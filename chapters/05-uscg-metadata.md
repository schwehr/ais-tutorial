# 5. USCG N-AIS metadata

> **What you will learn**
> * What the US Coast Guard's Nationwide AIS (N-AIS) was, how its feed
>   reached researchers through the AISUser client, and why its lines carry
>   extra fields *after* the NMEA checksum.
> * The "USCG R&D Center format": `s` (RSSI), `d` (dBm), `T` (time of
>   arrival within the UTC minute), `S` (slot), `x` (receiver counter),
>   `r`/`b` (station) and the bare UNIX time — and the five real variants of
>   it in the sample file, from 2007 CCOM logs to the 2011 `serial_logger.py`
>   convention.
> * The two clocks on every line — the logging computer's and the
>   receiver's — how far apart they can be, and how
>   `time_of_arrival_to_utc()` combines them into a precise UTC arrival
>   time.
> * Why precise arrival time matters: slot analysis, multilateration,
>   de-duplication across receivers, receiver health.
> * `uscg.py` and its C++/Rust/TypeScript twins: prefix-letter dispatch,
>   "keep what you cannot parse", and integer-vs-fractional time handling.

## 5.1 Where this fits

### N-AIS and how its data got to a laptop in New Hampshire

In the mid-2000s the US Coast Guard built a **Nationwide Automatic
Identification System (N-AIS)**: shore receivers (later transceivers)
along the coasts and inland waterways, feeding a central server. The
Research & Development Center (R&DC) in Groton, Connecticut ran the early
network and wrote the software that distributed it. A research group did
not get a radio feed; it got a TCP connection. The client program was
**AISUser**: it authenticated to an "AIS server" with a registered
`Client.Name`, requested regions, received the (compressed) stream, and
re-emitted it on local "monitor" ports for whatever program wanted it
[AISUser §2], [AISUser §3.6.3]. The configuration file preserved in
`sample-data/2010-deepwater-horizon/scripts/` shows the CCOM machine
talking to `nais.uscg.mil` on port 31415 (mirror `naisdr.uscg.mil`),
with `Monitor.Connection.1.OutputFormat=2` — the format this chapter is
about.

AISUser offered three output formats [AISUser §3.6.2.8]:

| `OutputFormat` | Meaning |
|---|---|
| 0 | Pass the feed through unchanged |
| 1 | NMEA 0183 v4.00 TAG block (chapter 4) |
| 2 | "Proprietary USCG R&D Center format (based on NMEA 0183 v3.01)" — metadata **appended after the checksum** |

Format 2 predates TAG blocks and is what every N-AIS capture in
`sample-data/` looks like. GPSD's documentation calls it "U.S. Coast Guard
Extended AIVDM" and describes it as "semi-obsolescent … never documented
well" [AIVDM "U.S. Coast Guard Extended AIVDM"]. That is fair: no
specification was ever published. What follows is assembled from the GPSD
text, the AISUser manual, and the logs themselves.

The idea spread beyond the USCG. Kurt Schwehr's `serial_logger.py`, which
recorded most of `sample-data/` from receivers on lighthouses and research
vessels, has a `--uscg-format` option that appends `r<station>,<epoch>` to
every sentence; older runs appended only the epoch (`sample-data/AGENTS.md`).
So the same parser must cope with six fields from a USCG receiver and one
field from a 2007 laptop.

### Operational use

The Coast Guard used the metadata for **network operations**: which
receiver heard a message (`r`), how strongly (`s`/`d`), when (`T`, time),
and whether the receiver was keeping up (`x`). For a watchstander looking
at a chart none of this is visible; the chart shows the ship. For the
people keeping two hundred receivers alive it was the whole picture: a
receiver whose dBm distribution drifts down by 10 dB has a wet connector;
a receiver whose `x` stops incrementing is dead; a receiver whose `T`
disagrees with its neighbours has lost GPS time.

### Unattended / automated uses

* **Receiver health monitoring.** Daily histograms of `d` per station;
  the median and the 95th percentile are the antenna/feeder health
  indicators. A sudden gain of the noise floor (more messages at −110 dBm
  that fail to decode) points at a new interferer.
* **Coverage maps.** Join each decoded position (chapter 3) with the `d`
  of the receiver that heard it and grid the result: a measured, not
  modelled, coverage map per station, refreshed every day from traffic
  that is there anyway.
* **De-duplication.** N-AIS merged all receivers into one stream, so a
  ship in Massachusetts Bay appears once per receiver that heard it —
  three times in the sample file (§5.5). A de-duplicator keys on
  (payload, precise arrival time rounded to the slot) and keeps the copy
  with the best `d`.
* **Timing-based anomaly and spoof detection.** A transmitter's reported
  `timestamp` field [M.1371-5 Annex 8 §3.1 Table 48] and the slot number in
  its self-organising TDMA (SOTDMA) communication state
  [M.1371-5 Annex 2 §3.3.7.2.3 Table 19]
  must agree with the receiver's `T`. A position that moves 20 km between
  two messages 26 ms apart, or a "ship" whose messages arrive at
  implausible slot times, is a candidate spoof. With `T` from three
  receivers whose positions are known, the differences in arrival time
  give the transmitter's position independently of what it claims
  (multilateration; 1 ms of time difference is 300 km of path difference,
  so sub-microsecond timing is needed for useful fixes — `T` as logged is
  not that good, but it shows the principle).
* **Slot-map reconstruction.** `T` (or `S`, where present) tells you which
  of the 2 250 slots per minute [M.1371-5 Annex 2 §3.1.2] a message
  occupied — the input for channel-loading studies and for finding
  stations that transmit in slots they should not.

