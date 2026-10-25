# 6. Messages 4 and 11: base station report and UTC/date response

> **What you will learn**
>
> * What an AIS *base station* is, why it broadcasts its clock and position
>   every ten seconds, and how mobile stations use that broadcast.
> * The 168-bit layout of message 4 — six UTC date/time fields, position,
>   EPFD type, the long-range transmission-control bit, RAIM and a SOTDMA
>   communication state — and why message 11 is the same layout with a
>   different message ID.
> * How to reuse the SOTDMA decoder from chapter 3 across modules in all
>   four languages, and how each language builds an optional ISO-8601
>   timestamp from six integers.
> * How to use base-station reports as a free clock reference: checking
>   receiver and logger clocks, spotting stations whose clocks are wrong,
>   and spotting stations that are not where (or who) they say they are.
> * What real base stations get wrong: 120-second clock offsets, wandering
>   "fixed" positions, non-standard MMSIs and comm-state fields filled with
>   nonsense.

## 6.1 Where this fits

Everything in chapter 3 was transmitted by ships. This chapter is about
the shore. An **AIS base station** is a fixed, land-based transceiver run
by a competent authority (a coast guard, a port, a vessel traffic service).
It listens to the VHF data link (VDL) like any receiver, but it also
transmits, and the one message it transmits continuously is **message 4,
the base station report**: "I am station *N*, the UTC date and time is
*T*, and I am at *P*" [M.1371-5 Annex 8 §3.2 Table 51]. A base station
"should normally transmit the base station report (Message 4) with a
minimum reporting interval of 10 s" [M.1371-5 Annex 2 §3.1.3.3.1]; the
nominal interval in the reporting-interval table is 10 s, dropping to
3 1/3 s "after the station detects that one or more stations are
synchronizing to the base station" [M.1371-5 Annex 1 §4.2.1 Table 2
note 1].

Message 4 exists for three reasons that the specification spells out:

1. **Synchronisation.** The TDMA scheme divides each UTC minute into 2 250
   slots [M.1371-5 Annex 2 §3.1.2]; every station must agree on where the
   minute starts. Stations with their own GNSS receiver are *UTC direct*
   (sync state 0). A station that has lost UTC should synchronise to
   stations that have it (*UTC indirect*, sync state 1), and failing that
   "should synchronize to the base station which indicates the highest
   number of received stations, provided that two reports have been
   received from that station in the last 40 s" (sync state 2, *base
   direct*) [M.1371-5 Annex 2 §3.1.1.1–§3.1.1.3]. Message 4 carries the
   number of received stations in its SOTDMA (self-organising TDMA,
   chapter 3) sub-message when the slot
   time-out is 3, 5 or 7 [M.1371-5 Annex 2 §3.1.1.3; §3.3.7.2.3 Table 19].
2. **Slot reservation.** A base station reserves slots for itself with
   message 20 (chapter 13), but "FATDMA reservations should consist of a
   base station report (message 4) in conjunction with a data link
   management message with the same base station ID (MMSI)… A data link
   management message (Message 20) without a base station report
   (Message 4) should be ignored" [M.1371-5 Annex 2 §3.3.4.3.1]. The
   reservation is only valid within 120 NM of the position in message 4.
3. **Coverage and long-range control.** "Message 4 is used by AIS stations
   for determining if it is within 120 NM for response to Messages 20 and
   23" [M.1371-5 Annex 8 §3.2]. Since M.1371-4, one bit of the former spare
   field tells Class A ships inside a base-station coverage area (defined
   by message 23) whether to keep sending the satellite long-range message
   27 [M.1371-5 Annex 8 Table 51; Annex 4 §3.3.3].

**Message 11** is the same 168 bits with message ID 11 instead of 4. It
is "UTC and position response from mobile station" and "is only
transmitted as a result of a UTC request message (Message 10)… on the
channel where the UTC request message was received" [M.1371-5 Annex 8
§3.2; §3.9 simply says "For Message 11 refer to description of
Message 4"]. Message 10 (chapter 11) is 72 bits: the asking MMSI and the
asked MMSI [M.1371-5 Annex 8 §3.8 Table 60]. In practice a ship sends
message 10 when its own GNSS has stopped delivering UTC; the recordings
in this chapter show exactly that.

### Operational use

* **Bridge.** Nothing on the bridge displays message 4 directly, but the
  transceiver uses it: a Class A that loses its GNSS fix stays on the air
  by synchronising to a base station (sync state 2) and keeps its slot map
  coherent with everyone else's. The transceiver also uses the position in
  message 4 to decide whether it is inside 120 NM of a base station whose
  message 20/22/23 commands it should obey.
* **VTS and ports.** A VTS centre usually *is* the base station. Its
  message 4 is also how the VTS announces its presence on the VDL; ships
  and other authorities can see which shore stations cover an area and
  whether they are alive.
* **Receiver networks.** Any coastal receiver — including the US Coast
  Guard's N-AIS network whose metadata we met in chapter 5 — sees message 4
  from every base station in range every 10 s. Operators use that steady
  stream as a heartbeat for both the base stations and the receivers.

### Unattended / automated uses

Because message 4 is periodic, timestamped at source and sent from a
fixed point, machines can do a great deal with it without anyone
watching:

* **Clock auditing.** Every message 4 carries UTC to the second. A logger
  that stamps each line with its own clock (the trailing epoch seconds in
  all of our recordings) can be checked against the base station, and the
  base station can be checked against the receiver. In §6.6 we find a
  Norfolk base station two minutes slow, a Boston receiver five seconds
  slow, and a bench base station with no GNSS at all that is 102 s fast.
  A monitoring script that computes `receive_time − message_utc` per
  (base station, receiver) pair and alarms on a change is a dozen lines
  long and catches both broken GNSS feeds and un-synced logging PCs.
* **Coverage monitoring.** A receiver that normally hears base station *X*
  every 10 s and suddenly does not has either lost its antenna or *X* has
  gone off the air. The *absence* of message 4 is a cheap, continuous test
  of a receiver site. The Great Bay receiver (`nhjel`) in `sample-data/tide1`
  heard **no** valid message 4 at all in August 2008 (only a dozen
  corrupt fragments whose first six bits happened to spell 4) — it was
  simply out of range
  of every base station — but it heard the message 11 answers that ships
  gave to each other's UTC requests.
* **Identifying the site.** A base station's position should not move.
  Surveyed stations (EPFD type 7) report the same coordinates in every
  message; GPS-fed stations wobble by a few metres. Either way the
  coordinates identify the physical site, which lets a data miner attach a
  station name to an MMSI and detect when a transmitter has been moved.
