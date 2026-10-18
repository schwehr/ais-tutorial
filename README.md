# An AIS Tutorial — from raw NMEA to every AIS message, in four languages

This repository teaches the maritime **Automatic Identification System (AIS)**
from the wire up, using real recordings and small, dependency-free decoders
written four times over — in **Python, C++, Rust and TypeScript** — so that you
can learn the protocol in the language you know and pick up the others by
comparison. Nothing is copied from an existing AIS or NMEA library; every line
was written for this tutorial from the specifications cited in the text.

The data are field recordings made between 2006 and 2011 around New
Hampshire, Massachusetts Bay and Hampton Roads: GPS receivers on aeroplanes,
AIS receivers on lighthouses and research vessels, base stations and
aids-to-navigation on the bench, the US Coast Guard's national AIS feed, and
the right-whale notice broadcasts of Stellwagen Bank. See
[`sample-data/AGENTS.md`](sample-data/AGENTS.md) for the full inventory. Only
small excerpts live in this repository ([`data/`](data/)); the raw logs
(650 MB) are kept outside git.

## Table of contents

| # | Chapter | Covers |
|---|---|---|
| 0 | [Introduction](chapters/00-introduction.md) | What AIS is, what you need, how the book and the code are organised |
| 1 | [NMEA 0183 basics](chapters/01-nmea-0183.md) | Sentence structure, talkers, checksum, `GGA`, `GSA`, `GSV`, `RMC`, proprietary sentences, log formats |
| 2 | [AIVDM and six-bit armouring](chapters/02-aivdm-armoring.md) | `!AIVDM`/`!AIVDO`, payload armouring, fill bits, multi-sentence assembly, bit extraction, six-bit text |
| 3 | [Position reports: messages 1, 2 and 3](chapters/03-position-reports-1-2-3.md) | Every field in depth, ROT coding, MMSI formats, SOTDMA/ITDMA communication state |
| 4 | [NMEA TAG blocks](chapters/04-tag-blocks.md) | NMEA 4.10 `\…\` metadata, grouping, converting legacy metadata |
| 5 | [USCG N-AIS metadata](chapters/05-uscg-metadata.md) | The `s`, `d`, `T`, `x`, `r`, time fields after the checksum |
| 6 | [Messages 4 and 11: base station report, UTC response](chapters/06-messages-4-11-base-station.md) | |
| 7 | [Message 5: static and voyage data](chapters/07-message-5-static-voyage.md) | |
| 8 | [Messages 6 and 8: binary messages and application identifiers](chapters/08-messages-6-8-binary.md) | DAC/FI, `BBM`/`ABM`/`ABK` sentences |
| 9 | [Application-specific messages in the wild](chapters/09-application-specific-messages.md) | Right-whale notices (366/34), met/hydro (1/11, 1/31), encrypted AIS (366/56–57) |
| 10 | [Message 9: SAR aircraft](chapters/10-message-9-sar-aircraft.md) | |
| 11 | [Messages 10, 15 and 16: inquiry, interrogation, assignment](chapters/11-messages-10-15-16-interrogation-assignment.md) | |
| 12 | [Messages 7, 12, 13 and 14: safety text and acknowledgements](chapters/12-messages-12-13-14-safety-text.md) | |
| 13 | [Messages 17, 20, 22 and 23: base-station control](chapters/13-messages-17-20-22-23-base-station-control.md) | DGNSS corrections, slot reservation, channel and group management |
| 14 | [Messages 18, 19 and 24: Class B](chapters/14-messages-18-19-24-class-b.md) | |
| 15 | [Message 21: aids to navigation](chapters/15-message-21-aton.md) | Physical and virtual AtoN, off-position, AtoN status bits |
| 16 | [Messages 25, 26 and 27: slot binaries and long range](chapters/16-messages-25-26-27.md) | |
| A | [References](chapters/appendix-a-references.md) | Citation keys |
| B | [Six-bit tables](chapters/appendix-b-six-bit-tables.md) | Armouring and six-bit ASCII tables |

## Running the code

All four decoders read NMEA log lines and print one JSON object per line
([`code/CONVENTIONS.md`](code/CONVENTIONS.md) defines the format). They
are built and checked against each other by `tests/run_tests.sh`.

```sh
# Python 3.10+ (no packages needed)
python3 code/python/decode.py data/ch03-position-reports.nmea

# C++17 (g++ or clang++)
make -C code/cpp && code/cpp/build/decode data/ch03-position-reports.nmea

# Rust 1.70+ (no crates)
(cd code/rust && cargo build --release) && code/rust/target/release/decode data/ch03-position-reports.nmea

# TypeScript (tsc 5+, node 18+; no npm packages)
(cd code/typescript && tsc -p tsconfig.json) && node code/typescript/dist/decode.js data/ch03-position-reports.nmea

# Everything, compared byte-for-byte with tests/expected/
tests/run_tests.sh
```

## Repository layout

```
chapters/      the tutorial text (Markdown), one file per chapter
code/          python/ cpp/ rust/ typescript/ — four implementations of the same decoder
data/          small NMEA excerpts used by the chapters (sources documented in each chapter)
tests/         run_tests.sh, regen_expected.sh, expected/*.jsonl
sample-data/   AGENTS.md guides to the raw recordings (the recordings themselves are not in git)
```

## Status

Work in progress. Chapters 0–5 and the core of the four decoders are written;
the message chapters are being added one at a time, each with decoders in all
four languages and test data.