## 5.2 The specification

### 5.2.1 Field table

There is no bit layout here: everything is text after the sentence's
`*hh`, comma-separated, each item tagged by its first character. Field
order is **not fixed** and every field except the final time is optional.

| Prefix | Field | Type / range | Meaning | Present in our data | Source |
|---|---|---|---|---|---|
| `s` | RSSI | unsigned integer 0–65535 | Received signal strength indicator in the receiver's own units; one of the inputs to `d`. Only when the receiver provides it. | lines 1–3, 6–9 | [AIVDM "U.S. Coast Guard Extended AIVDM"] |
| `d` | signal strength | signed integer, dBm | Received power in dBm (negative). Only when the receiver provides it. | lines 1–3, 6, 7 | [AIVDM "U.S. Coast Guard Extended AIVDM"] |
| `T` | time of arrival | decimal seconds, 0 ≤ T < 60 | Seconds since the start of the current UTC minute, by the receiver's clock. Only when the receiver provides it. | lines 1–3, 6, 7 | [AIVDM "U.S. Coast Guard Extended AIVDM"] |
| `S` | slot | unsigned integer 0–2249 | Slot in which the message was received. Documented, but **absent from every file in `sample-data/`**. | — | [AIVDM "U.S. Coast Guard Extended AIVDM"] |
| `x` | sentence counter | unsigned integer | A running count of sentences emitted by this receiver. | lines 1–7 | observed in the logs; not described by [AIVDM] |
| `r` | receiving station | string, usually 9 digits | Identifier of the station that received the message. "Always provided." | lines 1–9, 11 | [AIVDM "U.S. Coast Guard Extended AIVDM"] |
| `b` | base station | string | Same as `r`, used when the receiving station is a base station. | — | [AIVDM "U.S. Coast Guard Extended AIVDM"] |
| *(none)* | UNIX time | integer or decimal seconds since 1970-01-01 UTC | "Time tag based on the standard C time function" — the logging computer's clock. "Always provided." Last field. | all lines | [AIVDM "U.S. Coast Guard Extended AIVDM"] |

Our JSON names: `rssi`, `signal_dbm`, `time_of_arrival_s`, `slot`,
`sentence_counter`, `station` + `station_kind` (`"receiver"` for `r`,
`"base"` for `b`), `time`. Anything that fits none of the rules is kept
verbatim in an `unparsed` list rather than dropped.

### 5.2.2 The five variants in the sample file

`data/ch05-uscg-metadata.nmea` was chosen to show every shape the parser
must accept. Sources are the raw recordings described in
`sample-data/AGENTS.md`:

| data line(s) | shape after `*hh` | source |
|---|---|---|
| 1–3 | `,s…,d…,T…,x…,r003669946,<int>` — the full N-AIS set | `sample-data/RightwhaleAisProject/20081013.log` lines 4–6 (13 Oct 2008, N-AIS regional feed, Boston) |
| 4–5 | `,x…,r003669947,<int>` — a receiver that sends **no `s`/`d`/`T`** | same file, lines 7–8 |
| 6 | full set from a third receiver, `r003669945`, with a log time **4 s earlier** than lines 1–5 | same file, line 9 |
| 7 | full set; the sentence is a type 5 that a collection tool **stitched** into one over-long sentence | `sample-data/sbnms/2008/data/200801-5.ais.unique` line 1 (Jan 2008) |
| 8–9 | `,s…,r993689982,<int>` — 2007 N-AIS: RSSI, station and time only, on both fragments of a type 19 | `sample-data/AISmisc/classb.uscg.19.nmea` lines 3–4 (3 Dec 2007) |
| 10 | `,<decimal>` — a 2007 CCOM log with **only** the time | `sample-data/2007hydro/norfolk-log.ais` line 2 (14 May 2007, Norfolk VA) |
| 11 | `,rnhcml,<decimal>` — the `serial_logger.py --uscg-format` convention: station name and fractional time | `sample-data/tide3/ais/ais-2011-05-21` line 8 (UNH Coastal Marine Lab) |

Two things to notice before any code. The station ids `003669945`–`947`
look like maritime mobile service identities (MMSIs) — specifically like
coast-station MMSIs, which have the
form `00MIDxxxx` with MID 366 = United States [M.585-9 Annex 1 Section 2]
— and `993689982` looks like an aid-to-navigation (AtoN) format MMSI. But
these were
receive-only sites; the ids were **never transmitted on the air**. They are
network labels borrowed from the MMSI number space, nothing more. And
`nhcml` is not numeric at all: the parser must treat `r…` as an opaque
string.

### 5.2.3 Two clocks on every line

Each N-AIS line carries **two** times from **two** clocks:

* The bare UNIX time at the end: the clock of the computer that stamped
  the line — the receiver site's PC, or a server upstream. Integer seconds
  in the USCG feed; fractional in `serial_logger.py` files. It is late by
  whatever the serial port, the network and the software added, and it is
  only as right as that computer's clock.
* `T`: the receiver's own measurement of when the message *arrived*,
  as seconds into the UTC minute, typically from the receiver's GPS
  timebase. It is precise but it does not say *which* minute.

Lines 1, 4 and 6 are the same message (payload
`18153:h02BruMNFHJPhhe0Th0D05`) heard by three receivers:

| line | station | log time | `T` |
|---|---|---|---|
| 1 | `003669946` | 1223901985 (12:46:25 UTC) | 25.49701619 |
| 4 | `003669947` | 1223901985 (12:46:25 UTC) | *(none)* |
| 6 | `003669945` | 1223901981 (12:46:**21** UTC) | 25.49717248 |

