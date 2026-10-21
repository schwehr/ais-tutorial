# Chapter authoring guide

This file tells authors (human or agent) how every chapter of the tutorial
is built, so that the book reads as one voice. Read it together with
`../code/CONVENTIONS.md`.

## Audience and voice

* The reader is a programmer who has *not* seen NMEA or AIS before and who
  may be new to one or more of Python, C++, Rust and TypeScript. We teach
  the protocol **and** the languages at the same time: every time a chapter
  introduces a language construct for the first time, it gets a one- or
  two-sentence explanation in a "Language notes" box. Later chapters only
  explain *new* constructs.
* Be concrete. Every statement about a bit, a unit, a sentinel value or a
  rule is followed by a citation in square brackets, e.g.
  `[M.1371-5 §3.1 Table 48]`, `[IEC 61162-1 §7.2.3]`, `[AIVDM "Type 5"]`.
  Keys are defined in `appendix-a-references.md`.
* No links to files outside this repository. Refer to the companion
  repository by URL only. Inside the repository use relative links.
* Use real data. Every example sentence comes from `../data/` (which is
  excerpted from `../sample-data/`); name the source file and line number.
  If a construct does not occur in the data (TAG blocks did not), say so and
  explain how the example was produced.
* Keep excerpts small. Show at most ~10 sentences per example and at most
  one or two full JSON output objects; otherwise show selected fields.

## Chapter skeleton

```
# N. Title

> **What you will learn** — three to five bullets.

## N.1 Where this fits
Why the message/sentence exists, who sends it, how often, who consumes it.
Two sub-headings are mandatory for every AIS message chapter:
### Operational use        (bridge, VTS, SAR, port, fleet management …)
### Unattended / automated uses
   How machines (not people) use or abuse the message: data loggers,
   AtoN/buoy telemetry, fishing-gear markers that put battery voltage in
   the name field, satellite AIS, geofences, anomaly detection, spoofing
   detection, etc. Be specific; cite USCG Safety Alerts or IALA where
   possible.

## N.2 The specification
Field table: | Bits | Len | Field | Type/units | Not-available value | Notes |
Every row cites its table. Then explain each field in prose, especially the
odd ones (ROT square-root coding, two's-complement lat/lon, time stamp 60–63).

## N.3 Decoding one message by hand
Take ONE sentence from data/, show the armoured characters, the 6-bit
groups, the bit string with offsets, and each field value. Readers should
be able to follow with pencil and paper.

## N.4 The code
### Python   — full listing of the new module (or the relevant function)
### C++      — the same, with Language notes
### Rust     — the same, with Language notes
### TypeScript — the same, with Language notes
Point out *where the four differ* (integer widths, string handling, error
handling, optional values) rather than repeating identical explanations.

## N.5 Sample data and results
The data/chNN-*.nmea file (or part of it), the command to run each decoder,
and the parsed results from tests/expected/. Annotate interesting values.

## N.6 Pitfalls and real-world deviations
What equipment gets wrong; what the logs show; how to validate.

## N.7 Exercises
Three to six, increasing difficulty, at least one per language.

## N.8 References
The citation keys used, with the specific sections.
```

## Formatting rules

* GitHub-flavoured Markdown. Headings numbered as above. Code fences with
  language tags (`python`, `cpp`, `rust`, `typescript`, `text`, `json`).
* Bit offsets are **zero-based** everywhere (the way `Bits.uint(start, len)`
  takes them). When quoting the ITU table, which gives only lengths, write
  "bits 61–88 (28 bits)".
* Show payload bits in groups of six with the character above each group
  when doing hand decodes:
  ```text
  1      5      N      9      1      b
  000001 000101 011110 001001 000001 101010
  ```
* Tables of enumerations list *all* values the spec defines, not just the
  common ones.
* Terminology: "sentence" (NMEA), "message" (AIS), "payload" (the armoured
  text), "fragment" (one sentence of a multi-sentence message), "field".
  MMSI, SOG, COG, ROT, EPFD, RAIM, SOTDMA/ITDMA/CSTDMA/RATDMA/FATDMA are
  spelled out on first use in each chapter.

## Language notes — what to explain where

| Construct | First explained in |
|---|---|
| Python: functions, f-strings, slicing, `int(s, 2)`, dict, list, `dataclass`, type hints, `X | None`, exceptions, modules/packages | ch. 1–2 |
| C++: `#include`, `std::string`/`string_view`, `std::vector`, `struct`/`class`, `std::optional`, `std::map`, references vs values, `const`, exceptions, header-only design, `snprintf` | ch. 1–2 |
| Rust: `fn`, ownership and borrowing (`&str` vs `String`), `Option`/`Result` and `?`, `match`, `struct`/`impl`, `enum` with data, `Vec`, `BTreeMap`, `impl Display`, modules | ch. 1–2 |
| TypeScript: `interface`/`type`, `class`, union types `number | null`, `readonly`, `Map`, arrow functions, template literals, modules/imports, bitwise 32-bit pitfall, `toFixed` | ch. 1–2 |
| Enumerations as tables, two's complement, bit extraction loops | ch. 2–3 |
| Error handling across module boundaries (`DecodeError`) | ch. 3 |
| State across lines (fragment assembly) | ch. 2 |
| Later chapters: only genuinely new constructs (e.g. recursion for nested ASM payloads, iterators for repeated groups in messages 7/15/20/22). | |

## Review checklist (apply before marking a chapter done)

1. Every bit offset in the chapter matches the code in all four languages.
2. Every number in the "results" section comes from `tests/expected/`.
3. `tests/run_tests.sh` passes.
4. Every spec claim has a citation key that exists in Appendix A.
5. No absolute paths, no links outside the repository.
6. The "Unattended / automated uses" section exists and is specific.
7. Exercises have answers that a reader could verify with the code.
