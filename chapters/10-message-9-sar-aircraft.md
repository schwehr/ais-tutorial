# 10. Message 9: standard SAR aircraft position report

> **What you will learn**
> * Why search-and-rescue (SAR) aircraft have their own position report, how it differs from the Class A report of chapter 3 (altitude instead of rate of turn and heading, speed in whole knots, time stamp in a different place) and how often it is sent.
> * The 168-bit layout of message 9 [M.1371-5 Annex 8 §3.7 Table 59], its not-available and "at or above" sentinels (4095/4094 m, 1023/1022 kn) and the `111MIDxxx` MMSI format that identifies an aircraft [M.585-9].
> * How the communication state selector bit chooses between SOTDMA and ITDMA, and how one decoder function serves messages 1–3, 9, 18 and 26.
> * How to tell that a message type is *absent* from a recording — and how to build, and label, a synthetic example so the code can still be tested.
> * Language notes: almost none — this chapter deliberately reuses what chapters 3 and 6 introduced (and shares its selector-bit logic with message 18, chapter 14), so you can concentrate on the protocol.

## 10.1 Where this fits

AIS was designed for ships, but a maritime SAR operation is run from the air as much as from the sea: a fixed-wing aircraft flying a search pattern at 150 knots and 500 m, or a helicopter hovering over a casualty. If those aircraft carry an AIS transponder, the ships and the rescue coordination centre see them on the same display as everything else. ITU-R M.1371 therefore defines a dedicated station class — "search and rescue mobile aircraft equipment" — which "should transmit position report Message 9, and static data using Message 5 and Messages 24A and 24B" [M.1371-5 Annex 1 §2.1.4].

Message 9 is the Class A position report (chapter 3) rebuilt for something that flies:

| | Messages 1–3 (ship) | Message 9 (aircraft) |
|---|---|---|
| Navigational status | 4 bits | — |
| Rate of turn | 8 bits | — |
| **Altitude** | — | **12 bits, metres** |
| Speed over ground | 10 bits, 1/10 knot (max 102.2) | 10 bits, **whole knots** (max 1022) |
| True heading | 9 bits | — |
| Time stamp | bits 137–142 | bits 128–133 |
| Altitude sensor, DTE, assigned-mode flag | — | 1 bit each |
| Communication state | SOTDMA for 1/2, ITDMA for 3 (fixed by type) | selector bit + 19 bits (either) |

Sources: [M.1371-5 Annex 8 Table 48] and [Table 59]. A ship's SOG field tops out at 102.2 knots, which no aircraft would tolerate; an aircraft has no rate-of-turn indicator or gyro heading to report; and the one thing a ship never needs — height — becomes the most important field after the position.

"Stations other than aircraft involved in SAR operations should not transmit this message. The default reporting interval for this message should be 10 s" [M.1371-5 Annex 8 §3.7]; Annex 1 Table 2 gives the same 10 s nominal interval for "search and rescue aircraft (airborne mobile equipment)" with a footnote that "shorter RI down to 2 s could be used in the area of search and rescue operations" [M.1371-5 Annex 1 Table 2 note 2]. The message has link priority 1, like every position report, and may be sent with SOTDMA, RATDMA or ITDMA access (self-organising, random-access and incremental TDMA; chapter 3) [M.1371-5 Annex 8 Table 46].

The aircraft's identity is a maritime mobile service identity (MMSI) of the form `111MIDXXX`: three fixed digits `111`, the three-digit maritime identification digits (MID) of the administration, and three digits chosen by that administration, which "may use the seventh digit to differentiate" — `111MID1XX` for fixed-wing aircraft and `111MID5XX` for helicopters are the examples given [M.585-9 Annex 1 §3]. The special identity `111MID000` is a *group* call to all SAR aircraft of that administration [M.585-9 Annex 1 §3 item 5]. The aircraft's message 5 should carry the name `SAR AIRCRAFT NNNNNNN`, where `NNNNNNN` is the aircraft registration, and leaves IMO number, draught and the ship-dimension fields unused or at the administration's discretion [M.1371-5 Annex 8 Table 52].

### Operational use

* **On a ship's bridge**, a SAR aircraft appears as an AIS target like any other, but a display that follows the IMO presentation guidance can show its altitude and distinguish the aircraft symbol — useful when a helicopter is about to winch from your deck, or when you are the on-scene coordinator and need to know where the search aircraft is in its pattern.
* **For the aircraft crew**, the reverse is the point: the aircraft's AIS receiver shows every AIS-equipped ship in VHF range (which, from 500 m altitude, is far more than from a bridge — the VHF radio horizon to a ship's antenna is roughly 50–60 nautical miles instead of 20), with name, MMSI and course, which is exactly what a SAR crew needs to task nearby ships, verify a distress position, or find the one fishing vessel that has stopped answering the radio.
* **For the rescue coordination centre (RCC)**, aircraft position reports relayed through the coastal AIS network provide a live track of the search without voice reports. The 10 s interval (2 s in the search area) is fast enough to reconstruct the search pattern afterwards for coverage analysis.
* **Who actually transmits it.** In the United States the Coast Guard's aircraft carry AIS, and USCG aircraft MMSIs are in the `111366xxx` block (Air Station Cape Cod's aircraft, which fly over the waters our receivers covered, among them). Other administrations equip their coastguard and navy SAR aircraft similarly. Civil airliners do not carry AIS; neither do most private aircraft.