The two receivers that report `T` agree to **0.16 ms** — radio
propagation over the extra path to the farther receiver plus receiver
processing — while their logging computers disagree by **4 s**. One
message cannot arrive at 12:46:21 at one site and 12:46:25.497 at another
a few tens of kilometres away; the `T` values say it arrived at :25.497 at both, so the
`003669945` logger's clock was about 4.5 s slow (or its pipeline 4.5 s
long). The AIS message itself says the position was fixed at UTC second
24 (`timestamp` in the decoded `ais` object, chapter 3), 1.5 s before
arrival — a normal gap between fix and slot.

`time_of_arrival_to_utc(unix_time, toa_s)` reconstructs a precise arrival
time from the two:

1. Find the start of the minute containing the log time:
   `minute_start = unix_time − (unix_time mod 60)`.
2. Form three candidates: `minute_start + T`, and the same for the
   previous and the next minute.
3. Keep the candidate closest to the log time. If even that is more than
   30 s away, give up (`None`) — the two clocks disagree too much to trust
   either.

Worked for line 6: 1223901981 mod 60 = 21, so `minute_start` =
1223901960; candidates 1223901925.497, 1223901985.497, 1223902045.497;
distances from the log time 55.5 s, 4.5 s, 64.5 s; result
**1223901985.497**. For line 1: 1223901985 mod 60 = 25, same
`minute_start`, result **1223901985.497** — identical to the millisecond,
which is what a good timestamp for de-duplication should look like. The
±60 s candidates handle the case that matters most in practice: a log time
of `…:01` with a `T` of 59.8 belongs to the *previous* minute.

## 5.3 Decoding one line by hand

Line 1 of `data/ch05-uscg-metadata.nmea`:

```text
!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,s34486,d-074,T25.49701619,x78124,r003669946,1223901985
```

**Step 1 — the sentence.** Chapter 1's `parse_sentence` partitions at the
first `*`: the body `AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0`, the two
hex digits `02`, and — new here — everything after the next comma,
split into `after_checksum = ["s34486", "d-074", "T25.49701619", "x78124",
"r003669946", "1223901985"]`. The checksum is verified over the body only;
the extras are not covered by it.

**Step 2 — dispatch on the first character.**

| item | first char | rest | rule | result |
|---|---|---|---|---|
| `s34486` | `s` | `34486` | digits (after stripping any `-`) → integer | `rssi: 34486` |
| `d-074` | `d` | `-074` | digits after stripping `-` → integer; leading zero is just formatting | `signal_dbm: -74` |
| `T25.49701619` | `T` | `25.49701619` | decimal → float, printed with 8 decimals | `time_of_arrival_s: 25.49701619` |
| `x78124` | `x` | `78124` | digits → integer | `sentence_counter: 78124` |
| `r003669946` | `r` | `003669946` | non-empty → string (leading zeros kept!) | `station: "003669946"`, `station_kind: "receiver"` |
| `1223901985` | `1` | — | not a known prefix; all digits with at most one `.` → UNIX time | `time: 1223901985` (integer, because no `.`) |

**Step 3 — the precise arrival time** (not part of the JSON, see §5.2.3):
1223901985 mod 60 = 25 → `minute_start` 1223901960 → 1223901960 + 25.497
= 1223901985.497, i.e. 2008-10-13 12:46:25.497 UTC. The sentence's `d-074`
is the strongest signal in the file; this receiver was closest to the ship.

Why keep the station as a string: `int("003669946")` would print back as
`3669946` and never match the id in another log again.

## 5.4 The code

The module has two functions. `parse_uscg_fields(extras)` turns the list
of after-checksum items into the JSON sub-object; it is called by
`decode.*` whenever `parse_sentence` returned a non-empty
`after_checksum`, **before** the checksum verdict is examined, so that a
line with a bad sentence checksum still reports its metadata (line 7).
`time_of_arrival_to_utc(unix_time, toa_s)` is the clock-combination
helper of §5.2.3; the driver does not call it (the JSON reports the raw
fields), but the exercises do.

### Python

```python
def parse_uscg_fields(extras: list[str]) -> dict:
    out: dict = {}
    unparsed: list[str] = []
    for item in extras:
        if not item:
            continue
        code, rest = item[0], item[1:]
        try:
            if code == "s" and rest.lstrip("-").isdigit():
                out["rssi"] = int(rest)
            elif code == "d" and rest.lstrip("-").isdigit():
                out["signal_dbm"] = int(rest)
            elif code == "T":
                out["time_of_arrival_s"] = Fixed(float(rest), 8)
            elif code == "S" and rest.isdigit():
                out["slot"] = int(rest)
            elif code == "x" and rest.isdigit():
                out["sentence_counter"] = int(rest)
            elif code in ("r", "b") and rest:
                out["station"] = rest
                out["station_kind"] = "base" if code == "b" else "receiver"
            elif item.replace(".", "", 1).isdigit():
                # A bare number is the UNIX receive time.
                out["time"] = Fixed(float(item), 2) if "." in item else int(item)
            else:
                unparsed.append(item)
        except ValueError:
            unparsed.append(item)
    if unparsed:
        out["unparsed"] = unparsed
    return out


def time_of_arrival_to_utc(unix_time: int, toa_s: Optional[float]) -> Optional[Fixed]:
    if toa_s is None:
        return None
    minute_start = unix_time - (unix_time % 60)
    candidates = [minute_start + k + toa_s for k in (-60, 0, 60)]
    precise = min(candidates, key=lambda t: abs(t - unix_time))
    if abs(precise - unix_time) > 30:
        return None
    return Fixed(precise, 3)
```

