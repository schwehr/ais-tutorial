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
| 3 | [Messages 1, 2 and 3: Class A position reports](chapters/03-position-reports-1-2-3.md) | Every field in depth, ROT coding, MMSI formats, SOTDMA/ITDMA communication state |
| 4 | [NMEA TAG blocks](chapters/04-tag-blocks.md) | NMEA 4.10 `\…\` metadata, grouping, converting legacy metadata |
| 5 | [USCG N-AIS metadata](chapters/05-uscg-metadata.md) | The `s`, `d`, `T`, `x`, `r`, time fields after the checksum |
| 6 | [Messages 4 and 11: base station report and UTC/date response](chapters/06-messages-4-11-base-station.md) | UTC fields, base-station sync, clock-offset auditing, rogue base stations, message 10/11 pairs |
| 7 | [Message 5: ship static and voyage related data](chapters/07-message-5-static-voyage.md) | The 424-bit static/voyage report: ship dimensions and reference point, ETA sub-fields, IMO check digit, `@`-padded text, what real units get wrong |
| 8 | [Messages 6 and 8: binary messages, application identifiers and the BBM/ABM/ABK sentences](chapters/08-messages-6-8-binary.md) | Addressed and broadcast binary containers, DAC/FI application identifiers, slot limits and padding, the `BBM`/`ABM`/`ABK` sending side |
| 9 | [Application-specific messages in the wild](chapters/09-application-specific-messages.md) | Right-whale notices (366/34), met/hydro (1/11, 1/31), encrypted AIS (366/56–57) |
| 10 | [Message 9: standard SAR aircraft position report](chapters/10-message-9-sar-aircraft.md) | The 168-bit aircraft position report: altitude, whole-knot speed, `111MIDxxx` MMSIs, shared communication-state decoding, building a labelled synthetic example |
| 11 | [Messages 10, 15 and 16: UTC inquiry, interrogation and assignment](chapters/11-messages-10-15-16-interrogation-assignment.md) | UTC inquiry, the 88–160-bit interrogation shapes, assignment by report rate or slot increment, what ships and base stations actually ask each other |
| 12 | [Messages 7, 12, 13 and 14: safety-related text and acknowledgements](chapters/12-messages-12-13-14-safety-text.md) | Addressed and broadcast six-bit text, byte-alignment padding, acknowledgement sequence numbers and retries, the fixed AIS-SART/MOB/EPIRB texts |
| 13 | [Messages 17, 20, 22 and 23: base-station control of the data link](chapters/13-messages-17-20-22-23-base-station-control.md) | DGNSS corrections, slot reservation, channel and group management |
| 14 | [Messages 18, 19 and 24: Class B](chapters/14-messages-18-19-24-class-b.md) | Class B "CS" versus "SO", the fixed communication state, the two-part static data report, the vendor-ID field across editions, `$AISSD`/`$AIVSD` configuration, pairing part A with part B |
| 15 | [Message 21: aids to navigation](chapters/15-message-21-aton.md) | Physical and virtual AtoN, off-position, AtoN status bits |
| 16 | [Messages 25, 26 and 27: short binary messages and the long-range broadcast](chapters/16-messages-25-26-27.md) | Single- and multi-slot binaries with flag-dependent layouts, a communication state at the *end* of the payload, the 96-bit satellite position report, noise versus rare messages |
| A | [References and how we cite them](chapters/appendix-a-references.md) | Citation keys |
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

All seventeen chapters (0–16) and the two appendices are written, and every
AIS message type 1–27 is decoded by all four implementations. The decoders
print byte-identical JSON for every file in `data/`; `tests/run_tests.sh`
checks all four languages against `tests/expected/` and passes.

## Contributing

Before changing anything read [`AGENTS.md`](AGENTS.md) (repository rules),
[`chapters/AUTHORING.md`](chapters/AUTHORING.md) (chapter skeleton, voice and
citation style) and [`code/CONVENTIONS.md`](code/CONVENTIONS.md) (the output
contract the four decoders must honour). A change that touches `code/`,
`data/` or `tests/` must leave `tests/run_tests.sh` green in all four
languages.

## License

No license file is included yet; the author will choose one. Until then the
usual default applies — all rights reserved — and the specifications cited in
Appendix A remain the property of their publishers.
