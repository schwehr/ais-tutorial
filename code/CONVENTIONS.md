# Code conventions for the four decoders

The tutorial ships the *same* decoder four times — Python, C++, Rust and
TypeScript — so that a reader can learn the protocol in whichever language
they prefer and compare idioms across languages. This file is the contract
that keeps the four in step. **Read it before changing any of them.**

## 1. Directory layout

```
code/
  python/      aistut/<module>.py          decode.py (CLI)   tools/
  cpp/         <module>.hpp                decode.cpp        Makefile   (C++23, no dependencies)
  rust/        src/<module>.rs             src/main.rs       Cargo.toml (edition 2024, no crates)
  typescript/  src/<module>.ts             src/decode.ts     tsconfig.json (tsc + node, no npm packages)
```

The module names are identical in all four:

| module | purpose | chapter |
|---|---|---|
| `nmea` | sentence splitting, checksum, GGA/GSA/GSV/RMC | 1 |
| `bits` | armour decoding, `Bits` reader (uint/sint/flag/text), six-bit ASCII | 2 |
| `vdm` | `!AIVDM`/`!AIVDO` fields, multi-fragment `Assembler` | 2 |
| `msg_1_2_3` | position reports | 3 |
| `tagblock` | NMEA 4.10 TAG blocks | 4 |
| `uscg` | USCG N-AIS metadata after the checksum | 5 |
| `common` | `DecodeError`, EPFD and ship-type tables | — |
| `jsonout` | deterministic JSON writer | — |
| `msg_4_11`, `msg_5`, `msg_6`, `msg_7_13`, `msg_8`, `msg_9`, `msg_10`, `msg_12`, `msg_14`, `msg_15`, `msg_16`, `msg_17`, `msg_18`, `msg_19`, `msg_20`, `msg_21`, `msg_22`, `msg_23`, `msg_24`, `msg_25`, `msg_26`, `msg_27` | one per message (family) | 6+ |
| `asm` | application-specific payload decoders keyed by (DAC, FI); used by `msg_6`/`msg_8` | 8–9 |
| `decode` | CLI: lines in → JSON lines out | all |

Every message module exposes **one** function, `decode(bits) -> object`,
taking the complete payload bit string (fill bits already removed). It must
check its minimum length with `require_bits` and raise/return the shared
`DecodeError` text `message N too short: H bits < M`.

## 2. The output contract (what the tests compare)

`decode FILE` prints one JSON object per *relevant* input line, in input
order. The test-suite (`tests/run_tests.sh`) diffs the four programs'
output byte-for-byte against `tests/expected/*.jsonl`, which are generated
from the Python version (`tests/regen_expected.sh`) and reviewed by hand.

Rules that make byte-identical output possible:

1. **Keys sorted** (ASCII order) at every level of every object.
2. **No whitespace** in the JSON (`{"a":1,"b":[2,3]}`).
3. **Integers** print as integers. **Booleans** as `true`/`false`. Missing
   values as `null`.
4. **Non-integers always have an explicit, fixed number of decimals**,
   chosen per field (lat/lon 6, SOG 1 or 2, COG 1, ROT 1, …) and identical in
   all four languages. Python wraps them in `Fixed(value, decimals)`; C++
   uses `std::format("{:.{}f}")`; Rust `format!("{:.*}")`; TypeScript `toFixed()`.
   Negative zero is printed as `0.0…` (no minus sign). A `Fixed` whose
   value is NaN, ±infinity or has |x| ≥ 1e21 prints as `null` in all four
   (JSON has no `nan`/`inf` tokens, and `toFixed` switches to exponent
   notation at 1e21); such values only arise from corrupted text such as
   `hdop` = `inf`, never from a real receiver.
5. **Strings** are escaped minimally: `"` → `\"`, `\` → `\\`, control
   characters → `\n`, `\r`, `\t` or `\u00XX`. Non-ASCII is passed through
   as UTF-8 (it never occurs in six-bit text).
6. **Logger comments** (`# …`) and blank lines produce no output. A line
   that completes a multi-fragment message produces one object; the earlier
   fragments produce nothing.
7. Per-line object shapes (see `python/aistut/decode.py` docstring):
   * GPS: `{"line":N,"sentence":"GGA",…}`
   * AIS: `{"line":N,"ais":{…},"channel":"A","talker":"AI","own_ship":false,"payload_bits":168,"fragments":1}`
   * unknown formatter: `{"line":N,"address":"PMTK010","skipped":"no parser"}`
   * problems at the line / sentence / VDM-field layer:
     `{"line":N,"error":"<text>","raw":"<line>"}` with these exact texts:
     `no '$' or '!' in line`, `no checksum`, `checksum mismatch`,
     `unterminated TAG block`, `address field too short: '…'`,
     `VDM needs 6 fields, got N`, `fill bits must be 0..5, got N`,
     `invalid literal for int() with base 10: '…'`,
     `could not convert string to float: '…'`. The last two are Python's
     own `ValueError` messages (including its `repr()` quoting of the
     offending text); the C++/Rust/TypeScript ports reproduce them with
     their `py`-compat helpers, and they are what a malformed numeric
     field in a GGA/GSA/GSV/RMC sentence produces — the GPS parsers never
     abort the run.
   * problems at the AIS payload layer are emitted **without** `"raw"` but
     with `channel`/`talker`/`own_ship`/`fragments` and, once known,
     `payload_bits` and `msg_type`: `payload too short: N bits`,
     `unknown message type N`, `message N too short: H bits < M`,
     `illegal armour character 'c'`.
   * `"uscg":{…}` and `"tagblock":{…}` are added to any of the above when
     the line carried them.
   * A decoder that is still a stub returns `{"msg_type":N,"not_implemented":true}`.
