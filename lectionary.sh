#!/bin/sh
# lectionary: TLM Daily Mass Readings from your terminal
# Data: Divinum Officium project / Douay-Rheims Bible
# License: Public domain

set -e

SELF="$0"

get_data() {
	sed '1,/^#EOF$/d' < "$SELF" | tar xzf - -O "$1" 2>/dev/null || true
}

if [ -z "$PAGER" ]; then
	if command -v less >/dev/null; then
		PAGER="less"
	else
		PAGER="cat"
	fi
fi

PART=""
DATE_ARG=""

show_help() {
	exec >&2
	echo "usage: $(basename "$0") [options] [YYYY-MM-DD]"
	echo
	echo "  lectionary                   today's Mass propers"
	echo "  lectionary 2026-04-05        propers for a specific date"
	echo
	echo "Options:"
	echo "  --introit       show only the Introit"
	echo "  --epistle       show only the Epistle"
	echo "  --gradual       show only the Gradual"
	echo "  --gospel        show only the Gospel"
	echo "  --offertory     show only the Offertory"
	echo "  --communion     show only the Communion"
	echo "  --collect       show only the Collect (Oratio)"
	echo "  --all           show all propers (default)"
	echo "  -h, --help      show help"
	exit 2
}

while [ $# -gt 0 ]; do
	case "$1" in
		--introit)   PART="Introitus"; shift ;;
		--epistle)   PART="Lectio"; shift ;;
		--gradual)   PART="Graduale"; shift ;;
		--gospel)    PART="Evangelium"; shift ;;
		--offertory) PART="Offertorium"; shift ;;
		--communion) PART="Communio"; shift ;;
		--collect)   PART="Oratio"; shift ;;
		--all)       PART=""; shift ;;
		-h|--help|help) show_help ;;
		-*) show_help ;;
		*) DATE_ARG="$1"; shift ;;
	esac
done

# Get liturgical day info from helper binary
CAL_BIN="lectionary-cal"

# Look for the helper binary
for p in \
	"$(dirname "$SELF")/lectionary-cal" \
	"$(dirname "$SELF")/cal-helper/target/release/lectionary-cal" \
	"$(command -v lectionary-cal 2>/dev/null)"; do
	if [ -x "$p" ] 2>/dev/null; then
		CAL_BIN="$p"
		break
	fi
done

if [ -n "$DATE_ARG" ]; then
	CAL_INFO=$("$CAL_BIN" "$DATE_ARG")
else
	CAL_INFO=$("$CAL_BIN")
fi

TEMPORA_KEY=$(echo "$CAL_INFO" | cut -f1)
SANCTI_KEY=$(echo "$CAL_INFO" | cut -f2)
TITLE=$(echo "$CAL_INFO" | cut -f3)
COLOR=$(echo "$CAL_INFO" | cut -f4)
SOURCE=$(echo "$CAL_INFO" | cut -f5)

# Try to get Missa data: first Sancti, then Tempora
# Sancti have priority for fixed feasts
MISSA=""

# Check if Sancti file exists and has Mass propers
# Try Tempora first (always available for temporal cycle)
tempora_data=$(get_data "data/Tempora/${TEMPORA_KEY}.txt")

# Check if Sancti file has its own Mass propers
# Try main file first, then common suffixed variants (m3 for Christmas, etc.)
sancti_data=$(get_data "data/Sancti/${SANCTI_KEY}.txt")
has_sancti=""
if [ -n "$sancti_data" ] && echo "$sancti_data" | grep -q '^\[Introitus\]\|^\[Lectio\]\|^\[Evangelium\]'; then
	has_sancti="1"
fi
# If main file has no propers, try m3 variant (e.g., Christmas 3rd Mass)
if [ -z "$has_sancti" ]; then
	sancti_m3=$(get_data "data/Sancti/${SANCTI_KEY}m3.txt")
	if [ -n "$sancti_m3" ] && echo "$sancti_m3" | grep -q '^\[Introitus\]\|^\[Lectio\]\|^\[Evangelium\]'; then
		sancti_data="$sancti_m3"
		has_sancti="1"
	fi
fi

# Use source hint from calendar helper
if [ "$SOURCE" = "sancti" ] && [ -n "$has_sancti" ]; then
	MISSA="$sancti_data"
elif [ -n "$tempora_data" ]; then
	MISSA="$tempora_data"
elif [ -n "$has_sancti" ]; then
	MISSA="$sancti_data"
fi