* **Rogue and spoofed base stations.** A base station is a privileged
  transmitter: it can reserve slots (message 20), reassign channels
  (message 22), command ships (message 16, 23) and silence their long-range
  broadcasts (the bit in message 4). An attacker who can transmit message 4
  with a plausible MMSI can therefore disrupt a whole area; this is the
  threat behind IMO's resolution on protecting the VDL [MSC.140(76)].
  Automatic checks that catch the clumsy cases:
  * **MMSI (maritime mobile service identity) format.** Coast stations use
    `00MIDXXXX` — two leading zeros,
    the three-digit maritime identification digits of the administration,
    then four digits [M.585-9 Annex 1 Section 2]. The administration *may*
    use the sixth digit to distinguish kinds of station (`00MID5XXX` for
    AIS base stations), but that is "optional and should be used only as a
    guidance" [M.585-9 Annex 1 Section 2 §3–4]. The US stations in this
    chapter are `003669702`, `003669713`, `003669717`, `003669730`,
    `003669971` — all `00366xxxx`, none using the `5` convention. A message
    4 from an MMSI that does not start with `00` is wrong by construction.
    In §6.5 you will meet one (`312670`).
  * **Position.** A message 4 whose position moves more than tens of
    metres, is at sea, or is the "not available" sentinel (181°/91°) is
    either broken or not a real base station.
  * **Timing.** Real base stations are UTC-direct (sync state 0) with
    a GNSS or an external time source. A "base station" that reports
    sync state 3 (synchronised to another mobile) or whose UTC drifts
    against every other station is suspect.
* **Time synchronisation of mobiles and of data.** Post-processing
  software that merges logs from several receivers needs a common clock.
  Message 4 heard by all of them is the natural reference: in §6.5 the
  same transmission is logged by three N-AIS receivers whose epoch stamps
  disagree by 5 s, while the precise time-of-arrival field agrees to
  13 µs. The slot number in the SOTDMA state is a second, independent
  clock: slot *n* starts *n* × 26.667 ms after the top of the UTC minute.

## 6.2 The specification

Message 4 / 11 is 168 bits [M.1371-5 Annex 8 §3.2 Table 51]. Offsets
below are zero-based; the ITU table gives only lengths.

| Bits | Len | Field | Type / units | Not available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 4 = UTC and position report from base station; 11 = UTC and position response from mobile station [M.1371-5 Table 51] |
| 6–7 | 2 | Repeat indicator | uint | — | 0 default; 3 = do not repeat any more [M.1371-5 Table 51; Annex 2 §4.6.1] |
| 8–37 | 30 | User ID | uint | — | MMSI [M.1371-5 Table 51]; for a base station the format is `00MIDXXXX` [M.585-9 Annex 1 Section 2] |
| 38–51 | 14 | UTC year | uint, 1–9999 | 0 | [M.1371-5 Table 51] |
| 52–55 | 4 | UTC month | uint, 1–12 | 0 | 13–15 not used [M.1371-5 Table 51] |
| 56–60 | 5 | UTC day | uint, 1–31 | 0 | [M.1371-5 Table 51] |
| 61–65 | 5 | UTC hour | uint, 0–23 | 24 | 25–31 not used [M.1371-5 Table 51] |
| 66–71 | 6 | UTC minute | uint, 0–59 | 60 | 61–63 not used [M.1371-5 Table 51] |
| 72–77 | 6 | UTC second | uint, 0–59 | 60 | 61–63 not used [M.1371-5 Table 51] |
| 78 | 1 | Position accuracy | flag | 0 default | 1 = high (≤ 10 m), 0 = low (> 10 m); determined per Table 50 [M.1371-5 Table 51] |
| 79–106 | 28 | Longitude | sint, 1/10 000 min | 181° = 0x6791AC0 | East positive, two's complement [M.1371-5 Table 51] |
| 107–133 | 27 | Latitude | sint, 1/10 000 min | 91° = 0x3412140 | North positive, two's complement [M.1371-5 Table 51] |
| 134–137 | 4 | Type of EPFD | enum | 0 = undefined | see table below [M.1371-5 Table 51] |
| 138 | 1 | Transmission control for long-range broadcast message | flag | 0 default | 0 = Class A stops message 27 within base-station coverage area; 1 = request Class A to transmit message 27 within the coverage area [M.1371-5 Table 51; Annex 4 §3.3.3] |
| 139–147 | 9 | Spare | uint | — | "Not used. Should be set to zero" [M.1371-5 Table 51] |
| 148 | 1 | RAIM flag | flag | 0 default | 1 = receiver autonomous integrity monitoring (RAIM) in use [M.1371-5 Table 51] |
| 149–167 | 19 | Communication state | SOTDMA | — | Always SOTDMA (sync state 2 bits, slot time-out 3 bits, sub-message 14 bits) [M.1371-5 Table 51; Annex 2 §3.3.7.2.2 Table 18, §3.3.7.2.3 Table 19] |

**Type of electronic position fixing device (EPFD)** [M.1371-5 Annex 8
Table 51] — this table is shared with messages 5, 9, 19, 21 and 24, which
is why `epfd_text` lives in the `common` module:

| Code | Meaning | Code | Meaning |
|---|---|---|---|
| 0 | undefined (default) | 6 | integrated navigation system |
| 1 | GPS | 7 | surveyed |
| 2 | GLONASS | 8 | Galileo |
| 3 | combined GPS/GLONASS | 9–14 | not used |
| 4 | Loran-C | 15 | internal GNSS |
| 5 | Chayka | | |

The table says "Use of differential corrections is defined by field
position accuracy above" — i.e. EPFD type tells you the sensor, the PA
flag tells you whether it was differentially corrected [M.1371-5 Table 51].
"Surveyed" (7) is the value a properly installed base station should use:
its antenna position was measured once and typed in, so the position
never changes and PA = 1 is justified.

**Repeat indicator** — mobile stations "should always set the repeat
indicator to default = 0"; a repeater increments it, and a base station
transmitting on behalf of another entity (an aid to navigation, AtoN, for
example) sets it
non-zero [M.1371-5 Annex 2 §4.6.1]. The L-3 bench unit in §6.5 sends its
own report with repeat indicator 3.

**The six UTC fields** replace the single 6-bit "time stamp" of the
position report. Each has its own not-available sentinel — 0 for the date
fields, 24/60/60 for the time fields — and each sentinel is also the
default [M.1371-5 Table 51]. A base station without a time source therefore sends
year 0, month 0, day 0, hour 24, minute 60, second 60. Note that the
"not used" values (month 13–15, hour 25–31, minute/second 61–63) are *not*
the not-available value; a decoder should treat them as invalid, not as
"unknown". Our `_available` booleans test only the sentinel; the
`utc_iso` field is `null` whenever any component is a sentinel *or* out
of range (§6.4).

**Position, PA, RAIM** are bit-for-bit the same as in message 1
(chapter 3): 28/27-bit two's-complement 1/10 000-minute values with the
same 181°/91° sentinels [M.1371-5 Table 51; Table 48].

**Transmission control for long-range broadcast message** appeared in
M.1371-4; in M.1371-3 and earlier bits 138–147 were a 10-bit spare, and
the GPSD description still shows them that way [AIVDM "Type 4"]. The
semantics need message 23 as well: "Control of the long range AIS
broadcast message requires the reception of both the Message 4 with the
control setting 'off' for the transmission of Message 27 and a Message 23
with the definition of the base station coverage area" [M.1371-5 Annex 4
§3.3.3]. Without a message 23 a ship ignores the bit.

