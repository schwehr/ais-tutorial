# AGENTS.md — repository root

An AIS/NMEA tutorial that teaches the protocol and four programming languages
(Python, C++, Rust, TypeScript) at once. Start with `README.md` for the table
of contents. Rules for anyone — human or agent — changing this repository:

| Area | Guide to read first |
|---|---|
| `chapters/` | `chapters/AUTHORING.md` (chapter skeleton, voice, citation keys in `appendix-a-references.md`) |
| `code/` | `code/CONVENTIONS.md` (the four decoders must print byte-identical JSON; module and field naming; error texts) |
| `data/`, `tests/` | `code/CONVENTIONS.md` §4 — add a `data/chNN-*.nmea` excerpt, regenerate `tests/expected/`, review the diff |
| `sample-data/` | `sample-data/AGENTS.md` — raw recordings, **not tracked by git** except the `AGENTS.md` guides; never edit, only excerpt |

Hard rules:

* No third-party dependencies in any language. No code copied from existing
  AIS/NMEA libraries.
* No absolute paths and no links to anything outside this repository in
  committed text; cite external documents by name/URL (Appendix A).
* Never refer to an "AIS transponder" (or "transponder"). An AIS unit is always
  an "AIS transceiver" (or "transceiver").
* `tests/run_tests.sh` must pass for all four languages before a commit that
  touches `code/`, `data/` or `tests/`.
* Keep data excerpts small (tens of lines) and name their source file and
  line numbers in the chapter that uses them.
