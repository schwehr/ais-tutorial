# 0. Introduction

> **What you will learn**
> * What the Automatic Identification System (AIS) is, who transmits on it
>   and what the radio link looks like.
> * The layers between a transponder's antenna and the text file on your
>   disk — radio bits, AIS messages, NMEA sentences, metadata, log files —
>   and which chapter covers each.
> * What software you need (very little) and how to run the decoders and the
>   test-suite.
> * Where the recordings used in every example came from.
> * The conventions this book uses: zero-based bit offsets, citation keys,
>   and the "Language notes" boxes that teach Python, C++, Rust and
>   TypeScript side by side.

## 0.1 What AIS is

AIS is a VHF radio data system that ships, shore stations, buoys and
aircraft use to tell each other who and where they are. Every participating
station transmits short digital *messages* — position, course, speed,
identity, destination, and a few dozen other things — and every station
within radio range receives them. The International Maritime Organization
(IMO) adopted the performance standard in 1998 with three purposes in mind:
ship-to-ship collision avoidance, giving coastal states information about a
ship and its cargo, and acting as a vessel traffic service (VTS) tool
[MSC.74(69) Annex 3 §1.2]. Since 2004 the Safety of Life at Sea (SOLAS)
convention has required AIS on all ships of 300 gross tonnage and above on
international voyages, and many national rules (in the United States,
[33 CFR 164.46]) extend carriage to smaller commercial vessels.

The technical standard is ITU-R Recommendation M.1371, now in its fifth
edition [M.1371-5]. The essentials:

* **Frequencies.** Two 25 kHz channels in the VHF maritime band are
  reserved world-wide: AIS 1 on 161.975 MHz (marine channel 87B) and AIS 2
  on 162.025 MHz (channel 88B) [M.1371-5 Annex 1 §5; Annex 2 §2.1 Table 3].
  Shipborne stations receive both channels at once and alternate their
  transmissions between them [M.1371-5 Annex 2 §2.1.4]. Two further
  channels (75 and 76) are used for long-range (satellite) AIS
  [M.1371-5 Annex 4].
* **Modulation and bit rate.** Gaussian-filtered minimum shift keying (GMSK)
  at 9 600 bit/s [M.1371-5 Annex 2 §2.1 Table 3].
* **Time division.** Each channel is divided into frames of one minute, and
  each frame into 2 250 slots of 26.67 ms [M.1371-5 Annex 2 §3.1.2]. One
  slot carries one 256-bit transmission — enough for a 168-bit position
  report plus training sequence, flags and a 16-bit frame check sequence.
  Longer messages occupy up to five consecutive slots.
* **Access scheme.** There is no master station. Each Class A transponder
  listens for a minute, builds a map of which slots are in use, and then
  announces in each of its own transmissions which slot it will use next —
  *self-organising time division multiple access*, SOTDMA
  [M.1371-5 Annex 2 §3.3.4.4]. Three other schemes share the link: ITDMA
  (incremental, for announcing changes), RATDMA (random access, for
  unscheduled messages) and FATDMA (fixed, for base stations)
  [M.1371-5 Annex 2 §3.3.4.1–3.3.4.3]; low-cost Class B "CS" transponders
  use carrier-sense CSTDMA instead [M.1371-5 Annex 7]. Chapter 3 decodes the
  SOTDMA and ITDMA "communication state" bits that make this work.
* **Reporting rate.** A Class A ship under way reports its position every
  2–10 s depending on speed and turn rate, every 3 min when moored
  [M.1371-5 Annex 1 Table 1]; Class B, aids to navigation and base stations
  have their own intervals [M.1371-5 Annex 1 Table 2]. Static and voyage
  data go out every 6 min [MSC.74(69) Annex 3 §6.2].

The stations on the link come in several kinds [M.1371-5 Annex 1 §2;
IALA G1082 §4]:

| Station | What it is | Chapters |
|---|---|---|
| **Class A** shipborne mobile | SOLAS-mandated transponder; 12.5 W; SOTDMA; messages 1/2/3, 5, … | 3, 7 |
| **Class B** shipborne mobile | Voluntary fit for small craft; "SO" (SOTDMA, 5 W) or "CS" (carrier sense, 2 W); messages 18, 19, 24 | 14 |
| **Base station** | Shore station that can control the link: slot reservation, channel management, DGNSS corrections, interrogation | 6, 11, 13 |
| **Aid to navigation (AtoN)** | Transponder on (or representing) a buoy or beacon; message 21 | 15 |
| **SAR aircraft** | Search-and-rescue aircraft; message 9 | 10 |
| **AIS-SART / MOB / EPIRB-AIS** | Distress beacons that transmit message 1 and 14 bursts with MMSI 970…, 972…, 974… | 3, 12 |
| **Repeater, limited base station** | Coverage extension; no link control | 13 |

Every station is identified by a nine-digit *Maritime Mobile Service
Identity* (MMSI) whose leading digits encode the kind of station and the
country [M.585-9]; chapter 3 covers the formats.

AIS defines 27 message types [M.1371-5 Annex 8 Table 46; IALA G1082 §3.6].
Chapters 3 and 6–16 of this book take them in the order a reader is likely
to meet them on a real receiver: position reports first, then base
stations, static data, binary messages, and the rarer types.

## 0.2 The layered picture

You will never see radio bits. What reaches your program has been through
several layers, and the first five chapters of the book peel them off one at
a time from the outside in:

```text
   ┌────────────────────────────────────────────────────────────────┐
   │ 5. Log file         # comments, one sentence per line, \r\n    │  ch. 1
   │ ┌────────────────────────────────────────────────────────────┐ │
   │ │ 4. Metadata     \s:003669947,c:1223901985*02\  (TAG block) │ │  ch. 4
   │ │                 …*5C,s1234,d-119,r003669958,1085889680     │ │  ch. 5
   │ │ ┌────────────────────────────────────────────────────────┐ │ │
   │ │ │ 3. NMEA sentence  !AIVDM,1,1,,B,177KQJ5000G?tO…,0*5C  │ │ │  ch. 1–2
   │ │ │ ┌────────────────────────────────────────────────────┐ │ │ │
   │ │ │ │ 2. Six-bit armour  '1' → 000001, '7' → 000111, …  │ │ │ │  ch. 2
   │ │ │ │ ┌────────────────────────────────────────────────┐ │ │ │ │
   │ │ │ │ │ 1. AIS message bits  000001 | 00 | 30-bit MMSI… │ │ │ │ │  ch. 3+
   │ │ │ │ └────────────────────────────────────────────────┘ │ │ │ │
   │ │ │ └────────────────────────────────────────────────────┘ │ │ │
   │ │ └────────────────────────────────────────────────────────┘ │ │
   │ └────────────────────────────────────────────────────────────┘ │
   └────────────────────────────────────────────────────────────────┘
       0. Radio: GMSK at 9 600 bit/s, 26.67 ms slots, HDLC framing, 16-bit CRC
```

* **Layer 0 — radio.** A receiver demodulates the GMSK signal, finds the
  HDLC start flag, removes bit-stuffing and checks the CRC
  [M.1371-5 Annex 2 §3.2.2]. You get the message bits only if the CRC
  passed; this book never touches this layer.
* **Layer 1 — AIS message.** A string of 168, 424 or more bits whose first
  six bits are the message type, laid out exactly as the tables in
  [M.1371-5 Annex 8] describe. Chapters 3 and 6–16 are about this layer.