**Communication state** is always SOTDMA for messages 4 and 11
[M.1371-5 Table 51]. The 14-bit sub-message means, depending on the 3-bit slot
time-out: received stations (time-out 3, 5, 7), slot number 0–2 249
(2, 4, 6), UTC hour and minute (1), or slot offset (0) [M.1371-5 Annex 2
§3.3.7.2.3 Table 19]. Chapter 3 explained these; here the interesting
observation is that for a base station transmitting every 10 s, three
*different* slots are in use (10 s = 375 slots apart), each with its own
time-out counter, so consecutive reports from one station do not count
down 7, 6, 5… (see §6.5).

## 6.3 Decoding one message by hand

Line 3 of [`data/ch06-base-station-reports.nmea`](../data/ch06-base-station-reports.nmea)
(source: `sample-data/2007hydro/va-portsmouth-sr162-2007-05-17` line 16, an
SR162 receiver at the US Hydro 2007 conference in Norfolk, Virginia):

```text
!AIVDM,1,1,,B,403OviQuMHQG9rRO9>E6fE700@5r,0*44,1179364990.14
```

One fragment, channel B, fill bits 0, so the 28 payload characters give
exactly 28 × 6 = 168 bits. The trailing `1179364990.14` is the logger's
receive time (2007-05-17 01:23:10.14 UTC). Armour decoding (chapter 2):

```text
4      0      3      O      v      i      Q      u      M      H      Q      G      9      r
000100 000000 000011 011111 111110 110001 100001 111101 011101 011000 100001 010111 001001 111010

R      O      9      >      E      6      f      E      7      0      0      @      5      r
100010 011111 001001 001110 010101 000110 101110 010101 000111 000000 000000 010000 000101 111010
```

Now walk Table 51 from bit 0:

| Bits | Binary | Value | Meaning |
|---|---|---|---|
| 0–5 | `000100` | 4 | message 4, base station report |
| 6–7 | `00` | 0 | repeat indicator 0 |
| 8–37 | `000000001101111111111011000110` | 3669702 | MMSI 003669702 — `00` + `366` (USA) + `9702`, a valid coast-station identity [M.585-9 Annex 1 Section 2] |
| 38–51 | `00011111010111` | 2007 | UTC year |
| 52–55 | `0101` | 5 | UTC month |
| 56–60 | `10001` | 17 | UTC day |
| 61–65 | `00001` | 1 | UTC hour |
| 66–71 | `010111` | 23 | UTC minute |
| 72–77 | `001001` | 9 | UTC second → 2007-05-17T01:23:09Z |
| 78 | `1` | 1 | position accuracy high (≤ 10 m) |
| 79–106 | `1101010001001111100100100111` | −45 811 417 | longitude: MSB is 1 so negative; 0xD44F927 − 2²⁸ = −45 811 417; ÷ 600 000 = **−76.352362°** (West) |
| 107–133 | `001010100011010111001010100` | 22 130 260 | latitude: ÷ 600 000 = **36.883767°** (North) |
| 134–137 | `0111` | 7 | EPFD "surveyed" |
| 138 | `0` | 0 | transmission control: Class A stops message 27 in coverage area (default) |
| 139–147 | `000000000` | 0 | spare |
| 148 | `0` | 0 | RAIM not in use |
| 149–150 | `00` | 0 | sync state: UTC direct |
| 151–153 | `100` | 4 | slot time-out 4 → sub-message is a slot number [M.1371-5 Annex 2 Table 19] |
| 154–167 | `00000101111010` | 378 | slot number 378 |

Three cross-checks that cost nothing:

* The MMSI has the coast-station shape, EPFD is "surveyed" and PA is 1:
  this looks like a properly configured base station.
* Slot 378 begins 378 × 60/2 250 = 10.08 s after the top of the minute
  [M.1371-5 Annex 2 §3.1.2]; the logger stamped the line at 01:23:10.14,
  60 ms later. The message's own UTC second is 09, one second *before*
  the slot — a pattern we see from every station in these logs (§6.6).
* 36.883767° N, 76.352362° W is on the shore of Hampton Roads, Virginia,
  which is where the receiver was.

## 6.4 The code

The module is deliberately short: everything hard — bit extraction,
two's complement, the SOTDMA sub-message — was built in chapters 2 and 3.
The new work is (a) six small integer fields with sentinels, (b) a
derived ISO-8601 string that is present only when all six are valid, and
(c) reusing `decode_sotdma` from the message 1/2/3 module instead of
copying it. The `_available` booleans test the Table 51 sentinel and
nothing else; `utc_iso` applies the full validity rule.

### Python

[`code/python/aistut/msg_4_11.py`](../code/python/aistut/msg_4_11.py):

```python
from .bits import Bits
from .common import epfd_text, require_bits
from .jsonout import Fixed
from .msg_1_2_3 import decode_sotdma

# Not-available sentinels for the six UTC fields, Table 51.
YEAR_NA = 0
MONTH_NA = 0
DAY_NA = 0
HOUR_NA = 24
MINUTE_NA = 60
SECOND_NA = 60

TRANSMISSION_CONTROL: dict[int, str] = {
    0: "Class A stops message 27 within base station coverage (default)",
    1: "Class A requested to transmit message 27 within base station coverage",
}


def utc_iso(year: int, month: int, day: int, hour: int, minute: int, second: int) -> str | None:
    """Join the six UTC fields into 'YYYY-MM-DDTHH:MM:SSZ', or None.

    None is returned when *any* component is its not-available value or
    outside its valid range (Table 51: month 13-15, hour 25-31, minute and
    second 61-63 are "not used"). We do not check the calendar (31 April
    passes) — that is left to the reader as an exercise.
    """
    if not 1 <= year <= 9999 or not 1 <= month <= 12 or not 1 <= day <= 31:
        return None
    if hour > 23 or minute > 59 or second > 59:
        return None
    return f"{year:04d}-{month:02d}-{day:02d}T{hour:02d}:{minute:02d}:{second:02d}Z"


def decode(bits: Bits) -> dict:
    """Decode a message 4 or 11. Raises DecodeError if shorter than 168 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 168)
    year = bits.uint(38, 14)
    month = bits.uint(52, 4)
    day = bits.uint(56, 5)
    hour = bits.uint(61, 5)
    minute = bits.uint(66, 6)
    second = bits.uint(72, 6)
    lon_raw = bits.sint(79, 28)
    lat_raw = bits.sint(107, 27)
    epfd = bits.uint(134, 4)
    tx_control = bits.uint(138, 1)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "utc_year": year,
        "utc_year_available": year != YEAR_NA,
        "utc_month": month,
        "utc_month_available": month != MONTH_NA,
        "utc_day": day,
        "utc_day_available": day != DAY_NA,
        "utc_hour": hour,
        "utc_hour_available": hour != HOUR_NA,
        "utc_minute": minute,
        "utc_minute_available": minute != MINUTE_NA,
        "utc_second": second,
        "utc_second_available": second != SECOND_NA,
        "utc_iso": utc_iso(year, month, day, hour, minute, second),
        "position_accuracy": bits.uint(78, 1),
        "lon": Fixed(lon_raw / 600000.0, 6),
        "lat": Fixed(lat_raw / 600000.0, 6),
        "position_available": lon_raw != 0x6791AC0 and lat_raw != 0x3412140,
        "epfd": epfd,
        "epfd_text": epfd_text(epfd),
        "transmission_control": tx_control,
        "transmission_control_text": TRANSMISSION_CONTROL[tx_control],
        "spare": bits.uint(139, 9),
        "raim": bits.flag(148),
        "comm_state": decode_sotdma(bits, 149),
    }
    return out
```

