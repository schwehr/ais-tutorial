# 7. Message 5: ship static and voyage related data

> **What you will learn**
> * What a Class A transponder says about *itself* — identity, size, type, antenna position — and about its *voyage* — destination, ETA, draught — and how often it says it.
> * Every field of the 424-bit message 5 layout [M.1371-5 Annex 8 §3.3 Table 52], including the four-field ship-dimension block, the ETA sub-fields with their four different "not available" values, and the IMO number ranges.
> * How to validate an IMO number with its check digit, and why a valid check digit still proves very little.
> * How the data really looks: `@`-padding versus space-padding, free-text destinations instead of UN/LOCODEs, IMO numbers typed with extra zeros, draught 25.5 m on a 25-metre boat, and receivers that re-join the two fragments into one over-long sentence.
> * Six-bit text handling, string formatting and simple arithmetic on decimal digits in Python, C++, Rust and TypeScript.

## 7.1 Where this fits

Messages 1–3 (chapter 3) tell you *where* a ship is. Message 5 tells you *who* it is and *what it is doing*. It is the only Class A message that carries the ship's name, call sign, IMO number, type, dimensions, destination, estimated time of arrival (ETA) and draught [M.1371-5 Annex 8 §3.3]. A Class A shipborne station (and a SAR aircraft station — nobody else) broadcasts it every six minutes, and immediately after any of the values changes [M.1371-5 Annex 1 §4.2.1; Annex 8 §3.3]. At 424 bits it is the longest fixed-length message in the protocol and the first one in this tutorial that always arrives as two `!AIVDM` fragments (two slots on the air) [M.1371-5 Annex 8 Table 52, "Occupies 2 slots"].

The fields split into two groups that the IMO performance standard defines separately [MSC.74(69) Annex 3 §6.1–6.2]:

* **Static data** — MMSI (maritime mobile service identity), IMO number, call sign, name, type of ship, dimensions and reference point of the position antenna, EPFD (electronic position fixing device) type. Set once at installation, protected by a password on the unit [SN/Circ.227 §5.1].
* **Voyage related data** — draught, destination, ETA (plus the DTE flag and the navigational status, which travels in messages 1–3). Supposed to be typed in by the officer of the watch at the start of each voyage and kept up to date [A.1106(29) §13 Table 1, §23; Encoding Guide].

The first group is reliable to the extent that the installer did a careful job; the second depends on a human remembering to update it, and the GPSD documentation puts it bluntly: "the information in these fields (especially ETA and destination) is not reliable" [AIVDM "Type 5"]. A large part of this chapter is about telling the two apart in the data.

### Operational use

* **Bridge.** The name next to the radar target, the call sign to hail on VHF, the ship type and length to judge what the other vessel can do. A pilot boarding a ship reads A+B and C+D for the overall dimensions and the draught for the under-keel clearance.
* **Vessel traffic services and ports.** Arrival planning from destination and ETA; berth allocation from length, beam and draught; dangerous-cargo notification from the second digit of the ship type (categories X/Y/Z/OS) [M.1371-5 Annex 8 Table 53].
* **Search and rescue.** IMO number and call sign to look the ship up in the ITU and flag-state registers and reach the owner.
* **Compliance inspection.** The USCG checks the static data of boarded vessels against the Encoding Guide and can fine for wrong entries [Encoding Guide; Inspection; 33 CFR 164.46].

### Unattended / automated uses

Because message 5 is broadcast every six minutes by every Class A ship on earth, it is the backbone of every machine-built fleet database. The typical automated consumers — and the ways they go wrong — are:

