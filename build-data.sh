#!/bin/sh
# Build lectionary data from Divinum Officium Missa files
set -e

SRCDIR="${1:-../divinum-officium-data/web/www/missa/English}"
OUTDIR="data"

rm -rf "$OUTDIR"
mkdir -p "$OUTDIR/Tempora" "$OUTDIR/Sancti" "$OUTDIR/Commune"

# Copy Tempora (temporal cycle Mass propers)
for f in "$SRCDIR"/Tempora/*.txt; do
	cp "$f" "$OUTDIR/Tempora/"
done

# Copy Sancti (sanctoral cycle Mass propers)  
for f in "$SRCDIR"/Sancti/*.txt; do
	cp "$f" "$OUTDIR/Sancti/"
done

# Copy Commune (common Masses)
for f in "$SRCDIR"/Commune/*.txt; do
	cp "$f" "$OUTDIR/Commune/"
done

echo "Built data:"
echo "  Tempora: $(ls "$OUTDIR"/Tempora/*.txt | wc -l | tr -d ' ') files"
echo "  Sancti:  $(ls "$OUTDIR"/Sancti/*.txt | wc -l | tr -d ' ') files"
echo "  Commune: $(ls "$OUTDIR"/Commune/*.txt | wc -l | tr -d ' ') files"