> **Language notes (Python).** `from .msg_1_2_3 import decode_sotdma`
> imports one name from a sibling module of the same package (the leading
> dot means "relative to this package"). Python has no `private` keyword;
> by convention a name without a leading underscore is public, and
> `decode_sotdma` was written as public precisely so this chapter could
> reuse it. The f-string format spec `{year:04d}` means "decimal, zero-pad
> to width 4" — the same mini-language as `format(x, '.6f')` in
> `jsonout.py`. `str | None` is the type hint for "a `str` or `None`".

### C++

[`code/cpp/msg_4_11.hpp`](../code/cpp/msg_4_11.hpp):

```cpp
#include "msg_1_2_3.hpp"

// Not-available sentinels for the six UTC fields, Table 51.
inline constexpr std::uint64_t UTC_YEAR_NA = 0;
inline constexpr std::uint64_t UTC_MONTH_NA = 0;
inline constexpr std::uint64_t UTC_DAY_NA = 0;
inline constexpr std::uint64_t UTC_HOUR_NA = 24;
inline constexpr std::uint64_t UTC_MINUTE_NA = 60;
inline constexpr std::uint64_t UTC_SECOND_NA = 60;

inline const std::array<const char*, 2> TRANSMISSION_CONTROL = {
    "Class A stops message 27 within base station coverage (default)",
    "Class A requested to transmit message 27 within base station coverage",
};

inline std::optional<std::string> utc_iso(std::uint64_t year, std::uint64_t month, std::uint64_t day,
                                          std::uint64_t hour, std::uint64_t minute, std::uint64_t second) {
    if (year < 1 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31) return std::nullopt;
    if (hour > 23 || minute > 59 || second > 59) return std::nullopt;
    return std::format("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hour, minute, second);
}

// Decode a message 4 or 11. Throws DecodeError if shorter than 168 bits.
inline Json decode_msg_4_11(const Bits& bits) {
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 168);
    std::uint64_t year = bits.uint(38, 14);
    std::uint64_t month = bits.uint(52, 4);
    std::uint64_t day = bits.uint(56, 5);
    std::uint64_t hour = bits.uint(61, 5);
    std::uint64_t minute = bits.uint(66, 6);
    std::uint64_t second = bits.uint(72, 6);
    std::int64_t lon_raw = bits.sint(79, 28);
    std::int64_t lat_raw = bits.sint(107, 27);
    std::uint64_t epfd = bits.uint(134, 4);
    std::uint64_t tx_control = bits.uint(138, 1);

    std::optional<std::string> iso = utc_iso(year, month, day, hour, minute, second);
    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"utc_year", year},
        {"utc_year_available", year != UTC_YEAR_NA},
        {"utc_month", month},
        {"utc_month_available", month != UTC_MONTH_NA},
        {"utc_day", day},
        {"utc_day_available", day != UTC_DAY_NA},
        {"utc_hour", hour},
        {"utc_hour_available", hour != UTC_HOUR_NA},
        {"utc_minute", minute},
        {"utc_minute_available", minute != UTC_MINUTE_NA},
        {"utc_second", second},
        {"utc_second_available", second != UTC_SECOND_NA},
        {"utc_iso", iso ? Json(*iso) : Json(nullptr)},
        {"position_accuracy", bits.uint(78, 1)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600000.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600000.0, 6)},
        {"position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140},
        {"epfd", epfd},
        {"epfd_text", epfd_text(static_cast<int>(epfd))},
        {"transmission_control", tx_control},
        {"transmission_control_text", TRANSMISSION_CONTROL[tx_control]},
        {"spare", bits.uint(139, 9)},
        {"raim", bits.flag(148)},
        {"comm_state", decode_sotdma(bits, 149)},
    };
    return out;
}
```

> **Language notes (C++).** Including `msg_1_2_3.hpp` from a second header
> is safe because every function in it is `inline` and the file has
> `#pragma once`: the one-definition rule allows an `inline` function to be
> defined in several translation units as long as the definitions are
> identical. `std::optional<std::string>` is "a string or nothing";
> `std::nullopt` is the nothing, `*iso` extracts the string once you have
> tested `iso` in boolean context. `Json(nullptr)` selects the
> `std::nullptr_t` constructor and prints `null`. `std::format` (C++20)
> with `{:04}` zero-pads to four digits and `{:02}` to two — the same
> format specs as the Python f-string — and it is type-safe: it takes the
> `std::uint64_t` arguments as they are, where C's `%u` would have needed a
> `static_cast<unsigned>` for each. `inline constexpr`
> at namespace scope gives a compile-time constant with a single
> definition across all includers (C++17).

### Rust

[`code/rust/src/msg_4_11.rs`](../code/rust/src/msg_4_11.rs):

```rust
use crate::bits::Bits;
use crate::common::{epfd_text, require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};
use crate::msg_1_2_3::decode_sotdma;

/// Not-available sentinels for the six UTC fields, Table 51.
pub const UTC_YEAR_NA: u64 = 0;
pub const UTC_MONTH_NA: u64 = 0;
pub const UTC_DAY_NA: u64 = 0;
pub const UTC_HOUR_NA: u64 = 24;
pub const UTC_MINUTE_NA: u64 = 60;
pub const UTC_SECOND_NA: u64 = 60;

/// Transmission control for long-range broadcast message, Table 51.
pub fn transmission_control_text(code: u64) -> &'static str {
    match code {
        0 => "Class A stops message 27 within base station coverage (default)",
        _ => "Class A requested to transmit message 27 within base station coverage", // 1
    }
}

pub fn utc_iso(year: u64, month: u64, day: u64, hour: u64, minute: u64, second: u64) -> Option<String> {
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

/// Decode a message 4 or 11. Fails with `DecodeError::TooShort` if the
/// payload has fewer than 168 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 168)?;
    let year = bits.uint(38, 14);
    let month = bits.uint(52, 4);
    let day = bits.uint(56, 5);
    let hour = bits.uint(61, 5);
    let minute = bits.uint(66, 6);
    let second = bits.uint(72, 6);
    let lon_raw = bits.sint(79, 28);
    let lat_raw = bits.sint(107, 27);
    let epfd = bits.uint(134, 4);
    let tx_control = bits.uint(138, 1);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "utc_year", year);
    put(&mut out, "utc_year_available", year != UTC_YEAR_NA);
    put(&mut out, "utc_month", month);
    put(&mut out, "utc_month_available", month != UTC_MONTH_NA);
    put(&mut out, "utc_day", day);
    put(&mut out, "utc_day_available", day != UTC_DAY_NA);
    put(&mut out, "utc_hour", hour);
    put(&mut out, "utc_hour_available", hour != UTC_HOUR_NA);
    put(&mut out, "utc_minute", minute);
    put(&mut out, "utc_minute_available", minute != UTC_MINUTE_NA);
    put(&mut out, "utc_second", second);
    put(&mut out, "utc_second_available", second != UTC_SECOND_NA);
    put(&mut out, "utc_iso", utc_iso(year, month, day, hour, minute, second));
    put(&mut out, "position_accuracy", bits.uint(78, 1));
    put(&mut out, "lon", Json::fixed(lon_raw as f64 / 600000.0, 6));
    put(&mut out, "lat", Json::fixed(lat_raw as f64 / 600000.0, 6));
    put(
        &mut out,
        "position_available",
        lon_raw != 0x6791AC0 && lat_raw != 0x3412140,
    );
    put(&mut out, "epfd", epfd);
    put(&mut out, "epfd_text", epfd_text(epfd));
    put(&mut out, "transmission_control", tx_control);
    put(&mut out, "transmission_control_text", transmission_control_text(tx_control));
    put(&mut out, "spare", bits.uint(139, 9));
    put(&mut out, "raim", bits.flag(148));
    put(&mut out, "comm_state", decode_sotdma(bits, 149));
    Ok(Json::Obj(out))
}
```