### Unattended / automated uses

* **SAR pattern reconstruction and coverage.** Logged message 9 tracks let software compute the area actually searched (track × sensor swath), overlay it on the drift prediction and tell the RCC what is left — a task that used to be done on paper from radio position reports.
* **Aircraft tracking by shore networks.** A coastal AIS network sees aircraft tens of miles further than ships because of the altitude. Some shore systems forward message 9 into the same feeds as ship positions; consumers that assume "AIS = surface vessel" then draw a 150-knot "ship" moving over land. A pipeline should route on message type (9) and on the MMSI prefix (`111`) and keep aircraft in their own layer.
* **Validation rules that a program can apply.** A genuine message 9 has `payload_bits` = 168, an MMSI starting with `111`, altitude < 4094 (or exactly 4094/4095 with the right meaning), SOG < 1022, a position within VHF range of the receiver and a comm-state `sync_state` that makes sense. Anything else is noise or a mis-configured station (§10.6 shows what our receivers actually produced). Chapters 15 and 16 make the same point for messages 21 and 25–27: the rarer the type, the larger the share of garbage among the lines that claim to be it.
* **Spoofing and security.** A message 9 is as unauthenticated as every other AIS message. An attacker who wants to disrupt a search, or a hobbyist who thinks an "aircraft" on the AIS display is funny, needs only a software-defined radio. Data consumers that drive decisions (RCC displays, collision-avoidance for drones, airspace deconfliction) should cross-check message 9 tracks against radar or ADS-B where available. IMO's MSC.140(76) asks administrations to "take steps necessary to ensure the integrity of the radio channels used for AIS in their waters", which in practice means detecting and locating such transmitters [MSC.140(76)].
* **Interrogation and assignment.** A base station may ask an aircraft for a message 9 or 24 with a message 15, and may assign it a reporting schedule with a message 16; when so assigned the aircraft sets its *assigned mode flag* [M.1371-5 Annex 8 Table 65; Annex 2 §3.3.6]. Chapter 11 covers both messages; in an automated vessel traffic service (VTS) the aircraft is just another station to be polled.

## 10.2 The specification

Message 9 is fixed at 168 bits, exactly one slot [M.1371-5 Annex 8 §3.7 Table 59].

| Bits | Len | Field | Type/units | Not-available value | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 9 [Table 59] |
| 6–7 | 2 | Repeat indicator | uint | — | 0 default, 3 = do not repeat any more [Table 59] |
| 8–37 | 30 | User ID | MMSI | — | `111MIDxxx` for an aircraft [Table 59; M.585-9 Annex 1 §3] |
| 38–49 | 12 | Altitude (GNSS) | metres, 0–4094 | 4095 | "4 094 = 4 094 m or higher"; source per the altitude sensor bit [Table 59] |
| 50–59 | 10 | SOG | knots, 0–1022 | 1023 | whole knots; "1 022 = 1 022 knots or higher" [Table 59] |
| 60 | 1 | Position accuracy | flag | 0 | 1 = high (≤ 10 m), 0 = low (> 10 m); determined per Table 50 from the RAIM (receiver autonomous integrity monitoring) flag and differential status [Table 59; Table 50] |
| 61–88 | 28 | Longitude | signed, 1/10 000 min | 181° = 0x6791AC0 | East positive, two's complement [Table 59] |
| 89–115 | 27 | Latitude | signed, 1/10 000 min | 91° = 0x3412140 | North positive, two's complement [Table 59] |
| 116–127 | 12 | COG (course over ground) | 1/10 degree, 0–3599 | 3600 (0xE10) | "3 601–4 095 should not be used" [Table 59] |
| 128–133 | 6 | Time stamp | UTC second, 0–59 | 60 | 61 = positioning system in manual input mode, 62 = dead reckoning, 63 = inoperative [Table 59] |
| 134 | 1 | Altitude sensor | enum | — | 0 = GNSS, 1 = barometric source [Table 59] |
| 135–141 | 7 | Spare | — | 0 | "Not used. Should be set to zero" [Table 59] |
| 142 | 1 | DTE | flag | 1 | data terminal equipment ready: 0 = available, 1 = not available (default) [Table 59; §3.3.1] |
| 143–145 | 3 | Spare | — | 0 | [Table 59] |
| 146 | 1 | Assigned mode flag | flag | 0 | 0 = autonomous and continuous, 1 = assigned mode [Table 59] |
| 147 | 1 | RAIM flag | flag | 0 | 1 = RAIM in use [Table 59; Table 50] |
| 148 | 1 | Communication state selector | flag | — | 0 = SOTDMA state follows, 1 = ITDMA state follows [Table 59] |
| 149–167 | 19 | Communication state | struct | — | SOTDMA [Annex 2 §3.3.7.2.2 Tables 18/19] or ITDMA [Annex 2 §3.3.7.3.2 Table 20] |

Points worth a second look:

* **Altitude is an unsigned 12-bit integer in metres**, 0 to 4094, which is plenty for a SAR aircraft (a helicopter rarely exceeds 3 000 m; a maritime patrol aircraft on a search flies far lower). There is no sign bit, so an aircraft on the apron of an airfield below sea level reports 0. Two sentinels share the top of the range: 4095 means *not available* and 4094 means *4094 m or higher* [Table 59]. Our decoder reports both as booleans (`altitude_available`, `altitude_at_max`) so that a consumer does not mistake the ceiling for a measurement.
* **The altitude sensor bit (134)** says whether the number came from the GNSS receiver or a barometric altimeter. The two can differ by tens of metres (barometric altitude depends on the local pressure setting; GNSS height is geometric and, on most receivers, above the WGS-84 ellipsoid rather than mean sea level). The field is labelled "Altitude (GNSS)" in Table 59 but the description says "derived from GNSS or barometric (see altitude sensor parameter below)" [Table 59] — read the bit before you trust the number to a metre.
* **SOG is in whole knots.** Compare chapter 3: the ship report's 10 bits hold tenths of a knot (0–102.2); the same 10 bits here hold 0–1022 knots. The decoders therefore print `sog_knots` as an integer, not as a one-decimal `Fixed`, and the sentinels move from 1023/1022 tenths to 1023/1022 knots — `sog_available` and `sog_at_max` make them explicit [Table 59].
* **Position, COG and time stamp** use exactly the encodings of message 1 [Table 48]: two's-complement longitude/latitude in 1/10 000 of a minute (divide by 600 000 for degrees), COG in tenths of a degree with 3600 = not available, and the six-bit UTC second with 60–63 as status codes. Only the bit offsets differ: the time stamp is at 128 because the fields before it are 9 bits shorter than in message 1 (12 bits of altitude replace 4 + 8 + 9 bits of status, ROT and heading).
* **DTE** (bit 142) is the same flag as bit 422 of message 5: 0 means a display/keyboard is available for safety-related text, 1 (the default) means it is not [M.1371-5 Annex 8 §3.3.1; Table 59].
* **Assigned mode flag** (bit 146): a SAR aircraft station that has received a message 16 or 23 assignment "should set their assign mode flag to 'station operating in assigned mode'" [M.1371-5 Annex 2 §3.3.6]. (A Class A ship signals the same condition by switching from message 1 to message 2 instead; message 9 has no such twin, hence the bit.)
* **The communication state selector** (bit 148) and the 19 bits after it are identical in structure to message 18 (chapter 14) and message 26 (chapter 16): 0 → read bits 149–167 as the SOTDMA state (sync state, slot time-out, sub-message); 1 → read them as the ITDMA state (sync state, slot increment, number of slots, keep flag) [Table 59; Annex 2 Tables 18–20]. ITDMA is used during network entry and when changing reporting rate, SOTDMA in continuous operation [M.1371-5 Annex 8 Table 46 note 1].

Enumerations in full:

| Field | Values |
|---|---|
| Altitude sensor (bit 134) | 0 = GNSS, 1 = barometric source [Table 59] |
| DTE (bit 142) | 0 = available, 1 = not available (default) [Table 59] |
| Assigned mode (bit 146) | 0 = autonomous and continuous (default), 1 = assigned [Table 59] |
| Time stamp (bits 128–133) | 0–59 = UTC second; 60 = not available (default); 61 = manual input mode; 62 = estimated (dead reckoning); 63 = positioning system inoperative [Table 59] |
| Sync state (bits 149–150) | 0 = UTC direct, 1 = UTC indirect, 2 = synchronized to base station, 3 = synchronized to another station [Annex 2 Table 18, Table 20] |

## 10.3 Decoding one message by hand

### There is no recorded message 9

Before decoding anything we have to be honest about the data. The scan behind this chapter looked at every `!AIVDM`/`!AIVDO` line in `sample-data/` — about five receivers, 2006 to 2011, including the 49 MB USCG N-AIS capture in `sample-data/RightwhaleAisProject/20081013.log` that covers Cape Cod Bay, where Air Station Cape Cod's aircraft fly. It found **no** line whose first six bits are `001001` and whose payload is 168 bits long. The dozen or so lines that *start* like a message 9 are 40 to 328 bits long and come from the `nhjel` receiver in Great Bay, NH, whose checksummed garbage you have met in chapters 2, 3 and 6; three of them are in Part 2 of the data file and §10.5 shows what the decoder makes of them.

That is a real result, not a failure of the search: our receivers were in New Hampshire estuaries and on a hotel roof in Norfolk, and SAR aircraft with AIS were rare in 2007–2011. So, as chapters 11, 13 and 16 also have to do for their rarer messages, we build the examples ourselves and say so in the data file.

### Building a synthetic message 9

