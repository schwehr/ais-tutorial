//! `ais_tutorial` — the AIS tutorial's teaching decoder (Rust version).
//!
//! This crate is a line-by-line port of the Python package
//! `code/python/aistut`. The module names are the same as the Python file
//! names so that a reader can keep both versions open side by side:
//!
//! | module      | job                                             | chapter |
//! |-------------|-------------------------------------------------|---------|
//! | `nmea`      | sentence framing, checksum, GGA/GSA/GSV/RMC     | 1       |
//! | `bits`      | six-bit armouring and the `Bits` reader         | 2       |
//! | `vdm`       | `!AIVDM`/`!AIVDO` fields, multi-fragment `Assembler` | 2  |
//! | `msg_1_2_3` | position reports                                | 3       |
//! | `tagblock`  | NMEA 4.10 TAG blocks                            | 4       |
//! | `uscg`      | USCG N-AIS metadata after the checksum          | 5       |
//! | `common`    | `DecodeError`, EPFD and ship-type tables        | —       |
//! | `jsonout`   | deterministic JSON writer                       | —       |
//! | `msg_*`     | one per message (family); most are still stubs  | 6+      |
//! | `decode`    | CLI: lines in → JSON lines out                  | all     |
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `#![forbid(unsafe_code)]` is a crate-wide attribute: the compiler
//!   rejects any `unsafe` block anywhere in this crate. Nothing in a text
//!   decoder needs raw pointers, so we rule them out up front.
//! * `pub mod name;` tells the compiler to look for `src/name.rs` and make
//!   it a public sub-module of the crate. There is no `import` keyword:
//!   modules are *declared* here and *used* elsewhere with `use`.
//! * Comments starting with `//!` document the enclosing item (here: the
//!   crate); `///` documents the item that follows. `cargo doc` turns them
//!   into HTML.

#![forbid(unsafe_code)]

pub mod bits;
pub mod common;
pub mod decode;
pub mod jsonout;
pub mod nmea;
pub mod tagblock;
pub mod uscg;
pub mod vdm;

pub mod msg_1_2_3;
pub mod msg_4_11;
pub mod msg_5;
pub mod msg_6;
pub mod msg_7_13;
pub mod msg_8;
pub mod msg_9;
pub mod msg_10;
pub mod msg_12;
pub mod msg_14;
pub mod msg_15;
pub mod msg_16;
pub mod msg_17;
pub mod msg_18;
pub mod msg_19;
pub mod msg_20;
pub mod msg_21;
pub mod msg_22;
pub mod msg_23;
pub mod msg_24;
pub mod msg_25;
pub mod msg_26;
pub mod msg_27;