> **Language notes (Rust).** `use crate::msg_1_2_3::decode_sotdma;` works
> only because that function is `pub`; Rust privacy is per module and
> checked by the compiler, so "reuse a private helper" is not something
> you can do by accident. `utc_iso` returns `Option<String>`, and
> `jsonout.rs` has `impl<T: Into<Json>> From<Option<T>> for Json`, so
> `put(&mut out, "utc_iso", option)` prints either the string or `null`
> with no `match` at the call site. `(1..=9999).contains(&year)` is the
> idiomatic range test — `..=` is an inclusive range, and `contains` takes
> a reference, hence the `&`. `format!("{year:04}")` captures the local
> variable by name (an *inline format argument*) and zero-pads to width 4. The module also
> has `#[cfg(test)]` unit tests that decode two lines from the data file;
> run them with `cargo test` in `code/rust`.

### TypeScript

[`code/typescript/src/msg_4_11.ts`](../code/typescript/src/msg_4_11.ts):

```typescript
import { Bits } from "./bits";
import { epfdText, requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";
import { decodeSotdma } from "./msg_1_2_3";

// Not-available sentinels for the six UTC fields, Table 51.
export const UTC_YEAR_NA = 0;
export const UTC_MONTH_NA = 0;
export const UTC_DAY_NA = 0;
export const UTC_HOUR_NA = 24;
export const UTC_MINUTE_NA = 60;
export const UTC_SECOND_NA = 60;

export const TRANSMISSION_CONTROL: { readonly [code: number]: string } = {
  0: "Class A stops message 27 within base station coverage (default)",
  1: "Class A requested to transmit message 27 within base station coverage",
};

export function utcIso(
  year: number, month: number, day: number, hour: number, minute: number, second: number,
): string | null {
  if (year < 1 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31) {
    return null;
  }
  if (hour > 23 || minute > 59 || second > 59) {
    return null;
  }
  const p2 = (n: number): string => String(n).padStart(2, "0");
  return `${String(year).padStart(4, "0")}-${p2(month)}-${p2(day)}T${p2(hour)}:${p2(minute)}:${p2(second)}Z`;
}

/** Decode a message 4 or 11. Throws DecodeError if shorter than 168 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 168);
  const year = bits.uint(38, 14);
  const month = bits.uint(52, 4);
  const day = bits.uint(56, 5);
  const hour = bits.uint(61, 5);
  const minute = bits.uint(66, 6);
  const second = bits.uint(72, 6);
  const lonRaw = bits.sint(79, 28);
  const latRaw = bits.sint(107, 27);
  const epfd = bits.uint(134, 4);
  const txControl = bits.uint(138, 1);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    utc_year: year,
    utc_year_available: year !== UTC_YEAR_NA,
    utc_month: month,
    utc_month_available: month !== UTC_MONTH_NA,
    utc_day: day,
    utc_day_available: day !== UTC_DAY_NA,
    utc_hour: hour,
    utc_hour_available: hour !== UTC_HOUR_NA,
    utc_minute: minute,
    utc_minute_available: minute !== UTC_MINUTE_NA,
    utc_second: second,
    utc_second_available: second !== UTC_SECOND_NA,
    utc_iso: utcIso(year, month, day, hour, minute, second),
    position_accuracy: bits.uint(78, 1),
    lon: new Fixed(lonRaw / 600000.0, 6),
    lat: new Fixed(latRaw / 600000.0, 6),
    position_available: lonRaw !== 0x6791ac0 && latRaw !== 0x3412140,
    epfd,
    epfd_text: epfdText(epfd),
    transmission_control: txControl,
    transmission_control_text: TRANSMISSION_CONTROL[txControl],
    spare: bits.uint(139, 9),
    raim: bits.flag(148),
    comm_state: decodeSotdma(bits, 149),
  };
  return out;
}
```

> **Language notes (TypeScript).** `import { decodeSotdma } from
> "./msg_1_2_3"` imports an `export`ed function from a sibling module;
> the path is relative and written without the `.ts`/`.js` extension.
> `string | null` is the union type of chapter 1, and because `null` is
> already a member of `JsonValue`
> the object literal needs no special case. JavaScript has no `printf`;
> `String(n).padStart(2, "0")` is the zero-padding idiom, and the little
> arrow function `p2` keeps the template literal readable. `epfd,` on its
> own line is *shorthand property syntax*: `{ epfd }` means
> `{ epfd: epfd }`.

**Where the four differ.** The six UTC integers are plain `int`/`u64`/
`uint64_t`/`number` everywhere. The only real divergence is the optional
string: Python returns `None`, C++ `std::optional<std::string>` that is
unwrapped with `*iso` into a `Json`, Rust `Option<String>` that is
converted by a blanket `From<Option<T>>`, TypeScript `string | null`. The
zero-padding idiom is different in all four (`{:04d}`, `%04u`, `{:04}`,
`padStart`). Reuse of the SOTDMA decoder is spelled `from .msg_1_2_3
import`, `#include`, `use crate::`, `import { } from` — four syntaxes for
one idea.

## 6.5 Sample data and results

[`data/ch06-base-station-reports.nmea`](../data/ch06-base-station-reports.nmea)
has 25 lines, all real, from six recordings (`\r` stripped, nothing else
changed):