* **Layer 2 — six-bit armouring.** To carry binary over a text interface,
  the receiver cuts the bits into groups of six and maps each group to a
  printable ASCII character (`0`–`W`, `` ` ``–`w`)
  [IEC 61162-1 §7.3.3; AIVDM "AIVDM/AIVDO Payload Armoring"]. Chapter 2 and
  Appendix B.
* **Layer 3 — NMEA 0183 sentence.** The armoured text is one field of an
  `!AIVDM` sentence: comma-separated fields, a two-hex-digit XOR checksum,
  carriage return and line feed, at most 82 characters, with long messages
  split over several sentences [IEC 61162-1 §7; AIVDM "AIVDM/AIVDO Sentence
  Layer"]. Chapter 1 teaches the sentence format on plain GPS sentences;
  chapter 2 adds the `VDM`/`VDO` specifics and multi-sentence assembly.
* **Layer 4 — metadata.** Who received this, when, how strongly? Two
  conventions exist: the NMEA 4.10 *TAG block* `\s:…,c:…*hh\` in front of
  the sentence [NMEA 0183 v4.10 §7; IEC 61162-1 §7.1.2] (chapter 4), and the
  older US Coast Guard practice of appending `,s…,d…,T…,r…,<epoch>` fields
  *after* the checksum [AIVDM "U.S. Coast Guard Extended AIVDM"; AISUser
  §3.6.2.8] (chapter 5). Our recordings use the second.
* **Layer 5 — log file.** A text file, one sentence per line, with `#`
  comment lines from the logger and assorted line endings. Chapter 1 §1.2.6.

Keep this picture in mind: when a decode goes wrong, the first question is
always *which layer* failed — a bad checksum (3), a missing fragment (3), an
illegal armour character (2), or a payload that is too short for its message
type (1).

## 0.3 What you need

The whole book runs with nothing but stock language toolchains. There are
**no third-party packages in any language** — not because packages are bad,
but because the point is to see every line that touches the data.

| Language | Toolchain | Check with |
|---|---|---|
| Python | **3.14 or newer**, standard library only (the code uses PEP 604 `X \| None` unions, PEP 585 built-in generics, PEP 695 `type` aliases and PEP 649 deferred annotations; if your `python3` is older, run `python3.14 …` and `PYTHON=python3.14 tests/run_tests.sh`) | `python3 --version` |
| C++ | `g++` or `clang++` with C++17; `make` | `g++ --version` |
| Rust | 1.70 or newer (`cargo`, `rustc`); no crates | `cargo --version` |
| TypeScript | `tsc` 5 or newer and `node` 18 or newer; no npm packages | `tsc --version`, `node --version` |

If you only care about one language, install only that one;
`tests/run_tests.sh python` (or `cpp`, `rust`, `typescript`) runs a single
implementation. A text editor, `git`, and a terminal complete the kit.

You do not need a radio or a receiver. Every example uses real recorded
data that is already in the repository (§0.6).

## 0.4 How the book is organised

The table of contents is in [`README.md`](../README.md). Chapters 0–5 build
the foundation — sentence format, armouring, the position report that
accounts for most traffic, and the two metadata conventions. Chapters 6–16
then take the remaining message types one family per chapter. Two
appendices hold the [citation keys](appendix-a-references.md) and the
[six-bit tables](appendix-b-six-bit-tables.md).

Every message chapter has the same shape, described in
[`chapters/AUTHORING.md`](AUTHORING.md): *where this fits* (including a
section on how unattended systems use — and misuse — the message), *the
specification* as a field table with a citation on every row, *decoding one
message by hand* with pencil and paper, *the code* in all four languages,
*sample data and results*, *pitfalls*, *exercises* and *references*. Once
you have read chapters 1–3 you can read the rest in any order.

## 0.5 How the code is organised

The decoder exists four times, in [`code/python/`](../code/python/),
[`code/cpp/`](../code/cpp/), [`code/rust/`](../code/rust/) and
[`code/typescript/`](../code/typescript/). The four trees mirror each other
module for module — `nmea`, `bits`, `vdm`, `msg_1_2_3`, `tagblock`, `uscg`,
`common`, `jsonout`, `msg_N` for each message, and a `decode` command-line
driver — so that you can keep two versions open side by side and compare
the same function in two languages. [`code/CONVENTIONS.md`](../code/CONVENTIONS.md)
is the contract between them: identical module names, identical JSON field
names, identical error texts, and **byte-identical output**.

Each `decode` program reads NMEA log lines and prints one JSON object per
relevant line:

```sh
# Python (no build step)
python3 code/python/decode.py data/ch01-gps-itrek-flight.nmea

# C++17
make -C code/cpp && code/cpp/build/decode data/ch01-gps-itrek-flight.nmea

# Rust
(cd code/rust && cargo build --release) && code/rust/target/release/decode data/ch01-gps-itrek-flight.nmea

# TypeScript
(cd code/typescript && tsc -p tsconfig.json) && node code/typescript/dist/decode.js data/ch01-gps-itrek-flight.nmea
```

The test-suite builds all four, runs each over every file in `data/`, and
diffs the output against the reviewed reference output in
`tests/expected/`:

```sh
tests/run_tests.sh              # all four languages
tests/run_tests.sh rust         # just one
```

It prints `ok <language> <file>` for each pass. Every number quoted in a
chapter's results section was copied from `tests/expected/`, so the book,
the code and the tests cannot silently drift apart.

The JSON is deliberately plain: keys sorted, no whitespace, integers as
integers, missing values as `null`, and every non-integer printed with a
fixed number of decimals so that the four languages agree to the last
character ([`code/CONVENTIONS.md`](../code/CONVENTIONS.md) §2). Chapter 1 §1.5 shows the first objects.

## 0.6 Where the data came from

All the sentences in this book are excerpts from field recordings made
between 2006 and 2011 by Kurt Schwehr and colleagues at the University of
New Hampshire Center for Coastal and Ocean Mapping (CCOM), NOAA's Stellwagen
Bank National Marine Sanctuary and Cornell's Bioacoustics Research Program:

* a Bluetooth GPS receiver carried in an office, in a car and on airline
  flights (chapter 1);
* AIS receivers in lighthouses, on research vessels and in a Norfolk hotel
  room during the US Hydro 2007 conference;
* Class A and Class B transponders, base stations and AtoN units on the
  bench, sending test messages;
* the US Coast Guard's national AIS network feed;
* the right-whale notice broadcasts (binary message 8) in the Boston
  traffic separation scheme.

The raw logs amount to about 650 MB and are **not** in this repository; a
set of guides in [`sample-data/`](../sample-data/) (one `AGENTS.md` per
directory) describes every file so that a reader with the originals can
find more examples. What *is* in the repository is [`data/`](../data/): one
small excerpt per chapter, tens of lines each, with the source file and line
numbers named in the chapter that uses it.

Almost all of the recordings were made with the same small Python serial
logger, which gives them a recognisable shape: `#` comment lines with a
start banner, then one sentence per line, each followed by two extra fields
`,r<station-id>,<unix-time>` after the checksum. Chapter 1 explains the
file format and chapter 5 the extra fields. The data are historical: MMSIs,
names and positions were real in 2006–2011 and should not be treated as
current.

## 0.7 Conventions used in this book

* **Bit offsets are zero-based.** When we say "bits 8–37 (30 bits)" we mean
  that the first bit of the field is the ninth bit of the message, exactly
  the way `Bits.uint(8, 30)` in the code takes it. The ITU tables give only
  field *lengths*; we add the offsets. All four implementations use the same
  offsets and the test-suite checks that they agree.
* **Citations** look like `[M.1371-5 §3.1 Table 48]`: a key in square
  brackets, then the section or table inside the document. The keys are
  defined in [Appendix A](appendix-a-references.md). Every statement about
  a bit layout, a unit, a sentinel value or a procedure carries one. The
  ITU, IMO, IALA and USCG documents are free downloads; IEC 61162-1 and
  NMEA 0183 are paywalled, so for the sentence layer we always add a free
  source (usually the GPSD *AIVDM* document) alongside.
* **Terminology.** A *sentence* is an NMEA 0183 line; a *message* is an AIS
  message; the *payload* is the armoured text inside a `VDM` sentence; a
  *fragment* is one sentence of a multi-sentence message; a *field* is a
  comma-separated item in a sentence or a bit range in a message.
* **Hand decodes** show the payload in groups of six bits with the armour
  character above each group, so you can follow with pencil and paper.
* **Language notes.** The first time a chapter uses a construct of one of
  the four languages — a Python `dataclass`, a C++ `std::optional`, a Rust
  `match`, a TypeScript union type — a short *Language notes* box explains
  it. Later chapters explain only what is new. If you already know a
  language, skip its boxes.
* **Results come from the tests.** Every value quoted in a "Sample data and
  results" section is copied from `tests/expected/*.jsonl`.

## 0.8 Why four languages

Because the protocol is the point and the language is not — and because
seeing the *same* thirty-line function in four languages teaches more about
each of them than four unrelated examples would. Each version is written in
the idiom its own community would recognise, not as a transliteration:

* **Python** is the reference. Functions and `dataclass`es, dictionaries
  for JSON objects, `X | None` type hints for missing values, exceptions for errors.
  It is the shortest and the one the test fixtures are generated from.
* **C++17** shows what the same design looks like with static types and no
  garbage collector: header-only modules, `struct`s as data bundles,
  `std::optional` where Python has `None`, `std::map` for objects (which
  sorts the keys for free), `std::string_view` for borrowed text, and a
  small "Python-compatibility" header so that `strip`, `split` and
  `partition` behave exactly like their Python namesakes.
* **Rust** adds ownership and exhaustive error handling: `&str` versus
  `String`, `Option` and `Result` with the `?` operator instead of
  exceptions, `match` on `enum`s that carry data, `BTreeMap` for sorted
  objects, and `#![forbid(unsafe_code)]` at the top of the crate. Where the
  Python reference would crash on a malformed number, the Rust version
  returns an error object with Python's own wording.
* **TypeScript** runs on plain `node` with `strict` typing: `class` where
  there is behaviour and `interface` where there is only data, union types
  such as `number | null`, `readonly` fields, `Map` for tables keyed by
  input text, template literals as f-strings, and a hand-rolled JSON writer
  because `JSON.stringify` neither sorts keys nor fixes decimals. It is also
  where you learn that bitwise operators are 32-bit and that `toFixed`
  rounds ties differently from C — both of which matter for AIS.

Each code directory has a `README.md` listing which language feature each
module demonstrates. Chapter 1 opens all four boxes for the first time.

## 0.9 References

* [M.1371-5] Annex 1 §2 (station types), §5 (frequency band), Tables 1–2
  (reporting intervals); Annex 2 §2.1 Table 3 (channels, bit rate, GMSK),
  §2.1.4 (dual-channel operation), §3.1.2 (frames and slots), §3.2.2 (packet
  format), §3.3.4 (access schemes); Annex 4 (long-range channels); Annex 7
  (Class B CS); Annex 8 Table 46 (message summary).
* [M.585-9] — MMSI formats.
* [MSC.74(69)] Annex 3 §1.2 (purpose), §6.1–6.2 (information and update
  rates).
* [IALA G1082] §1.2 (what AIS is), §3 (how it operates), §3.6 (messages),
  §4 (station types), §6.2 (satellite AIS).
* [33 CFR 164.46] — US carriage requirement.
* [IEC 61162-1] §7 (sentence structure), §7.1.2 (TAG blocks), §7.3.3
  (six-bit conversion).
* [NMEA 0183] v4.10 §7 (TAG blocks).
* [AIVDM] "AIVDM/AIVDO Sentence Layer", "Talker IDs", "AIVDM/AIVDO Payload
  Armoring", "U.S. Coast Guard Extended AIVDM".
* [AISUser] §3.6.2.8 — USCG output formats.
