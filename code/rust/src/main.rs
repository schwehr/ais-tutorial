//! `decode` — the command-line entry point (see `decode.rs` for the logic).
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * A binary crate needs a `fn main()`. Ours is three lines: collect the
//!   command-line arguments, hand them to the library, and turn the
//!   returned status code into the process exit code.
//! * `std::env::args()` yields `String`s and *panics* on arguments that are
//!   not valid UTF-8; `args_os()` would give raw `OsString`s. File names in
//!   this tutorial are plain ASCII, so the simple form is fine.
//! * `.skip(1)` drops the program name, which Python's `sys.argv[1:]` also
//!   does; `.collect()` gathers the iterator into a `Vec<String>`.

#![forbid(unsafe_code)]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ais_tutorial::decode::main(&args));
}