| Lines | Source (`sample-data/…`) | Source lines | What they show |
|---|---|---|---|
| 1–7 | `2007hydro/va-portsmouth-sr162-2007-05-17` | 8, 14, 16, 19, 27, 32, 33 | Three Hampton Roads base stations over 20 s: GPS-fed, surveyed, and one with a 2-minute clock error |
| 8–9 | `misc/rv-auk-boston-harbor-log.2008-08-20` | 6152, 13785 | Two Boston Harbor base stations heard from the R/V *Auk* |
| 10–14 | `RightwhaleAisProject/20081013.log` | 45, 48, 49, 50, 51 | The same two transmissions logged by three N-AIS receivers with full USCG metadata |
| 15–19 | `AISmisc/2008-03-27-bs-send-test/bs1-2008-03-27` | 26–30 | An L-3 base station's own `!BSVDO` message 4 on the bench with no GNSS antenna, plus its `$BSADS` status and `$BSALR` alarms |
| 20 | `AISmisc/2008-03-27-bs-send-test/sr162-2008-03-27` | 22 | The same bench station as heard by an SR162 receiver |
| 21–22 | `tide1/ais/ais-2008-08-22` | 679, 680 | A message 10 UTC inquiry and the message 11 answer 0.24 s later |
| 23–24 | `tide1/ais/ais-2008-08-21` | 372, 926 | Two more message 11 answers from different ships |
| 25 | `tide1/ais/ais-2007-02-25` | 45 | A corrupt 72-bit "message 11" from the noisy `nhjel` receiver |

Run any of the four decoders over it:

```sh
python3 code/python/decode.py data/ch06-base-station-reports.nmea
code/cpp/build/decode data/ch06-base-station-reports.nmea
code/rust/target/release/decode data/ch06-base-station-reports.nmea
node code/typescript/dist/decode.js data/ch06-base-station-reports.nmea
```

All four print the 25 lines of
[`tests/expected/ch06-base-station-reports.jsonl`](../tests/expected/ch06-base-station-reports.jsonl)
byte for byte. Selected results follow.

### Three base stations in Hampton Roads (lines 1–7)

| Line | Rx time (UTC) | Ch | MMSI | `utc_iso` | `lon` | `lat` | PA | EPFD | time-out | slot |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 01:23:00.33 | A | 3669717 | 2007-05-17T01:22:59Z | −76.325072 | 36.963052 | 0 | 1 GPS | 6 | 10 |
| 2 | 01:23:09.19 | B | 3669730 | 2007-05-17T**01:21:09**Z | −76.304695 | 36.844297 | 0 | 1 GPS | 6 | 380 |
| 3 | 01:23:10.14 | B | 3669702 | 2007-05-17T01:23:09Z | −76.352362 | 36.883767 | 1 | 7 surveyed | 4 | 378 |
| 4 | 01:23:10.33 | B | 3669717 | 2007-05-17T01:23:09Z | −76.325070 | 36.963050 | 0 | 1 GPS | 6 | 385 |
| 5 | 01:23:19.19 | A | 3669730 | 2007-05-17T**01:21:19**Z | −76.304695 | 36.844295 | 0 | 1 GPS | 6 | 755 |
| 6 | 01:23:20.14 | A | 3669702 | 2007-05-17T01:23:19Z | −76.352362 | 36.883767 | 1 | 7 surveyed | 2 | 753 |
| 7 | 01:23:20.33 | A | 3669717 | 2007-05-17T01:23:19Z | −76.325068 | 36.963047 | 0 | 1 GPS | 6 | 760 |

(The "Rx time" column is the logger's epoch stamp converted to UTC; the
other columns are the decoder's output.) Things to notice:

* **Cadence and channels.** Each station reports every 10 s and alternates
  channels A/B between consecutive reports, as periodic messages should
  [M.1371-5 Annex 2 §4.1.2]. Consecutive slots of one station are 375
  apart (10 → 385 → 760; 378 → 753): 375 slots × 26.667 ms = 10.000 s.
* **Station 3669730 is two minutes slow.** Its `utc_iso` says 01:21:09 and
  01:21:19 while the two other stations and the logger all agree on
  01:23:09–01:23:20. Its slot numbers (380, 755) are nevertheless
  consistent with the others, so its *frame* timing was fine — only the
  calendar time it writes into message 4 was off. A ship synchronising
  to this station would have kept its slots but, if it trusted the
  message for date/time, would have been two minutes wrong.
* **Surveyed vs GPS.** 3669702 (EPFD 7, PA 1) reports exactly the same
  position every time. 3669717 (EPFD 1, PA 0) wobbles in the sixth decimal
  — a few tenths of a metre per report — because it is retransmitting a live GPS
  fix instead of a surveyed antenna position.
* **The UTC second is one behind the slot.** Line 1 was sent in slot 10
  (0.27 s into the minute) and says second 59 of the previous minute;
  line 3 in slot 378 (10.08 s) says second 09. Every station in all of our
  recordings behaves this way: the time in message 4 appears to be the
  time at which the transceiver *built* the message, roughly one second
  before the slot in which it went out.

### Boston Harbor from the R/V *Auk* (lines 8–9)

```json
{"ais":{"comm_state":{"received_stations":40,"scheme":"SOTDMA","slot_timeout":7,"sync_state":0,"sync_state_text":"UTC direct"},"epfd":1,"epfd_text":"GPS","lat":42.346745,"lon":-70.954472,"mmsi":3669971,"msg_type":4,"position_accuracy":0,"position_available":true,"raim":false,"repeat_indicator":0,"spare":0,"transmission_control":0,"transmission_control_text":"Class A stops message 27 within base station coverage (default)","utc_day":20,"utc_day_available":true,"utc_hour":11,"utc_hour_available":true,"utc_iso":"2008-08-20T11:23:29Z","utc_minute":23,"utc_minute_available":true,"utc_month":8,"utc_month_available":true,"utc_second":29,"utc_second_available":true,"utc_year":2008,"utc_year_available":true},"channel":"B","fragments":1,"line":8,"own_ship":false,"payload_bits":168,"talker":"AI","uscg":{"station":"auk","station_kind":"receiver","time":1219231410.31}}
```

Station 3669971 at 42.346745° N, 70.954472° W (eastern Boston Harbor)
says it currently receives 40 other stations. Line 9 is station 3669713
at 42.368338° N, 71.053098° W (the downtown waterfront) receiving 36.
Both are UTC direct and within about a second of the *Auk*'s logging clock
(line 8 received at 11:23:30.31, message says 11:23:29). If you scan the
whole `rv-auk` log you will find that 3669971 is *only ever heard on
channel B* and *always* reports slot time-out 7 — it is running on
FATDMA-reserved slots and never counts down — whereas 3669713 alternates
A/B and cycles its time-outs like a mobile. Both behaviours are legal; a
base station's slot use is configured by its operator [M.1371-5 Annex 2
§3.3.4.3.1].

### One transmission, three receivers (lines 10–14)

Lines 10, 11 and 14 are *the same* message 4 from 3669713 — identical
payload `403OvlAuRVdfMJrgbDH?Tli008Ab` — as logged by N-AIS receivers
003669946, 003669945 and 003669947. The decoded `ais` object is identical;
the `uscg` metadata (chapter 5) is not:

| Line | `uscg.station` | `uscg.time` | `time_of_arrival_s` | `signal_dbm` |
|---|---|---|---|---|
| 10 | 003669946 | 1223901990 (12:46:30) | 30.13634059 | −102 |
| 11 | 003669945 | 1223901985 (12:46:25) | 30.13632757 | −92 |
| 14 | 003669947 | 1223901990 (12:46:30) | — | — |