> **Language notes (Python)**
> * `item[0], item[1:]` — one character and the rest. For this
>   single-letter-prefix format that *is* the key:value split; there is no
>   separator to `partition` on.
> * `rest.lstrip("-").isdigit()` strips *all* leading minus signs and then
>   asks "only digits?". `int("--5")` would then raise `ValueError`, which
>   the surrounding `try` turns into an `unparsed` entry. The pattern
>   "validate loosely, convert strictly, catch the exception" is idiomatic
>   Python; the other three languages have no exception to catch and
>   validate up front instead.
> * **Integer vs float time.** `int(item)` when there is no `.`,
>   `Fixed(float(item), 2)` when there is one. Python's `int` is arbitrary
>   precision and prints exactly; a `float` prints as whatever `repr`
>   decides, which is why the JSON writer insists on `Fixed` with an
>   explicit decimal count (chapter 1). The result is that the same field
>   prints as `1223901985` on one line and `1179170462.87` on another —
>   faithful to the input.
> * `min(candidates, key=…)` returns the *first* minimum on ties. The C++,
>   Rust and TypeScript versions reproduce that with a strict `<`.
> * `unix_time % 60` is never negative in Python for a positive divisor;
>   C++ and JavaScript differ (see below).

### C++

```cpp
inline Json parse_uscg_fields(const std::vector<std::string>& extras) {
    Json out = Json::object();
    Json::Array unparsed;
    for (const std::string& item : extras) {
        if (item.empty()) continue;
        char code = item[0];
        std::string rest = item.substr(1);
        bool handled = false;   // set to true once a branch consumed the item
        if (code == 's' && py::isdigit(py::lstrip(rest, '-'))) {
            if (auto v = py::to_int(rest)) { out["rssi"] = *v; handled = true; }
        } else if (code == 'd' && py::isdigit(py::lstrip(rest, '-'))) {
            if (auto v = py::to_int(rest)) { out["signal_dbm"] = *v; handled = true; }
        } else if (code == 'T') {
            if (auto v = py::to_float(rest)) { out["time_of_arrival_s"] = Json::fixed(*v, 8); handled = true; }
        } else if (code == 'S' && py::isdigit(rest)) {
            if (auto v = py::to_int(rest)) { out["slot"] = *v; handled = true; }
        } else if (code == 'x' && py::isdigit(rest)) {
            if (auto v = py::to_int(rest)) { out["sentence_counter"] = *v; handled = true; }
        } else if ((code == 'r' || code == 'b') && !rest.empty()) {
            out["station"] = rest;
            out["station_kind"] = (code == 'b') ? "base" : "receiver";
            handled = true;
        } else {
            std::string without_dot = item;
            std::size_t dot = without_dot.find('.');
            if (dot != std::string::npos) without_dot.erase(dot, 1);
            if (py::isdigit(without_dot)) {
                if (dot != std::string::npos) {
                    if (auto v = py::to_float(item)) { out["time"] = Json::fixed(*v, 2); handled = true; }
                } else {
                    if (auto v = py::to_int(item)) { out["time"] = *v; handled = true; }
                }
            }
        }
        if (!handled) unparsed.emplace_back(item);
    }
    if (!unparsed.empty()) out["unparsed"] = unparsed;
    return out;
}

inline Json time_of_arrival_to_utc(long long unix_time, std::optional<double> toa_s) {
    if (!toa_s) return Json();
    // Python's % is never negative for a positive divisor; C++'s can be, so
    // normalise (only matters for times before 1970).
    long long minute_start = unix_time - (((unix_time % 60) + 60) % 60);
    double best = 0.0;
    double best_distance = -1.0;
    for (int k : {-60, 0, 60}) {
        double candidate = static_cast<double>(minute_start + k) + *toa_s;
        double distance = std::abs(candidate - static_cast<double>(unix_time));
        if (best_distance < 0 || distance < best_distance) {   // Python's min() keeps the first minimum
            best = candidate;
            best_distance = distance;
        }
    }
    if (best_distance > 30) return Json();
    return Json::fixed(best, 3);
}
```

> **Language notes (C++)**
> * `item[0]` is a `char`, compared with single-quoted character literals
>   (`code == 's'`). `item.substr(1)` is Python's `item[1:]`.
> * Every conversion returns `std::optional`; `if (auto v = py::to_int(rest))`
>   declares `v` and tests it in one go. The `handled` flag replaces
>   Python's `try/except`: an item is `unparsed` unless some branch
>   explicitly claimed it. Both designs are fine; optionals are preferred
>   in C++ when failure is routine rather than exceptional.
> * **Integer vs float.** `long long` for whole-second times (64-bit;
>   `int` is only guaranteed 16 bits and is 32 on common platforms, which
>   overflows in 2038) and `double` for fractional ones. The two JSON paths
>   `out["time"] = *v` (integer) and `Json::fixed(*v, 2)` (two decimals) are
>   chosen by the presence of a `.`, exactly as in Python.
> * `%` on a negative `long long` yields a negative remainder in C++; the
>   `((x % 60) + 60) % 60` idiom makes it non-negative. Irrelevant for
>   times after 1970, but the habit is worth having.
> * `for (int k : {-60, 0, 60})` iterates an initializer list — the C++
>   way to loop over three literal values.

### Rust

