# 15. Message 21: aids to navigation

> **What you will learn**
> * What an AIS aid-to-navigation (AtoN) station is, how it differs from a
>   ship's transponder, and why the same message 21 serves a lighthouse, a
>   buoy, a wreck marker that does not physically exist, and a NOAA shore
>   station broadcasting whale notices.
> * Every field of message 21 [M.1371-5 §3.19 Table 73]: the 32 AtoN types,
>   the 20-character name and its 0–14 character extension, the
>   dimension/reference-point block, the off-position flag and the eight
>   "AtoN status" bits that ITU leaves to IALA and national authorities.
> * A message with an optional **variable-length** tail: how to find the
>   end of the name extension from the payload length, and what the
>   byte-boundary padding rule means for your decoder.
> * How to read the four decoders' handling of a field that *may not be
>   there*, and the small differences between `min()` in Python, C++, Rust
>   and TypeScript.
> * What a real AtoN unit does on the bench: a serial number masquerading as
>   an MMSI, `!ANVDO` with channel codes `X` and `Y`, an off-position flag
>   that is *correct* because the buoy's configured position is 130 km away,
>   and time stamps 15 seconds ahead of UTC.

## 15.1 Where this fits

Everything before this chapter was about *ships* (and the base stations
that control them). Message 21 is the voice of the **aid to navigation**:
lighthouses, beacons, buoys, offshore platforms, and increasingly things that
exist only as data. IALA, the body that standardises aids to navigation,
defines an AtoN as "a device or system external to vessels designed and
operated to enhance safe and efficient navigation of vessels and/or vessel
traffic" — a definition ITU quotes verbatim in the message description
[M.1371-5 §3.19].

An AIS AtoN station transmits message 21 autonomously every **3 minutes**
[M.1371-5 Annex 1 §4.2.1 Table 2], or at a rate set by a base station's
assignment command (message 16, chapter 11), and must retransmit at once
whenever a parameter changes [M.1371-5 §3.19]. The message always occupies
two slots: 272 bits without a name extension, up to 360 with one
[M.1371-5 Table 73].

Three kinds of station send it [IALA R0126 §4.2], [IALA G1062 §6]:

| Kind | What exists | Who transmits | Virtual flag | MMSI (recommended) |
|---|---|---|---|---|
| **Physical** (ITU: "real") | a buoy or beacon with an AIS unit on it | the unit on the aid | 0 | `99MID1xxx` |
| **Synthetic** | a physical aid *without* its own AIS; "monitored" if something confirms its state, "predicted" if not | a base station or another AtoN station elsewhere | 0, repeat indicator ≠ 0 | `99MID1xxx` |
| **Virtual** | nothing — "a digital information object" | a base station or AtoN station | 1 | `99MID6xxx` |

The maritime mobile service identity (MMSI) format is set by ITU: `99` + the three-digit maritime identification
digits (MID) of the licensing country + four digits, i.e. 10 000 identities
per MID [M.585-9 Annex 1 §4]. The sixth digit is *recommended* (not
required) to distinguish physical (`1`), virtual (`6`) and mobile (`8`) aids
[M.585-9 Annex 1 §4 ¶4], [IALA R0126 §4.3.1 Table 2]. US MIDs are 338 and
366–369, so a US virtual aid would be `993676xxx`. You will meet two
exceptions to the rule in this chapter's data: a coast-station style
`003669739` that the USCG assigned to a NOAA shore installation, and a bare
serial number.