The message says `"utc_iso":"2008-10-13T12:46:29Z"` and
`"slot_number":1130`. Slot 1130 starts 1130 × 26.667 ms = 30.133 s into
the minute; both receivers' time-of-arrival fields say 30.136 s, agreeing
with the slot to 3 ms and with each other to 13 µs. The whole-second
epoch stamps, however, disagree by five seconds: receiver 003669945's
logging clock was ~5 s behind the other two throughout this file, while
003669946 and 003669947 were within a second of the base station. The precise `T`
field comes from the receiver's own GNSS-disciplined clock; the epoch
stamp comes from a PC. Trust accordingly.

### A base station with no GNSS (lines 15–20)

Lines 15–19 come from the serial port of an L-3 base station on a bench
at CCOM with no GNSS antenna connected. The transceiver's own
transmissions appear as `!BSVDO` — talker `BS` for base station, `VDO`
for "own vessel" — and the driver marks them `"own_ship":true`,
`"talker":"BS"`. Line 16 decodes as:

```json
{"ais":{"comm_state":{"scheme":"SOTDMA","slot_number":750,"slot_timeout":2,"sync_state":3,"sync_state_text":"synchronized to another station"},"epfd":0,"epfd_text":"undefined","lat":91.000000,"lon":181.000000,"mmsi":312670,"msg_type":4,"position_accuracy":0,"position_available":false,"raim":false,"repeat_indicator":3,"spare":0,"transmission_control":0,"transmission_control_text":"Class A stops message 27 within base station coverage (default)","utc_day":27,"utc_day_available":true,"utc_hour":22,"utc_hour_available":true,"utc_iso":"2008-03-27T22:09:54Z","utc_minute":9,"utc_minute_available":true,"utc_month":3,"utc_month_available":true,"utc_second":54,"utc_second_available":true,"utc_year":2008,"utc_year_available":true},"channel":"A","fragments":1,"line":16,"own_ship":true,"payload_bits":168,"talker":"BS","uscg":{"station":"bs1","station_kind":"receiver","time":1206655691.67}}
```

Almost every field is instructive:

* `"position_available":false` with `lon` 181.000000 and `lat` 91.000000 —
  the not-available sentinels, exactly as Table 51 prescribes for a
  station with no fix. `"epfd":0` "undefined". The `$BSALR` on line 19,
  `AIS: No sensor position in use`, says the same thing in words (the
  `$BSALR` and `$BSADS` lines print as `"skipped":"no parser"` — alarm
  sentences are the `ALR` alarm formatter [IEC 61162-1 §8] with a `BS`
  talker, for which the driver has no parser).
* `"mmsi":312670` — a six-digit number, not a `00MIDXXXX` coast-station
  identity [M.585-9 Annex 1 Section 2]. It is whatever the unit shipped
  with. On the air this would be an obvious rogue.
* `"repeat_indicator":3` — "do not repeat any more" on an original
  transmission, which a base station should not do for its own report
  [M.1371-5 Annex 2 §4.6.1].
* `"sync_state":3`, "synchronized to another station" — a base station
  with no UTC source, admitting it. Line 17 (the next report, time-out 1)
  puts `"utc_hour":1,"utc_minute":45` in its SOTDMA sub-message while the
  message body says 22:10 — the sub-message is supposed to carry UTC
  "if the station has access to UTC" [M.1371-5 Annex 2 Table 19], and this
  station does not, so it emitted garbage there.
* **The UTC fields are populated anyway**, from a free-running clock.
  Line 15, the station's `$BSADS` status sentence, says `220947.00` while
  the logging PC stamped it 22:08:05.20; line 16 says 22:09:54 at
  22:08:11.67; line 20, the same station heard 44 minutes earlier by the
  SR162 in the next room, says 21:26:05 at 21:24:23.08. The station is a
  consistent 102 s ahead of the PC. (Which clock is wrong? The base
  station has no time source at all, so it is the prime suspect — but
  from one pair of clocks alone you cannot tell; you need a third, which
  is the point of §6.6.)

### Message 10 asked, message 11 answered (lines 21–24)

```text
!AIVDM,1,1,,B,:5Qnbf0tw9hP,0*23,rnhjel,1219390927.42
!AIVDM,1,1,,B,;3ktW21uR;7b6JstMHHbOQ100000,0*64,rnhjel,1219390927.66
```

Line 21 is a 72-bit message 10 (first character `:` = 10). Its decoder
belongs to chapter 11; the driver already prints its result,
`{"dest_mmsi":255797000,"mmsi":371043000,"msg_type":10,…}`, and you can
confirm it by reading bits 8–37 and 40–69 by hand [M.1371-5 Annex 8
Table 60]: source 371043000 asking destination 255797000. Line 22,
0.24 s later on the same channel B as the spec requires [M.1371-5 Annex 8
§3.2], is the answer:

```json
{"ais":{"comm_state":{"scheme":"SOTDMA","slot_offset":0,"slot_timeout":0,"sync_state":0,"sync_state_text":"UTC direct"},"epfd":1,"epfd_text":"GPS","lat":43.103367,"lon":-70.790967,"mmsi":255797000,"msg_type":11,"position_accuracy":0,"position_available":true,"raim":false,"repeat_indicator":0,"spare":0,"transmission_control":0,"transmission_control_text":"Class A stops message 27 within base station coverage (default)","utc_day":22,"utc_day_available":true,"utc_hour":7,"utc_hour_available":true,"utc_iso":"2008-08-22T07:42:06Z","utc_minute":42,"utc_minute_available":true,"utc_month":8,"utc_month_available":true,"utc_second":6,"utc_second_available":true,"utc_year":2008,"utc_year_available":true},"channel":"B","fragments":1,"line":22,"own_ship":false,"payload_bits":168,"talker":"AI","uscg":{"station":"nhjel","station_kind":"receiver","time":1219390927.66}}
```

Ship 255797000 (MID 255, a Portuguese-flag vessel moored in the
Piscataqua River above Portsmouth, NH) answers with its own UTC —
07:42:06 against the receiver's 07:42:07.66 — and its own GPS position.
The comm state is `slot_timeout 0, slot_offset 0`: "this was the last
transmission in this slot… If the slot offset is zero, the slot should be
de-allocated after transmission" [M.1371-5 Annex 2 Table 18, Table 19] —
exactly right for a one-off reply. Lines 23 and 24 are replies from
255797000 again (02:09:24, received 02:09:26.24) and from 353825000, a
ship offshore in the Gulf of Maine (42.941567° N, 70.650283° W, PA 1).

Why was 371043000 asking? Its own position reports in the same log (not
excerpted) show it moored with sync state 1 or 3 — it had lost UTC from
its GNSS and was polling its neighbours. In three days the `nhjel`
receiver logged 26 message 10s from this one ship and 161 message 11s;
on 20 August it logged 29 answers and no questions at all, because the
asking ship was out of the receiver's range while the answering ships
were not.

### A corrupt line (line 25)