```rust
pub fn parse_uscg_fields(extras: &[String]) -> Obj {
    let mut out = Obj::new();
    let mut unparsed: Vec<Json> = Vec::new();
    for item in extras {
        let Some(first) = item.chars().next() else {
            continue; // empty item
        };
        let (code, rest) = item.split_at(first.len_utf8());
        let parsed = match code {
            "s" if is_signed_digits(rest) => py_int_radix(rest, 10).map(|v| put(&mut out, "rssi", v)),
            "d" if is_signed_digits(rest) => {
                py_int_radix(rest, 10).map(|v| put(&mut out, "signal_dbm", v))
            }
            "T" => py_float(rest)
                .ok()
                .map(|v| put(&mut out, "time_of_arrival_s", Json::fixed(v, 8))),
            "S" if is_ascii_digits(rest) => py_int_radix(rest, 10).map(|v| put(&mut out, "slot", v)),
            "x" if is_ascii_digits(rest) => {
                py_int_radix(rest, 10).map(|v| put(&mut out, "sentence_counter", v))
            }
            "r" | "b" if !rest.is_empty() => {
                put(&mut out, "station", rest);
                put(&mut out, "station_kind", if code == "b" { "base" } else { "receiver" });
                Some(())
            }
            _ => parse_bare_number(item).map(|v| put(&mut out, "time", v)),
        };
        if parsed.is_none() {
            unparsed.push(Json::from(item.as_str()));
        }
    }
    if !unparsed.is_empty() {
        put(&mut out, "unparsed", unparsed);
    }
    out
}

fn parse_bare_number(item: &str) -> Option<Json> {
    if !is_ascii_digits(&item.replacen('.', "", 1)) {
        return None;
    }
    if item.contains('.') {
        py_float(item).ok().map(|v| Json::fixed(v, 2))
    } else {
        py_int_radix(item, 10).map(Json::Int)
    }
}

pub fn time_of_arrival_to_utc(unix_time: i64, toa_s: Option<f64>) -> Option<Json> {
    let toa_s = toa_s?;
    let minute_start = (unix_time - unix_time.rem_euclid(60)) as f64;
    let unix = unix_time as f64;
    let mut precise = minute_start - 60.0 + toa_s;
    for k in [0.0, 60.0] {
        let candidate = minute_start + k + toa_s;
        if (candidate - unix).abs() < (precise - unix).abs() {
            precise = candidate;
        }
    }
    if (precise - unix).abs() > 30.0 {
        return None;
    }
    Some(Json::fixed(precise, 3))
}
```

> **Language notes (Rust)**
> * Strings are UTF-8 and indexing by byte can split a character, so the
>   first character comes from `item.chars().next()` (an `Option<char>`),
>   and `split_at(first.len_utf8())` cuts at a character boundary. The
>   data here is ASCII, but the compiler does not know that.
> * `match code { "s" if cond => …, "r" | "b" if cond => …, _ => … }` —
>   string patterns, `|` alternatives and *match guards* (`if cond`). A
>   guard that fails falls through to the next arm, so an `s--5` item ends
>   up in the `_` arm and then in `unparsed`, like Python's `except`.
> * Each arm evaluates to `Option<()>`: `Some(())` when something was
>   recorded, `None` otherwise. `py_int_radix(rest, 10).map(|v| put(…))`
>   runs the closure only on success and yields `Option<()>` — the
>   combinator form of "if it parsed, store it". `.ok()` turns a `Result`
>   into an `Option` first when the helper returns `Result`.
> * **Integer vs float.** `i64` for whole seconds, `f64` once a fraction is
>   involved; the conversion is explicit (`as f64`) — Rust never promotes
>   silently. `rem_euclid(60)` is the always-non-negative remainder, the
>   one-call equivalent of C++'s `((x % 60) + 60) % 60`.
> * `toa_s?` on an `Option` inside a function returning `Option` is the
>   early `return None`.

### TypeScript

```typescript
export function parseUscgFields(extras: readonly string[]): JsonObject {
  const out: JsonObject = {};
  const unparsed: string[] = [];
  for (const item of extras) {
    if (!item) {
      continue;
    }
    const code = item[0];
    const rest = item.slice(1);
    try {
      if (code === "s" && isDigits(rest.replace(/^-+/, ""))) {
        out["rssi"] = pyInt(rest);
      } else if (code === "d" && isDigits(rest.replace(/^-+/, ""))) {
        out["signal_dbm"] = pyInt(rest);
      } else if (code === "T") {
        out["time_of_arrival_s"] = new Fixed(pyFloat(rest), 8);
      } else if (code === "S" && isDigits(rest)) {
        out["slot"] = pyInt(rest);
      } else if (code === "x" && isDigits(rest)) {
        out["sentence_counter"] = pyInt(rest);
      } else if ((code === "r" || code === "b") && rest) {
        out["station"] = rest;
        out["station_kind"] = code === "b" ? "base" : "receiver";
      } else if (isDigits(item.replace(".", ""))) { // replace() without /g replaces the first only
        out["time"] = item.includes(".") ? new Fixed(pyFloat(item), 2) : pyInt(item);
      } else {
        unparsed.push(item);
      }
    } catch (e) {
      if (!(e instanceof ValueError)) {
        throw e;
      }
      unparsed.push(item);
    }
  }
  if (unparsed.length > 0) {
    out["unparsed"] = unparsed;
  }
  return out;
}

export function timeOfArrivalToUtc(unixTime: number, toaS: number | null): Fixed | null {
  if (toaS === null) {
    return null;
  }
  const minuteStart = unixTime - (unixTime % 60);
  const candidates = [-60, 0, 60].map((k) => minuteStart + k + toaS);
  let precise = candidates[0];
  for (const t of candidates) {
    if (Math.abs(t - unixTime) < Math.abs(precise - unixTime)) {
      precise = t; // like Python's min(key=...): the first minimum wins
    }
  }
  if (Math.abs(precise - unixTime) > 30) {
    return null;
  }
  return new Fixed(precise, 3);
}
```