# Handle @references (DO cross-file references)
# e.g., @Tempora/Pent03-0r or @Commune/C4a
resolve_refs() {
	_data="$1"
	# Check if first line is a @reference
	_first=$(echo "$_data" | head -1)
	case "$_first" in
		@*)
			_ref=$(echo "$_first" | sed 's/^@//')
			_resolved=$(get_data "data/${_ref}.txt")
			if [ -n "$_resolved" ]; then
				# Merge: resolved file first, then rest of original
				_rest=$(echo "$_data" | tail -n +2)
				echo "$_resolved"
				echo "$_rest"
				return
			fi
			;;
	esac
	echo "$_data"
}

MISSA=$(resolve_refs "$MISSA")

if [ -z "$MISSA" ]; then
	echo "No Mass propers found for ${TITLE} (${SANCTI_KEY})" >&2
	exit 1
fi

# Resolve inline @references within a section
resolve_inline_refs() {
	while IFS= read -r _line; do
		case "$_line" in
			@*)
				# Parse @Path/File:Section or just @Path/File
				_ref=$(echo "$_line" | sed 's/^@//')
				_refsect=""
				case "$_ref" in
					*:*) _refsect=$(echo "$_ref" | cut -d: -f2); _ref=$(echo "$_ref" | cut -d: -f1) ;;
				esac
				_refdata=$(get_data "data/${_ref}.txt")
				if [ -n "$_refdata" ] && [ -n "$_refsect" ]; then
					echo "$_refdata" | awk -v sect="[${_refsect}]" '
						$0 == sect { found=1; next }
						/^\[/ && found { exit }
						found { print }
					'
				elif [ -n "$_refdata" ]; then
					# No section specified — skip (whole-file refs not useful in section context)
					:
				else
					echo "$_line"
				fi
				;;
			*) echo "$_line" ;;
		esac
	done
}

# Extract a section from the Missa data
get_section() {
	_section="$1"
	_data="$2"
	echo "$_data" | awk -v sect="[${_section}]" '
		$0 == sect { found=1; next }
		/^\[/ && found { exit }
		found { print }
	' | resolve_inline_refs
}

# Format DO markup for terminal
format_text() {
	sed '
		s/^v\. /  /
		s/^V\. /  ℣. /
		s/^R\. /  ℟. /
		s/^!\(.*\)/  [\1]/
		s/^\$.*//
		s/^&Gloria/  Glory be to the Father, and to the Son, and to the Holy Ghost. As it was in the beginning, is now, and ever shall be, world without end. Amen./
		s/^_$//
	' | grep -v '^$' | head -100
}

# Print a section with header
print_section() {
	_name="$1"
	_label="$2"
	_content=$(get_section "$_name" "$MISSA")
	if [ -n "$_content" ]; then
		printf '\n  ── %s ──\n\n' "$_label"
		echo "$_content" | format_text | while IFS= read -r line; do
			printf '  %s\n' "$line"
		done
	fi
}

# Output
(
if [ -n "$PART" ]; then
	# Single part requested
	case "$PART" in
		Introitus)   print_section "Introitus" "Introit" ;;
		Lectio)      print_section "Lectio" "Epistle" ;;
		Graduale)    print_section "Graduale" "Gradual" ;;
		Evangelium)  print_section "Evangelium" "Gospel" ;;
		Offertorium) print_section "Offertorium" "Offertory" ;;
		Communio)    print_section "Communio" "Communion" ;;
		Oratio)      print_section "Oratio" "Collect" ;;
	esac
else
	# Full propers
	printf '══════════════════════════════════════\n'
	printf '  %s\n' "$TITLE"
	if [ -n "$DATE_ARG" ]; then
		printf '  %s\n' "$DATE_ARG"
	else
		printf '  %s\n' "$(date '+%A, %B %d, %Y')"
	fi
	printf '  Color: %s\n' "$COLOR"
	printf '══════════════════════════════════════\n'

	print_section "Introitus" "Introit"
	print_section "Oratio" "Collect"
	print_section "Lectio" "Epistle"
	print_section "Graduale" "Gradual"
	print_section "Tractus" "Tract"
	print_section "Sequentia" "Sequence"
	print_section "Evangelium" "Gospel"
	print_section "Offertorium" "Offertory"
	print_section "Secreta" "Secret"
	print_section "Communio" "Communion"
	print_section "Postcommunio" "Postcommunion"

	printf '\n══════════════════════════════════════\n'
fi
) | ${PAGER}