```text
!AIVDM,1,1,,A,;P@SGS1`i30t,0*05,rnhjel,1172370520.01
```

A valid checksum, first character `;` = 11, and only 12 characters =
72 bits. The decoder reports
`"error":"message 11 too short: 72 bits < 168"` and goes on to the next
line. The `nhjel` receiver produced many such lines (chapter 2); the first
six bits happening to spell a legal message type is no evidence of
anything.

## 6.6 Pitfalls and real-world deviations

* **Clocks disagree, and message 4 shows you by how much.** Across the
  recordings used here: a Norfolk base station 120 s slow (lines 2, 5); a
  bench base station 102 s fast (lines 15–20); N-AIS receiver 003669945
  3–5 s slow for the whole of `20081013.log`, receiver 003669947 within
  2 s most of the time but ~128 s slow for short stretches (a scan of the
  first 150 000 lines; exercise 4 reproduces it). The robust procedure is
  to compute `receive_time − utc_iso` per (station, receiver) pair, look
  at the *distribution*, and attribute the offset to whichever party is
  inconsistent with everyone else. One base station against one receiver
  tells you only that *one of them* is wrong.
* **The UTC second lags the slot by about one second** in every base
  station we looked at (§6.5). Do not use the message's second to
  identify the slot; use the SOTDMA slot number or the receiver's
  time-of-arrival field.
* **"Fixed" positions that move.** GPS-fed base stations (EPFD 1, PA 0)
  wander by metres; surveyed ones (EPFD 7, PA 1) do not. If you key a
  station database on exact coordinates you will get one entry per
  report for the former. Round to ~10 m, or key on MMSI and store the
  centroid.
* **The not-available position is not an error** — it is the correct
  thing for a station without a fix to send (line 16). But a base station
  *on the air* with 181°/91° cannot anchor FATDMA reservations or the
  120 NM rule and should be flagged.
* **MMSI formats.** Real US base stations use `00366xxxx` without the
  optional `00MID5XXX` sixth-digit convention; do not reject a base
  station because digit six is not 5. Do reject one that does not start
  with `00`.
* **Bits 138–147.** Decoders written against M.1371-3 (and the GPSD table
  [AIVDM "Type 4"]) treat these as a 10-bit spare; M.1371-4 and later
  split off bit 138 as transmission control. Every recorded message here
  has 0 there, so the two readings agree today; they will not if a base
  station ever sets the bit.
* **Comm-state sub-messages from stations without UTC** are garbage
  (line 17: 01:45 in a message that says 22:10). Only trust the hour/
  minute sub-message from a station whose sync state is 0.
* **Message 11 is rare and short-lived.** You will see it only when some
  ship nearby has lost UTC and is polling; each answer de-allocates its
  slot immediately (`slot_timeout 0, slot_offset 0`). Receivers at the
  edge of coverage often see the answers without the questions.
* **The noisy receiver problem** (line 25): a 72-bit payload whose first
  six bits say 11 is not a message 11. Always check `payload_bits`
  against the fixed length before believing a decode; `require_bits`
  does it for you, but only if you call it.

## 6.7 Exercises

1. **(Pencil)** Decode line 2 of the data file by hand as in §6.3 and
   confirm `utc_iso` 2007-05-17T01:21:09Z and slot number 380. Then
   compute how many seconds into the minute slot 380 starts and compare
   with the logger's stamp 1179364989.19.
2. **(Python)** Write a script that reads a decoded `.jsonl` file, and for
   every message 4 or 11 with a `uscg.time` prints
   `mmsi, station, receive_time − utc_iso` in seconds. Run it on
   `tests/expected/ch06-base-station-reports.jsonl`; you should get about
   +120 for 3669730, −102 for 312670, +1 for receiver 003669946 and −4
   for receiver 003669945 on the same transmission.
   (`datetime.datetime.fromisoformat` accepts the `Z` suffix directly and
   `.timestamp()` turns it into UNIX seconds.)
3. **(TypeScript)** `utcIso` accepts 31 April and 30 February. Add a
   calendar check (28/29/30/31 days per month, leap years) and make it
   return `null` for impossible dates. Keep the output identical for every
   line in `data/` — none of them has an impossible date — and confirm
   with `tests/run_tests.sh typescript`.
4. **(Python, larger data)** Using the raw
   `sample-data/RightwhaleAisProject/20081013.log`, compute the
   distribution of `receive_time − utc_iso` per receiver (`uscg.station`)
   for message 4. Which receiver is 3–5 s slow? Which one has stretches
   of ~128 s? Then do the same with `time_of_arrival_s` against the
   message's `utc_second` and slot number — how much better is it?
5. **(Rust)** Add a unit test to `msg_4_11.rs` that builds an all-default
   message 4 (year 0, month 0, day 0, hour 24, minute 60, second 60,
   position 181°/91°, EPFD 0) from a bit string, decodes it, and asserts
   that all six `_available` fields are `false`, `utc_iso` is `Json::Null`
   and `position_available` is `false`. (Hint: `Bits::new(vec![...])`
   takes a `Vec<u8>` of 0/1 values; write a small helper that turns a
   `&str` of `'0'`/`'1'` into it.)
6. **(C++, design)** A base-station watchdog: read lines from standard
   input, keep a `std::map<std::uint64_t, Record>` keyed by MMSI holding
   the last receive time, position and a running mean clock offset, and
   print a warning when (a) a known station has not been heard for 60 s,
   (b) a station's position moves more than 50 m from its first report,
   or (c) a station's MMSI does not start with `00`. Test it on the data
   file: it should warn about 312670 for reason (c) and about nothing else.

## 6.8 References

* [M.1371-5 Annex 8 §3.2 Table 51] — message 4 / 11 layout, UTC field
  sentinels, EPFD codes, transmission control, SOTDMA comm state.
* [M.1371-5 Annex 8 §3.8 Table 60], [§3.9] — message 10 layout; message
  11 "refer to message 4".
* [M.1371-5 Annex 8 Table 50] — position accuracy flag determination.
* [M.1371-5 Annex 1 §4.2.1 Table 2 note 1] — base station reporting
  interval 10 s, 3 1/3 s when being synchronised to.
* [M.1371-5 Annex 2 §3.1.1.1–§3.1.1.4] — UTC direct/indirect, base
  station and received-stations synchronisation; [§3.1.2] frame of 2 250
  slots; [§3.1.3.3.1] base station operation.
* [M.1371-5 Annex 2 §3.3.4.3.1] — FATDMA reservations require message 4;
  120 NM rule.
* [M.1371-5 Annex 2 §3.3.7.2.2 Table 18], [§3.3.7.2.3 Table 19] — SOTDMA
  communication state and sub-messages.
* [M.1371-5 Annex 2 §4.1.2] — channel alternation of periodic messages;
  [§4.6.1] repeat indicator.
* [M.1371-5 Annex 4 §3.3.3] — AIS shore-station qualifier for the
  long-range message 27 (message 4 together with message 23).
* [M.585-9 Annex 1 Section 2] — coast-station identities `00MIDXXXX`
  and the optional sixth-digit scheme.
* [MSC.140(76)] — protection of the AIS VHF data link.
* [IEC 61162-1 §8] — the `ALR` alarm sentence formatter seen with the
  `BS` talker in the bench log.
* [AIVDM "Type 4"] — GPSD's field table (note the 10-bit spare, pre-M.1371-4).
