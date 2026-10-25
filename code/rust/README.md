# AIS tutorial decoder — Rust version

A line-by-line port of the Python reference (`../python/aistut/`) to Rust,
edition 2024 (`rust-version = "1.93"`, i.e. rustc/cargo 1.93 or newer).
Standard library only; no crates. The binary prints exactly the same JSON
lines as the Python, C++ and TypeScript versions (the test-suite diffs them
byte for byte).

## Build and run

```sh
cd code/rust
cargo build --release              # warning-free; binary at target/release/decode
target/release/decode ../../data/ch03-position-reports.nmea
head -200 ../../sample-data/2007hydro/norfolk-log.ais | target/release/decode -
cargo test                         # unit tests: checksum, armouring, sint, JSON, assembler …
cargo clippy --release --all-targets -- -D warnings   # lint-clean, no `#[allow]`s
```

From the repository root, `tests/run_tests.sh rust` builds and compares the
output for every `data/*.nmea` against `tests/expected/*.jsonl`.

## Module map

Same names as the Python files, one file per module under `src/`:

| module | job | chapter | Rust features it shows |
|---|---|---|---|
| `lib.rs` | crate root: `#![forbid(unsafe_code)]`, `pub mod` list | — | crate/module layout, `//!` docs |
| `main.rs` | three-line binary wrapper | — | `std::env::args`, `process::exit` |
| `jsonout.rs` | deterministic JSON writer: `enum Json`, `Obj = BTreeMap<String, Json>`, `put()` | — | enums with data, `match`, `BTreeMap` (sorted keys for free), `impl Display`, `impl From<T>`, inline format arguments (`format!("{value:.decimals$}")`), `u32::from(char)` |
| `nmea.rs` | `checksum`, `Sentence`, `parse_sentence`, GGA/GSA/GSV/RMC, Python-compat helpers (`py_strip`, `py_int`, `py_float`, `py_repr`) | 1 | `&str` vs `String`, `split_once`, `chars()` vs byte indexing, `Option`/`Result` and `?`, function-pointer dispatch (`gps_parser`) |
| `bits.rs` | six-bit armouring, `Bits` with `uint`/`sint`/`flag`/`text`/`raw`/`has`/`len` | 2 | `Vec<u8>`, slices and ranges, `Result` for input errors vs `assert!` for programmer errors, explicit integer casts |
| `vdm.rs` | `Vdm`, `parse_vdm`, `Message`, `Assembler` | 2 | ownership and moves (`asm.add(frag)` consumes the fragment), `.clone()` when a struct must own its data, `HashMap` with a tuple key, `let … else`, slice patterns, let chains (`if let Some(g) = … && cond`), `Option::is_some_and` |
| `msg_1_2_3.rs` | position reports, ROT, SOTDMA/ITDMA comm state | 3 | `match` tables returning `&'static str`, `Result` + `?`, `BTreeMap::extend` as `dict.update` |
| `tagblock.rs` | NMEA 4.10 TAG blocks | 4 | returning a slice of the input (lifetime elision), `strip_prefix`, `BTreeMap<String, String>`, let chains |
| `uscg.rs` | USCG N-AIS fields after the checksum | 5 | `match` with guards (`"s" if …`), `Option` combinators (`map`, `ok()`), `split_at(len_utf8())` |
| `commstate.rs` | TDMA communication-state schedule and UTC clock helpers | 17 | `pub use` re-exports, `i64::rem_euclid`, `if let` chains |
| `common.rs` | `DecodeError` (`TooShort{..}`, `NotImplemented(n)`), `require_bits`, EPFD and ship-type tables | — | error enums with `Display`/`Error`, range patterns (`1..=19`, `200..`) |
| `msg_*.rs` | one module per message family: `decode(&Bits) -> Result<Json, DecodeError>` | 6+ | `matches!`, range patterns, `if` as an expression (`let pos = if … { 144 } else { 96 }`, msg_16), `u64::from` for lossless widening (msg_5) |
| `decode.rs` | line loop, dispatch table, output shapes, universal newlines, `-` for stdin | all | `fn` pointers as a dispatch table, `BufWriter`, `from_utf8_lossy`, closures (`FnMut`), broken-pipe handling |

## Edition 2024 and the idioms used

The crate targets edition 2024 and builds warning-free with
`cargo build --release` and `cargo clippy --release --all-targets -- -D warnings`
on rustc 1.93. The migration from edition 2021 needed no source changes
(`cargo fix --edition` was a no-op: no `unsafe`, no `gen` identifiers, no
`impl Trait` lifetime captures, no `Box<[T]>` iteration). Where a modern
spelling makes the teaching code clearer we use it, and the module's "Rust
notes" explain it the first time it appears:

* **let chains** — `if let Some(h) = hexpart && h.chars().count() >= 2`
  (`tagblock.rs`) and `else if let Some(group) = map.get_mut(&key) && …`
  (`vdm.rs`) replace nested `if let` + `if`.
* **`let … else`** — `let Some(decoder) = decoder_for(t) else { … return };`
  (`decode.rs`, `tagblock.rs`, `uscg.rs`, `vdm.rs` slice pattern).
* **`Option::is_some_and`** — `last.is_some_and(|g| g.own_ship)` (`vdm.rs`).
* **inline format arguments** — `format!("{name}{name_extension}")`
  (`msg_21.rs`), `format!("{value:.decimals$}")` (`jsonout.rs`),
  `format!("{month:02}")` (`msg_5.rs`).
* **lossless conversions with `From`** — `u64::from(bit)` (`bits.rs`),
  `u64::from(b - b'0')` (`msg_5.rs`), `u32::from(c)` (`jsonout.rs`);
  `as` is kept only where a narrowing or float conversion is intended and
  commented.
* **`if` as an expression** — `let pos = if bits.has(92, 52) { …; 144 } else { …; 96 };`
  (`msg_16.rs`).

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