* **Fleet and port databases.** Receivers log the latest message 5 per MMSI and join it to the position reports by MMSI. The `sbnms/2008` files used in this chapter are exactly such a product: one line per *unique* static report seen by the Boston-area N-AIS receivers each month, generated for a ship-strike risk study. A joined database is only as good as the join key: when two ships share an MMSI (chapter 3), their names flip back and forth in the database every six minutes.
* **Registry consistency checks.** Automated systems compare the broadcast MMSI, IMO number, name and call sign against the ITU MARS database, the IMO GISIS register and the national (FCC) licence database. Any mismatch — a name that belongs to a different IMO number, an IMO number whose check digit fails, a call sign from the wrong country — is a flag for a mis-programmed unit, a vessel that changed name without re-programming, or deliberate identity spoofing. The IMO check digit (§7.2) is the cheapest first filter; the `imo_valid` field of our decoder exists for this reason.
* **ETA-driven scheduling and destination text mining.** Port-community systems read destination and ETA to pre-plan pilots, tugs and berths; data vendors mine destination strings for trade-route statistics. Both must survive free-text spelling: in one month of Boston data the same port appears as `BOSTON`, `BOSTON,MA`, `BOSTON MA.`, `BOSTON MASS` and `BOSTON MA. USA`. The IMO asked for UN/LOCODEs precisely to make this machine-readable [SN/Circ.244 §3–§7]; almost nobody in our 2007–2011 recordings uses them.
* **Anomaly and spoofing detection.** A tanker whose reported length changes between reports, a ship type that flips between "tug" and "fishing" (we show one below), a vessel called `NAUTICAST` (the transponder's brand name) with MMSI 1193046, or a message 5 whose dimensions disagree with the hull visible on satellite imagery, all point at mis-configured or counterfeit equipment. The USCG has repeatedly warned about "a high percentage of inaccurate and improper AIS messaging data" [Safety Alerts, Alert 05-10] and reminds users that broadcasting inaccurate information is a civil-penalty offence [Safety Alerts, Alert 10-08].
* **Telemetry squeezed into the name field.** Devices that were never meant to be ships use message 5 (or its Class B cousin, message 24, chapter 14) as a free 20-character channel: fishing-gear markers that put a buoy number or a battery voltage in the name (a name such as `NET 3 12.4V` — an illustration, not from our recordings), and bench test units that announce `NOAA TEST` bound for `NOT A VESSEL` (our first example below). The USCG safety alerts in our reference set deal with inaccurate static data in general [Safety Alerts, Alert 05-10]; the flood of non-type-approved fishing-gear AIS devices is a later problem, addressed in USCG Marine Safety Information Bulletins that are not in Appendix A. Unattended parsers should treat the name as untrusted text, not as an identity, and should expect the pattern `<word> <number>` to be a device as often as a ship.
* **Satellite AIS.** From orbit, message 5 is harder to receive than the position reports because it needs both slots of a two-slot transmission to survive the collision-prone footprint. Satellite data vendors therefore keep a long-lived cache of static data per MMSI and merge it with terrestrial feeds; stale names and destinations in such products are a known artefact.

## 7.2 The specification

Message 5 is 424 bits long and every field has a fixed position [M.1371-5 Annex 8 §3.3 Table 52]. Offsets below are zero-based bit positions as `Bits.uint(start, len)` takes them.

| Bits | Len | Field | Type / units | Not available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 5 [Table 52] |
| 6–7 | 2 | Repeat indicator | uint | — | 0–3, 3 = do not repeat [Table 52; Annex 2 §4.6.1] |
| 8–37 | 30 | User ID | uint | — | MMSI [Table 52] |
| 38–39 | 2 | AIS version indicator | enum | — | 0 = M.1371-1, 1 = M.1371-3 or later, 2 = M.1371-5 or later, 3 = future editions [Table 52] |
| 40–69 | 30 | IMO number | uint | 0 | 1–999 999 not used; 1 000 000–9 999 999 valid IMO number; 10 000 000–1 073 741 823 official flag state number; not applicable to SAR aircraft [Table 52] |
| 70–111 | 42 | Call sign | 7 × six-bit ASCII | `@@@@@@@` | Craft attached to a parent vessel: `A` + last six digits of the parent MMSI [Table 52] |
| 112–231 | 120 | Name | 20 × six-bit ASCII | 20 × `@` | As on the station radio licence; SAR aircraft: `SAR AIRCRAFT NNNNNNN` [Table 52; Table 47] |
| 232–239 | 8 | Type of ship and cargo type | enum | 0 | 1–99 per Table 53; 100–199 regional; 200–255 future [Table 52; §3.3.2] |
| 240–248 | 9 | Dimension A (to bow) | metres | 0 (see text) | 511 = 511 m or more [Figure 41] |
| 249–257 | 9 | Dimension B (to stern) | metres | 0 | 511 = 511 m or more [Figure 41] |
| 258–263 | 6 | Dimension C (to port) | metres | 0 | 63 = 63 m or more [Figure 41] |
| 264–269 | 6 | Dimension D (to starboard) | metres | 0 | 63 = 63 m or more [Figure 41] |
| 270–273 | 4 | Type of EPFD | enum | 0 = undefined | Same list as message 4 [Table 52; Table 51] |
| 274–277 | 4 | ETA month | 1–12 UTC | 0 | [Table 52] |
| 278–282 | 5 | ETA day | 1–31 UTC | 0 | [Table 52] |
| 283–287 | 5 | ETA hour | 0–23 UTC | 24 | [Table 52] |
| 288–293 | 6 | ETA minute | 0–59 UTC | 60 | [Table 52] |
| 294–301 | 8 | Maximum present static draught | 1/10 m | 0 | 255 = 25.5 m or more; SAR aircraft set 0 [Table 52] |
| 302–421 | 120 | Destination | 20 × six-bit ASCII | 20 × `@` | [Table 52] |
| 422 | 1 | DTE | flag | 1 = not available (default) | 0 = data terminal available [Table 52; §3.3.1] |
| 423 | 1 | Spare | — | — | should be 0 [Table 52] |

All citations in the table are to ITU-R M.1371-5 Annex 8 unless marked otherwise.

### Text fields and padding

Character strings in AIS use the six-bit ASCII of Table 47 [M.1371-5 Annex 8 Table 47] (reproduced in Appendix B): values 0–31 are `@` and the capital letters `A`–`Z` plus `[\]^_`, values 32–63 are space, the punctuation `!"#$%&'()*+,-./`, the digits and `:;<=>?`. There are no lower-case letters. Unused positions "should be represented by the @ symbol, and they should be placed at the end of the string" [M.1371-5 Annex 2 §3.3.7]. An all-`@` string therefore means "not available" [M.1371-5 Annex 8 Table 52]. In practice half the ships pad with spaces instead (§7.6), and some do both. Our `Bits.text()` (chapter 2) cuts the string at the first `@` and strips trailing spaces, which handles both conventions and turns the "not available" value into an empty string.

### The AIS version indicator

Two bits that say which edition of the Recommendation the transponder was built to [M.1371-5 Annex 8 Table 52]. Everything in our 2007–2011 recordings says 0 (M.1371-1). A consumer could use it to guess, for example, whether a unit knows the navigational statuses 11 and 12 that older devices lack [Encoding Guide, note 5]; in practice the field is rarely updated by firmware upgrades and is of little use.

### The IMO number and its check digit

The IMO ship identification number is the seven-digit number assigned to a hull for its whole life under the IMO ship identification number scheme; it stays with the ship through renamings and reflaggings, which is what makes it the best key for a fleet database ([Encoding Guide] note 2 says where to look one up). Table 52 divides the 30-bit field into four ranges [M.1371-5 Annex 8 Table 52]:

| Value | Meaning |
|---|---|
| 0 | not available (default) |
| 1 – 999 999 | not used |
| 1 000 000 – 9 999 999 | IMO number |
| 10 000 000 – 1 073 741 823 | official flag state number |

The last range is for vessels without an IMO number: the USCG asks them to enter the US official number prefixed with a `1` and zero-padded, e.g. `100123456` [Encoding Guide, "IMO Number"].

A genuine IMO number has a check digit: multiply the first six digits by 7, 6, 5, 4, 3, 2 in turn, add the products, and the last digit of the sum must equal the seventh digit. For USNS *Sacagawea* (line 10 of our data): 9 2 7 1 4 2 **0** → 9·7 + 2·6 + 7·5 + 1·4 + 4·3 + 2·2 = 63 + 12 + 35 + 4 + 12 + 4 = 130 → last digit **0** ✓. The decoder reports this as `imo_valid`. Two cautions: a seven-digit *official number* that happens to pass the test will look valid (line 19 below, `1098224`, passes), and a correctly-formed IMO number that belongs to a *different ship* also passes. The check digit catches typing errors, not lies; registry lookup catches the rest.

### Type of ship and cargo type (Table 53)

The 8-bit field is read as two decimal digits [M.1371-5 Annex 8 §3.3.2 Table 53]. The first digit is the vessel category; for categories 2, 4, 6, 7, 8 and 9 the second digit is the hazardous-cargo category; for category 3 the second digit is an activity; and the 5x values are individually named special craft. The complete table:

| First digit | Category |
|---|---|
| 0 (code 0) | not available or no ship (default) |
| 1 | reserved for future use (Table 53 lists no first digit 0, so codes 1–9 are undefined as well; our decoder prints "reserved for future use" for 1–19) |
| 2 | wing-in-ground (WIG) craft |
| 3 | vessels engaged in an activity — second digit from the "3x" column |
| 4 | high-speed craft (HSC) |
| 5 | special craft — individually listed below |
| 6 | passenger ships |
| 7 | cargo ships |
| 8 | tankers |
| 9 | other types of ship |

| Second digit (for 2x, 4x, 6x, 7x, 8x, 9x) | Meaning |
|---|---|
| 0 | all ships of this type |
| 1 | carrying DG, HS or MP, IMO hazard or pollutant category X |
| 2 | … category Y |
| 3 | … category Z |
| 4 | … category OS |
| 5–8 | reserved for future use |
| 9 | no additional information |

| 3x code | Activity | | 5x code | Special craft |
|---|---|---|---|---|
| 30 | fishing | | 50 | pilot vessel |
| 31 | towing | | 51 | search and rescue vessel |
| 32 | towing and length of tow > 200 m or breadth > 25 m | | 52 | tug |
| 33 | engaged in dredging or underwater operations | | 53 | port tender |
| 34 | engaged in diving operations | | 54 | vessel with anti-pollution facilities or equipment |
| 35 | engaged in military operations | | 55 | law enforcement vessel |
| 36 | sailing | | 56 | spare — for assignments to local vessels |
| 37 | pleasure craft | | 57 | spare — for assignments to local vessels |
| 38 | reserved for future use | | 58 | medical transport (1949 Geneva Conventions) |
| 39 | reserved for future use | | 59 | ships and aircraft of States not parties to an armed conflict (RR Resolution 18) |

DG = dangerous goods, HS = harmful substances, MP = marine pollutants; the categories X, Y, Z and OS were called A, B, C and D in earlier editions [M.1371-5 Annex 8 Table 53 note (2)] — which is why our shared `ship_type_text()` (and most other decoders) still print "hazardous category A–D". Codes 100–199 are reserved for regional use and 200–255 for the future [M.1371-5 Annex 8 Table 52]. The USCG adds domestic guidance on top: tugs pushing barges should use 57, research and buoy-tending vessels 33, and the "hazardous category" second digits should not be used by US vessels at all [Encoding Guide, "Ship Type and Cargo"].

### Dimensions and the reference point

The 30-bit dimension block is four distances, in metres, from the position-fixing antenna to the bow (A, 9 bits), stern (B, 9 bits), port side (C, 6 bits) and starboard side (D, 6 bits) [M.1371-5 Annex 8 §3.3.3 Figure 41]. A + B is the overall length and C + D the beam; the point where the four meet is where the reported latitude/longitude refers to. A and B saturate at 511 m, C and D at 63 m [M.1371-5 Annex 8 §3.3.3 Figure 41]. A is always towards the bow, i.e. in the direction of the transmitted heading [M.1371-5 Annex 8 §3.3.3 Figure 41].

Two special patterns are defined [M.1371-5 Annex 8 §3.3.3 Figure 41]: A = B = C = D = 0 means "neither reference point nor dimensions available" (the default), and A = C = 0 with B ≠ 0 and D ≠ 0 means "dimensions known but reference point unknown". The installation guideline notices that an antenna in the port-side corner of a square bow would *legitimately* give A = C = 0 and asks installers to enter 1 instead [SN/Circ.227 §5.3]. Installers are supposed to enter the overall dimensions of the hull *including* tows pushed ahead or alongside, which is why tug values change from voyage to voyage [Encoding Guide, "EPFS Antenna Position"]. Our decoder outputs the four raw values plus `length_m`, `beam_m`, `dimensions_available` and `reference_point_available`.

### ETA

Twenty bits, MMDDHHMM in UTC: month (4 bits, 1–12), day (5 bits, 1–31), hour (5 bits, 0–23) and minute (6 bits, 0–59) [M.1371-5 Annex 8 Table 52]. Each part has its own "not available" value — 0, 0, 24 and 60 respectively — so a fully default ETA is `0/0/24/60`, and partially filled ones like `month 0, day 13` occur. There is no year: an ETA of `01-03` seen on 31 December is next year. The USCG insists on UTC, not local time, and says the ETA is "not applicable to vessels at anchor, moored, or on variable schedule (e.g. workboats)" [Encoding Guide, "Estimated Time of Arrival"]. We output the four integers, `eta_text` (`MM-DD HH:MM` with `??` for missing parts) and `eta_available` (true only when all four parts are valid).

### Draught

The maximum present static draught in 1/10 m: 0 = not available, 255 = 25.5 m or more [M.1371-5 Annex 8 Table 52]. It is a *voyage* value (it changes with loading) and the Encoding Guide reminds mariners that it is metres, not feet [Encoding Guide, "Static Draft"].

### Destination

Twenty six-bit characters of free text [M.1371-5 Annex 8 Table 52]. IMO's recommended content is the UN/LOCODE of the port of departure, a `>` separator and the UN/LOCODE of the next port, e.g. `AE DXB>NL RTM`; `?? ???` for an unknown next port, `XX XXX` for a departure port without a code, and `===` followed by the plain name when the destination has no code [SN/Circ.244 §6–§10]. The USCG variant drops the space (`USNYC>NLRTM`), adds two-digit berth suffixes and `US^` geographic identifiers, and uses `<>` and `<<` for ships that stay in one area or on station [Encoding Guide, "Destination"]. Our recordings pre-date most of this guidance and show plain names — `BOSTON`, `NORFOLK VIRGINIA USA`, `HOME`, `NOT A VESSEL`.

### DTE

One bit, "data terminal equipment ready": 0 means the transmitting station has at least the minimum keyboard and display, so a text message sent to it will be seen by a human; 1 (the default) means it does not [M.1371-5 Annex 8 §3.3.1 Table 52]. Note the inverted sense — 0 is the *good* value — which is why we print both `dte` and `dte_text` rather than a boolean.

## 7.3 Decoding one message by hand

Lines 4–5 of [`data/ch07-message-5-static-voyage.nmea`](../data/ch07-message-5-static-voyage.nmea) are a message 5 received by an SR162G receiver in the basement of the Cape Cod National Seashore visitor centre on 5 October 2008 (`sample-data/RightwhaleAisProject/20081005-capecod-visitor/20081005-capecod-visitor-center-basement`, lines 9–10). The sender is not a ship but a test transponder:

```text
!AIVDM,2,1,1,A,503OvqP00000ptpD000pt461@E=@00000000000W000004@PJ03Sm80H,0*01,rcapecod-before,1223219995.92
!AIVDM,2,2,1,A,5QDliC000000000,2*43,rcapecod-before,1223219995.92
```

Two fragments (`2,1` and `2,2`) with the same sequential message identifier `1` on channel A; the first has 56 payload characters and 0 fill bits, the second has 15 characters and 2 fill bits. The `Assembler` of chapter 2 concatenates the payloads — 71 characters, 426 bits — and drops the 2 fill bits: 424 bits, exactly Table 52's length. The characters, their six-bit values and the fields they fall into:

```text
5      0      3      O      v      q      P      0      0      0        chars 0-9,   bits 0-59
000101 000000 000011 011111 111110 111001 100000 000000 000000 000000

0      0      p      t      p      D      0      0      0      p        chars 10-19, bits 60-119
000000 000000 111000 111100 111000 010100 000000 000000 000000 111000

t      4      6      1      @      E      =      @      0      0        chars 20-29, bits 120-179
111100 000100 000110 000001 010000 010101 001101 010000 000000 000000

0      0      0      0      0      0      0      0      0      W        chars 30-39, bits 180-239
000000 000000 000000 000000 000000 000000 000000 000000 000000 100111

0      0      0      0      0      4      @      P      J      0        chars 40-49, bits 240-299
000000 000000 000000 000000 000000 000100 010000 100000 011010 000000

3      S      m      8      0      H      5      Q      D      l        chars 50-59, bits 300-359
000011 100011 110101 001000 000000 011000 000101 100001 010100 110100

i      C      0      0      0      0      0      0      0      0        chars 60-69, bits 360-419
110001 010011 000000 000000 000000 000000 000000 000000 000000 000000

0                                                                       char 70,     bits 420-425
000000   (bits 424-425 are the two fill bits and are discarded)
```

Field boundaries do not line up with character boundaries — the MMSI, for example, starts two bits into character 1 and ends two bits into character 6 — so write the 424 bits out in one line, number them from 0 and cut at the offsets of Table 52. Field by field:

| Bits | Binary | Value | Field |
|---|---|---|---|
| 0–5 | `000101` | 5 | message ID |
| 6–7 | `00` | 0 | repeat indicator |
| 8–37 | `000000001101111111111011100110` | 3 669 734 | MMSI. Nine digits: 003669734 — the `00` prefix of a coast station [M.585-9 Annex 1], MID 366 = USA |
| 38–39 | `00` | 0 | AIS version: M.1371-1 |
| 40–69 | thirty `0` | 0 | IMO number not available |
| 70–111 | `001110 001111 001110 000101 000000 000000 000000` | 14 15 14 5 0 0 0 | call sign `N O N E @ @ @` → `NONE` |
| 112–231 | `001110 001111 000001 000001 100000 010100 000101 010011 010100` then eleven `000000` | | name `N O A A ␠ T E S T` + 11 × `@` → `NOAA TEST` |
| 232–239 | `00100111` | 39 | ship type 39 — "reserved for future use" in the 3x column [M.1371-5 Annex 8 Table 53] |
| 240–248 | `000000000` | 0 | A |
| 249–257 | `000000000` | 0 | B |
| 258–263 | `000000` | 0 | C |
| 264–269 | `000000` | 0 | D — all four zero: no dimensions, no reference point [M.1371-5 Annex 8 §3.3.3 Figure 41] |
| 270–273 | `0001` | 1 | EPFD: GPS [M.1371-5 Annex 8 Table 51] |
| 274–277 | `0001` | 1 | ETA month: January |
| 278–282 | `00001` | 1 | ETA day: 1 |
| 283–287 | `00000` | 0 | ETA hour: 00 |
| 288–293 | `011010` | 26 | ETA minute: 26 — so the ETA reads `01-01 00:26`, which is almost certainly a never-edited default of the unit rather than a plan |
| 294–301 | `00000000` | 0 | draught not available |
| 302–421 | `001110 001111 010100 100000 000001 100000 010110 000101 010011 010011 000101 001100` then eight `000000` | | destination `N O T ␠ A ␠ V E S S E L` + 8 × `@` → `NOT A VESSEL` |
| 422 | `0` | 0 | DTE: available |
| 423 | `0` | 0 | spare |

Take a moment on the name: the six-bit value 32 (`100000`) is a *space* in Table 47, whereas 0 (`000000`) is `@`. "NOAA TEST" is followed by `@`s, so this unit pads the way the standard asks. Compare the sentences from the `sbnms` files below, where the padding after the name is `100000`, i.e. spaces.

The decoder's output for these two lines is the first object in [`tests/expected/ch07-message-5-static-voyage.jsonl`](../tests/expected/ch07-message-5-static-voyage.jsonl):

```json
{"ais":{"ais_version":0,"ais_version_text":"ITU-R M.1371-1","beam_m":0,"call_sign":"NONE",
"destination":"NOT A VESSEL","dim_bow_m":0,"dim_port_m":0,"dim_starboard_m":0,"dim_stern_m":0,
"dimensions_available":false,"draught_available":false,"draught_m":0.0,"dte":0,"dte_text":"available",
"epfd":1,"epfd_text":"GPS","eta_available":true,"eta_day":1,"eta_hour":0,"eta_minute":26,"eta_month":1,
"eta_text":"01-01 00:26","imo":0,"imo_available":false,"imo_range_text":"not available (default)",
"imo_valid":false,"length_m":0,"mmsi":3669734,"msg_type":5,"name":"NOAA TEST",
"reference_point_available":false,"repeat_indicator":0,"ship_type":39,"ship_type_text":"reserved",
"spare":0},"channel":"A","fragments":2,"line":5,"own_ship":false,"payload_bits":424,"talker":"AI",
"uscg":{"station":"capecod-before","station_kind":"receiver","time":1223219995.92}}
```

(The real file has the object on one line; it is wrapped here for reading.) Note `"line":5` and `"fragments":2`: the object is reported on the line that *completed* the message, and the first fragment printed nothing (chapter 2).

## 7.4 The code

The decoder is a straight transcription of Table 52 plus three small helpers: the IMO range text, the IMO check digit and the ETA text. All four versions keep the same function names and produce byte-identical JSON. Only the Python version is listed in full; for the other three we show the parts that differ and refer you to the files for the identical field-by-field block.

### Python

[`code/python/aistut/msg_5.py`](../code/python/aistut/msg_5.py):

```python
from .bits import Bits
from .common import epfd_text, require_bits, ship_type_text
from .jsonout import Fixed

# Table 52, "AIS version indicator".
AIS_VERSION = {
    0: "ITU-R M.1371-1",
    1: "ITU-R M.1371-3 or later",
    2: "ITU-R M.1371-5 or later",
    3: "future edition",
}

# Table 52, "DTE" (data terminal equipment ready), §3.3.1.
DTE_TEXT = {0: "available", 1: "not available (default)"}


def imo_range_text(imo: int) -> str:
    """Which of the four ranges of Table 52 the 30-bit IMO field falls in."""
    if imo == 0:
        return "not available (default)"
    if imo <= 999999:
        return "not used"
    if imo <= 9999999:
        return "IMO number"
    return "official flag state number"


def imo_check_digit_ok(imo: int) -> bool:
    """True if `imo` is a seven-digit IMO number whose check digit is right.

    An IMO number (Lloyd's Register / IHS number, made mandatory by SOLAS
    V/3 and shown as "IMO 9074729" on the hull) has six digits plus one
    check digit. Multiply the six digits by 7, 6, 5, 4, 3, 2 in turn, add
    the products, and the last digit of the sum is the check digit:

        9074729:  9*7 + 0*6 + 7*5 + 4*4 + 7*3 + 2*2 = 139  ->  9  ok

    Numbers outside 1000000..9999999 are never valid IMO numbers, so the
    "official flag state number" range of Table 52 returns False here;
    look at `imo_range_text` to tell the two cases apart.
    """
    if not 1000000 <= imo <= 9999999:
        return False
    digits = [int(c) for c in str(imo)]
    total = sum(d * w for d, w in zip(digits[:6], (7, 6, 5, 4, 3, 2)))
    return total % 10 == digits[6]


def eta_text(month: int, day: int, hour: int, minute: int) -> str:
    """'MM-DD HH:MM' in UTC with '??' for each part that is not available."""
    mm = f"{month:02d}" if 1 <= month <= 12 else "??"
    dd = f"{day:02d}" if 1 <= day <= 31 else "??"
    hh = f"{hour:02d}" if hour <= 23 else "??"
    mi = f"{minute:02d}" if minute <= 59 else "??"
    return f"{mm}-{dd} {hh}:{mi}"


def decode(bits: Bits) -> dict:
    """Decode a message 5. Raises DecodeError if shorter than 424 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 424)
    ais_version = bits.uint(38, 2)
    imo = bits.uint(40, 30)
    ship_type = bits.uint(232, 8)
    dim_a = bits.uint(240, 9)      # to bow
    dim_b = bits.uint(249, 9)      # to stern
    dim_c = bits.uint(258, 6)      # to port
    dim_d = bits.uint(264, 6)      # to starboard
    epfd = bits.uint(270, 4)
    month = bits.uint(274, 4)
    day = bits.uint(278, 5)
    hour = bits.uint(283, 5)
    minute = bits.uint(288, 6)
    draught_raw = bits.uint(294, 8)
    dte = bits.uint(422, 1)

    return {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "ais_version": ais_version,
        "ais_version_text": AIS_VERSION[ais_version],
        "imo": imo,
        "imo_available": imo != 0,
        "imo_range_text": imo_range_text(imo),
        "imo_valid": imo_check_digit_ok(imo),
        "call_sign": bits.text(70, 42),
        "name": bits.text(112, 120),
        "ship_type": ship_type,
        "ship_type_text": ship_type_text(ship_type),
        "dim_bow_m": dim_a,
        "dim_stern_m": dim_b,
        "dim_port_m": dim_c,
        "dim_starboard_m": dim_d,
        # Figure 41: all four zero = neither reference point nor dimensions;
        # A = C = 0 with B, D non-zero = dimensions but no reference point.
        "dimensions_available": not (dim_a == 0 and dim_b == 0 and dim_c == 0 and dim_d == 0),
        "reference_point_available": not (dim_a == 0 and dim_c == 0),
        "length_m": dim_a + dim_b,
        "beam_m": dim_c + dim_d,
        "epfd": epfd,
        "epfd_text": epfd_text(epfd),
        "eta_month": month,
        "eta_day": day,
        "eta_hour": hour,
        "eta_minute": minute,
        "eta_text": eta_text(month, day, hour, minute),
        "eta_available": 1 <= month <= 12 and 1 <= day <= 31 and hour <= 23 and minute <= 59,
        "draught_m": Fixed(draught_raw / 10.0, 1),
        "draught_available": draught_raw != 0,
        "destination": bits.text(302, 120),
        "dte": dte,
        "dte_text": DTE_TEXT[dte],
        "spare": bits.uint(423, 1),
    }
```

Three things are new compared with chapter 3:

* **Text fields.** `bits.text(70, 42)` reads seven six-bit characters through Table 47 and applies the `@`/space trimming described in §7.2. The same call with 120 bits reads the name and the destination. Nothing in the decoder cares about padding style.
* **Digits of a number.** `str(imo)` turns 9271420 into `"9271420"`, the list comprehension `[int(c) for c in …]` into `[9, 2, 7, 1, 4, 2, 0]`, and `zip(digits[:6], (7, 6, 5, 4, 3, 2))` pairs each of the first six digits with its weight. Chained comparisons such as `1000000 <= imo <= 9999999` are a Python speciality — the other three languages need `&&`.
* **The length rule.** `require_bits(…, 424)` demands the full message. A 426-bit message (a sender that mis-counted its fill bits, see §7.6) decodes fine because we never look beyond bit 423; a 416- or 420-bit one is rejected as `message 5 too short`, because we cannot know which field the sender lost.

### C++

[`code/cpp/msg_5.hpp`](../code/cpp/msg_5.hpp). The decoder body is the same field list as a `Json::Object` initialiser (compare `decode_msg_1_2_3` in chapter 3); the helpers are:

```cpp
inline const std::array<const char*, 4> AIS_VERSION = {
    "ITU-R M.1371-1",
    "ITU-R M.1371-3 or later",
    "ITU-R M.1371-5 or later",
    "future edition",
};

inline const std::array<const char*, 2> DTE_TEXT = {"available", "not available (default)"};

inline std::string imo_range_text(std::uint64_t imo) {
    if (imo == 0) return "not available (default)";
    if (imo <= 999999) return "not used";
    if (imo <= 9999999) return "IMO number";
    return "official flag state number";
}

inline bool imo_check_digit_ok(std::uint64_t imo) {
    if (imo < 1000000 || imo > 9999999) return false;
    std::string digits = std::to_string(imo);  // exactly 7 characters here
    int total = 0;
    for (int i = 0; i < 6; ++i) {
        total += (digits[i] - '0') * (7 - i);
    }
    return total % 10 == digits[6] - '0';
}

inline std::string eta_text(int month, int day, int hour, int minute) {
    auto two = [](int v) {
        char buf[16];  // "%02d" of any int fits in 11 characters plus NUL
        std::snprintf(buf, sizeof buf, "%02d", v);
        return std::string(buf);
    };
    std::string mm = (1 <= month && month <= 12) ? two(month) : "??";
    std::string dd = (1 <= day && day <= 31) ? two(day) : "??";
    std::string hh = hour <= 23 ? two(hour) : "??";
    std::string mi = minute <= 59 ? two(minute) : "??";
    return mm + "-" + dd + " " + hh + ":" + mi;
}
```

and the start of the decoder:

```cpp
inline Json decode_msg_5(const Bits& bits) {
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 424);
    std::uint64_t ais_version = bits.uint(38, 2);
    std::uint64_t imo = bits.uint(40, 30);
    // ... one local per field, then:
    return Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"ais_version", ais_version},
        {"ais_version_text", AIS_VERSION[ais_version]},
        {"imo", imo},
        {"imo_available", imo != 0},
        {"imo_range_text", imo_range_text(imo)},
        {"imo_valid", imo_check_digit_ok(imo)},
        {"call_sign", bits.text(70, 42)},
        {"name", bits.text(112, 120)},
        // ... identical to the Python dictionary ...
        {"spare", bits.uint(423, 1)},
    };
}
```

> **Language notes (C++).**
> * `std::to_string(imo)` is Python's `str(imo)`; a `std::string` can be indexed like an array and `digits[i] - '0'` turns the character `'7'` into the integer 7, because characters are small integers in C++.
> * `auto two = [](int v) { … };` is a *lambda*: a function defined inside another function. The `[]` is the (empty) capture list — the lambda uses nothing from the enclosing scope.
> * There is no built-in zero-padded formatting of integers into a `std::string` in C++17, hence `snprintf` into a stack buffer. The buffer is deliberately large enough for any `int` so that the compiler's `-Wformat-truncation` check is satisfied.
> * `1 <= month && month <= 12`: a chained `1 <= month <= 12` compiles in C++ but means `(1 <= month) <= 12`, i.e. `true <= 12`, which is always true. Keep the `&&`.

### Rust

[`code/rust/src/msg_5.rs`](../code/rust/src/msg_5.rs):

```rust
pub fn ais_version_text(code: u64) -> &'static str {
    match code {
        0 => "ITU-R M.1371-1",
        1 => "ITU-R M.1371-3 or later",
        2 => "ITU-R M.1371-5 or later",
        _ => "future edition", // 3
    }
}

pub fn imo_range_text(imo: u64) -> &'static str {
    match imo {
        0 => "not available (default)",
        1..=999_999 => "not used",
        1_000_000..=9_999_999 => "IMO number",
        _ => "official flag state number",
    }
}

pub fn imo_check_digit_ok(imo: u64) -> bool {
    if !(1_000_000..=9_999_999).contains(&imo) {
        return false;
    }
    let digits: Vec<u64> = imo.to_string().bytes().map(|b| (b - b'0') as u64).collect();
    let total: u64 = digits[..6].iter().zip([7, 6, 5, 4, 3, 2]).map(|(d, w)| d * w).sum();
    total % 10 == digits[6]
}

pub fn eta_text(month: u64, day: u64, hour: u64, minute: u64) -> String {
    let two = |v: u64| format!("{v:02}");
    let mm = if matches!(month, 1..=12) { two(month) } else { "??".to_string() };
    let dd = if matches!(day, 1..=31) { two(day) } else { "??".to_string() };
    let hh = if hour <= 23 { two(hour) } else { "??".to_string() };
    let mi = if minute <= 59 { two(minute) } else { "??".to_string() };
    format!("{mm}-{dd} {hh}:{mi}")
}

pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 424)?;
    let ais_version = bits.uint(38, 2);
    let imo = bits.uint(40, 30);
    // ... one local per field, then:
    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "ais_version", ais_version);
    put(&mut out, "ais_version_text", ais_version_text(ais_version));
    put(&mut out, "imo", imo);
    put(&mut out, "imo_available", imo != 0);
    put(&mut out, "imo_range_text", imo_range_text(imo));
    put(&mut out, "imo_valid", imo_check_digit_ok(imo));
    put(&mut out, "call_sign", bits.text(70, 42));
    put(&mut out, "name", bits.text(112, 120));
    // ... identical to the Python dictionary ...
    put(&mut out, "spare", bits.uint(423, 1));
    Ok(Json::Obj(out))
}
```

> **Language notes (Rust).**
> * Range patterns in `match` (`1..=999_999 =>`) replace Python's `if … <= … <= …` chains; underscores in numeric literals are ignored and only help the eye.
> * `imo.to_string().bytes()` yields the ASCII bytes of the decimal text; `b - b'0'` (a byte literal) is the same digit trick as in C++. `.map(…).collect()` builds a `Vec`, and `zip` with a fixed array pairs digits with weights; `.sum()` needs the type annotation `let total: u64` so the compiler knows what to add into.
> * `matches!(month, 1..=12)` expands to a `match` that returns a `bool` — handy inside an `if`.
> * The `#[cfg(test)] mod tests` block at the end of the file contains unit tests (`cargo test msg_5` runs them) for the check digit, the ETA defaults and the hand-decoded Cape Cod message; the other languages rely on the integration test only.

### TypeScript

[`code/typescript/src/msg_5.ts`](../code/typescript/src/msg_5.ts):

```typescript
export function imoRangeText(imo: number): string {
  if (imo === 0) {
    return "not available (default)";
  }
  if (imo <= 999999) {
    return "not used";
  }
  if (imo <= 9999999) {
    return "IMO number";
  }
  return "official flag state number";
}

export function imoCheckDigitOk(imo: number): boolean {
  if (imo < 1000000 || imo > 9999999) {
    return false;
  }
  const digits = String(imo).split("").map(Number);
  const weights = [7, 6, 5, 4, 3, 2];
  const total = weights.reduce((sum, w, i) => sum + digits[i] * w, 0);
  return total % 10 === digits[6];
}

export function etaText(month: number, day: number, hour: number, minute: number): string {
  const two = (v: number): string => String(v).padStart(2, "0");
  const mm = 1 <= month && month <= 12 ? two(month) : "??";
  const dd = 1 <= day && day <= 31 ? two(day) : "??";
  const hh = hour <= 23 ? two(hour) : "??";
  const mi = minute <= 59 ? two(minute) : "??";
  return `${mm}-${dd} ${hh}:${mi}`;
}

export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 424);
  const aisVersion = bits.uint(38, 2);
  const imo = bits.uint(40, 30);
  // ... one const per field, then:
  return {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    ais_version: aisVersion,
    ais_version_text: AIS_VERSION[aisVersion],
    imo,
    imo_available: imo !== 0,
    imo_range_text: imoRangeText(imo),
    imo_valid: imoCheckDigitOk(imo),
    call_sign: bits.text(70, 42),
    name: bits.text(112, 120),
    // ... identical to the Python dictionary ...
    spare: bits.uint(423, 1),
  };
}
```

> **Language notes (TypeScript).**
> * `String(imo).split("").map(Number)` — convert to text, split into one-character strings, convert each back to a number. `Number` is passed as a function value, like `int` would be in Python's `map(int, …)`.
> * `reduce((sum, w, i) => …, 0)` folds an array to a single value; the callback receives the running total, the element and its index.
> * `"7".padStart(2, "0")` is the zero-padding; template literals `` `${mm}-${dd}` `` are the f-string.
> * `imo,` on its own inside an object literal is shorthand for `imo: imo` — the same shorthand you saw with `timestamp,` in chapter 3.
> * A 30-bit value is exact in JavaScript's 64-bit floating-point numbers; no `BigInt` is needed anywhere in this message.

## 7.5 Sample data and results

[`data/ch07-message-5-static-voyage.nmea`](../data/ch07-message-5-static-voyage.nmea) has 26 lines, seven of them `#` comments that the decoders ignore, and 19 sentences. The sentences come from four recordings:

| Data lines | Source | What it shows |
|---|---|---|
| 4–5 | `sample-data/RightwhaleAisProject/20081005-capecod-visitor/20081005-capecod-visitor-center-basement` lines 9–10 | the hand-decoded test transponder, `@`-padded, all defaults |
| 7–12 | `sample-data/2007hydro/norfolk-log.ais` lines 60–61, 77–78, 215–216 | three two-fragment messages from Hampton Roads, May 2007: a tug, a US Navy supply ship with a valid IMO number and a free-text destination, and a ship with every voyage field at its default |
| 14 | `sample-data/sbnms/2008/data/200801-5.ais.unique` line 1 | one receiver-stitched single-sentence message 5 exactly as recorded; its checksum does not verify |
| 16–23 | `sample-data/sbnms/2008/data/200801-5.ais.unique` lines 4, 3, 48, 53, 55, 93, 114, 119 | eight more of the same with the checksum recomputed (§7.6 explains how and why): a tanker, a passenger ship, two fishing vessels, two tugs, a Coast Guard boat and a pilot boat |
| 25–26 | `sample-data/tide3/ais/ais-2010-12-28` lines 152–153 | a two-fragment message whose second fragment is 8 bits short |

Run any of the four decoders over it:

```sh
python3 code/python/decode.py data/ch07-message-5-static-voyage.nmea
code/cpp/build/decode data/ch07-message-5-static-voyage.nmea
code/rust/target/release/decode data/ch07-message-5-static-voyage.nmea
node code/typescript/dist/decode.js data/ch07-message-5-static-voyage.nmea
```

and compare with [`tests/expected/ch07-message-5-static-voyage.jsonl`](../tests/expected/ch07-message-5-static-voyage.jsonl) (14 objects: 12 decoded messages and 2 errors). The interesting fields, taken from that file:

| Line | MMSI | `name` | `call_sign` | `imo` / `imo_valid` | `ship_type` | A/B/C/D | `eta_text` | `draught_m` | `destination` |
|---|---|---|---|---|---|---|---|---|---|
| 5 | 3669734 | `NOAA TEST` | `NONE` | 0 / false | 39 reserved | 0/0/0/0 | `01-01 00:26` | 0.0 | `NOT A VESSEL` |
| 8 | 366877840 | `DANIEL MCALLISTER` | `WDB2615` | 0 / false | 31 towing | 11/22/2/8 | `11-15 23:00` | 5.5 | (empty) |
| 10 | 369855000 | `USNS SACAGAWEA` | `NSAC` | 9271420 / true | 79 cargo, no additional information | 110/100/8/8 | `05-11 06:00` | 8.5 | `NORFOLK VIRGINIA USA` |
| 12 | 367861000 | `USNS LARAMIE` | `NLAR` | 8906626 / true | 35 military operations | 0/0/0/0 | `??-?? ??:??` | 9.0 | (empty) |
| 14 | — | *checksum mismatch* | | | | | | | |
| 16 | 538002219 | `NOR'EASTER` | `V7HF2` | 9298703 / true | 84 tanker, hazardous category D | 148/33/13/13 | `12-31 12:00` | 11.0 | `PORTSMOUTH` |
| 17 | 367130160 | `BRANDYWINE` | `WDD3364` | 0 / false | 31 towing | 10/20/6/2 | `??-?? ??:??` | 4.0 | `BOSTON` |
| 18 | 366888690 | `LISA ANN 2  F/V` | `WDB3573` | 0 / false | 30 fishing | 10/7/3/3 | `12-30 17:00` | 3.0 | `NEWBURYPORT MA` |
| 19 | 367008480 | `VERNON C` | `WCZ9536` | 1098224 / true | 30 fishing | 10/5/21/20 | `??-?? ??:??` | 0.0 | (empty) |
| 20 | 366941510 | `MORTONBOUCHARD IV` | `WDB7380` | 926974000 / false | 52 tug | 0/40/2/12 | `01-03 03:00` | 7.0 | `BOSTON,MA` |
| 21 | 367288000 | `COAST GUARD 908` | (empty) | 0 / false | 0 not available | 0/0/0/0 | `??-?? ??:??` | 25.5 | (empty) |
| 22 | 246028000 | `NOORDAM` | `PHET` | 9230115 / true | 69 passenger, no additional information | 30/259/16/16 | `01-08 10:00` | 7.8 | `TORTOLA` |
| 23 | 80269 | `NORTHEAST PILOT IV` | `0` | 0 / false | 0 not available | 8/8/3/3 | `??-?? ??:??` | 0.0 | `HOME` |
| 26 | — | *message 5 too short: 416 bits < 424* | | | | | | | |

Things to notice, object by object:

* **Line 10, USNS *Sacagawea*.** The model citizen: IMO 9271420 passes the check digit, call sign `NSAC` is a US Navy call sign, A + B = 210 m. But C + D = 16 m, and a dry-cargo ship of that length is roughly twice as wide — whoever entered the dimensions seems to have put the *half*-beam on each side. Static data is only as good as the installer. The destination `NORFOLK VIRGINIA USA` is free text; the UN/LOCODE would be `USORF`.
* **Line 12, USNS *Laramie*.** Type 35 "engaged in military operations", all dimensions zero (`dimensions_available: false`), ETA at all four defaults (`eta_available: false`), destination empty, draught 9.0 m. A valid IMO number with nothing else filled in is common for naval auxiliaries.
* **Line 16, *Nor'easter*.** Ship type 84 — tanker, second digit 4. The apostrophe in the name is six-bit value 39 (`'`), a legal character. Destination `PORTSMOUTH` is ambiguous between New Hampshire, Virginia, England and a dozen other places, which is the problem SN/Circ.244 was written to solve.
* **Line 18, *Lisa Ann 2*.** The name `LISA ANN 2  F/V` (two spaces, then the type prefix) violates two rules of the Encoding Guide — no vessel-type precursors such as `F/V`, no padding inside the name [Encoding Guide, "Vessel Names"] — and EPFD 0 "undefined" means the unit does not know what its own position source is. Still a perfectly useful record for a fisheries database.
* **Line 19, *Vernon C*.** Draught 0 and ETA defaults (a fishing boat has no "voyage"), destination all `@`, and `imo` 1098224 with `imo_valid: true` — the check digit passes by chance. The number is almost certainly the US official number 1098224 (a documented vessel number) typed without the leading `1` + zeros prefix. Elsewhere in the same month's file this MMSI reports ship type 31, 32 and 21, and the name changes to `TUG VERNON C` — somebody was experimenting with the menu.
* **Line 20, *Morton Bouchard IV*.** `imo` 926974000 lands in the "official flag state number" range and fails the check — because it is the genuine IMO number 9269740 followed by two extra zeros (9269740 passes: 9·7+2·6+6·5+9·4+7·3+4·2 = 170 → 0). The Encoding Guide's "use leading zeroes, not trailing zeroes" rule exists because of this exact mistake [Encoding Guide, "IMO Number"]. Note also A = 0 with C = 2 — an antenna at the very bow is unlikely on a tug; more probably the dimensions describe the tug alone although it pushes a barge, which the Encoding Guide says they should not [Encoding Guide, "EPFS Antenna Position"].
* **Line 21, `COAST GUARD 908`.** Call sign all `@` (empty), type 0, no dimensions, and `draught_m: 25.5` — the raw value 255, which means "25.5 m or more" [M.1371-5 Annex 8 Table 52], on a Coast Guard small boat. The unit's draught field was never set and holds all ones. `draught_available` is `true` because 0 is the only defined "not available" value; a consumer must apply its own plausibility limit.
* **Line 22, *Noordam*.** Type 69 passenger ship, A + B = 289 m — the largest `length_m` in the file — bound for `TORTOLA` with a proper ETA and `dte: 0`. The stern distance B = 259 is well below the 511 saturation.
* **Line 23, `NORTHEAST PILOT IV`.** MMSI 80269 — nine digits `000080269`, which would be a coast station with MID 008, which does not exist [M.585-9 Annex 1] — call sign `0`, type 0, destination `HOME`. A pilot boat whose installer never finished the job. Any database keyed on MMSI will merge this boat with every other unit that was left at a factory default.
* **Line 26.** The one message that does not decode; see the next section.

## 7.6 Pitfalls and real-world deviations

**`@` padding versus space padding.** The standard says unused characters are `@` [M.1371-5 Annex 2 §3.3.7]. In the January 2008 Boston file (462 unique reports) 267 names are space-padded (`BRANDYWINE          `), 191 are `@`-padded (`NOR'EASTER@@@@@@@@@@`), 4 fill all 20 characters, and `COLLEEN   MCALLISTER` has three spaces *inside* the name. Trim at the first `@` and then strip trailing spaces, as `Bits.text()` does, and never compare names for equality without normalising white space. Beware the opposite failure: a decoder that strips `@` from both ends will turn a name that legitimately starts with `@`… into nothing — but no real name starts with `@`, so the first-`@` rule is safe.

**Receiver-stitched single-sentence messages.** The `sbnms/2008/data/*-5.ais.unique` files were produced by a collection tool that re-joined the two fragments of each message 5 into one `!AIVDM,1,1,…` sentence with a 71-character payload and fill bits 2. Such a sentence is about 150 characters long, well over the 82-character limit of the sentence layer [IEC 61162-1 §7], and — as data line 14 shows — the checksum the tool wrote does not match the text it wrote. We tried the obvious explanations (the checksum of either original fragment, with every sequence number and channel) and none fits every line, so we do not know what it was computed over. Our decoders report `checksum mismatch` and stop, which is the right default. To use the data anyway you have two honest options: recompute the checksum (which is what we did for lines 16–23 with a one-line script — the payload, fill bits and metadata are untouched, and the data file says so in a comment), or add a `--no-checksum` flag to the driver (exercise 5). Once past the checksum, the payload decodes exactly like a two-fragment message: 71 × 6 − 2 = 424 bits.

**426-bit messages.** The most common length error on the air is a sender that reports 2 fill bits too few, so the message assembles to 426 bits; GPSD notes that this "seems for some reason to be most common on Type 5 messages" and recommends accepting up to 5 bits over length [AIVDM "Interaction with AIVDM padding"]. Our decoders do exactly that by never reading past bit 423. We looked for such a message in all of the recordings (more than 100 000 message 5 instances) and found none, so the behaviour is covered by reasoning rather than by a test line.

**Short messages.** The same source says 420- and 422-bit messages are "fairly common" [AIVDM "Type 5"]. In our recordings, leaving aside the `nhjel` receiver (next paragraph), the only message 5 that is not 424 bits is data lines 25–26: the first fragment is a normal 56 characters, the second is 14 characters with fill 4 instead of 15 characters with fill 2, so the message is 416 bits. If you decode what *is* there (exercise 3), the name is `KEOKUK(YTB-771)` — a Navy yard tug, with parentheses in the name — and the destination reads `HARBOQSK/)EPPPPPPPP`: the tail of the second fragment looks like a space-padded destination shifted by one bit (`P` is `010000`, a space `100000` shifted right by one). Somewhere between the transponder and the serial port bits were lost, the fill count was adjusted to match, and the checksum was computed over the damaged text. Our decoders report `message 5 too short: 416 bits < 424` and leave the forensics to you; the point is that a syntactically valid sentence is not a semantically valid message.

**The `nhjel` receiver.** The `tide1/` recordings contain dozens of apparent message 5s with 56, 104, 136, 168, 192 and 248 bits and valid checksums; none of them is a message 5. Filter on `payload_bits` before trusting the first six bits.

**ETA has no year, and two of the four parts default to out-of-range values.** `0/0/24/60` is the default; `??-13 00:00` (month 0, day 13) and `01-03 03:00` seen on 31 December both occur in the Boston data. Build the date in the receiver's calendar, treat month or day 0 as unknown, and allow for the ETA being up to a year in the future *or* months in the past (ships forget to update it after arrival).

**Draught 25.5.** Any value of 255 should be read as "25.5 m or more" [M.1371-5 Annex 8 Table 52] and, on anything that is not a ULCC, as "not set". Likewise length 0 with a non-zero draught, or a tug with `length_m` 40 one hour and 120 the next (tow included or not), are normal.

**IMO number in the wrong range.** Trailing zeros (line 20), the official number without its `1` prefix (line 19), six-digit numbers in the "not used" range and nine-digit numbers that look more like MMSIs all occur in one month of the Boston data. Use `imo_range_text` and `imo_valid` together, then a registry.

**Mis-matched MMSI.** A message 5 from MMSI 80269 or 1193046 (a *Nauticast* unit never programmed) is a Class A transponder that has not been configured. M.1371-5 says a station "should only transmit if an appropriate MMSI … is programmed" [M.1371-5 Annex 1 §3]; many units transmit anyway. Everything in such a message is suspect, including the name.

**Ship type 39 and other "reserved" values.** The test transponder on line 5 uses 39, reserved in Table 53. Decoders must print *something* for every one of the 256 values; ours prints `reserved`, `reserved for regional use` or `reserved for future use` depending on the range, and never raises.

**Destination as a status field.** `HOME`, `SHIFTING`, `ENROUTE`, `ESCORT VESSEL`, `SCIENCE` and `US BTC` all appear in one month of Boston data, and the Cape Cod test unit says `NOT A VESSEL`. Text mining must expect the destination to be an operator's note, not a place.

## 7.7 Exercises

1. **(Pencil)** Decode the first 40 bits of data line 9 (`55Pf:602…`, USNS *Sacagawea*) by hand and confirm the MMSI 369855000 and AIS version 0. Then find the 30 bits of the IMO number and check the digit sum given in §7.2.
2. **(Python)** Write `imo_check_digit(n: int) -> int` that *computes* the check digit for a six-digit stem, and use it to generate the correct IMO number from the stem `926974` (line 20). Verify with `imo_check_digit_ok`.
3. **(Rust or TypeScript)** Make the decoder tolerant of short messages: when the payload has at least 302 bits, decode every field that is fully present, output `destination` from whatever six-bit characters exist (`Bits.text()` already stops at the end of the data), set `dte` and `spare` to `null` when missing, and add `"truncated": true`. Run it on data lines 25–26 and confirm the name `KEOKUK(YTB-771)`. Decide, and document, whether a 426-bit message should also be flagged.
4. **(C++)** Add a `dimensions_text` field such as `"210 m x 16 m, antenna 110 m from bow, 8 m from port side"`, honouring the saturation values 511 and 63 ("511 m or more") and the two special patterns of Figure 41.
5. **(Any language)** Add a `--no-checksum` option to the driver that decodes sentences whose checksum does not verify, but adds `"checksum_ok": false` to the output object. Run it on the unmodified `sample-data/sbnms/2008/data/200801-5.ais.unique` and count how many distinct destination spellings of Boston you find.
6. **(Harder)** Using the position reports of chapter 3 and the message 5s of this chapter, write a program that keeps the latest static record per MMSI and flags: (a) a name that changes between two reports from the same MMSI; (b) `length_m` that changes by more than 10 m; (c) an MMSI whose first three digits are not a valid MID [M.585-9]; (d) a draught of 25.5. Which of the ships in the data file trips which rule?

## 7.8 References

* [M.1371-5] Annex 1 §3 (MMSI must be programmed), §4.2.1 (reporting intervals); Annex 2 §3.3.7 (character strings, `@` padding); Annex 8 §3.3 Table 52 (message 5), §3.3.1 (DTE), §3.3.2 Table 53 (ship and cargo type), §3.3.3 Figure 41 (dimensions and reference point), Table 47 (six-bit ASCII), Table 51 (EPFD types).
* [M.585-9] Annex 1 — MMSI formats (coast station `00MIDxxxx`, MIDs).
* [IEC 61162-1] §7 — sentence structure, the 82-character limit and the checksum.
* [AIVDM] "Type 5: Static and Voyage Related Data"; "Interaction with AIVDM padding" (the 426-bit variant).
* [MSC.74(69)] Annex 3 §6 — static vs. voyage related information.
* [A.1106(29)] §13 Table 1, §23 — which data the officer of the watch must enter and keep current.
* [SN/Circ.227] §5.1–§5.3 — installation: static data entry, reference point, dimensions A/B/C/D and the A = C = 0 trap.
* [SN/Circ.244] §3–§10 — UN/LOCODE in the destination field.
* [Encoding Guide] — US rules for name, call sign, IMO/official number, EPFS, dimensions, draught, destination syntax, ETA in UTC, ship type table.
* [Inspection], [33 CFR 164.46] — enforcement.
* [Safety Alerts] Alert 05-10 (inaccurate AIS data), Alert 10-08 (civil penalties for inaccurate information).