We choose field values for a plausible fixed-wing SAR aircraft over Stellwagen Bank, east of Boston, and write each one as a bit string of the Table 59 width (two's complement for the signed fields, as in chapter 2):

| Field | Value | Bits |
|---|---|---|
| message ID | 9 | `001001` |
| repeat indicator | 0 | `00` |
| MMSI | 111366101 (`111` + MID 366 + `101`, a fixed-wing aircraft by the M.585 example scheme) | `000110101000110100111111010101` |
| altitude | 457 m (1 500 ft) | `000111001001` |
| SOG | 150 kn | `0010010110` |
| position accuracy | 1 | `1` |
| longitude | −70.325° → −70.325 × 600 000 = −42 195 000 → 2²⁸ − 42 195 000 = 226 240 456 | `1101011111000010011111001000` |
| latitude | 42.4167° → 42.4167 × 600 000 = 25 450 020 | `001100001000101011000100100` |
| COG | 225.5° → 2255 | `100011001111` |
| time stamp | 17 | `010001` |
| altitude sensor | 0 (GNSS) | `0` |
| spare | 0 | `0000000` |
| DTE | 0 (available) | `0` |
| spare | 0 | `000` |
| assigned mode | 0 | `0` |
| RAIM | 0 | `0` |
| comm state selector | 0 (SOTDMA) | `0` |
| SOTDMA: sync state 0, slot time-out 3, sub-message 150 (received stations) | | `00` `011` `00000010010110` |

Concatenated that is 168 bits, 28 six-bit groups, no fill. Converting each group with the armour table of chapter 2 (add 48; add 8 more if the result exceeds 87) and computing the NMEA checksum gives line 5 of [`data/ch10-sar-aircraft.nmea`](../data/ch10-sar-aircraft.nmea):

```text
!AIVDM,1,1,,A,91b=?mAjBFrv4v@HAF98kl@00<2F,0*4B
```

### Reading it back

Six-bit groups under their characters (Appendix B has the table):

```text
9      1      b      =      ?      m      A      j      B      F      r      v      4      v
001001 000001 101010 001101 001111 110101 010001 110010 010010 010110 111010 111110 000100 111110

@      H      A      F      9      8      k      l      @      0      0      <      2      F
010000 011000 010001 010110 001001 001000 110011 110100 010000 000000 000000 001100 000010 010110
```

| Bits | Value | Field |
|---|---|---|
| 0–5 | `001001` = 9 | message ID |
| 6–7 | `00` = 0 | repeat indicator |
| 8–37 | `000110101000110100111111010101` = **111366101** | MMSI; starts with 111 → aircraft; MID 366 → USA [M.585-9] |
| 38–49 | `000111001001` = **457** | altitude, metres; ≠ 4095 so available, ≠ 4094 so not clipped |
| 50–59 | `0010010110` = **150** | SOG, whole knots |
| 60 | `1` | position accuracy high |
| 61–88 | `1101011111000010011111001000` | longitude: leading 1 → negative; 226 240 456 − 2²⁸ = −42 195 000 → ÷ 600 000 = **−70.325°** |
| 89–115 | `001100001000101011000100100` = 25 450 020 | latitude: ÷ 600 000 = **42.4167°** |
| 116–127 | `100011001111` = 2255 | COG **225.5°** |
| 128–133 | `010001` = **17** | time stamp, UTC second |
| 134 | `0` | altitude sensor: GNSS |
| 135–141 | `0000000` | spare |
| 142 | `0` | DTE available |
| 143–145 | `000` | spare |
| 146 | `0` | autonomous mode |
| 147 | `0` | RAIM not in use |
| 148 | `0` | SOTDMA state follows |
| 149–150 | `00` | sync state 0, UTC direct |
| 151–153 | `011` = 3 | slot time-out 3 |
| 154–167 | `00000010010110` = 150 | sub-message; with time-out 3 it is the number of received stations [Annex 2 Table 19] |

Compare with the first object in [`tests/expected/ch10-sar-aircraft.jsonl`](../tests/expected/ch10-sar-aircraft.jsonl): `altitude_m` 457, `sog_knots` 150, `lon` −70.325000, `lat` 42.416700, `cog_deg` 225.5, `timestamp` 17, `comm_state.received_stations` 150. The remaining three synthetic sentences (lines 7, 9 and 11 of the data file) were produced the same way; their field choices are listed in §10.5.

## 10.4 The code

The decoder is a close relative of messages 1–3 and 18. The only new logic is the pair of sentinel checks on altitude and SOG; the communication state is read by the two functions from the messages 1–3 module, chosen by the selector bit exactly as message 18 does.

### Python

[`code/python/aistut/msg_9.py`](../code/python/aistut/msg_9.py):

```python
from .bits import Bits
from .common import require_bits
from .jsonout import Fixed
from .msg_1_2_3 import decode_itdma, decode_sotdma

ALTITUDE_NOT_AVAILABLE = 4095
ALTITUDE_MAX = 4094          # "4 094 m or higher"
SOG_NOT_AVAILABLE = 1023
SOG_MAX = 1022               # "1 022 knots or higher"

ALTITUDE_SENSOR = {0: "GNSS", 1: "barometric"}
ASSIGNED_MODE = {0: "autonomous and continuous", 1: "assigned"}


def decode(bits: Bits) -> dict:
    """Decode a message 9. Raises DecodeError if shorter than 168 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 168)
    altitude = bits.uint(38, 12)
    sog = bits.uint(50, 10)
    lon_raw = bits.sint(61, 28)
    lat_raw = bits.sint(89, 27)
    cog_raw = bits.uint(116, 12)
    sensor = bits.uint(134, 1)
    assigned = bits.uint(146, 1)
    selector = bits.uint(148, 1)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "altitude_m": altitude,
        "altitude_available": altitude != ALTITUDE_NOT_AVAILABLE,
        "altitude_at_max": altitude == ALTITUDE_MAX,
        # SOG is an integer number of knots here, so no Fixed() wrapper:
        # the JSON shows 150, not 150.0, and the reader can see that the
        # field has no fractional part.
        "sog_knots": sog,
        "sog_available": sog != SOG_NOT_AVAILABLE,
        "sog_at_max": sog == SOG_MAX,
        "position_accuracy": bits.uint(60, 1),
        "lon": Fixed(lon_raw / 600000.0, 6),
        "lat": Fixed(lat_raw / 600000.0, 6),
        "position_available": lon_raw != 0x6791AC0 and lat_raw != 0x3412140,
        "cog_deg": Fixed(cog_raw / 10.0, 1),
        "cog_available": cog_raw != 3600,
        "timestamp": bits.uint(128, 6),
        "altitude_sensor": sensor,
        "altitude_sensor_text": ALTITUDE_SENSOR[sensor],
        "spare": bits.uint(135, 7),
        "dte": bits.uint(142, 1),
        "spare2": bits.uint(143, 3),
        "assigned_mode": assigned,
        "assigned_mode_text": ASSIGNED_MODE[assigned],
        "raim": bits.flag(147),
        "comm_state_selector": selector,
    }
    if selector == 1:
        out["comm_state"] = decode_itdma(bits, 149)
    else:
        out["comm_state"] = decode_sotdma(bits, 149)
    return out
```

Two spare fields exist, so the second is `spare2` — the same convention chapter 14 uses for message 18. `sog_knots` is a bare `int`; everything else that is not an integer (`lon`, `lat`, `cog_deg`) keeps the `Fixed` wrapper and the decimal counts of chapter 3.

> **Language notes (Python).** Nothing new. `from .msg_1_2_3 import decode_itdma, decode_sotdma` is the relative import you saw in chapter 6.

### C++

[`code/cpp/msg_9.hpp`](../code/cpp/msg_9.hpp), the body of the function (the constants and the two `std::array` text tables sit in a `namespace msg_9`):

```cpp
inline Json decode_msg_9(const Bits& bits) {
    using namespace msg_9;
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 168);
    std::uint64_t altitude = bits.uint(38, 12);
    std::uint64_t sog = bits.uint(50, 10);
    std::int64_t lon_raw = bits.sint(61, 28);
    std::int64_t lat_raw = bits.sint(89, 27);
    std::uint64_t cog_raw = bits.uint(116, 12);
    std::uint64_t sensor = bits.uint(134, 1);
    std::uint64_t assigned = bits.uint(146, 1);
    std::uint64_t selector = bits.uint(148, 1);

    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"altitude_m", altitude},
        {"altitude_available", altitude != ALTITUDE_NOT_AVAILABLE},
        {"altitude_at_max", altitude == ALTITUDE_MAX},
        {"sog_knots", sog},   // whole knots: an integer, not Json::fixed
        {"sog_available", sog != SOG_NOT_AVAILABLE},
        {"sog_at_max", sog == SOG_MAX},
        {"position_accuracy", bits.uint(60, 1)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600000.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600000.0, 6)},
        {"position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140},
        {"cog_deg", Json::fixed(static_cast<double>(cog_raw) / 10.0, 1)},
        {"cog_available", cog_raw != 3600},
        {"timestamp", bits.uint(128, 6)},
        {"altitude_sensor", sensor},
        {"altitude_sensor_text", ALTITUDE_SENSOR[sensor]},
        {"spare", bits.uint(135, 7)},
        {"dte", bits.uint(142, 1)},
        {"spare2", bits.uint(143, 3)},
        {"assigned_mode", assigned},
        {"assigned_mode_text", ASSIGNED_MODE[assigned]},
        {"raim", bits.flag(147)},
        {"comm_state_selector", selector},
    };
    if (selector == 1) {
        out["comm_state"] = decode_itdma(bits, 149);
    } else {
        out["comm_state"] = decode_sotdma(bits, 149);
    }
    return out;
}
```

> **Language notes (C++).**
> * `constexpr std::uint64_t ALTITUDE_NOT_AVAILABLE = 4095;` gives the constant the same unsigned 64-bit type that `bits.uint` returns, so `altitude != ALTITUDE_NOT_AVAILABLE` compares like with like and `-Wextra` stays quiet. (Chapter 16 does the same with `std::int64_t` for the *signed* sentinels of message 27.)
> * `using namespace msg_9;` inside the function body brings the constants into scope for that function only — a smaller hammer than a file-wide `using`.

### Rust

[`code/rust/src/msg_9.rs`](../code/rust/src/msg_9.rs), the parts that differ from the Python:

```rust
use crate::msg_1_2_3::{decode_itdma, decode_sotdma};

pub const ALTITUDE_NOT_AVAILABLE: u64 = 4095;
pub const ALTITUDE_MAX: u64 = 4094; // "4 094 m or higher"
pub const SOG_NOT_AVAILABLE: u64 = 1023;
pub const SOG_MAX: u64 = 1022; // "1 022 knots or higher"

pub fn altitude_sensor_text(code: u64) -> &'static str {
    match code {
        0 => "GNSS",
        _ => "barometric",
    }
}

pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 168)?;
    let altitude = bits.uint(38, 12);
    let sog = bits.uint(50, 10);
    // ...
    let mut out = Obj::new();
    put(&mut out, "altitude_m", altitude);
    put(&mut out, "altitude_available", altitude != ALTITUDE_NOT_AVAILABLE);
    put(&mut out, "altitude_at_max", altitude == ALTITUDE_MAX);
    put(&mut out, "sog_knots", sog); // whole knots: an integer, not Json::fixed
    put(&mut out, "sog_available", sog != SOG_NOT_AVAILABLE);
    put(&mut out, "sog_at_max", sog == SOG_MAX);
    // ...
    let comm_state = if selector == 1 {
        decode_itdma(bits, 149)
    } else {
        decode_sotdma(bits, 149)
    };
    put(&mut out, "comm_state", comm_state);
    Ok(Json::Obj(out))
}
```

> **Language notes (Rust).** A one-bit enumeration as `match code { 0 => …, _ => … }` is what chapter 14 uses for the Class B flags; the `_` arm keeps the match exhaustive without listing `1` separately. The `#[cfg(test)]` module at the bottom of the file decodes the synthetic sentence and the 120-bit garbage line and checks the exact error text — run `cargo test` in `code/rust` to see it.

### TypeScript

[`code/typescript/src/msg_9.ts`](../code/typescript/src/msg_9.ts), the parts that differ:

```typescript
import { decodeItdma, decodeSotdma } from "./msg_1_2_3";

export const ALTITUDE_NOT_AVAILABLE = 4095;
export const ALTITUDE_MAX = 4094; // "4 094 m or higher"
export const SOG_NOT_AVAILABLE = 1023;
export const SOG_MAX = 1022; // "1 022 knots or higher"

export const ALTITUDE_SENSOR = ["GNSS", "barometric"];
export const ASSIGNED_MODE = ["autonomous and continuous", "assigned"];

export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 168);
  const altitude = bits.uint(38, 12);
  const sog = bits.uint(50, 10);
  // ...
  const out: JsonObject = {
    // ...
    altitude_m: altitude,
    altitude_available: altitude !== ALTITUDE_NOT_AVAILABLE,
    altitude_at_max: altitude === ALTITUDE_MAX,
    sog_knots: sog, // whole knots: an integer, not Fixed
    sog_available: sog !== SOG_NOT_AVAILABLE,
    sog_at_max: sog === SOG_MAX,
    // ...
  };
  if (selector === 1) {
    out["comm_state"] = decodeItdma(bits, 149);
  } else {
    out["comm_state"] = decodeSotdma(bits, 149);
  }
  return out;
}
```

> **Language notes (TypeScript).** JavaScript has one `number` type, so "integer SOG" versus "SOG in tenths" is decided only by what we hand to the JSON writer: a bare integral `number` prints as `150`, a `new Fixed(x, 1)` prints `15.0`. The writer checks `Number.isInteger` at run time (chapter 2).

### Where the four differ

| Concern | Python | C++ | Rust | TypeScript |
|---|---|---|---|---|
| Typed sentinel constants | plain names | `constexpr std::uint64_t` | `pub const …: u64` | `export const` (literal type) |
| 0/1 → text | dict | `std::array<const char*, 2>` | `match` with `_` arm | string array |
| Integer vs fixed-point output | `int` vs `Fixed` | `std::uint64_t` vs `Json::fixed` | `u64` vs `Json::fixed` | `number` vs `Fixed` |
| Comm-state reuse | `from .msg_1_2_3 import` | `#include "msg_1_2_3.hpp"` | `use crate::msg_1_2_3::…` | `import { … } from "./msg_1_2_3"` |

## 10.5 Sample data and results

[`data/ch10-sar-aircraft.nmea`](../data/ch10-sar-aircraft.nmea) has two parts:

1. **Synthetic, labelled** (lines 5, 7, 9, 11) — four message 9 sentences built as in §10.3:
   * line 5: aircraft 111366101, 457 m GNSS altitude, 150 kn, SOTDMA (the hand decode above);
   * line 7: the same aircraft ten seconds later — 460 m *barometric*, 152 kn, RAIM on, ITDMA state (increment 375, 1 slot, keep flag set);
   * line 9: helicopter 111366501 with every field at its default: altitude 4095, SOG 1023, position 181°/91°, COG 3600, time stamp 60, DTE 1;
   * line 11: aircraft 111366102 at the "or higher" sentinels — altitude 4094, SOG 1022 — in assigned mode, time stamp 61 (manual input), synchronised to a base station.
2. **Recorded, garbage** (lines 15, 17, 19) — three lines from the `nhjel` receiver that begin with a message-9 identifier: `sample-data/tide1/ais/ais-2008-11-27` line 127 (120 bits), `sample-data/tide1/ais/ais-2008-10-09` line 113 (160 bits) and `sample-data/tide1/ais/ais-2008-11-27` line 398 (328 bits).

Run any of the four decoders:

```sh
python3 code/python/decode.py data/ch10-sar-aircraft.nmea
code/cpp/build/decode data/ch10-sar-aircraft.nmea
code/rust/target/release/decode data/ch10-sar-aircraft.nmea
node code/typescript/dist/decode.js data/ch10-sar-aircraft.nmea
```

All four print the seven lines of [`tests/expected/ch10-sar-aircraft.jsonl`](../tests/expected/ch10-sar-aircraft.jsonl). Selected objects (the `"ais"` part only):

**Line 5 — the hand-decoded sentence:**

```json
{"altitude_at_max":false,"altitude_available":true,"altitude_m":457,"altitude_sensor":0,"altitude_sensor_text":"GNSS","assigned_mode":0,"assigned_mode_text":"autonomous and continuous","cog_available":true,"cog_deg":225.5,"comm_state":{"received_stations":150,"scheme":"SOTDMA","slot_timeout":3,"sync_state":0,"sync_state_text":"UTC direct"},"comm_state_selector":0,"dte":0,"lat":42.416700,"lon":-70.325000,"mmsi":111366101,"msg_type":9,"position_accuracy":1,"position_available":true,"raim":false,"repeat_indicator":0,"sog_at_max":false,"sog_available":true,"sog_knots":150,"spare":0,"spare2":0,"timestamp":17}
```

**Line 7 — ITDMA and barometric altitude.** `altitude_sensor` 1 / `"barometric"`, `altitude_m` 460, `sog_knots` 152, `raim` true, `comm_state_selector` 1 and

```json
"comm_state":{"keep_flag":true,"scheme":"ITDMA","slot_increment":375,"slot_offset":375,"slots":1,"slots_raw":0,"sync_state":0,"sync_state_text":"UTC direct"}
```

— an increment of 375 slots is 10 s at 2 250 slots per minute [M.1371-5 Annex 2 §3.1.2], i.e. the aircraft announcing its next report one nominal interval ahead, exactly as a ship does in message 3 (chapter 3).

**Line 9 — every default.** `altitude_m` 4095 with `altitude_available` false; `sog_knots` 1023 with `sog_available` false; `lon` 181.000000, `lat` 91.000000, `position_available` false; `cog_deg` 360.0, `cog_available` false; `timestamp` 60; `dte` 1. This is what a transponder sends before its GNSS has a fix — and, for the time stamp, what some send forever (§10.6).

**Line 11 — the ceilings.** `altitude_m` 4094 with `altitude_available` true **and** `altitude_at_max` true; `sog_knots` 1022 with `sog_at_max` true; `assigned_mode` 1 / `"assigned"`; `timestamp` 61; `sync_state` 2 `"synchronized to base station"`. A consumer that plots altitude must treat 4094 as "≥ 4094", not as a measurement.

**Lines 15 and 17 — too short.** `{"error":"message 9 too short: 120 bits < 168", …}` and `… 160 bits < 168`. The 160-bit line is the instructive one: it is only eight bits short, and a decoder that did not check the length would read a communication state out of the fill.

**Line 19 — long enough to decode, and wrong in every field.** 328 bits decode (from the first 168) to MMSI **69358984** — eight digits, no `111` prefix — at **35.615162° N 12.811213° E**, which is the Strait of Sicily, heard by a receiver in New Hampshire; altitude 1 966 m at 276 kn, `spare` 13 (should be 0), `received_stations` 2 592 (more stations than there are slots in a frame). Every one of those is a validation failure; `payload_bits` 328 ≠ 168 alone is enough to discard it.

## 10.6 Pitfalls and real-world deviations

* **Absence is the normal case.** Across our recordings there is not one valid message 9, and a shore receiver anywhere outside a busy coastguard air-station's area may go months without one. [AIVDM "Type 9"] likewise notes that the type is rare. Build your pipeline so that an empty result for a message type is not an error — and so that the garbage that *does* arrive labelled "9" is rejected by length and range checks, not counted (§10.5, line 19).
* **Length first.** The payload must be exactly 168 bits. The 160-bit line above shows how close noise can come; a 168-bit *minimum* check (which is what `require_bits` enforces) accepts the 328-bit line, so a consumer that wants only genuine reports should additionally require `payload_bits == 168`. We leave that to the layer above the decoder, as chapter 16 does for the 168-bit message 27s that [AIVDM] reports in the wild.
* **MMSI format.** A SAR aircraft MMSI starts with `111` [M.585-9 Annex 1 §3]. In practice some aircraft have been seen transmitting message 9 with a ship-format MMSI, and some ships' transponders can be configured as "SAR aircraft" by mistake; a message 9 with a `2xx`–`7xx` MMSI deserves a look at the message 5 name (`SAR AIRCRAFT …` is the rule [M.1371-5 Table 52]) before you believe either the type or the identity.
* **Altitude units and datum.** The field is metres, not feet; the bit at 134 tells you whether it is GNSS (geometric, usually above the ellipsoid) or barometric (pressure altitude or QNH-corrected, depending on the installation). Do not compare one aircraft's barometric altitude with another's GNSS altitude to the metre, and do not expect agreement with an ADS-B pressure altitude.
* **Speed is whole knots.** A decoder written by copy-and-paste from message 1 that divides by 10 will show a 150-knot aircraft doing 15.0 knots — a plausible boat speed, which makes the bug hard to notice. The field widths are identical; only the unit changed [Table 48 vs Table 59].
* **Time stamp at 128, not 137.** The second copy-and-paste trap: all the fields after the position shift by nine bits relative to message 1. A decoder with message 1 offsets reads the aircraft's COG low bits and the time stamp as a "true heading" and part of the sensor/spare bits as the "time stamp". The symptom is a heading on an aircraft report (message 9 has none) and time stamps that never change.
* **Reporting interval.** 10 s nominal, 2 s in the search area [M.1371-5 Annex 1 Table 2]; a base station can assign a different rate (chapter 11). An aircraft that reports every 2 s uses 30 slots a minute per channel; several aircraft on scene are a noticeable load on the link, which is one reason the assignment mechanism exists.
* **M.1371-4 versus M.1371-5, and older editions.** The message 9 table is Table 59 in M.1371-5 and Table 56 in M.1371-4; the layout is the same in both. It was *not* the same in M.1371-1 and -2: there, bits 134–141 were eight bits "reserved for definition by a competent regional authority" and there was no altitude-sensor bit and no assigned-mode flag — M.1371-3 (2007) turned the regional byte into *altitude sensor + 7 spare* and carved the assigned-mode flag out of the spare bits before RAIM [M.1371-5 Table 59] (see the edition history under [M.1371-5] in Appendix A). A decoder written against the 2001 text will label bit 134 "regional" and miss the two flags; the order in Table 59 is altitude sensor (134), DTE (142), assigned mode (146), RAIM (147), selector (148).

## 10.7 Exercises

1. **(Pencil)** Decode line 11 of the data file, `91b=?mgwgvJwjmPGgi`3Q?@0=D07`, by hand far enough to read the altitude, SOG, time stamp and assigned-mode bit. Check the four values against `tests/expected/ch10-sar-aircraft.jsonl`.
2. **(Python)** Write `encode_msg_9(mmsi, altitude_m, sog_knots, lon, lat, cog_deg, timestamp)` that returns a complete `!AIVDM` sentence (the inverse of the chapter 2 armouring plus the chapter 1 checksum), with every other field at its Table 59 default and a SOTDMA state of all zeros. Round-trip it through `decode.py` for a position in the southern and western hemispheres and for altitude 4094.
3. **(TypeScript)** Add `isPlausibleSar(obj, receiverLat, receiverLon)` that returns the list of reasons a decoded message 9 is suspect: MMSI not starting with 111, `payload_bits` ≠ 168 (you will need the outer object), `spare` or `spare2` non-zero, position more than 150 nautical miles from the receiver, `sog_knots` > 400 for a `111MID5xx` (helicopter) identity. Run it on the expected output and confirm it flags line 19 and passes lines 5 and 7.
4. **(C++)** `decode_msg_9` and `decode_msg_18` share the selector-bit logic verbatim. Factor it into `inline Json decode_comm_state(const Bits& bits, std::size_t selector_bit)` in `msg_1_2_3.hpp` (in a copy of the tree — the header is owned by chapter 3) and use it from both. Does the test-suite still pass byte-for-byte?
5. **(Rust)** Extend the unit tests in `msg_9.rs` with a 168-bit payload whose selector bit is 1 and whose last 19 bits are `1100000000000000110` — the fixed value Class B "CS" units put in message 18 (chapter 14). What does `decode_itdma` make of it (sync state, `slots_raw`, `slots`, `keep_flag`), and why would such a comm state be nonsense in a real message 9?
6. **(Any language, data)** Write a small scanner that lists every line in `sample-data/tide1/ais/` whose first character is `9` and tabulate payload lengths. How many are 168 bits? What is the distribution of the decoded MMSI's leading digits for those that are long enough to decode? Write two sentences on what this says about counting message types without validation.

## 10.8 References

* [M.1371-5] Annex 8 §3.7 and Table 59 (message 9; Table 56 in M.1371-4); Table 48 (message 1 encodings of position, COG, time stamp that Table 59 reuses); Table 50 (position accuracy flag from RAIM/differential status); Table 52 (message 5 for SAR aircraft: name `SAR AIRCRAFT NNNNNNN`, fields not applicable); Table 46 (priority 1, access schemes); Table 65 (who may interrogate a SAR aircraft and for what); §3.3.1 (DTE); Annex 1 §2.1.4 (SAR aircraft station transmits messages 9, 5, 24A/B) and Table 2 with note 2 (10 s interval, 2 s in the search area); Annex 2 §3.1.2 (2 250 slots per minute), §3.3.6 (assigned operation; SAR aircraft set the assigned-mode flag), §3.3.7.2.2 Tables 18/19 (SOTDMA) and §3.3.7.3.2 Table 20 (ITDMA).
* [M.585-9] Annex 1 Section 3 — aircraft identities `111MIDXXX`, the `111MID1XX`/`111MID5XX` example scheme, the `111MID000` group identity.
* [MSC.140(76)] protection of the AIS VHF data link.
* [AIVDM] "Type 9: Standard SAR Aircraft Position Report".
* Chapter 3 (messages 1–3, communication states), chapter 11 (interrogation and assignment of aircraft), chapter 14 (message 18, the other selector-bit message), chapter 16 (synthetic examples and noise validation).
* Appendix B for the six-bit armour table used in §10.3.