Physical AtoN stations come in three hardware classes [IALA G1062 §6.1],
[IEC 62320-1/-2/-3]: **Type 1** transmit-only (fixed-access TDMA, FATDMA, slots reserved for it
by a base station's message 20, chapter 13), **Type 2** with a receiver for
remote configuration only, and **Type 3** with a full receiver that can use random-access TDMA,
RATDMA, like a ship and respond to interrogation (message 15, chapter 11). All
three sleep between transmissions to save battery [IALA G1062 §6.1].

### Operational use

* **On the bridge**, message 21 draws a diamond at the aid's position — solid
  for a physical or synthetic aid, dashed for a virtual one [IALA R0126 §4.2]
  — labelled with the name, and in good ECDIS implementations also flagged
  when `off_position` is set. A lighthouse with EPFD (electronic position fixing device) type 7 ("surveyed") doubles
  as a radar reference target: the mariner can compare the radar echo with
  the AIS position to check the radar's alignment [M.1371-5 Table 73],
  [IALA G1062 §3].
* **Marking new dangers.** When a wreck happens, a virtual AtoN can be on the
  chart within hours — long before a buoy tender arrives — and is removed
  when the physical mark is laid. IALA treats this as the primary use of
  virtual aids and limits "temporary" use to about six months
  [IALA R0143 §5.1], [IALA R0126 §4.2.3].
* **Buoy position verification.** Every 3 minutes the authority gets a GNSS
  fix from each buoy; if the fix leaves the "guard zone" around the charted
  position, the station sets the off-position flag and the authority (and
  every ship in range) knows the buoy has dragged [IALA R0126 §4.8.1].
* **Hazard broadcasts.** A station that detects it is off position or
  malfunctioning may also send a safety-related broadcast, message 14
  (chapter 12), at the authority's discretion [M.1371-5 §3.19 note].

### Unattended / automated uses

An AtoN is unmanned *by definition*, so almost every consumer of message 21
is a machine. This is where the message's quirks matter most.

1. **Health monitoring through the status bits.** ITU reserves the 8 status
   bits with no meaning beyond "0 = default" [M.1371-5 Table 73]. IALA fills
   them in: the top three bits select a *page*, and page 7 (`111`) packs
   RACON state (2 bits), main-light state (2 bits) and a one-bit health
   alarm into the remaining five [IALA R0126 §4.8.4 Figure 3]. The point is
   explicitly to let a monitoring centre read "light failed" from the
   broadcast every ship already receives, *without* polling the buoy with an
   addressed message 6 and loading the VHF data link
   [IALA R0126 §4.8.4 note 2]. The alarm bit is deliberately coarse: a low
   battery must *not* set it unless an AtoN function has actually failed
   [IALA R0126 §4.8.4 note 7c] — telemetry is for the authority, the alarm
   is for the mariner. Page `101` carries mobile-AtoN movement codes and
   page `001` is reserved for regional (inland waterway) use
   [IALA R0126 §4.8.4 Figures 4–5]. Our decoder interprets page 7 and
   reports the page number for the others.
2. **Missing-report alarms.** Because the interval is fixed at 3 minutes, a
   shore network can alarm on *silence*: no message 21 from buoy X for 10
   minutes means a dead battery, a sunk buoy or a jammed channel. In
   `sample-data/2010-11-l3-aton/ccom-sr162g-2010-09.aivdm` the gaps between
   the 429 reports are 180 s (340 times), 360 s (86 times), once 540 s and
   once 48 minutes — the 360 s
   gaps are reports the monitor receiver *missed*, one bench-test away from
   a false alarm. The 2024 USCG safety alert on VHF keying blocking AIS
   reception names "Aid to Navigation reports" as a casualty of exactly
   this effect [Safety Alerts, Safety Alert 01-24].
3. **Buoy telemetry proper.** Battery and solar voltage, lamp current,
   mooring tension and the like go in an addressed binary message 6
   (chapter 8) to the authority's MMSI, ideally sent right after message 21
   so the unit wakes only once [IALA R0126 §4.4.2.1]; met/hydro sensors on
   a buoy use broadcast message 8 (chapters 8–9) [IALA G1062 §6.1].
4. **The temptation of the name field.** Twenty characters of free text,
   repeated every three minutes and shown on every chart plotter, are hard
   to resist. IALA asks for *short* names that do not duplicate other
   fields and warns that not all shipborne equipment displays the 14
   character extension [IALA R0126 §4.8.2]. In practice the AtoN name and
   the 8 status bits are where undocumented telemetry ends up (battery
   voltage, a site code, a software version). A decoder should pass the
   name through untouched and make the raw status byte available — which is
   why we output `aton_status_bits` as a string of eight `0`/`1` characters
   next to its integer value.
5. **Non-AtoN uses of an AtoN message.** The NOAA/USCG right-whale notice
   station whose output fills this chapter's data file was registered with
   the USCG *as an AtoN* [AtoN form v4]: a shore computer in the Cape Cod
   National Seashore visitor centre, type of AtoN 0 ("not in the water and
   not fixed to any reference point"), name `NOAA WHALE NOTICES`, EPFD 1
   (GPS), and message 8 with designated area code (DAC) 366 every 5 minutes for ten acoustic-buoy
   zones (the form says function identifier (FI) 22; what is on the air, including line 3 of the
   same recording, is FI 34 — chapter 9). Message 21 was the station's *identity beacon*, so that
   anyone seeing the whale notices could find out who sent them. Fishing-gear
   markers, private moorings and race marks have since followed the same
   path with much less paperwork; a monitoring system must therefore
   expect message 21 from MMSIs that are not `99…` and types that are 0.
6. **Spoofing and data mining.** A virtual AtoN is just a message; anyone
   with a transmitter can put a "safe water" mark anywhere. Consumers should
   cross-check the source: a virtual aid should arrive with repeat
   indicator 0 from an authority's base station [M.1371-5 Annex 2 §4.6.1.2],
   its MMSI should be registered, and a physical aid's EPFD should not
   change from report to report. Conversely, long archives of message 21
   are a free, world-wide inventory of marked hazards and of buoy drift
   statistics.

## 15.2 The specification

Zero-based bit offsets; lengths from [M.1371-5 Table 73]. The message is
272 bits plus an optional extension of up to 88 bits (84 of text and up to 6
of padding, with the constraint that the total ends on a byte boundary, see
below).

| Bits | Len | Field | Type / units | Not available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 21 [M.1371-5 Table 73] |
| 6–7 | 2 | Repeat indicator | uint | — | 0 default, 3 = do not repeat [M.1371-5 Table 73]; non-zero when a base station transmits on behalf of a synthetic or virtual aid [M.1371-5 Annex 2 §4.6.1.2] |
| 8–37 | 30 | MMSI | uint | — | `99MIDxxxx` for AtoN [M.585-9 Annex 1 §4] |
| 38–42 | 5 | Type of AtoN | enum, Table 15.1 | 0 | [M.1371-5 Table 73, Table 74] |
| 43–162 | 120 | Name | 20 × six-bit ASCII | all `@` | [M.1371-5 Table 73], alphabet in [M.1371-5 Table 47] |
| 163 | 1 | Position accuracy | flag | 0 | 1 = high (≤ 10 m), determined as in message 1 [M.1371-5 Table 73, Table 50] |
| 164–191 | 28 | Longitude | sint, 1/10 000 min | 181° = `0x6791AC0` | East positive [M.1371-5 Table 73] |
| 192–218 | 27 | Latitude | sint, 1/10 000 min | 91° = `0x3412140` | North positive [M.1371-5 Table 73] |
| 219–227 | 9 | Dimension A | uint, m | 0 | reference point to "bow"; for fixed and virtual aids A points true north [M.1371-5 Table 73 note 1, Fig. 41] |
| 228–236 | 9 | Dimension B | uint, m | 0 | to "stern" [M.1371-5 Fig. 41] |
| 237–242 | 6 | Dimension C | uint, m | 0 | to port [M.1371-5 Fig. 41] |
| 243–248 | 6 | Dimension D | uint, m | 0 | to starboard [M.1371-5 Fig. 41] |
| 249–252 | 4 | Type of EPFD | enum, as message 4 | 0 | 7 = surveyed: "for fixed AtoN and virtual AtoN, the charted position should be used" [M.1371-5 Table 73] |
| 253–258 | 6 | Time stamp | uint, UTC second | 60 | 61 manual input, 62 dead reckoning, 63 positioning system inoperative [M.1371-5 Table 73] |
| 259 | 1 | Off-position indicator | flag | 0 | floating aids only; valid only if time stamp ≤ 59 [M.1371-5 Table 73 note] |
| 260–267 | 8 | AtoN status | bits | `00000000` | "reserved for the indication of the AtoN status" [M.1371-5 Table 73]; IALA page layout [IALA R0126 §4.8.4] |
| 268 | 1 | RAIM flag | flag | 0 | receiver autonomous integrity monitoring [M.1371-5 Table 73] |
| 269 | 1 | Virtual AtoN flag | flag | 0 | 1 = "does not physically exist" [M.1371-5 Table 73] |
| 270 | 1 | Assigned mode flag | flag | 0 | 1 = operating under a message 16 assignment [M.1371-5 Table 73] |
| 271 | 1 | Spare | — | 0 | [M.1371-5 Table 73] |
| 272– | 0, 6, …, 84 | Name extension | 0–14 × six-bit ASCII | *omitted* | "only the required number of characters should be transmitted, i.e. no @-character should be used" [M.1371-5 Table 73] |
| after | 0, 2, 4 or 6 | Spare | — | 0 | "used only when … Extension is used … adjusted in order to observe byte boundaries" [M.1371-5 Table 73] |

**Table 15.1 — Type of aid to navigation** [M.1371-5 Table 74]. The grouping
headings are the spec's own; GPSD's "1–15 fixed, 16–31 floating" [AIVDM
"Type 21"] does not match the table. Code 4 was "spare" in M.1371-4 and
became the emergency wreck marking buoy in M.1371-5. Text in the right
column is what our decoders print as `aton_type_text`.

| Code | Spec definition | `aton_type_text` |
|---|---|---|
| 0 | Default, type of AtoN not specified | `not specified (default)` |
| 1 | Reference point | `reference point` |
| 2 | RACON | `RACON` |
| 3 | Fixed structures off-shore, such as oil platforms, wind farms (an obstruction fitted with an AtoN AIS station) | `fixed structure off-shore` |
| 4 | Emergency wreck marking buoy | `emergency wreck marking buoy` |
| *Fixed AtoN* | | |
| 5 | Light, without sectors | `light, without sectors` |
| 6 | Light, with sectors | `light, with sectors` |
| 7 | Leading light front | `leading light front` |
| 8 | Leading light rear | `leading light rear` |
| 9–12 | Beacon, cardinal N / E / S / W | `beacon, cardinal N` … `W` |
| 13 | Beacon, port hand | `beacon, port hand` |
| 14 | Beacon, starboard hand | `beacon, starboard hand` |
| 15 | Beacon, preferred channel port hand | `beacon, preferred channel port hand` |
| 16 | Beacon, preferred channel starboard hand | `beacon, preferred channel starboard hand` |
| 17 | Beacon, isolated danger | `beacon, isolated danger` |
| 18 | Beacon, safe water | `beacon, safe water` |
| 19 | Beacon, special mark | `beacon, special mark` |
| *Floating AtoN* | | |
| 20–23 | Cardinal mark N / E / S / W | `cardinal mark N` … `W` |
| 24 | Port hand mark | `port hand mark` |
| 25 | Starboard hand mark | `starboard hand mark` |
| 26 | Preferred channel port hand | `preferred channel port hand` |
| 27 | Preferred channel starboard hand | `preferred channel starboard hand` |
| 28 | Isolated danger | `isolated danger` |
| 29 | Safe water | `safe water` |
| 30 | Special mark | `special mark` |
| 31 | Light vessel / LANBY / rigs | `light vessel / LANBY / rig` |

Whether an aid is *lighted* is not encoded here; the spec notes the
ambiguity and suggests authorities use the status bits for it
[M.1371-5 Table 74 note 2].

### The fields that need explaining

**Name and name extension.** The 20-character name is padded with `@` like
every other AIS text field. If the name is longer than 20 characters, the
station appends the *remaining* characters as the extension — the spec's
example of 34 characters in total is the whole point of the two-slot
message [M.1371-5 Table 73]. The extension is sent without any `@` padding
and *omitted entirely* when not needed, so the receiver learns its length
only from the payload length: `extension_bits = payload_bits − 272`. Then
0, 2, 4 or 6 spare bits are appended so that the whole message ends on a
byte boundary — the AIS transport layer works in bytes, not bits
[M.1371-5 Table 73]. Two consequences for a decoder:

* Decode `floor(extension_bits / 6)` characters, never more; the 2 or 4
  left-over bits are padding.
* Six padding bits are indistinguishable from a `@` character. If a station
  pads with 6, your decoder will see one extra `@`, which the usual
  cut-at-`@` rule removes anyway. We therefore print the raw
  `name_extension_bits` and leave the arithmetic to the reader.

GPSD describes the field as "272–360, 88 bits" [AIVDM "Type 21"]; the 88 is
84 + a 4-bit remainder and is a *maximum*, not a length.

**Dimensions / reference point.** The 30 bits are laid out exactly like
message 5's dimension block — A (9 bits), B (9), C (6), D (6), distances in
metres from the reported position to the four edges [M.1371-5 §3.3.3
Fig. 41]. For an AtoN the "bow" direction A is true north for fixed aids,
virtual aids and offshore structures; a floating aid larger than 2 m × 2 m
reports a circle, A = B = C = D ≠ 0 (its orientation is unknown); an object
of 2 m × 2 m or less reports A = B = C = D = 1; a virtual aid or a
"reference point" reports all zeros [M.1371-5 Table 73 notes 1–2],
[IALA R0126 §4.8.3]. These are the dimensions of the aid *itself*, not of
the guard zone or danger area around it [IALA R0126 §4.8.3]. Our decoder
outputs the block with the keys of chapter 7 — `dim_bow_m`, `dim_stern_m`,
`dim_port_m`, `dim_starboard_m`, `length_m`, `beam_m`,
`dimensions_available`, `reference_point_available` — where "bow" is to be
read as "true north" for a fixed or virtual aid.

**Time stamp and EPFD.** As in message 1: the UTC second when the position
was generated, 60 if unavailable, 61/62/63 for manual, dead-reckoning and
inoperative positioning [M.1371-5 Table 73]. Fixed and virtual aids are
expected to report the *charted* (surveyed) position with EPFD 7
[M.1371-5 Table 73], [IALA R0126 §4.8.6]; with no live fix behind it the
time stamp is then 60, "not available" [M.1371-5 Table 73]. A floating aid
reports the live GNSS fix.

**Off-position indicator.** Set when the station's GNSS position is outside
the guard zone configured at installation (mooring length + water depth +
positioning error + tolerance) [M.1371-5 Table 73], [IALA R0126 §4.8.1].
Neither ITU nor IEC 62320-2 prescribes the algorithm; IALA asks that a
single bad fix never set the flag [IALA R0126 §4.8.1]. The spec says the
receiver should consider the flag **only** for a floating aid **and** only
when the time stamp is ≤ 59 [M.1371-5 Table 73 note] — a flag computed
from an inoperative GNSS is meaningless. We implement that rule as
`off_position_valid`, taking "floating" to be Table 74 codes 20–31 plus
code 4 (the emergency wreck marking *buoy*, listed above the "Floating
AtoN" heading but a buoy nonetheless).

**AtoN status, IALA page 7.** With the most significant bit numbered 8
(IALA's convention) the layout is `111 RR LL H` [IALA R0126 §4.8.4
Figure 3]:

| Bits (IALA numbering) | Field | Values |
|---|---|---|
| 8–6 | page id | `111` = page 7, `000` = no data (default), `101` = mobile AtoN, `001` = regional |
| 5–4 | RACON | 0 none installed, 1 installed not monitored, 2 operational, 3 error (built-in test or power failure) |
| 3–2 | Light | 0 no light or no monitoring, 1 on, 2 off, 3 fail or reduced range |
| 1 | Health | 0 good, 1 alarm — any AtoN device failed |

In our zero-based terms: page = `status >> 5`, RACON = `(status >> 3) & 3`,
light = `(status >> 1) & 3`, health = `status & 1`.

**Assigned mode.** 1 when a base station has put the station into assigned
mode with message 16 (a different reporting interval or fixed slots)
[M.1371-5 Table 73].

## 15.3 Decoding one message by hand

Line 8 of `data/ch15-aton-reports.nmea`, which is line 1 of
`sample-data/2010-11-l3-aton/ccom-sr162g-2010-09.aivdm`:

```text
ccom-sr162g-2010-09-15:!AIVDM,1,1,,B,E03Ovrh77PPh;T0V2h77b4QRaP0MM`Lp<Em<000000V@20,4*4B,rccom-office,1284576313.39
```

The `ccom-sr162g-2010-09-15:` prefix is a file name left by `grep`; our
sentence parser discards everything before the `!` (chapter 1). The payload
has 46 characters and 4 fill bits: 46 × 6 − 4 = **272 bits**, the minimum
for message 21 — so there is no name extension. The six-bit groups
(armouring table in Appendix B):

```text
E      0      3      O      v      r      h      7      7      P      P      h
010101 000000 000011 011111 111110 111010 110000 000111 000111 100000 100000 110000
bits 0-71

;      T      0      V      2      h      7      7      b      4      Q      R
001011 100100 000000 100110 000010 110000 000111 000111 101010 000100 100001 100010
bits 72-143

a      P      0      M      M      `      L      p      <      E      m      <
101001 100000 000000 011101 011101 101000 011100 111000 001100 010101 110101 001100
bits 144-215

0      0      0      0      0      0      V      @      2      0
000000 000000 000000 000000 000000 000000 100110 010000 000010 000000
bits 216-275 (the last four bits are fill and are dropped)
```

Reading the fields at their offsets:

| Bits | Field | Binary | Value |
|---|---|---|---|
| 0–5 | message id | `010101` | 21 |
| 6–7 | repeat | `00` | 0 |
| 8–37 | MMSI | `000000001101111111111011101011` | 3669739 → `003669739`, a coast-station format (`00` + MID 366) |
| 38–42 | AtoN type | `00000` | 0, not specified |
| 43–162 | name | `001110 001111 000001 000001 100000 010111 001000 000001 001100 000101 100000 001110 001111 010100 001001 000011 000101 010011 000000 000000` | N O A A ␣ W H A L E ␣ N O T I C E S @ @ → `NOAA WHALE NOTICES` |
| 163 | position accuracy | `1` | high |
| 164–191 | longitude | `1101011101101000011100111000` | sign bit set: 225 871 672 − 2²⁸ = −42 563 784 → ÷ 600 000 = **−70.939640°** |
| 192–218 | latitude | `001100010101110101001100000` | 25 881 184 ÷ 600 000 = **43.135307°** |
| 219–248 | A, B, C, D | all zero | 0, 0, 0, 0 m — no dimensions, "reference point" style |
| 249–252 | EPFD | `0001` | 1 = GPS |
| 253–258 | time stamp | `001100` | 12 |
| 259 | off position | `1` | **set** |
| 260–267 | AtoN status | `00000000` | 0, page 0 |
| 268 | RAIM | `1` | in use |
| 269 | virtual | `0` | physical |
| 270 | assigned | `0` | autonomous |
| 271 | spare | `0` | |

43.1353° N, 70.9396° W is the CCOM building at the University of New
Hampshire in Durham — the unit was on a desk. The off-position bit is set
because the unit had been *configured* with its intended deployment
position on Cape Cod (42.074048° N, 70.205338° W on the registration form
[AtoN form v4]; the unit's own debug console in
`sample-data/2010-11-l3-aton/ccom-l3-2-2010-11-02` line 10 prints
`Cfg Lat/Lon [42 4.4428 N, 70 12.3202 W]`). Durham is about 130 km from
there, far outside any guard zone, so the unit was right. Note, though,
that with AtoN type 0 the spec tells a *receiver* to ignore the flag — our
decoder reports `off_position: true, off_position_valid: false`.

The USCG metadata after the checksum (chapter 5) says the SR162G receiver
`rccom-office` logged it at 1284576313.39, i.e. 2010-09-15 18:45:13 UTC;
the time stamp in the message says the fix was taken at second 12. Three
minutes later (line 9 of the data file) the next report arrived, on the
other channel.

## 15.4 The code

All four decoders share the shape of the message-1 decoders from chapter 3.
The new elements are (a) a 32-entry type table, (b) the variable-length
tail, (c) a second small function, `decode_status`, that adds keys only
when the IALA page is 7, and (d) two booleans derived from other fields
(`off_position_valid`, `position_available`). The dimension block is the
one part that is *not* new: it is decoded by the `decode_dimensions` helper
of chapter 14, so that message 21 prints the same eight dimension keys as
messages 5, 19 and 24B.

### Python

`code/python/aistut/msg_21.py` (tables omitted — see Table 15.1):

```python
MIN_BITS = 272            # Table 73: the fixed part (two slots)
MAX_EXTENSION_CHARS = 14  # Table 73: "up to 14 additional 6-bit-ASCII characters"


def is_floating(aton_type: int) -> bool:
    """True for the Table 74 codes that describe a floating aid (a buoy)."""
    return aton_type == 4 or 20 <= aton_type <= 31


def decode_status(status: int) -> dict:
    """Interpret the 8 AtoN status bits (IALA R0126 §4.8.4)."""
    page = status >> 5
    out: dict = {"aton_status_page": page}
    if page == 7:
        racon = (status >> 3) & 0x3
        light = (status >> 1) & 0x3
        out["racon_status"] = racon
        out["racon_status_text"] = RACON_STATUS[racon]
        out["light_status"] = light
        out["light_status_text"] = LIGHT_STATUS[light]
        out["health_alarm"] = (status & 0x1) == 1
    return out


def decode(bits: Bits) -> dict:
    """Decode a message 21. Raises DecodeError if shorter than 272 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), MIN_BITS)
    aton_type = bits.uint(38, 5)
    lon_raw = bits.sint(164, 28)
    lat_raw = bits.sint(192, 27)
    epfd = bits.uint(249, 4)
    timestamp = bits.uint(253, 6)
    status = bits.uint(260, 8)

    # --- the variable-length name extension -------------------------------
    # Table 73: "0, 6, 12, … 84" bits of six-bit characters followed by 0, 2,
    # 4 or 6 spare bits so that the message ends on a byte boundary. Only
    # whole characters are decoded; a partial character is ignored.
    extension_bits = len(bits) - MIN_BITS
    extension_chars = min(extension_bits // 6, MAX_EXTENSION_CHARS)
    name = bits.text(43, 120)
    name_extension = bits.text(MIN_BITS, extension_chars * 6)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "aton_type": aton_type,
        "aton_type_text": aton_type_text(aton_type),
        "name": name,
        "name_extension": name_extension,
        "name_extension_bits": extension_bits,
        "full_name": name + name_extension,
        "position_accuracy": bits.uint(163, 1),
        "lon": Fixed(lon_raw / 600000.0, 6),
        "lat": Fixed(lat_raw / 600000.0, 6),
        "position_available": lon_raw != 0x6791AC0 and lat_raw != 0x3412140,
        "epfd": epfd,
        "epfd_text": epfd_text(epfd),
        "timestamp": timestamp,
        "off_position": bits.flag(259),
        # Table 73 note 1: the flag means something only for a floating aid
        # whose time stamp is a real second (0–59).
        "off_position_valid": is_floating(aton_type) and timestamp <= 59,
        "aton_status": status,
        "aton_status_bits": bits.raw(260, 8),
        "raim": bits.flag(268),
        "virtual_aton": bits.flag(269),
        "assigned_mode": bits.flag(270),
        "spare": bits.uint(271, 1),
    }
    # Bits 219-248: the same 30-bit A/B/C/D block as messages 5, 19 and 24B,
    # with the same keys (dim_bow_m … beam_m) — one definition in msg_19.py.
    out.update(decode_dimensions(bits, 219))
    out.update(decode_status(status))
    return out
```

Three things to notice. `bits.text(start, 0)` is legal and returns `""`, so
the no-extension case needs no special branch. `Bits.text` already
tolerates a length that is not a multiple of six (it only reads whole
characters), but we compute the whole-character count ourselves so that the
C++, Rust and TypeScript versions do not have to rely on the same
tolerance. And the dimension block is not decoded here at all: the module
imports `decode_dimensions` from the message 19 module of chapter 14 (`from
.msg_19 import decode_dimensions`), which outputs `dim_bow_m`,
`dim_stern_m`, `dim_port_m`, `dim_starboard_m`, `dimensions_available`,
`reference_point_available`, `length_m` and `beam_m` exactly as messages 5,
19 and 24B do. For an AtoN "bow" means true north (§15.2), `length_m` is the
north–south extent and `beam_m` the east–west one, and
`reference_point_available` is false for the all-zero block a virtual aid or
reference point sends.

> **Language notes (Python).** `a // b` is floor division; `min(a, b)`
> works on any comparable values. `out.update(other)` merges dictionaries
> (seen in chapter 3). Nothing else here is new.

### C++

`code/cpp/msg_21.hpp`:

```cpp
constexpr std::size_t MSG21_MIN_BITS = 272;            // Table 73: the fixed part (two slots)
constexpr std::size_t MSG21_MAX_EXTENSION_CHARS = 14;  // "up to 14 additional 6-bit-ASCII characters"

inline bool is_floating_aton(std::uint64_t aton_type) {
    return aton_type == 4 || (20 <= aton_type && aton_type <= 31);
}

inline Json decode_aton_status(std::uint64_t status) {
    std::uint64_t page = status >> 5;
    Json out = Json::Object{{"aton_status_page", page}};
    if (page == 7) {
        std::uint64_t racon = (status >> 3) & 0x3;
        std::uint64_t light = (status >> 1) & 0x3;
        out["racon_status"] = racon;
        out["racon_status_text"] = RACON_STATUS[racon];
        out["light_status"] = light;
        out["light_status_text"] = LIGHT_STATUS[light];
        out["health_alarm"] = (status & 0x1) == 1;
    }
    return out;
}

inline Json decode_msg_21(const Bits& bits) {
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), MSG21_MIN_BITS);
    std::uint64_t aton_type = bits.uint(38, 5);
    std::int64_t lon_raw = bits.sint(164, 28);
    std::int64_t lat_raw = bits.sint(192, 27);
    std::uint64_t epfd = bits.uint(249, 4);
    std::uint64_t timestamp = bits.uint(253, 6);
    std::uint64_t status = bits.uint(260, 8);

    std::size_t extension_bits = bits.size() - MSG21_MIN_BITS;
    std::size_t extension_chars = std::min(extension_bits / 6, MSG21_MAX_EXTENSION_CHARS);
    std::string name = bits.text(43, 120);
    std::string name_extension = bits.text(MSG21_MIN_BITS, extension_chars * 6);

    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"aton_type", aton_type},
        {"aton_type_text", aton_type_text(aton_type)},
        {"name", name},
        {"name_extension", name_extension},
        {"name_extension_bits", extension_bits},
        {"full_name", name + name_extension},
        {"position_accuracy", bits.uint(163, 1)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600000.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600000.0, 6)},
        {"position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140},
        {"epfd", epfd},
        {"epfd_text", epfd_text(static_cast<int>(epfd))},
        {"timestamp", timestamp},
        {"off_position", bits.flag(259)},
        {"off_position_valid", is_floating_aton(aton_type) && timestamp <= 59},
        {"aton_status", status},
        {"aton_status_bits", bits.raw(260, 8)},
        {"raim", bits.flag(268)},
        {"virtual_aton", bits.flag(269)},
        {"assigned_mode", bits.flag(270)},
        {"spare", bits.uint(271, 1)},
    };
    out.update(decode_dimensions(bits, 219));   // from msg_19.hpp
    out.update(decode_aton_status(status));
    return out;
}
```

> **Language notes (C++).**
> * `#include "msg_19.hpp"` brings in `decode_dimensions`. The header is
>   also included by `msg_24.hpp` and by the driver; `#pragma once` and
>   `inline` make that harmless (chapter 14).
> * `std::min(a, b)` is a template that deduces *one* type for both
>   arguments. `bits.size()` is a `std::size_t`, so the constant must be a
>   `std::size_t` as well — `std::min(extension_bits / 6, 14)` would fail to
>   compile because `14` is an `int`. This is the single most common reason
>   `std::min`/`std::max` refuse to compile.
> * `constexpr` declares a compile-time constant with a type; prefer it to
>   `#define`, which has no type and no scope.
> * `std::array<const char*, 32>` indexed by a 5-bit value can never go out
>   of range, so no bounds check is needed — the same reasoning as the
>   16-entry navigation-status table in chapter 3.
> * `status` is kept unsigned (`std::uint64_t`) so that `>>` is a logical
>   shift; shifting a negative *signed* value right was implementation-
>   defined before C++20 (C++20 pins it down as arithmetic), and an
>   unsigned type still documents that these are bit flags, not a number.

### Rust

`code/rust/src/msg_21.rs`:

```rust
pub const MIN_BITS: usize = 272;
pub const MAX_EXTENSION_CHARS: usize = 14;

pub fn is_floating(aton_type: u64) -> bool {
    matches!(aton_type, 4 | 20..=31)
}

pub fn decode_status(status: u64) -> Obj {
    let page = status >> 5;
    let mut out = Obj::new();
    put(&mut out, "aton_status_page", page);
    if page == 7 {
        let racon = (status >> 3) & 0x3;
        let light = (status >> 1) & 0x3;
        put(&mut out, "racon_status", racon);
        put(&mut out, "racon_status_text", racon_status_text(racon));
        put(&mut out, "light_status", light);
        put(&mut out, "light_status_text", light_status_text(light));
        put(&mut out, "health_alarm", (status & 0x1) == 1);
    }
    out
}

pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), MIN_BITS)?;
    let aton_type = bits.uint(38, 5);
    let lon_raw = bits.sint(164, 28);
    let lat_raw = bits.sint(192, 27);
    let epfd = bits.uint(249, 4);
    let timestamp = bits.uint(253, 6);
    let status = bits.uint(260, 8);

    let extension_bits = bits.len() - MIN_BITS;
    let extension_chars = usize::min(extension_bits / 6, MAX_EXTENSION_CHARS);
    let name = bits.text(43, 120);
    let name_extension = bits.text(MIN_BITS, extension_chars * 6);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "aton_type", aton_type);
    put(&mut out, "aton_type_text", aton_type_text(aton_type));
    put(&mut out, "full_name", format!("{name}{name_extension}"));
    put(&mut out, "name", name);
    put(&mut out, "name_extension", name_extension);
    put(&mut out, "name_extension_bits", extension_bits);
    put(&mut out, "position_accuracy", bits.uint(163, 1));
    put(&mut out, "lon", Json::fixed(lon_raw as f64 / 600000.0, 6));
    put(&mut out, "lat", Json::fixed(lat_raw as f64 / 600000.0, 6));
    put(&mut out, "position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140);
    out.extend(decode_dimensions(bits, 219));   // use crate::msg_19::decode_dimensions
    put(&mut out, "epfd", epfd);
    put(&mut out, "epfd_text", epfd_text(epfd));
    put(&mut out, "timestamp", timestamp);
    put(&mut out, "off_position", bits.flag(259));
    put(&mut out, "off_position_valid", is_floating(aton_type) && timestamp <= 59);
    put(&mut out, "aton_status", status);
    put(&mut out, "aton_status_bits", bits.raw(260, 8));
    put(&mut out, "raim", bits.flag(268));
    put(&mut out, "virtual_aton", bits.flag(269));
    put(&mut out, "assigned_mode", bits.flag(270));
    put(&mut out, "spare", bits.uint(271, 1));
    out.extend(decode_status(status));
    Ok(Json::Obj(out))
}
```

> **Language notes (Rust).**
> * `matches!(x, 4 | 20..=31)` expands to a `match` that returns `true` for
>   the listed patterns; `20..=31` is an inclusive range pattern (`20..31`
>   would exclude 31).
> * **Ownership shows up in the order of the `put` calls.** `full_name` is
>   built first with `format!("{name}{name_extension}")` (inline format arguments), which only
>   *borrows* the two strings. Had we written `name + &name_extension`, the
>   `+` would *move* `name` into the result and the later
>   `put(&mut out, "name", name)` would not compile ("use of moved value").
>   Python and TypeScript have no such concern because their strings are
>   shared by reference; C++ copies silently.
> * `usize::min(a, b)` is the same function as `a.min(b)`; written in the
>   prefix form it reads like the other three languages. As in C++ both
>   arguments must have the same type — `extension_bits / 6` is a `usize`
>   because `bits.len()` is.
> * `BTreeMap::extend` is `dict.update`.

### TypeScript

`code/typescript/src/msg_21.ts`:

```typescript
export const MIN_BITS = 272;
export const MAX_EXTENSION_CHARS = 14;

export function isFloating(atonType: number): boolean {
  return atonType === 4 || (20 <= atonType && atonType <= 31);
}

export function decodeStatus(status: number): JsonObject {
  const page = status >> 5;
  const out: JsonObject = { aton_status_page: page };
  if (page === 7) {
    const racon = (status >> 3) & 0x3;
    const light = (status >> 1) & 0x3;
    out["racon_status"] = racon;
    out["racon_status_text"] = RACON_STATUS[racon];
    out["light_status"] = light;
    out["light_status_text"] = LIGHT_STATUS[light];
    out["health_alarm"] = (status & 0x1) === 1;
  }
  return out;
}

export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, MIN_BITS);
  const atonType = bits.uint(38, 5);
  const lonRaw = bits.sint(164, 28);
  const latRaw = bits.sint(192, 27);
  const epfd = bits.uint(249, 4);
  const timestamp = bits.uint(253, 6);
  const status = bits.uint(260, 8);

  const extensionBits = bits.length - MIN_BITS;
  const extensionChars = Math.min(Math.floor(extensionBits / 6), MAX_EXTENSION_CHARS);
  const name = bits.text(43, 120);
  const nameExtension = bits.text(MIN_BITS, extensionChars * 6);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    aton_type: atonType,
    aton_type_text: atonTypeText(atonType),
    name,
    name_extension: nameExtension,
    name_extension_bits: extensionBits,
    full_name: name + nameExtension,
    position_accuracy: bits.uint(163, 1),
    lon: new Fixed(lonRaw / 600000.0, 6),
    lat: new Fixed(latRaw / 600000.0, 6),
    position_available: lonRaw !== 0x6791ac0 && latRaw !== 0x3412140,
    epfd,
    epfd_text: epfdText(epfd),
    timestamp,
    off_position: bits.flag(259),
    off_position_valid: isFloating(atonType) && timestamp <= 59,
    aton_status: status,
    aton_status_bits: bits.raw(260, 8),
    raim: bits.flag(268),
    virtual_aton: bits.flag(269),
    assigned_mode: bits.flag(270),
    spare: bits.uint(271, 1),
  };
  Object.assign(out, decodeDimensions(bits, 219));   // import from "./msg_19"
  Object.assign(out, decodeStatus(status));
  return out;
}
```

> **Language notes (TypeScript).**
> * JavaScript has one numeric type and `/` always divides as floating
>   point: `40 / 6` is `6.666…`, so `Math.floor` is needed where Python
>   uses `//` and C++/Rust get integer division for free. Forgetting it
>   would pass `6.666… * 6 = 40` bits to `text()`, which happens to work
>   here but is wrong in principle.
> * `Math.min` accepts any numbers; there is no type-matching problem as in
>   C++ and Rust.
> * `status >> 5` is safe because the value is 8 bits wide (the 32-bit
>   bitwise-operator pitfall from chapter 2 only bites above 31 bits).
> * `const ATON_TYPES: readonly string[]` is a read-only array type;
>   assigning to an element is a compile-time error, like `const` on a
>   `std::array` in C++.

## 15.5 Sample data and results

### Where the data comes from

Message 21 is rare in this tutorial's recordings: a scan of every file under
`sample-data/` finds it in exactly one directory, `2010-11-l3-aton/`, the
bench tests of an L-3 Communications AtoN station at CCOM in September and
November 2010 — the unit that was about to replace the NOAA right-whale
notice transmitter on Cape Cod [AtoN form v4]. Receivers in Great Bay and
New Castle (`tide1`, `tide3`) never heard a USCG AtoN, and the 2008 Boston
log in `RightwhaleAisProject/` contains none either. Consequently there is
**no recorded virtual AtoN, no name extension and no IALA status page**;
section 5 of the data file supplies synthetic examples, built as described
below and labelled as such.

`data/ch15-aton-reports.nmea` has six sections (comment lines in the file
name the source lines):

1. Three `!AIVDM` from the SR162G monitor receiver, September 2010
   (`ccom-sr162g-2010-09.aivdm` lines 1, 2, 4) — with the `file-name:`
   prefix that `grep -r` left behind.
2. Four `!ANVDO` from the AtoN's own NMEA port on 4 September 2010
   (`ccom-l3-3-2010-09-04` lines 9, 11, 24, 344), *before* the USCG had
   issued an MMSI.
3. The 2 November power-up sequence from the second NMEA port
   (`ccom-l3-4-2010-11-02` lines 7, 918, 919, 922, 926, 1030, 1031): a
   report with no GPS fix, the `$ANALR` "UTC Lost" alarm and `$ANADS`
   device-status sentence, then reports with a fix.
4. What the SR162G heard at the same moment (`ccom-sr162g-2010-11-02` lines
   1182, 1183, 1544), including the receiver's own stale `!AIVDO`.
5. Two synthetic sentences (virtual aid with name extension; floating aid
   with IALA status page 7).
6. A synthetic truncated sentence.

Run any of the four decoders:

```text
python3 code/python/decode.py data/ch15-aton-reports.nmea
code/cpp/build/decode        data/ch15-aton-reports.nmea
code/rust/target/release/decode data/ch15-aton-reports.nmea
node code/typescript/dist/decode.js data/ch15-aton-reports.nmea
```

The output (identical for all four, see
`tests/expected/ch15-aton-reports.jsonl`) has 20 objects. The hand-decoded
sentence, line 8, comes out as:

```json
{"ais":{"assigned_mode":false,"aton_status":0,"aton_status_bits":"00000000","aton_status_page":0,"aton_type":0,"aton_type_text":"not specified (default)","beam_m":0,"dim_bow_m":0,"dim_port_m":0,"dim_starboard_m":0,"dim_stern_m":0,"dimensions_available":false,"epfd":1,"epfd_text":"GPS","full_name":"NOAA WHALE NOTICES","lat":43.135307,"length_m":0,"lon":-70.939640,"mmsi":3669739,"msg_type":21,"name":"NOAA WHALE NOTICES","name_extension":"","name_extension_bits":0,"off_position":true,"off_position_valid":false,"position_accuracy":1,"position_available":true,"raim":true,"reference_point_available":false,"repeat_indicator":0,"spare":0,"timestamp":12,"virtual_aton":false},"channel":"B","fragments":1,"line":8,"own_ship":false,"payload_bits":272,"talker":"AI","uscg":{"station":"ccom-office","station_kind":"receiver","time":1284576313.39}}
```

### A serial number as MMSI (lines 16–19)

Two months earlier the same unit identified itself as MMSI **631850**, AtoN
type 3 `fixed structure off-shore`, name `L3_ATON`, position accuracy 0:

```json
"aton_type":3,"aton_type_text":"fixed structure off-shore","full_name":"L3_ATON","lat":43.135330,"lon":-70.939568,"mmsi":631850,…,"timestamp":10
```

631850 has six digits, not nine; it is not an MMSI at all but the unit's
**serial number** — the registration form lists "L3 Desktop ATON …
serial 000631850" [AtoN form v4]. Factory defaults on the air are a
recurring AIS theme (chapter 3 had MMSI 0 and 1193046); for an AtoN they
are worse, because a chart plotter will draw a "fixed structure" wherever
the unit is switched on. The `sample-data/2010-11-l3-aton/AGENTS.md` guide
says this file uses a `993672…` identity; that is not what the bits say.

Line 19 is the same unit without a GPS fix: `"lat":91.000000,"lon":181.000000,
"position_available":false,"timestamp":63` — the sentinels from the table,
with `off_position` back to `false` because nothing can be compared.

### Power-up, alarm, fix (lines 26–32)

| line | `channel` | `timestamp` | `position_available` | `off_position` | `aton_status` | `raim` | logged at |
|---|---|---|---|---|---|---|---|
| 26 | `X` | 63 | false | false | 1 | false | 1288708137.39 |
| 29 | `X` | 21 | true | true | 1 | true | 1288710426.57 |
| 30 | `Y` | 30 | true | true | 0 | true | 1288710435.57 |
| 31 | `Y` | 27 | true | true | 0 | true | 1288710552.59 |
| 32 | `B` | 28 | true | true | 0 | true | 1288710553.33 |

Between lines 26 and 29 the unit printed `$ANALR,000000.00,007,A,V,AIS: UTC
Lost` and `$ANADS,L3 AIS ID,,A,4,I,N` (lines 27–28; `ALR` is the IEC 61162-1
alarm sentence, `ADS` is an AIS device status sentence that is not among the
formatters in [IEC 61162-1 §8]; both are reported by our driver as
`"skipped":"no parser"`). The status byte is
`00000001` while the unit has no fix or has just acquired one, and
`00000000` afterwards. ITU gives that bit no meaning, IALA's page 0 means
"no data" — this is the manufacturer's own use of the field, and a good
illustration of why the raw bits must be preserved.

Line 32, sent on channel `B` at 1288710553.33, is the transmission that the
SR162G logged 60 ms later as line 38 (`"time":1288710553.39`): the `ais`
objects are byte-for-byte identical (`"lat":43.135227,"lon":-70.939447,
"timestamp":28`); only the envelope differs — `"talker":"AN","own_ship":true`
versus `"talker":"AI","own_ship":false` — and the checksum, because the
talker is part of it. Line 37 in between is the receiver's own `!AIVDO`, a
message 1 for MMSI 265000000 at 22.539202° N 114.066683° E: a factory
default position in Hong Kong from a GPS that never got a fix in the CCOM
office.

### The synthetic examples (lines 49–50)

Both were generated by a short script that assembles the Table 73 fields
into a bit string, armours it (chapter 2) and computes the checksum; no
decoder library was involved. The values were chosen to exercise what the
recordings lack:

*Line 49 — a virtual emergency wreck marking buoy with a 26-character name.*
MMSI 993676001 (`99` + MID 367 + `6` for virtual), type 4, name
`STELLWAGEN WRECK MAR` + extension `K ALFA`, position the Cape Cod
deployment site from the registration form, dimensions 0, EPFD 7
"surveyed", time stamp 60, virtual flag 1. Six extension characters are 36
bits, 272 + 36 = 308, so 4 spare bits bring the total to 312 = 39 bytes:

```json
"full_name":"STELLWAGEN WRECK MARK ALFA","name":"STELLWAGEN WRECK MAR","name_extension":"K ALFA","name_extension_bits":40,…,"epfd":7,"epfd_text":"surveyed","timestamp":60,"virtual_aton":true,"off_position":false,"off_position_valid":false
```

with `"payload_bits":312` in the envelope. Check: 40 bits → `40 // 6 = 6`
characters, 4 bits left over, exactly the padding we put in. Note that the
split falls in the middle of the word "MARK": the extension is a
continuation, not a second line.

*Line 50 — a physical special mark, off position, with status page 7.*
MMSI 993671002, type 30, dimensions 3/3/3/3 (a buoy approximated by a 6 m
circle), GPS fix at second 17, off-position set, status `11110111`:

```json
"aton_status":247,"aton_status_bits":"11110111","aton_status_page":7,"racon_status":2,"racon_status_text":"RACON operational","light_status":3,"light_status_text":"light fail or at reduced range","health_alarm":true,"off_position":true,"off_position_valid":true
```

This is the message a monitoring centre wants to see least: a buoy that has
dragged *and* whose light has failed.

### The truncated line (line 54)

Line 8's payload cut to 42 characters gives 252 bits:

```json
{"channel":"B","error":"message 21 too short: 252 bits < 272","fragments":1,"line":54,"msg_type":21,"own_ship":false,"payload_bits":252,"talker":"AI"}
```

All four decoders print the same error text, produced by `require_bits`
(chapter 3).

## 15.6 Pitfalls and real-world deviations

* **Channel codes `X` and `Y`.** The AtoN's `!ANVDO` sentences use `X` and
  `Y` in the channel field for the reports it emits every 3 s while idle,
  and `A`/`B` for the ones it actually transmitted (the `A`/`B` lines are
  the ones the SR162G received). IEC 61162-1 defines only `A`, `B`, `1`,
  `2` (and empty) [IEC 61162-1 §8 VDM]. Our `vdm` module passes the
  character through; a parser that validates the channel against the
  standard would have discarded the whole bench test.
* **The time stamp is 14–15 s ahead of the clock.** In the November log the
  reports logged at hh:mm:13 carry time stamps 27–28, and the one logged at
  hh:mm:06 carries 21; in September the stamps match the wall clock to the
  second (logged at :13.39, stamp 12). 15 s was the difference between GPS
  time and UTC in 2010. A GNSS receiver that has not yet received the
  leap-second parameters from the satellite almanac — up to 12.5 minutes
  after a cold start, longer with the poor indoor reception the `UTC Lost`
  alarms suggest — outputs GPS time, and the AtoN copied it into the field.
  A consumer matching `timestamp` against reception time to detect delayed
  repeats would reject every one of these.
* **Off-position on a non-floating type.** The unit set the flag with type
  0. It was factually right (the unit was 130 km from its configured
  position) but the spec tells receivers to ignore the flag unless the type
  is a floating aid — so a compliant display would have shown nothing.
  Authorities that run shore stations with type 0 lose this diagnostic;
  monitoring software should look at the raw flag anyway.
* **A coast-station MMSI on an AtoN.** 003669739 follows the `00MIDxxxx`
  coast-station format [M.585-9 Annex 1 §2], not `99MIDxxxx`. The USCG issued
  it because the "aid" was a shore computer; ITU allows a base station
  co-located with an AtoN station to use a base-station identity
  [M.585-9 Annex 1 §4 ¶2]. Do not filter message 21 by MMSI prefix.
* **Type 0 everywhere.** Both real identities in the data use type 0. A
  display that keys its symbol on the type will show a generic diamond; a
  data miner that counts "buoys" will miss them. The field is "regional
  defined set up by IALA" and voluntary in practice.
* **Name-extension length guessing.** Older decoders assumed the extension
  is always a multiple of 6 bits or always 88 bits; some stations pad with
  6 bits (which looks like a trailing `@`), some with the minimum. Compute
  from `payload_bits`, decode whole characters, cut at `@`, and never
  assume the total is 360.
* **Table numbering drift.** This message is Table 70/71 in M.1371-4 and
  Table 73/74 in M.1371-5 (message layout / type codes); the AtoN types were "Table 34bis" in IALA A-126
  edition 1.1 [AtoN form v4]. When a document cites a table number, check
  which edition it means. Code 4 changed meaning between editions
  (spare → emergency wreck marking buoy), so an M.1371-4-era decoder shows
  a modern wreck buoy as "spare".
* **Dimensions are not a guard zone.** The A–D block describes the aid's
  own size (or a circle around a buoy); several early implementations put
  the swing radius or the danger area there. Both IALA and the ITU note
  forbid it [IALA R0126 §4.8.3], [M.1371-5 Table 73 note 1].
* **Reception gaps are not outages.** §15.1 counted 86 gaps of 360 s
  in 429 reports from a transmitter in the same building. Alarm on *several*
  missed intervals, and weight the two channels separately — the station
  alternates strictly between the channels (253 reports on A, 176 on B in
  the September file, 360 s apart on each), so a receiver with one poor
  channel misses a large share of reports; here nearly every missed report
  was a channel B one.

## 15.7 Exercises

1. *(pencil and paper)* Decode line 19 of the data file
   (`E00VT:QVIgPb7W00000000000006NAc0J2@`000000wP00`, fill 4). Confirm the
   MMSI 631850 from the 30 bits at offset 8, and show that the longitude and
   latitude fields hold exactly `0x6791AC0` and `0x3412140`.
2. *(Python)* Write a function that takes a decoded message 21 and returns
   the IALA/ITU MMSI category: `"physical"` (`99MID1xxx`), `"virtual"`
   (`99MID6xxx`), `"mobile"` (`99MID8xxx`), `"other AtoN"` (other `99…`),
   `"coast station"` (`00MID…`), or `"not an AtoN format"`. Run it over the
   expected output; it should return `"virtual"` for line 49,
   `"physical"` for line 50, `"coast station"` for 003669739 and `"not an
   AtoN format"` for 631850. Then check whether the `virtual_aton` flag
   agrees with the MMSI category.
3. *(C++)* The decoder prints `name_extension_bits`. Add a
   `name_extension_pad_bits` key equal to `extension_bits % 6` and make the
   other three languages match, so that the tests still pass (regenerate
   `tests/expected/` with the Python version first and review the diff).
   Then explain why this number is 4 for line 49 and what it would be if
   the station had padded with 6 bits instead.
4. *(Rust)* Write a unit test that builds a 360-bit payload (14 extension
   characters, no padding) by hand and checks that `decode` returns the
   full 34-character `full_name`. Then feed it 366 bits and show that the
   fifteenth character is ignored (`MAX_EXTENSION_CHARS`).
5. *(TypeScript)* Write a small monitor: read the JSON lines of
   `tests/expected/ch15-aton-reports.jsonl`, group message 21 by MMSI, and
   print for each MMSI the last position, whether any report had
   `health_alarm` or `off_position_valid && off_position`, and the longest
   gap between consecutive `uscg.time` values, separately per `talker`. For 003669739 as heard by
   the SR162G (`talker` `AI`) the November gap between lines 38 and 39 is
   180 s.
6. *(any language, harder)* Using the whole
   `sample-data/2010-11-l3-aton/ccom-l3-4-2010-11-02` file (if you have
   it), pair each `!ANVDO` on channel `A`/`B` with the `!AIVDM` of the same
   payload in `ccom-sr162g-2010-11-02` and compute the distribution of the
   reception delay. Then explain the `X`/`Y` lines: how many are there per
   `A`/`B` line, and what do their time stamps suggest about when the unit
   decides what to transmit?

## 15.8 References

* [M.1371-5] Annex 8 §3.19 and Table 73 (message 21 layout); Table 74
  (types of AtoN); §3.3.3 and Figure 41 (dimension/reference point);
  Table 50 (position accuracy), Table 51 (EPFD), Table 47 (six-bit ASCII);
  Annex 1 §4.2.1 Table 2 (reporting interval, 3 min); Annex 2 §4.6.1.2
  (repeat indicator for messages sent on behalf of synthetic/virtual aids).
* [M.585-9] Annex 1 §4 (AtoN identities `99MIDxxxx`, sixth-digit
  convention, co-located base station); Annex 1 §2 (coast-station
  identities).
* [IALA R0126] §4.2 (physical, synthetic, virtual), §4.3 (MMSI,
  Table 2), §4.4 (reporting intervals, message 6/8 scheduling), §4.8.1
  (position monitoring and guard zone), §4.8.2 (name), §4.8.3
  (dimensions), §4.8.4 Figures 3–5 (status bit pages), §4.8.5 (Table 3,
  AtoN codes), §4.8.6 (EPFD).
* [IALA R0143] §5.1 (temporary virtual AtoN).
* [IALA G1062] §3 (benefits), §6.1 (station types 1–3, messages
  transmitted), §6.2 (synthetic AtoN), §9.1 (monitoring).
* [IEC 62320-1/-2/-3] AtoN station equipment standard (cited for the
  off-position algorithm being left to the manufacturer, via
  [IALA R0126 §4.8.1]).
* [IEC 61162-1] §8 VDM/VDO (channel field values), ALR sentence.
* [AIVDM] "Type 21: Aid-to-Navigation Report" (field table and the
  differing fixed/floating split).
* [AtoN form v4] USCG AIS AtoN specification sheet for the NOAA whale
  notice station: MMSI, type 0 rationale, name, EPFD, serial number
  000631850, intended position, message 8 DAC 366 FI 22 schedule (FI 34 is what the 2008 recordings carry; chapter 9).
* [Safety Alerts] Safety Alert 01-24 (VHF keying delays AtoN reports).
* [Toils] for the general theme that the spec, the guidelines and the
  equipment disagree.