> **Language notes (TypeScript)**
> * `try { … } catch (e) { … }` catches *everything*, typed `unknown`.
>   There is no `except ValueError:` filter, so the catch re-throws
>   anything that is not our `ValueError` — otherwise a genuine bug would be
>   silently filed under `unparsed`.
> * `rest.replace(/^-+/, "")` is a regular-expression replace (strip
>   leading minus signs); `item.replace(".", "")` with a *string* pattern
>   replaces only the first occurrence — which is exactly Python's
>   `replace(".", "", 1)` here, but is a classic surprise when you wanted
>   all of them (use `/\./g`).
> * **Integer vs float.** JavaScript has one number type, a 64-bit double.
>   Whole seconds up to 2<sup>53</sup> are exact, so `pyInt("1223901985")`
>   prints as an integer without any special handling; fractional times
>   are wrapped in `Fixed(…, 2)` because `toFixed` is the only way to get a
>   stable number of decimals. The pitfall is elsewhere: `Number("")` is
>   `0` and `Number("1e3")` is `1000`, which is why the strict helpers
>   `pyInt`/`pyFloat` in `common.ts` exist.
> * `%` on a negative number is negative in JavaScript, like C++.
> * `[-60, 0, 60].map((k) => …)` builds the candidates with an arrow
>   function; the loop that follows is the explicit "first minimum" rule.

**Where the four differ.** The dispatch is the same prefix-letter chain
everywhere. Error strategy differs — Python and TypeScript *catch*,
C++ tests *optionals*, Rust uses *match guards* and `Option` combinators.
Only the integer/float distinction needs real care: Python and Rust have
two distinct types and convert explicitly; C++ has them too but will
convert silently if you let it; TypeScript has one type and leans on the
JSON writer to print each shape correctly.

## 5.5 Sample data and results

`data/ch05-uscg-metadata.nmea`:

```text
!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,s34486,d-074,T25.49701619,x78124,r003669946,1223901985
!AIVDM,1,1,,B,15N2RMPP1cJuFalHdv1`:wvf0@=k,0*14,s27800,d-103,T25.55037535,x78125,r003669946,1223901985
!AIVDM,1,1,,A,14V1HP01BCK5EgBGJCcu=:Rl0@?2,0*5E,s26220,d-110,T25.65775774,x78126,r003669946,1223901985
!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,x104270,r003669947,1223901985
!AIVDM,1,1,,A,14V1HP01BCK5EgBGJCcu=:Rl0@?2,0*5E,x104271,r003669947,1223901985
!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,s27450,d-103,T25.49717248,x101179,r003669945,1223901981
!AIVDM,1,1,1,A,55Mq@Q02=KjmL@;?OODlE8D@U@R18DTp5DE:220lDHN<<7?UNDPSlm3kc3@H88888888880,2*5C,s26731,d-107,T54.25801979,x117264,r003669945,1199163535
!AIVDM,2,1,6,A,CWjqao;wMoB<<grwIguOUrBls;5s7quwNOKgoCLsPSR,0*64,s22983,r993689982,1196666940
!AIVDM,2,2,6,A,LumqliqdL,0*69,s22983,r993689982,1196666940
!AIVDM,1,1,,A,15N91b?P00JRlJ4E56Qe0?v20@14,0*2F,1179170462.87
!AIVDM,1,1,,B,H5MiOp1:uJ0MDhJ0<P4hhDpLE80,2*33,rnhcml,1305936051.69
```

```sh
python3 code/python/decode.py data/ch05-uscg-metadata.nmea
code/cpp/build/decode data/ch05-uscg-metadata.nmea
code/rust/target/release/decode data/ch05-uscg-metadata.nmea
node code/typescript/dist/decode.js data/ch05-uscg-metadata.nmea
```

All four print the ten objects in `tests/expected/ch05-uscg-metadata.jsonl`.
The `"uscg"` sub-object of each:

| line | `uscg` |
|---|---|
| 1 | `{"rssi":34486,"sentence_counter":78124,"signal_dbm":-74,"station":"003669946","station_kind":"receiver","time":1223901985,"time_of_arrival_s":25.49701619}` |
| 2 | `{"rssi":27800,"sentence_counter":78125,"signal_dbm":-103,"station":"003669946","station_kind":"receiver","time":1223901985,"time_of_arrival_s":25.55037535}` |
| 3 | `{"rssi":26220,"sentence_counter":78126,"signal_dbm":-110,"station":"003669946","station_kind":"receiver","time":1223901985,"time_of_arrival_s":25.65775774}` |
| 4 | `{"sentence_counter":104270,"station":"003669947","station_kind":"receiver","time":1223901985}` |
| 5 | `{"sentence_counter":104271,"station":"003669947","station_kind":"receiver","time":1223901985}` |
| 6 | `{"rssi":27450,"sentence_counter":101179,"signal_dbm":-103,"station":"003669945","station_kind":"receiver","time":1223901981,"time_of_arrival_s":25.49717248}` |
| 7 | `{"rssi":26731,"sentence_counter":117264,"signal_dbm":-107,"station":"003669945","station_kind":"receiver","time":1199163535,"time_of_arrival_s":54.25801979}` — with `"error":"checksum mismatch"` beside it |
| 9 | `{"rssi":22983,"station":"993689982","station_kind":"receiver","time":1196666940}` — the completing fragment's metadata; `"fragments":2`, `"payload_bits":312`, type 19 |
| 10 | `{"time":1179170462.87}` |
| 11 | `{"station":"nhcml","station_kind":"receiver","time":1305936051.69}` |

Line 1 in full, to show where `uscg` sits:

```json
{"ais":{"cog_available":true,"cog_deg":18.0,"comm_state":{"received_stations":5,"scheme":"SOTDMA","slot_timeout":5,"sync_state":0,"sync_state_text":"UTC direct"},"heading_available":true,"lat":42.666992,"lon":-70.459822,"mmsi":538002219,"msg_type":1,"nav_status":0,"nav_status_text":"under way using engine","position_accuracy":1,"position_available":true,"raim":false,"repeat_indicator":0,"rot_deg_per_min":0.0,"rot_raw":0,"rot_text":"not turning","sog_available":true,"sog_knots":14.6,"spare":0,"special_manoeuvre":0,"special_manoeuvre_text":"not available","timestamp":24,"true_heading":18},"channel":"A","fragments":1,"line":1,"own_ship":false,"payload_bits":168,"talker":"AI","uscg":{"rssi":34486,"sentence_counter":78124,"signal_dbm":-74,"station":"003669946","station_kind":"receiver","time":1223901985,"time_of_arrival_s":25.49701619}}
```

Things worth noticing:

* **Three receivers, one message.** Lines 1, 4 and 6 decode to the same
  `ais` object (MMSI 538002219, 42.666992 N 70.459822 W, 14.6 kn). The
  signal was −74 dBm at `003669946` and −103 dBm at `003669945`; a
  de-duplicator would keep line 1. Lines 3 and 5 are a second pair
  (MMSI 308304000, heard by `946` at −110 dBm and by `947`, which reports no
  level).
* **Receiver counters are per receiver.** `x` is 78124–78126 at `946`,
  104270–104271 at `947`, 101179 at `945`: consecutive within a station,
  unrelated between stations.
* **The receiver's `T` versus the transmitter's clock.** For line 3 the
  SOTDMA communication state (chapter 3) carries `slot_number: 962`
  [M.1371-5 Annex 2 §3.3.7.2.3 Table 19]. A slot lasts 60/2250 s = 26.67 ms
  [M.1371-5 Annex 2 §3.1.2], and 25.65775774 s / 26.67 ms = slot 962.2 —
  the receiver timed the arrival to the slot the ship said it was using.
  For line 2 the same arithmetic gives slot 958.1 against a reported
  `slot_number: 883` (a 75-slot = 2.000 s discrepancy) while its
  `timestamp` is 23 against an arrival at 25.55 s; both point the same
  way, at a transmitter whose idea of UTC lagged by about two seconds.
  Exercise 5.7.4 pursues this.
* **The fragments share their metadata.** Lines 8 and 9 carry the same
  `s22983`; the receiver measured one message and copied the result onto
  both sentences. Only the completing fragment's metadata reaches the
  output.
* **Integer and fractional times print differently** — `1223901985`
  versus `1179170462.87` — because the input was different. Both are
  seconds since 1970; a consumer should accept either.

## 5.6 Pitfalls and real-world deviations

* **Field order is not fixed.** The GPSD description says `S` "would
  appear after the checksum and before the station identifier" and shows
  `s,d,T,r,time`; our 2008 lines have `s,d,T,x,r,time`; 2007 lines have
  `s,r,time`; loggers have `r,time` or just `time`. Dispatch on the prefix
  letter, never on position. The only positional rule that has held in
  every file is "the bare number is last".
* **The station ids are MMSI-shaped but are not MMSIs on the air.**
  `003669946` is a label for a receive site. Do not join it against the
  `mmsi` column of decoded messages, and do not expect a message 4 from
  it.
* **The trailing-field logger variants.** `serial_logger.py` wrote two
  fields (`r<station>,<epoch>`) when run with `--uscg-format` and one
  (`<epoch>`) otherwise, and the oldest files put the epoch on a `#`
  comment line *before* the sentence instead (`sample-data/AGENTS.md`).
  A parser that insists on a station id will reject a third of the
  collection.
* **The receiver counter resets.** `x` is not a 16-bit counter that wraps
  at 65 535 — values above 200 000 occur. In the 13 Oct 2008 feed each of
  the three Boston receivers' counters jumps back to 1 within a few minutes
  of 04:00 UTC on 14 Oct (03:57, 04:00 and 04:02), i.e. local midnight
  (EDT): a daily reset, at
  slightly different moments per receiver. Use `x` for gap detection
  within a day, and treat a decrease as a day boundary, not as data loss.
* **Fractional vs integer times.** The USCG feed stamps whole seconds; the
  loggers stamp hundredths. Our JSON preserves the distinction
  (`1223901985` vs `1305936051.69`). Software that stores the field as an
  integer loses the fraction; software that stores it as a double prints
  `1223901985.0` and breaks string comparisons. Store what you were given
  and convert at use.
* **Precision theatre.** `T25.49701619` has eight decimals — ten
  nanoseconds. No receiver of that era timed arrivals to better than a
  slot or so; the digits are the `printf` of a double. The two receivers in
  lines 1 and 6 differ by 0.16 ms, which is a fair estimate of the real
  precision.
* **Logger clocks drift.** The 4 s offset of `003669945` in line 6 is
  typical; offsets of minutes happen when a site PC loses its time source.
  `time_of_arrival_to_utc` tolerates ±30 s and returns `None` beyond that
  — a signal to flag the receiver, not to guess.