8. **Integer range.** Numeric NMEA fields are parsed into 64-bit integers
   (Python's unbounded `int` and JavaScript's doubles agree with that for
   every value a receiver can emit). Fields whose value does not fit in
   64 bits are out of scope: the four decoders may then differ (Python
   prints the big integer, C++/Rust report `invalid literal…`, TypeScript
   rounds), and the tests do not cover them.

## 3. Field naming

* `snake_case`, units in the name when not obvious: `sog_knots`, `cog_deg`,
  `rot_deg_per_min`, `altitude_m`, `draught_m`, `time_of_arrival_s`.
* Raw enumerations are output as the integer **and** a `_text` companion:
  `nav_status` + `nav_status_text`, `epfd` + `epfd_text`, `ship_type` +
  `ship_type_text`. The text strings are defined once per language in the
  message module (or `common`) and must match across languages exactly.
* Availability of "not available" sentinel values is reported with a
  boolean companion where the spec defines a sentinel: `sog_available`,
  `position_available`, `cog_available`, `heading_available`.
* Binary payloads that are not decoded are output as `"data":"0101…"`
  (a string of bits) plus `"data_bits":N`.
* Spare/reserved fields are output (`"spare":0`) — readers should see them.
* Exception for ASM sub-objects (`application` in messages 6/8, chapter 9):
  sensor fields with a not-available sentinel print `null` instead of a
  per-field `_available` boolean, because met/hydro payloads have dozens of
  optional readings; enumerations keep their `_text` companions.

## 4. Adding a message (checklist for chapter authors)

1. Write the Python decoder first in `python/aistut/msg_N.py`, following
   the spec table cited in the module docstring. Run it over real data.
2. Pick a *small* set of real sentences from `sample-data/` that exercise
   the fields, save them as `data/chNN-<topic>.nmea` (tens of lines; strip
   `\r`; note source file and line numbers in the chapter).
3. `tests/regen_expected.sh`; read the new `tests/expected/chNN-*.jsonl`
   line by line against the spec and the chapter's hand-decoded example.
4. Port to C++ (`cpp/msg_N.hpp`), Rust (`rust/src/msg_N.rs`), TypeScript
   (`typescript/src/msg_N.ts`) with the same field names and texts.
5. `tests/run_tests.sh` must print `ok` for all four languages.
6. Write the chapter; every bit offset and enumeration must cite its spec
   table.

## 5. Style per language (teaching code, not production code)

* Prefer clarity over speed: explicit loops, named constants, comments that
  say *why*. No clever metaprogramming.
* No third-party dependencies anywhere. Standard library only.
* C++23 (`-std=c++23 -Wall -Wextra -pedantic`; GCC 14+ / Clang 18+ with
  libstdc++ 14+), header-only modules, `std::map` for objects (sorted for
  free), `std::optional` (with the C++23 monadic `transform`),
  `std::expected` where an error is returned rather than thrown
  (`parse_vdm`), `std::format`/`std::println` for all text output,
  `std::string_view`/`std::span` for borrowed data, `std::views::chunk` for
  repeated bit groups, designated initialisers for structs, no exceptions
  across module boundaries except `DecodeError`.
* Rust edition 2024 (`rust-version = "1.93"`), `#![forbid(unsafe_code)]`,
  `BTreeMap` for objects, `Result` for errors, no `unwrap()` on
  input-derived values; `cargo clippy --all-targets -- -D warnings` clean.
  Prefer the modern spellings when they read better: `let … else`, let
  chains (`if let Some(x) = a && cond`), `Option::is_some_and`, inline
  format arguments (`format!("{name}")`), `u64::from(x)` over `as` for
  widening.
* TypeScript `strict`, ES2022 target, CommonJS modules for plain `node`,
  `Map`-free plain objects with a custom sorted stringify.
* Python 3.14+, standard library only, type hints, dataclasses. Modern
  typing only: `X | None` (PEP 604), built-in generics `list[str]` /
  `dict[int, str]` (PEP 585), `collections.abc.Callable`, `type` aliases
  (PEP 695); no `typing.Optional`/`List`, no quoted forward references and
  no `from __future__ import annotations` (PEP 649 makes them unnecessary).
  `decode.py` refuses to run on an older interpreter; the test scripts
  honour `PYTHON=python3.14` when plain `python3` is older.
