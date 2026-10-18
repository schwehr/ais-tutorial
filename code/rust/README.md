# AIS tutorial decoder — Rust version

A line-by-line port of the Python reference (`../python/aistut/`) to Rust
2021. Standard library only; no crates. The binary prints exactly the same
JSON lines as the Python, C++ and TypeScript versions (the test-suite diffs
them byte for byte).

## Build and run

```sh
cd code/rust
cargo build --release              # warning-free; binary at target/release/decode
target/release/decode ../../data/ch03-position-reports.nmea
head -200 ../../sample-data/2007hydro/norfolk-log.ais | target/release/decode -
cargo test                         # unit tests: checksum, armouring, sint, JSON, assembler …
```

From the repository root, `tests/run_tests.sh rust` builds and compares the
output for every `data/*.nmea` against `tests/expected/*.jsonl`.

## Module map

Same names as the Python files, one file per module under `src/`:

| module | job | chapter | Rust features it shows |
|---|---|---|---|
| `lib.rs` | crate root: `#![forbid(unsafe_code)]`, `pub mod` list | — | crate/module layout, `//!` docs |
| `main.rs` | three-line binary wrapper | — | `std::env::args`, `process::exit` |
| `jsonout.rs` | deterministic JSON writer: `enum Json`, `Obj = BTreeMap<String, Json>`, `put()` | — | enums with data, `match`, `BTreeMap` (sorted keys for free), `impl Display`, `impl From<T>`, `format!("{:.*}")` |
| `nmea.rs` | `checksum`, `Sentence`, `parse_sentence`, GGA/GSA/GSV/RMC, Python-compat helpers (`py_strip`, `py_int`, `py_float`, `py_repr`) | 1 | `&str` vs `String`, `split_once`, `chars()` vs byte indexing, `Option`/`Result` and `?`, function-pointer dispatch (`gps_parser`) |
| `bits.rs` | six-bit armouring, `Bits` with `uint`/`sint`/`flag`/`text`/`raw`/`has`/`len` | 2 | `Vec<u8>`, slices and ranges, `Result` for input errors vs `assert!` for programmer errors, explicit integer casts |
| `vdm.rs` | `Vdm`, `parse_vdm`, `Message`, `Assembler` | 2 | ownership and moves (`asm.add(frag)` consumes the fragment), `.clone()` when a struct must own its data, `HashMap` with a tuple key, `let … else`, slice patterns |
| `msg_1_2_3.rs` | position reports, ROT, SOTDMA/ITDMA comm state | 3 | `match` tables returning `&'static str`, `Result` + `?`, `BTreeMap::extend` as `dict.update` |
| `tagblock.rs` | NMEA 4.10 TAG blocks | 4 | returning a slice of the input (lifetime elision), `strip_prefix`, `BTreeMap<String, String>` |
| `uscg.rs` | USCG N-AIS fields after the checksum | 5 | `match` with guards (`"s" if …`), `Option` combinators (`map`, `ok()`), `split_at(len_utf8())` |
| `common.rs` | `DecodeError` (`TooShort{..}`, `NotImplemented(n)`), `require_bits`, EPFD and ship-type tables | — | error enums with `Display`/`Error`, range patterns (`1..=19`, `200..`) |
| `msg_*.rs` | one stub per message family: `decode(&Bits) -> Result<Json, DecodeError>` | 6+ | `Err(DecodeError::NotImplemented(n))` instead of an exception |
| `decode.rs` | line loop, dispatch table, output shapes, universal newlines, `-` for stdin | all | `fn` pointers as a dispatch table, `BufWriter`, `from_utf8_lossy`, closures (`FnMut`), broken-pipe handling |

## Design notes

* Every decoder returns `Result<Json, DecodeError>`. The driver turns
  `TooShort` into `{"error":"message N too short: H bits < M"}` and
  `NotImplemented(n)` into `{"msg_type":n,"not_implemented":true}`.
* `Bits::uint`/`sint` **assert** that the field is inside the message — a
  decoder must call `require_bits` first (same contract as Python, where an
  `IndexError` would be a bug). Everything that depends on *input* (bad
  armour character, bad fill count, bad numbers) is a `Result`.
* Python's `str.strip()`, `int()`, `float()` and `repr()` accept and print
  slightly different things from Rust's `trim()`, `parse()` and `{:?}`.
  `nmea.rs` contains small helpers that copy Python's rules so that even
  malformed lines are reported identically.
* Where the Python reference would crash with an uncaught `ValueError` (a
  GPS numeric field such as `4312.2x`), the Rust version prints
  `{"line":N,"error":"could not convert string to float: '…'","raw":…}`
  using Python's own wording. This never happens with the tutorial data.