* **Transmitters' clocks drift too.** Line 2's two-second lag between `T`
  and the ship's own `timestamp`/`slot_number` (§5.5) is the ship's
  problem, not the receiver's. Sync state 0 ("UTC direct") is what the
  transponder *claims*; `T` is what the network *measured*.
* **The stitched type 5 (line 7).** The `sbnms` extraction tool joined the
  two fragments of each message 5 into one `!AIVDM,1,1,1,A,<71 characters>,2*5C`
  sentence — 91 characters, longer than the 82-character limit
  [IEC 61162-1 §7.2], with a sequential-id field (`1`) that single-sentence
  messages normally leave empty — and kept a checksum that does not
  cover the joined body (the XOR of the stitched body is `0x49`, not
  `0x5C`). The decoder therefore reports `"error":"checksum mismatch"`,
  but — because `parse_uscg_fields` runs before the verdict — still
  reports the full metadata. If you want the ship's name out of that
  file, you must consciously decide to trust the payload despite the
  checksum — a decision for the message 5 chapter, not for the metadata
  parser.
* **`S` is documented but unseen.** None of the recordings has a slot
  field. The parser accepts it anyway; do not remove code for a field just
  because your data lacks it — the next feed may have it.
* **Metadata is outside the checksum.** A corrupted `d-1O3` (letter O)
  passes the sentence checksum and lands in `unparsed`. Always look at
  `unparsed` counts when validating a new feed.

## 5.7 Exercises

1. **Precise arrival times (Python).** Add `--toa-utc` to `decode.py`
   that, when both `time` and `time_of_arrival_s` are present, adds
   `"arrival_utc": <seconds with 3 decimals>` using
   `time_of_arrival_to_utc`. Check that lines 1 and 6 produce the same
   value (`1223901985.497`) and that line 7's arrival, `1199163534.258`,
   is 0.74 s *before* its whole-second log time — the receiver's `T`
   beats the logger's clock, as it should.
2. **De-duplication (Rust).** Write a filter that reads
   `data/ch05-uscg-metadata.nmea`, groups lines by (payload, arrival time
   rounded to 26.67 ms — use the log time when `T` is missing), and emits
   one line per group, preferring the copy with the highest `signal_dbm`.
   The 11 input lines should become 8.
3. **RSSI calibration (any language).** Lines 1–3, 6 and 7 give five
   (`rssi`, `signal_dbm`) pairs from receivers of the same type. Fit a
   straight line dBm = a·RSSI + b. How many raw units per dB? Predict the
   dBm of line 9's `s22983` (whose receiver did not report `d`). State why
   that prediction is only valid if `993689982` was the same hardware.
4. **Clock audit (TypeScript).** For every type 1/2/3 whose communication
   state carries a `slot_number` and whose metadata carries `T`, compute
   `floor(T / (60/2250))` and the difference from `slot_number`; also
   compute `T − timestamp`. Over the first few thousand lines of a feed
   like `20081013.log`, plot both per MMSI. Ships with a consistent
   non-zero offset have a clock problem; receivers with an offset for
   *every* ship have one themselves.
5. **Feed validator (C++).** Write `uscg_check.cpp` that reads a log and
   prints, per station: line count, min/max/median `signal_dbm`, number
   of `x` gaps and resets, number of lines with `unparsed`, and the
   largest |log time − reconstructed arrival| seen. Run it over
   `data/ch05-uscg-metadata.nmea`; `003669947` should show no dBm
   statistics at all.
6. **Round trip (Python).** Convert `data/ch05-uscg-metadata.nmea` to TAG
   blocks with `uscg_to_tagblock.py` (chapter 4) and back with the
   reverse converter from exercise 4.7.5. List, field by field, what the
   round trip lost. Then propose a `t:` encoding that keeps the RSSI *and*
   the dBm *and* the arrival time within 15 characters — and say what you
   had to give up.

## 5.8 References

* [AIVDM "U.S. Coast Guard Extended AIVDM"] — the only public description
  of the fields `s`, `d`, `T`, `S`, `r`/`b` and the trailing C-time.
* [AISUser §2] data flow server → AISUser → monitor ports; §3.6.2.8
  `OutputFormat` 0/1/2 (format 2 = "proprietary USCG R&D Center format
  (based on NMEA 0183 v3.01)"); §3.6.3 server connection settings.
* [M.1371-5 Annex 2 §3.1.2] frame = 1 min = 2 250 slots;
  [M.1371-5 Annex 2 §3.3.7.2.3 Table 19] SOTDMA sub-message: slot number
  (slot time-out 2, 4, 6) / received stations (3, 5, 7);
  [M.1371-5 Annex 8 §3.1 Table 48] the `timestamp` field = UTC second of
  the position fix.
* [M.585-9 Annex 1 Section 2] coast-station MMSI format `00MIDxxxx`.
* [IEC 61162-1 §7.2] 82-character sentence limit; §7.2.3 checksum.
* `sample-data/AGENTS.md`, `sample-data/RightwhaleAisProject/AGENTS.md`,
  `sample-data/sbnms/AGENTS.md`, `sample-data/AISmisc/AGENTS.md`,
  `sample-data/2007hydro/AGENTS.md`, `sample-data/tide3/AGENTS.md`,
  `sample-data/2010-deepwater-horizon/scripts/AGENTS.md` — provenance of
  each excerpt and of the AISUser INI file.
* Chapter 1 (`parse_sentence` and `after_checksum`), chapter 3 (the
  `timestamp` and `comm_state` fields used in the clock cross-checks),
  chapter 4 (TAG blocks, the format that was meant to replace this one).
