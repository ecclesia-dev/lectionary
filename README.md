# lectionary

**TLM Daily Mass Readings from your terminal.**

`1962 Missal · 938 Propers · Full Temporal & Sanctoral Cycles · Public Domain`

A command-line tool for reading the Mass propers of the Traditional Latin Mass (Extraordinary Form). Covers the full temporal and sanctoral cycles — Introit, Collect, Epistle, Gradual, Gospel, Offertory, and Communion texts for every day of the liturgical year.

Built for Catholics who live in the terminal. One command, today's Mass.

---

## Quick Start

```
$ lectionary
══════════════════════════════════════
  1st Sunday of Lent
  Sunday, February 22, 2026
  Color: violet
══════════════════════════════════════

  ── Introit ──
  He shall call upon Me, and I will answer him...

  ── Epistle ──
  A reading from the Second Letter of St. Paul...

  ── Gospel ──
  Continuation of the Holy Gospel according to Matthew...
```

## Installation

Requires Rust toolchain for the calendar helper binary.

```sh
git clone https://github.com/ecclesia-dev/lectionary.git
cd lectionary
make
sudo make install
```

To uninstall:

```sh
sudo make uninstall
```

## Usage

```
lectionary [options] [YYYY-MM-DD]
```

### Options

| Flag | Description |
|------|-------------|
| `--introit` | Introit only |
| `--epistle` | Epistle only |
| `--gradual` | Gradual only |
| `--gospel` | Gospel only |
| `--offertory` | Offertory only |
| `--communion` | Communion only |
| `--collect` | Collect only |
| `--all` | All propers (default) |
| `-h, --help` | Show help |

### Examples

```sh
$ lectionary                      # today's propers
$ lectionary 2026-12-25           # Christmas Day
$ lectionary --gospel             # just today's Gospel
$ lectionary --epistle 2026-04-05 # Easter Epistle
```

### Piping

```sh
lectionary --gospel | fmt -w 60              # rewrap Gospel text
lectionary 2026-12-25 | grep -A5 "Introit"   # extract Christmas Introit
```

## Architecture

Two components:

1. **`lectionary`** — POSIX shell script with embedded Mass propers data (tar archive appended to script)
2. **`lectionary-cal`** — Small Rust binary that computes the 1962 liturgical calendar (maps civil dates to Divinum Officium Missa file keys)

The shell script calls `lectionary-cal` for calendar computation, then extracts and formats the appropriate propers from its embedded data. Single-command install, no runtime dependencies beyond the two binaries.

## Data

Mass propers from the [Divinum Officium](https://github.com/DivinumOfficium/divinum-officium) project:

| Cycle | Files | Coverage |
|-------|-------|----------|
| **Tempora** | 480 | Advent through Christ the King |
| **Sancti** | 458 | Fixed feasts of the sanctoral cycle |

Each file contains the full text of Introit, Epistle, Gradual, Gospel, Offertory, Communion, and prayers.

## Related Projects

| Tool | Description |
|------|-------------|
| **[drb](https://github.com/ecclesia-dev/drb)** | Douay-Rheims Bible |
| **[martyrology](https://github.com/ecclesia-dev/martyrology)** | Roman Martyrology |
| **[opus](https://github.com/ecclesia-dev/opus)** | Traditional Divine Office |
| **[pray](https://github.com/ecclesia-dev/pray)** | Catholic prayers (Rosary, Angelus, and more) |
| **[calendar-ios](https://github.com/ecclesia-dev/calendar-ios)** | 1962 Liturgical Calendar for iOS |

*Ad Maiorem Dei Gloriam.*
