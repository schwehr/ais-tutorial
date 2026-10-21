# 9. Application-specific messages in the wild

> **What you will learn**
> * What is actually inside the message 6/8 containers of chapter 8 in real recordings from 2007–2011: a reverse-engineered whale-notice format, two generations of the IMO meteorological/hydrographic message, encrypted military traffic, test broadcasts and a mis-aligned area notice.
> * How to reverse-engineer an unpublished binary layout from an encoder's debug log and confirm it against what was heard on the air — and how to say honestly which parts are certain and which are guesses.
> * The IMO SN/Circ.236 (2004) and SN.1/Circ.289 (2010) met/hydro messages field by field, why the second is incompatible with the first, and how "not available" is encoded in each.
> * How to recognise what you *cannot* decode — encrypted AIS, DAC 0 tests, private DACs — and what to do with it.
> * A catalogue of the other ASM families (Circ.289 FI 16–40, Seaway/PAWSS, UNH water levels, inland AIS) and how to read an ASM registry entry.
> * Language notes: decoders with dozens of optional fields (`None`/`null` versus `_available` booleans), repeated fixed-size records, offset-coded versus two's-complement numbers, helper functions that take a sentinel.

## 9.1 Where this fits

Chapter 8 established that a message 6 or 8 is an envelope with a 16-bit address label (designated area code and function identifier, DAC/FI) and up to 920/952 bits of content defined by whoever owns the label. This chapter opens the envelopes found in `sample-data/`. The inventory, from a scan of every `!AIVDM` with message type 6 or 8 across the collection (validating the length of each before counting it):

| (msg, DAC, FI) | Where | Count | What |
|---|---|---|---|
| 8 / 366 / 56, 57 | everywhere 2007–2011 | tens of thousands | USCG encrypted AIS: 312-bit position, 568-bit static [EAIS IDD] |
| 8 / 366 / 34 | `RightwhaleAisProject`, 2008 | 5 060 in one day | right-whale zone notices, 2008 form — unpublished |
| 6 / 1 / 0 | `RightwhaleAisProject/20081013.log` | 31 | ITU text telegram (IFM 0), empty text, from two base stations |
| 8 / 55 / 58 | `2007hydro`, Norfolk | 6 677 | 312-bit broadcasts from MMSI 61616, DAC 55 — undocumented |
| 8 / 91 / 37 | `2010-11-l3-aton` | 24 | L-3 AtoN bench test; really DAC 366 FI 22 shifted by two bits (§9.6) |
| 6 / 0 / 0, 8 / 0 / 11 | `tide3`, `tide1` | 2 | DAC 0 test messages |
| 8 / 117 / 16, 322 / 55, 1003 / 42, … | assorted | 1–6 each | unregistered DACs; noise or private experiments |
| 8 / 1 / 11, 8 / 1 / 31 | — | **0** | no IMO met/hydro message was recorded by any of our receivers |

Two things stand out. First, the ASM traffic that *exists* in US waters in these years is almost entirely machine-to-machine and either encrypted or project-specific; the international IMO messages that most tutorials describe do not appear at all. Second, the one open format with real traffic — the whale notice — was never published, so we have to reconstruct it. That is representative: anyone who works with ASMs spends more time with registries, drafts and encoder source code than with the ITU recommendation.

### Operational use

* **Met/hydro on the bridge.** Circ.289's FI 31 message is what a port's PORTS-style sensor network is meant to broadcast: wind, pressure, water level and current for a point, time-stamped, every few minutes [Circ.289 Annex §1; IHR 2010]. An ECDIS that implements Circ.290 shows the values near the station's position; one that does not shows nothing.
* **Hazard areas.** Circ.289's Area Notice (FI 22/23) lets a vessel traffic service (VTS) draw a circle, rectangle, sector or polygon on the chart with a 7-bit *notice description* — "Caution Area: Marine mammals in area – reduce speed" is code 1 — a start time and a duration [Circ.289 §11, Tables 11.1–11.11]. The whale notices below are its direct ancestor: the 2008 project encoded exactly that information in a home-made layout, and the lessons from it fed the 2010 standard [IHR 2010; AtoN form v4].
* **Text.** The IFM 0 text telegram is a legal way to send free text to one station or to all inside a binary message, with a sequence number for long texts [M.1371-5 Annex 5 §5.1].

### Unattended / automated uses

This chapter is, essentially, a chapter about unattended uses:

* **Environmental sensor networks.** A tide gauge or weather station with a serial port, a formatter that fills in Circ.289 Table 1.1, a transponder. Nobody types anything. The UNH 2006 experiments did this with a home-grown layout before IMO had one [UNH RFC; RTCM SC121]; NOAA PORTS and the Seaway run such networks operationally [Seaway R41 §1.2].
* **Whale-notice automation.** Ten acoustic buoys → Cornell's detection server → an XML status feed → a script that encodes one 112-bit record per buoy → `!VTBBM` → three shore transponders → every ship in Massachusetts Bay, every five minutes, for months. §9.3 shows the bits; chapter 8 showed the plumbing.
* **Encrypted blue-force tracking.** Government vessels that must not reveal their positions to everyone still want their own side to see them. The USCG's answer was to put an encrypted position report into a message 8 (FI 56) and an encrypted static report into another (FI 57), both from a station's ordinary maritime mobile service identity (MMSI), on the ordinary channels, next to the ship's *unencrypted* Class A reports — or instead of them, when the transponder runs in a "secure" mode [EAIS IDD] (the `$PCNSS,BFP,…` mode commands are in `sample-data/RightwhaleAisProject/cns-secure-dos.txt` and siblings). To every other receiver these are opaque 256- and 512-bit blocks that change completely each time.
* **Lock scheduling and traffic ordering.** The Seaway's FI 2 messages (lockage order, estimated lock times) are generated by the traffic management system and read by the ships' pilots' software [Seaway R41 §7–8].
* **Pitfalls that only machines notice.** No delivery guarantee: a lost 2-slot message simply never happens, and the sender's `ABK` type 3 only says it was transmitted (chapter 8). Slot cost: a 5-slot message 8 monopolises 1 280 bits of the channel, and Circ.289 says "messages with more than 3 slots should be avoided" because they "are less likely to be received" [Circ.289 §11.7]. Display support: a perfectly encoded FI 31 is invisible on a bridge whose ECDIS predates 2010. Registration: a DAC/FI that is not registered with IALA will eventually collide with one that is — the 2008 FI 34 was re-registered as FI 22 in 2010, and Circ.289 independently chose FI 22 under DAC 1 for its area notice.

## 9.2 The specification

The message 6/8 headers are in chapter 8. Here every offset is written as `o + n`, where `o` is the first bit after the application identifier: **88** in a message 6, **56** in a message 8.

### DAC 1 FI 0 — IFM 0 text telegram

[M.1371-5 Annex 5 §5.1 Tables 25/26]:

| Bits | Len | Field | Notes |
|---|---|---|---|
| o+0 | 1 | Acknowledge required flag | 1 = reply required (addressed only); 0 for broadcasts [Table 25] |
| o+1–11 | 11 | Text sequence number | incremented by the application for long texts; "all zeros indicates that sequence numbers are not being used" [Table 25] |
| o+12– | 6 × n | Text string | six-bit ASCII (Table 47); 6–906 bits in message 6, 6–936 in message 8 [Tables 25/26] |
| last | 0, 2, 4 or 6 | Spare | "to maintain byte boundaries". NOTE 1: a 6-bit spare "will be interpreted as a valid 6-bit character (all zeros is the '@' character)" [Table 25] |

Output: `ack_required`, `text_sequence_number`, `text`, `text_chars`, `spare_bits`. The decoder takes as many whole characters as fit and reports the remainder as spare.

### DAC 1 FI 11 — IMO SN/Circ.236 meteorological and hydrological data (2004)

Circ.236's "trial message 1", 352 bits in a message 8 (296 data bits), later deprecated: FI 11–15 "not to be used after 1 Jan 2013" [Circ.289 Table 1]. IMO's PDF is not freely available; the layout below follows the free description in [AIVDM "Type 8(1/11)"], which transcribes Circ.236 and notes its quirks. Three of them matter for a decoder: **latitude comes before longitude**; temperatures and water level are *offset*-coded, not two's complement; and "if there is no data available, default value to be transmitted is the highest available binary value for that particular data field" [Circ.236 as quoted in AIVDM], so every not-available value is all ones.

| Bits | Len | Field | Units | Not available |
|---|---|---|---|---|
| o+0–23 | 24 | Latitude | 1/1 000 min, signed | 0x7FFFFF |
| o+24–48 | 25 | Longitude | 1/1 000 min, signed | 0xFFFFFF |
| o+49–53 | 5 | UTC day | 1–31 | 31 |
| o+54–58 | 5 | UTC hour | 0–23 | 31 |
| o+59–64 | 6 | UTC minute | 0–59 | 63 |
| o+65–71 | 7 | Average wind speed | knots | 127 |
| o+72–78 | 7 | Wind gust | knots | 127 |
| o+79–87 | 9 | Wind direction | ° true | 511 |
| o+88–96 | 9 | Wind gust direction | ° true | 511 |
| o+97–107 | 11 | Air temperature | 0.1 °C, value/10 − 60 | 2047 |
| o+108–114 | 7 | Relative humidity | % | 127 |
| o+115–124 | 10 | Dew point | 0.1 °C, value/10 − 20 | 1023 |
| o+125–133 | 9 | Air pressure | hPa, value + 800 | 511 |
| o+134–135 | 2 | Pressure tendency | 0 steady, 1 decreasing, 2 increasing | 3 |
| o+136–143 | 8 | Horizontal visibility | 0.1 NM | 255 |
| o+144–152 | 9 | Water level | 0.1 m, value/10 − 10 | 511 |
| o+153–154 | 2 | Water level trend | as tendency | 3 |
| o+155–162 | 8 | Surface current speed | 0.1 kn | 255 |
| o+163–171 | 9 | Surface current direction | ° true | 511 |
| o+172–179 | 8 | Current speed 2 | 0.1 kn | 255 |
| o+180–188 | 9 | Current direction 2 | ° true | 511 |
| o+189–193 | 5 | Measuring depth 2 | m | 31 |
| o+194–201 | 8 | Current speed 3 | 0.1 kn | 255 |
| o+202–210 | 9 | Current direction 3 | ° true | 511 |
| o+211–215 | 5 | Measuring depth 3 | m | 31 |
| o+216–223 | 8 | Significant wave height | 0.1 m | 255 |
| o+224–229 | 6 | Wave period | s | 63 |
| o+230–238 | 9 | Wave direction | ° true | 511 |
| o+239–246 | 8 | Swell height | 0.1 m | 255 |
| o+247–252 | 6 | Swell period | s | 63 |
| o+253–261 | 9 | Swell direction | ° true | 511 |
| o+262–265 | 4 | Sea state | Beaufort (table below) | 13 |
| o+266–275 | 10 | Water temperature | 0.1 °C, value/10 − 10 | 1023 |
| o+276–278 | 3 | Precipitation type | WMO 4.201 (table below) | 7 |
| o+279–287 | 9 | Salinity | 0.1 ‰ | 511 |
| o+288–289 | 2 | Ice | 0 no, 1 yes, 2 reserved | 3 |
| o+290–295 | 6 | Spare | | |

[AIVDM "Type 8(1/11)"; Circ.236]. AIVDM lists the measuring depths as "units of 0.1 m", which a 5-bit field for 0–30 m cannot be; Circ.289 says "1 metre steps" for the same field and we use metres for both [Circ.289 Table 1.1].

### DAC 1 FI 31 — IMO SN.1/Circ.289 meteorological and hydrographic data (2010)

360 bits in a message 8 (304 data bits), two slots [Circ.289 Annex Table 1.1]. Longitude first, two's complement throughout, centimetre water level, distinct not-available values, and the *position must be present*: "this message should not be transmitted when positional information or time of measurement are not available" [Circ.289 Annex §1.2].

| Bits | Len | Field | Units | Not available | Notes |
|---|---|---|---|---|---|
| o+0–24 | 25 | Longitude | 1/1 000 min, signed | 181° = 10 860 000 | |
| o+25–48 | 24 | Latitude | 1/1 000 min, signed | 91° = 5 460 000 | |
| o+49 | 1 | Position accuracy | 1 = high (< 10 m) | default 0 | |
| o+50–54 | 5 | UTC day | 1–31 | 0 | |
| o+55–59 | 5 | UTC hour | 0–23 | 24 | |
| o+60–65 | 6 | UTC minute | 0–59 | 60 | |
| o+66–72 | 7 | Average wind speed | knots | 127 | 126 = ≥ 126 kn |
| o+73–79 | 7 | Wind gust | knots | 127 | 126 = ≥ 126 kn |
| o+80–88 | 9 | Wind direction | ° true | 360 | 361–511 reserved |
| o+89–97 | 9 | Wind gust direction | ° true | 360 | |
| o+98–108 | 11 | Air temperature | 0.1 °C, two's complement | −1024 | −60.0…+60.0 |
| o+109–115 | 7 | Relative humidity | % | 101 | |
| o+116–125 | 10 | Dew point | 0.1 °C, two's complement | 501 | −20.0…+50.0 |
| o+126–134 | 9 | Air pressure | 1–401 = 800–1200 hPa | 511 | 0 = ≤ 799, 402 = ≥ 1201 |
| o+135–136 | 2 | Pressure tendency | 0 steady, 1 decreasing, 2 increasing | 3 | |
| o+137–144 | 8 | Horizontal visibility | low 7 bits in 0.1 NM; MSB = "greater than" | 127 | 0.0–12.6 NM |
| o+145–156 | 12 | Water level | 0.01 m, value/100 − 10 | 4001 | −10.0…+30.0 m |
| o+157–158 | 2 | Water level trend | as tendency | 3 | |
| o+159–166 | 8 | Surface current speed | 0.1 kn | 255 | 251 = ≥ 25.1 |
| o+167–175 | 9 | Surface current direction | ° true | 360 | |
| o+176–183 | 8 | Current speed 2 | 0.1 kn | 255 | |
| o+184–192 | 9 | Current direction 2 | ° true | 360 | |
| o+193–197 | 5 | Measuring level 2 | m | 31 | |
| o+198–205 | 8 | Current speed 3 | 0.1 kn | 255 | |
| o+206–214 | 9 | Current direction 3 | ° true | 360 | |
| o+215–219 | 5 | Measuring level 3 | m | 31 | |
| o+220–227 | 8 | Significant wave height | 0.1 m | 255 | 251 = ≥ 25.1 |
| o+228–233 | 6 | Wave period | s | 63 | |
| o+234–242 | 9 | Wave direction | ° true | 360 | |
| o+243–250 | 8 | Swell height | 0.1 m | 255 | |
| o+251–256 | 6 | Swell period | s | 63 | |
| o+257–265 | 9 | Swell direction | ° true | 360 | |
| o+266–269 | 4 | Sea state | Beaufort | 13 | Table 1.2 |
| o+270–279 | 10 | Water temperature | 0.1 °C, two's complement | 501 | −10.0…+50.0 |
| o+280–282 | 3 | Precipitation type | WMO 4.201 | 7 | |
| o+283–291 | 9 | Salinity | 0.1 ‰ | 510 | 501 = ≥ 50.1; **511 = sensor not available** |
| o+292–293 | 2 | Ice | 0 no, 1 yes | 3 | |
| o+294–303 | 10 | Spare | "Not used. Set to zero" | | |

[Circ.289 Annex Table 1.1]. The enumerations shared by both versions:

| Sea state (Beaufort) [Circ.289 Table 1.2] | | Precipitation [Circ.289 Table 1.1, WMO 306 4.201] |
|---|---|---|
| 0 calm (flat) · 1 light air · 2 light breeze · 3 gentle breeze · 4 moderate breeze · 5 fresh breeze · 6 strong breeze · 7 near gale · 8 gale · 9 strong gale · 10 storm · 11 violent storm · 12 hurricane · 13 not available · 14–15 reserved | | 0 reserved · 1 rain · 2 thunderstorm · 3 freezing rain · 4 mixed/ice · 5 snow · 6 reserved · 7 not available |

Our output uses the same field names for both versions (`wind_speed_knots`, `air_temp_c`, `water_level_m`, …) so that a consumer can treat them alike; FI 31 adds `position_accuracy`, `visibility_greater_than` and `salinity_sensor_available`. **Not-available values print as `null`.** Chapter 3's convention of a `_available` boolean next to each value would add thirty-odd keys per message here; `null` says the same thing in one.

### DAC 366 FI 34 — USCG/NOAA right-whale zone notice, 2008 form (reverse-engineered)

> **This layout is not in any published document.** It is reconstructed from three sources in `sample-data/RightwhaleAisProject`: (1) the encoder's debug output, which prints each field's bit range and value as it encodes (`20081015-boston-pog/20081015-pog-update.log` lines 46–60 and the `ZoneMsg`/`ZoneCircular` dumps at lines 164–178); (2) the `BMS,…` request lines, which carry the 112 payload bits next to the zone number, status and time (`20081015-boston-pog/whale-update-2008-10-15.log` lines 4–13; `20081003-AIS/QM-Cornell-Tampa-2772008.log` lines 5–14); (3) what receivers heard (`20081013.log`, `20081003-AIS/ccom-sr162g2008-10-03.vdm`). The header fields are certain — the encoder names them. The circle's internal split is an inference explained below.

The encoder log for zone 1 on 15 October 2008:

```text
set_from_bits zonemsg w/ dac fi
dac 0 10 366
fi 10 16 34
 16 21 1
 21 28 0
 28 33 15
 33 38 15
 38 44 10
 44 52 10
setfrombits ZoneMsg 366:34 - 1 0 15 15 10 10 [zone_type: 0 [Right Whales NOT OBSERVED] ]
  length bits ... 128
  zones length ... 76
  num zones ... 1
zonebits (76): 0110101111000111010111011001000110000011101100110111000100100100001011000000
```

and the object dump that follows names the six header fields in order — `zone_id`, `zone_type`, `start_utc_day`, `start_utc_hour`, `start_utc_minute`, `zone_duration_minutes` — then "num zones: 1" and a `ZoneCircular` with `radius: 9260m`, `longitude: -70.678318`, `latitude: 42.347282`. Bit ranges are given relative to the start of the application identifier, so `16 21` is 5 bits immediately after DAC/FI. The data is therefore a sequence of **112-bit records** (36-bit header + one 76-bit zone); Cornell's queue manager packed up to five such records into one message 8 (§9.5), CCOM's script one per message.

| Bits (record-relative) | Len | Field | Notes | Evidence |
|---|---|---|---|---|
| r+0–4 | 5 | Zone id | 1–10, one per acoustic buoy TSS-AB01…AB10 | encoder `16 21`, BMS field 7 |
| r+5–11 | 7 | Zone type | 0 = right whales not observed, 1 = right whales present | encoder `21 28`, `zone_type: 0 [Right Whales NOT OBSERVED]`, `1 [Right Whales PRESENT]` |
| r+12–16 | 5 | Start UTC day | | encoder `28 33` |
| r+17–21 | 5 | Start UTC hour | | encoder `33 38` |
| r+22–27 | 6 | Start UTC minute | | encoder `38 44` |
| r+28–35 | 8 | Duration | **units of 10 minutes**: BMS payloads carry 1 where the dump says 10 minutes and 144 where it says 1440 | encoder `44 52`; BMS bits vs. dump |
| r+36 | 1 | Shape selector | always 0; see below | inference |
| r+37–64 | 28 | Longitude | 1/10 000 min, two's complement — the message 1 encoding | matches `-70.678318` |
| r+65–91 | 27 | Latitude | 1/10 000 min, two's complement | matches `42.347282` |
| r+92–107 | 16 | Radius | metres; always 9 260 = 5 NM | matches `9260m` |
| r+108–111 | 4 | Spare | zero | |

**What is inferred.** The 76 zone bits are `0 1101011110001110101110110010 001100000111011001101110001 0010010000101100 0000`. Reading bits 1–28 as a 28-bit two's-complement longitude in 1/10 000 minute gives −42 406 990 → −70.678317°, and bits 29–55 as a 27-bit latitude gives 25 408 369 → 42.347282°, both matching the dump to the last printed digit; the next 16 bits are 9 260. That fixes the position and radius. What the leading bit (or bits) and the trailing four bits *mean* cannot be decided from circles alone: a 2-bit shape field followed by a 27-bit longitude would decode to the same numbers, because the sign bit of a negative longitude repeats. We chose the M.1371 28/27-bit widths and call the first bit a shape selector, since the encoder's class is named `ZoneCircular` and the 2010 redesign (and Circ.289) begins every sub-area with a shape code [Circ.289 Table 11.4]. If you find a non-circular zone in another log, this is the first thing to revisit.

Compare the standardised successor [Circ.289 Table 11.1, 11.5]: message linkage id (10), notice description (7; 0 = "Caution Area: Marine mammals habitat", 1 = "… in area – reduce speed"), start month (4), day (5), hour (5), minute (6), duration in minutes (18), then 87-bit sub-areas: shape (3), scale factor (2), longitude (25) and latitude (24) in 1/1 000 min, precision (3), radius (12) × scale, spare (18). The 2008 form has no month, no linkage id, a coarser duration, a finer position and a wider radius — and it was 112 bits per zone instead of 55 + 87. The USCG registration of the project's 2010 equipment lists "Msg 8:366:22: every 5 minutes – 1 msg for each of 10 zones for the acoustic buoys", AtoN name `NOAA WHALE NOTICES`, replacing "a system that was using MMSI 3669739" [AtoN form v4] — i.e. by 2010 the project had moved to a USCG-registered FI 22 derived from the Circ.289 area notice, and the FI 34 form described here was retired.

### DAC 366 FI 56 and 57 — USCG encrypted AIS

Observable facts only, since the interface design description is not public [EAIS IDD]:

* FI 56 messages are always **312 bits** (256 data bits); FI 57 messages always **568 bits** (512 data bits), sent as two or three fragments.
* They come from ordinary ship MMSIs (366999699, 369493068, 369493092 in our data) and from a dedicated shore station (MMSI 3692… in `tide3` late 2010).
* The data block is different in every message, including consecutive ones from the same MMSI seconds apart, with no fixed sub-fields visible — consistent with a block cipher over a changing plaintext, inconsistent with a plain structured layout.
* They coexist with, or replace, the station's unencrypted messages 1/3/5 depending on the transponder's mode (`$PCNSS,BFP,…` "standard" / "silent" / "secure" in `sample-data/RightwhaleAisProject/cns-*-dos.txt`).

The decoder emits `dac`, `fi`, `data`, `data_bits` and nothing else. Do not add FI 56/57 to the registry with a guessed layout: a wrong decode of an encrypted message is worse than none.

### DAC 0 — test

"The test application identifier (DAC = 0) with any function identifier (0 to 63) should be used for testing purposes. The function identifier is arbitrary" [M.1371-5 Annex 5 §2.1.1]. Nothing can be decoded; the FI carries no information.

### How to read an ASM registry entry, and the families we do not decode

IALA keeps the international register of regional ASMs; the AIVDM document reproduces the commonly seen entries [AIVDM "Type 6", "Type 8"]. An entry is identified by **message type(s), DAC, FI** and sometimes a further *message ID* or *version* field inside the data (the Seaway family follows the application identifier with a 2-bit "Reserved" field and a 6-bit "Message ID" that selects the report type within the FI [Seaway R41 §1.1]; the UNH RFC proposed an "extended FID" for DAC 366 FI 63 [UNH RFC]). The families you are most likely to meet:

| Family | DAC / FI | Container | Content | Source |
|---|---|---|---|---|
| IMO international, 2004 trial set | 1 / 11–17 | 8 (11, 13, 15, 17), 6 (12, 14, 16) | met/hydro, dangerous cargo, fairway closed, tidal window, extended static, persons on board, pseudo-AIS targets; FI 11–15 withdrawn 2013 | [Circ.236; Circ.289 Table 1] |
| IMO international, 2010 | 1 / 16–32 | 6 and 8 | 16 persons on board, 17 VTS-generated targets, 18 clearance time, 19 marine traffic signal, 20 berthing, 21 weather from ship, 22/23 area notice, 24 extended static, 25 dangerous cargo, 26 environmental, 27/28 route, 29/30 text, 31 met/hydro, 32 tidal window; 33–63 reserved | [Circ.289 Table 1; Circ.290] |
| ITU system IFMs | 1 / 0–5 | 6, 8, 25, 26 | 0 text, 2 interrogation for an IFM, 3 capability interrogation, 4 capability reply, 5 application acknowledgement | [M.1371-5 Annex 5 Table 24, §5] |
| St. Lawrence Seaway / PAWSS | 316 or 366 / 1, 2, 32 | 8 (and 6) | FI 1: wind, weather station, water level, water flow (plus PAWSS hydro/current, salinity/temperature); FI 2: lockage order, estimated lock times, vessel procession order; FI 32: version | [Seaway R41 §1.1, §3–9; AIVDM "Type 8"] |
| USCG whale notices | 366 / 34 (2008), 366 / 22 (2010) | 8 | this chapter; the 2010 form is an area notice | [AtoN form v4] |
| USCG encrypted | 366 / 56, 57 | 8 | encrypted position and static | [EAIS IDD] |
| UNH water level experiment | 366 / various, proposal for 63 as extended FID | 8 | tide-gauge water level with XML-defined layouts; the `tide1` recordings | [UNH RFC; RTCM SC121] |
| Inland AIS (Europe) | 200 / 10, 21–24, 40, 55 | 8 (10, 23, 24, 40), 6 (21, 22, 55) | inland ship static data, ETA/RTA at lock, EMMA warning, water levels, signal status, persons on board | [AIVDM "Inland AIS"] |
| UK / Ireland AtoN monitoring | 235, 250 / 10 | 6 | AtoN monitoring data | [AIVDM "Type 6"] |

A registry entry is only half the story. Before writing a decoder, find the *length* the layout implies and reject messages that do not match it (chapter 16 makes the same point about noise); check whether a version or message-id sub-field exists; and check the date — the same DAC/FI may have changed meaning (Circ.236 → Circ.289) or been re-used (FI 22 under DAC 1 and DAC 366 are unrelated).

## 9.3 Decoding one message by hand

### The whale notice, bits 56–167

Chapter 8 §8.3 decoded the header of line 10 of [`data/ch09-asm.nmea`](../data/ch09-asm.nmea) (the same sentence as chapter 8's line 8): message 8 from 3 669 734, DAC 366, FI 34. The 112 data bits, with the six-bit groups they come from. Bit 56 is the third bit of the tenth character (`P`, bits 54–59), so the record straddles character boundaries from the start:

```text
P      P      7      f      b      0      F      g      7      G
100000 100000 000111 101110 101010 000000 010110 101111 000111 010111
  ^ bit 56

I      6      3      d      o      4      T      ;      0
011001 000110 000011 101100 110111 000100 100100 001011 000000
```

Record-relative offsets (r = 56):

| Bits | Value | Field |
|---|---|---|
| 56–60 | `00001` = 1 | zone id → buoy TSS-AB01 |
| 61–67 | `0000000` = 0 | zone type → right whales not observed |
| 68–72 | `01111` = 15 | start day |
| 73–77 | `01110` = 14 | start hour |
| 78–83 | `101010` = 42 | start minute → 15 Oct 14:42 UTC, which is when the sentence was logged (`1224081725` = 14:42:05) |
| 84–91 | `00000001` = 1 | duration → 10 minutes |
| 92 | `0` | shape selector |
| 93–120 | `1101011110001110101110110010` = 226 028 466 → −42 406 990 | longitude: /600 000 = **−70.678317°** |
| 121–147 | `001100000111011001101110001` = 25 408 369 | latitude: /600 000 = **42.347282°** |
| 148–163 | `0010010000101100` = **9 260** | radius, metres (5 NM) |
| 164–167 | `0000` | spare |

So: *no whales heard at buoy 1 in the last 10 minutes; the 5-mile circle around 42.3473° N 70.6783° W is clear.* A "present" record (line 11, zone 3) differs in type 1, a start time 9 hours earlier (05:31, the time of the last detection) and duration 144 → 1 440 minutes: the 24-hour speed restriction counted from the detection [RTCM SC121].

### A Circ.289 met/hydro message (synthetic)

No met/hydro message was recorded by our receivers, so line 27 was **built by hand** for this chapter: a plausible observation for a station at Portsmouth Harbor, New Hampshire, on 15 October at 14:30 UTC, from a made-up coast-station MMSI 003661234. The field values were chosen first, encoded with the Table 1.1 widths, armoured and checksummed; the sentence is then decoded like any other. The first 20 characters:

```text
8      0      3      O      M      d      P      0      G      u      v      Q      U      1      >
001000 000000 000011 011111 011101 101100 100000 000000 010111 111101 111110 100001 100101 000001 001110

o      N      U      s      W
110111 011110 100101 111011 100111
```

| Bits | Value | Field |
|---|---|---|
| 40–49 | `0000000001` = 1 | DAC international |
| 50–55 | `011111` = 31 | FI 31 |
| 56–80 | `1101111110100001100101000` → −4 242 648 | longitude: /60 000 = **−70.7108°** |
| 81–104 | `001001110110111011110100` = 2 584 308 | latitude: /60 000 = **43.0718°** |
| 105 | `1` | position accuracy high |
| 106–110 | `01111` = 15 | day |
| 111–115 | `01110` = 14 | hour |
| 116–121 | `011110` = 30 | minute |
| 122–128 | `0001100` = 12 | wind 12 kn |
| 129–135 | `0010010` = 18 | gust 18 kn |
| 136–144 | `011100001` = 225 | wind direction |
| 145–153 | `011100110` = 230 | gust direction |
| 154–164 | `00001110010` = 114 | air temperature 11.4 °C |
| 165–171 | `1001110` = 78 | humidity 78 % |
| 172–181 | `0001001100` = 76 | dew point 7.6 °C |
| 182–190 | `011010110` = 214 | pressure 214 + 799 = 1 013 hPa |
| 191–192 | `01` = 1 | decreasing |
| 193–200 | `01010000` = 80 | visibility 8.0 NM, MSB 0 so not "greater than" |
| 201–212 | `010001100011` = 1 123 | water level 11.23 − 10 = **+1.23 m** |
| 213–214 | `10` = 2 | rising |
| 215–222 | `00001110` = 14 | surface current 1.4 kn |
| 223–231 | `001011111` = 95 | setting 095° |

The remaining fields were set to "not available" except waves (0.5 m, 4 s, 110°), sea state 2, water temperature 13.2 °C, salinity 31.5 ‰ and ice 0. Line 28 is the same message with *every* field at its not-available value, and line 29 the same observation in the Circ.236 FI 11 layout — note that its longitude is −70.7108 too but the water level is **1.2 m**, because FI 11 only has decimetres.

## 9.4 The code

The message modules are in chapter 8. [`code/python/aistut/asm.py`](../code/python/aistut/asm.py) holds the four ASM decoders; the C++, Rust and TypeScript twins are [`asm.hpp`](../code/cpp/asm.hpp), [`asm.rs`](../code/rust/src/asm.rs) and [`asm.ts`](../code/typescript/src/asm.ts). Each decoder has the signature `(bits, o, data_bits) → fields`, where `o` is the first data bit.

### Python

Two helpers do most of the work for the met/hydro tables:

```python
def _na(raw: int, sentinel: int) -> Optional[int]:
    """`raw` unless it equals the not-available sentinel, in which case None."""
    return None if raw == sentinel else raw


def _scaled(raw: int, sentinel: int, divisor: float, decimals: int,
            offset: float = 0.0) -> Optional[Fixed]:
    """raw / divisor + offset with `decimals` places, or None if raw == sentinel."""
    if raw == sentinel:
        return None
    return Fixed(raw / divisor + offset, decimals)
```

The whale notice is a loop over fixed-size records:

```python
WHALE_RECORD_BITS = 112
WHALE_ZONE_TYPE = {0: "right whales not observed", 1: "right whales present"}


def decode_whale_notice(bits: Bits, o: int, data_bits: int) -> dict:
    notices: list[dict] = []
    r = o
    while bits.has(r, WHALE_RECORD_BITS):
        zone_type = bits.uint(r + 5, 7)
        duration_raw = bits.uint(r + 28, 8)
        z = r + 36
        notices.append({
            "zone_id": bits.uint(r, 5),
            "zone_type": zone_type,
            "zone_type_text": WHALE_ZONE_TYPE.get(zone_type, "unknown"),
            "start_day": bits.uint(r + 12, 5),
            "start_hour": bits.uint(r + 17, 5),
            "start_minute": bits.uint(r + 22, 6),
            "duration_raw": duration_raw,
            "duration_minutes": duration_raw * 10,
            "shape": bits.uint(z, 1),
            "lon": Fixed(bits.sint(z + 1, 28) / 600000.0, 6),
            "lat": Fixed(bits.sint(z + 29, 27) / 600000.0, 6),
            "radius_m": bits.uint(z + 56, 16),
            "spare": bits.uint(z + 72, 4),
        })
        r += WHALE_RECORD_BITS
    return {"notices": notices, "trailing_bits": o + data_bits - r}
```

The Circ.289 decoder is a long literal; the interesting lines are the ones where the two editions differ:

```python
def decode_met_hydro_289(bits: Bits, o: int, data_bits: int) -> dict:
    lon_raw = bits.sint(o, 25)
    lat_raw = bits.sint(o + 25, 24)
    ...
    vis_raw = bits.uint(o + 137, 8)
    ...
    salinity_raw = bits.uint(o + 283, 9)
    return {
        "lon": None if lon_raw == LON_1000_NOT_AVAILABLE else Fixed(lon_raw / 60000.0, 6),
        "lat": None if lat_raw == LAT_1000_NOT_AVAILABLE else Fixed(lat_raw / 60000.0, 6),
        "position_accuracy": bits.uint(o + 49, 1),
        "day": _na(bits.uint(o + 50, 5), 0),
        "hour": _na(bits.uint(o + 55, 5), 24),
        ...
        "air_temp_c": _scaled(bits.sint(o + 98, 11), -1024, 10.0, 1),      # two's complement
        ...
        "pressure_hpa": _pressure_289(bits.uint(o + 126, 9)),
        ...
        "visibility_nm": _scaled(vis_raw & 0x7F, 127, 10.0, 1),
        "visibility_greater_than": (vis_raw >> 7) == 1,
        "water_level_m": _scaled(bits.uint(o + 145, 12), 4001, 100.0, 2, -10.0),
        ...
        "salinity_ppt": None if salinity_raw >= 510 else Fixed(salinity_raw / 10.0, 1),
        "salinity_sensor_available": salinity_raw != 511,
        ...
    }
```

versus the Circ.236 version:

```python
def decode_met_hydro_236(bits: Bits, o: int, data_bits: int) -> dict:
    lat_raw = bits.sint(o, 24)            # latitude FIRST in Circ.236
    lon_raw = bits.sint(o + 24, 25)
    ...
    return {
        "lat": None if lat_raw == 0x7FFFFF else Fixed(lat_raw / 60000.0, 6),
        "lon": None if lon_raw == 0xFFFFFF else Fixed(lon_raw / 60000.0, 6),
        "day": _na(bits.uint(o + 49, 5), 31),
        ...
        "air_temp_c": _scaled(bits.uint(o + 97, 11), 2047, 10.0, 1, -60.0),   # unsigned + offset
        ...
        "pressure_hpa": None if bits.uint(o + 125, 9) == 511 else bits.uint(o + 125, 9) + 800,
        ...
        "water_level_m": _scaled(bits.uint(o + 144, 9), 511, 10.0, 1, -10.0),
        ...
    }
```

And the text telegram:

```python
def decode_text_telegram(bits: Bits, o: int, data_bits: int) -> dict:
    text_bits = (data_bits - 12) // 6 * 6
    return {
        "ack_required": bits.flag(o),
        "text_sequence_number": bits.uint(o + 1, 11),
        "text": bits.text(o + 12, text_bits),
        "text_chars": text_bits // 6,
        "spare_bits": data_bits - 12 - text_bits,
    }
```

> **Language notes (Python).** `Optional[Fixed]` is "a `Fixed` or `None`"; the JSON writer prints `None` as `null`. `(data_bits - 12) // 6 * 6` rounds down to a multiple of 6 — integer division then multiplication. `dict.get(key, default)` is how `zone_type_text` survives a value the table does not list. Nothing else here is new since chapter 3; the message is long, the constructs are not.

### C++

The two helpers return `Json` directly, because a `Json` already has a null state:

```cpp
inline Json na(std::uint64_t raw, std::uint64_t sentinel) {
    if (raw == sentinel) return nullptr;
    return Json(static_cast<long long>(raw));
}

inline Json scaled(std::int64_t raw, std::int64_t sentinel, double divisor, int decimals,
                   double offset = 0.0) {
    if (raw == sentinel) return nullptr;
    return Json::fixed(static_cast<double>(raw) / divisor + offset, decimals);
}
```

The record loop:

```cpp
inline Json decode_whale_notice(const Bits& bits, std::size_t o, std::size_t data_bits) {
    Json::Array notices;
    std::size_t r = o;
    while (bits.has(r, WHALE_RECORD_BITS)) {
        std::uint64_t zone_type = bits.uint(r + 5, 7);
        std::uint64_t duration_raw = bits.uint(r + 28, 8);
        std::size_t z = r + 36;
        Json notice = Json::Object{
            {"zone_id", bits.uint(r, 5)},
            {"zone_type", zone_type},
            {"zone_type_text", whale_zone_type_text(zone_type)},
            // ...
            {"lon", Json::fixed(static_cast<double>(bits.sint(z + 1, 28)) / 600000.0, 6)},
            {"lat", Json::fixed(static_cast<double>(bits.sint(z + 29, 27)) / 600000.0, 6)},
            {"radius_m", bits.uint(z + 56, 16)},
            {"spare", bits.uint(z + 72, 4)},
        };
        notices.push_back(std::move(notice));
        r += WHALE_RECORD_BITS;
    }
    return Json::Object{
        {"notices", notices},
        {"trailing_bits", o + data_bits - r},
    };
}
```

A Circ.289 field that differs in signedness from its Circ.236 cousin:

```cpp
// Circ.289: two's complement, sentinel -1024
{"air_temp_c", scaled(bits.sint(o + 98, 11), -1024, 10.0, 1)},
// Circ.236: unsigned with an offset, sentinel 2047
{"air_temp_c", scaled(static_cast<std::int64_t>(bits.uint(o + 97, 11)), 2047, 10.0, 1, -60.0)},
```

> **Language notes (C++).**
> * `scaled` takes `std::int64_t` so that one helper serves both the signed Circ.289 fields (from `bits.sint`) and the unsigned Circ.236 fields; the `static_cast<std::int64_t>` at the unsigned call sites documents the conversion, which `-Wextra` would otherwise flag.
> * `return nullptr;` from a function returning `Json` uses the `Json(std::nullptr_t)` constructor — C++'s spelling of `None`.
> * A `const char* const T[] = {…}` inside a function with `static` storage is a string table built once; indexing with `code & 3` keeps a 2-bit value in range without a bounds check.
> * `notices.push_back(std::move(notice))` moves the object into the vector instead of copying it. Building the object in a named variable first also sidesteps a spurious "may be used uninitialized" warning that some GCC versions emit for a temporary `Json::Object` passed straight to `push_back`.

### Rust

```rust
fn na(raw: u64, sentinel: u64) -> Option<u64> {
    if raw == sentinel { None } else { Some(raw) }
}

fn scaled(raw: i64, sentinel: i64, divisor: f64, decimals: usize, offset: f64) -> Option<Json> {
    if raw == sentinel { None } else { Some(Json::fixed(raw as f64 / divisor + offset, decimals)) }
}
```

Inside the Circ.236/289 decoders a small closure saves repeating `bits.uint(o + …) as i64`:

```rust
    let u = |start: usize, len: usize| bits.uint(o + start, len) as i64;
    // ...
    put(&mut out, "air_temp_c", scaled(u(97, 11), 2047, 10.0, 1, -60.0));   // Circ.236
    put(&mut out, "air_temp_c", scaled(bits.sint(o + 98, 11), -1024, 10.0, 1, 0.0));   // Circ.289
```

The pressure rule as a `match` with ranges:

```rust
fn pressure_289(raw: u64) -> Option<u64> {
    match raw {
        511 => None,
        0 => Some(799),          // "pressure 799 hPa or less"
        402..=510 => Some(1201), // "pressure 1201 hPa or greater"
        _ => Some(raw + 799),
    }
}
```

And the record loop:

```rust
pub fn decode_whale_notice(bits: &Bits, o: usize, data_bits: usize) -> Obj {
    let mut notices: Vec<Json> = Vec::new();
    let mut r = o;
    while bits.has(r, WHALE_RECORD_BITS) {
        let zone_type = bits.uint(r + 5, 7);
        let duration_raw = bits.uint(r + 28, 8);
        let z = r + 36;
        let mut n = Obj::new();
        put(&mut n, "zone_id", bits.uint(r, 5));
        // ...
        put(&mut n, "lon", Json::fixed(bits.sint(z + 1, 28) as f64 / 600000.0, 6));
        put(&mut n, "lat", Json::fixed(bits.sint(z + 29, 27) as f64 / 600000.0, 6));
        put(&mut n, "radius_m", bits.uint(z + 56, 16));
        put(&mut n, "spare", bits.uint(z + 72, 4));
        notices.push(Json::Obj(n));
        r += WHALE_RECORD_BITS;
    }
    let mut out = Obj::new();
    put(&mut out, "notices", notices);
    put(&mut out, "trailing_bits", o + data_bits - r);
    out
}
```

> **Language notes (Rust).**
> * `Option<u64>` and `Option<Json>` go straight into `put(...)` because `jsonout.rs` implements `From<Option<T>> for Json`; `None` becomes `null`.
> * `let u = |start, len| bits.uint(o + start, len) as i64;` is a *closure* that captures `bits` and `o` by reference; it is only valid while they are.
> * `402..=510 =>` is an inclusive range pattern in a `match` arm. The compiler checks that the arms together cover every `u64` (the `_` arm does).
> * `raw as f64` and `raw as i64` are explicit numeric casts; Rust never converts integer widths silently.

### TypeScript

```typescript
function na(raw: number, sentinel: number): number | null {
  return raw === sentinel ? null : raw;
}

function scaled(raw: number, sentinel: number, divisor: number, decimals: number, offset = 0): Fixed | null {
  if (raw === sentinel) return null;
  return new Fixed(raw / divisor + offset, decimals);
}
```

The visibility field, where C++/Rust/Python use `& 0x7F` and `>> 7`:

```typescript
    visibility_nm: scaled(visRaw % 128, 127, 10.0, 1), // low 7 bits
    visibility_greater_than: visRaw >= 128, // the MSB
```

The record loop:

```typescript
export function decodeWhaleNotice(bits: Bits, o: number, dataBits: number): JsonObject {
  const notices: JsonObject[] = [];
  let r = o;
  while (bits.has(r, WHALE_RECORD_BITS)) {
    const zoneType = bits.uint(r + 5, 7);
    const durationRaw = bits.uint(r + 28, 8);
    const z = r + 36;
    notices.push({
      zone_id: bits.uint(r, 5),
      zone_type: zoneType,
      zone_type_text: WHALE_ZONE_TYPE[zoneType] ?? "unknown",
      // ...
      lon: new Fixed(bits.sint(z + 1, 28) / 600000.0, 6),
      lat: new Fixed(bits.sint(z + 29, 27) / 600000.0, 6),
      radius_m: bits.uint(z + 56, 16),
      spare: bits.uint(z + 72, 4),
    });
    r += WHALE_RECORD_BITS;
  }
  return { notices, trailing_bits: o + dataBits - r };
}
```

> **Language notes (TypeScript).**
> * `Fixed | null` and `number | null` are the union types of chapter 1; the JSON writer prints `null`. `WHALE_ZONE_TYPE[zoneType] ?? "unknown"` is the nullish-coalescing default of chapter 1 — it fires on an index miss, not on `0`.
> * `visRaw % 128` and `visRaw >= 128` avoid the bitwise operators on purpose: `&` and `>>` convert to 32-bit integers in JavaScript, harmless for an 8-bit field but a habit chapter 2 told you to drop.
> * `offset = 0` in a parameter list is a default argument, as in Python; its type is inferred as `number`.

### Where the four differ

| Concern | Python | C++ | Rust | TypeScript |
|---|---|---|---|---|
| "value or not available" | `Optional[...]`, `None` | `Json` with `nullptr` | `Option<T>` → `null` via `From` | `T \| null` |
| Signed/unsigned helpers | one function, Python ints | `std::int64_t` + `static_cast` | `as i64` casts, closure `u` | one `number` type |
| Enumeration tables | `dict` with `.get` | `static const char*[]` indexed with a mask | `match` with `_` arm | object with `??` |
| Bit masks | `& 0x7F`, `>> 7` | same | same | `% 128`, `>= 128` |
| Record list | `list[dict]` | `Json::Array` + `std::move` | `Vec<Json>` | `JsonObject[]` |

## 9.5 Sample data and results

[`data/ch09-asm.nmea`](../data/ch09-asm.nmea) has six groups:

1. Whale notices: one record each from Boston (lines 10–11, zone 1 "not observed" and zone 3 "present"), one record plus padding from Provincetown via N-AIS (line 12), and two two-fragment messages from the Tampa bench test with five and four records (lines 13–16; `sample-data/RightwhaleAisProject/20081003-AIS/ccom-sr162g2008-10-03.vdm` lines 21–24).
2. The text telegram (line 20).
3. **Synthetic** met/hydro: FI 31 realistic (27), FI 31 all-defaults (28), FI 11 realistic (29).
4. Encrypted FI 56 (33) and FI 57 (34–35) from New Castle NH, December 2010.
5. DAC 0: a message 6 (40) and a 120-bit message 8 from a ship (41).
6. The L-3 AtoN bench test (46–47).

```sh
python3 code/python/decode.py data/ch09-asm.nmea
code/cpp/build/decode data/ch09-asm.nmea
code/rust/target/release/decode data/ch09-asm.nmea
node code/typescript/dist/decode.js data/ch09-asm.nmea
```

All four produce the fifteen lines of [`tests/expected/ch09-asm.jsonl`](../tests/expected/ch09-asm.jsonl). Selected objects (`"ais"` part unless stated):

**Line 11 — zone 3, whales present:**

```json
{"application":{"application_name":"USCG/NOAA right whale zone notice (2008)","notices":[{"duration_minutes":1440,"duration_raw":144,"lat":42.333247,"lon":-70.454335,"radius_m":9260,"shape":0,"spare":0,"start_day":15,"start_hour":5,"start_minute":31,"zone_id":3,"zone_type":1,"zone_type_text":"right whales present"}],"trailing_bits":0},"dac":366,"data":"0001100000010111100101011111100100000110101111010111110001010011100110000011100100101000110000100100001011000000","data_bits":112,"fi":34,"mmsi":3669734,"msg_type":8,"repeat_indicator":0,"spare":0}
```

Compare with the `BMS` line that produced it (`whale-update-2008-10-15.log` line 6): `…,3,1001,5,1224167556,0101101110100010,0001100000010111100101011111100100000110…` — the `data` string is the BMS payload, bit for bit, and the request's "1001, 5" corresponds to `zone_type` 1.

**Line 14 — five records in one 616-bit message** (`notices` abbreviated):

```json
"notices":[{"zone_id":1,"zone_type":0,"start_day":3,"start_hour":14,"start_minute":11,"duration_minutes":10,"lon":-70.678317,"lat":42.347282,...},{"zone_id":2,...,"lon":-70.566315,"lat":42.340320,...},{"zone_id":3,...,"start_hour":9,"start_minute":18,"duration_minutes":1440,"lon":-70.454335,"lat":42.333247,...},{"zone_id":4,...,"duration_minutes":1440,"lon":-70.342377,"lat":42.326065,...},{"zone_id":5,...,"start_hour":7,"start_minute":0,"duration_minutes":1440,"lon":-70.230442,"lat":42.318773,...}],"trailing_bits":0
```

`payload_bits` 616 = 56 + 5 × 112, `fragments` 2. The five circles march eastward along the Boston traffic separation scheme, about 9.2 km apart. Notice that zones 3–5 have 1 440-minute durations and detection-time starts but **zone type 0**: on 3 October the Cornell encoder was still sending "not observed" for buoys that had heard whales, while its queue manager's request said `1001,5` ("present"). By 15 October (line 11) the type bit was being set. A reverse-engineered format is only as stable as the software that writes it — which is one more reason to keep `data`.

**Line 16 — four records, 512 bits:** zones 6, 8, 9, 10 (buoy 7 is absent) and `"trailing_bits":8`: 4 × 112 = 448 data bits plus one byte of padding, 456 in all.

**Line 12 — Provincetown, one record:** `"data_bits":120,"trailing_bits":8` — the chapter 8 fill-bit story, seen from the application layer: the record decodes cleanly and the loop stops because a second record does not fit.

**Line 20 — the text telegram:** `"application":{"ack_required":false,"application_name":"IFM 0 text telegram","spare_bits":2,"text":"","text_chars":3,"text_sequence_number":0}`.

**Line 27 — the synthetic FI 31:**

```json
{"air_temp_c":11.4,"application_name":"IMO Circ.289 meteorological and hydrographic data","current_2_depth_m":null,"current_2_dir_deg":null,"current_2_speed_knots":null,"current_3_depth_m":null,"current_3_dir_deg":null,"current_3_speed_knots":null,"current_dir_deg":95,"current_speed_knots":1.4,"day":15,"dew_point_c":7.6,"hour":14,"humidity_pct":78,"ice":0,"ice_text":"no","lat":43.071800,"lon":-70.710800,"minute":30,"position_accuracy":1,"precipitation":7,"precipitation_text":"not available","pressure_hpa":1013,"pressure_tendency":1,"pressure_tendency_text":"decreasing","salinity_ppt":31.5,"salinity_sensor_available":true,"sea_state":2,"sea_state_text":"light breeze (small wavelets)","spare":0,"swell_dir_deg":null,"swell_height_m":null,"swell_period_s":null,"visibility_greater_than":false,"visibility_nm":8.0,"water_level_m":1.23,"water_level_trend":2,"water_level_trend_text":"increasing","water_temp_c":13.2,"wave_dir_deg":110,"wave_height_m":0.5,"wave_period_s":4,"wind_dir_deg":225,"wind_gust_dir_deg":230,"wind_gust_knots":18,"wind_speed_knots":12}
```

**Line 28 — FI 31, everything not available:** every sensor value is `null`; the enumerations show their sentinel codes with text — `"ice":3,"ice_text":"not available"`, `"pressure_tendency":3`, `"sea_state":13`, `"precipitation":7` — and `"position_accuracy":0`, `"salinity_sensor_available":true` (510 means "not available", only 511 means "no sensor").

**Line 29 — FI 11:** the same observation; `"application_name":"IMO Circ.236 meteorological and hydrological data"`, `"water_level_m":1.2`, no `position_accuracy`, no `visibility_greater_than`, no `salinity_sensor_available`, and `"data_bits":296`.

**Lines 33 and 35 — encrypted:** `"dac":366,"fi":56,"data_bits":256,"mmsi":369493068` and `"fi":57,"data_bits":512`, no `application`.

**Lines 40–41 — DAC 0:** the message 6 from MMSI 8 388 608 (`"dac":0,"fi":0,"data_bits":136`) and the message 8 from ship 369101122 (`"dac":0,"fi":11,"data_bits":64`). FI 11 under DAC 0 has nothing to do with met/hydro — and at 64 data bits could not be one anyway.

**Lines 46–47 — the L-3 AtoN:** `"dac":91,"fi":37,"data_bits":152` and `"data_bits":240`, from MMSI 3669739. §9.6 explains what they really are.

## 9.6 Pitfalls and real-world deviations

* **The L-3 message that is two bits off.** Lines 46–47 come from an AIS aid-to-navigation (AtoN) station under bench test at UNH in November 2010 (`sample-data/2010-11-l3-aton/ccom-sr162g-2010-11-02`). At the Table 57 offsets they read DAC 91 FI 37 — no such DAC. Read the same 16 bits **two positions later**, bits 42–57, and they are `0101101110 010110`: DAC **366**, FI **22**, the USCG-registered area notice [AtoN form v4]. Continuing at that offset with the Circ.289 header layout [Circ.289 Table 11.1] gives message linkage id 10, notice description 0 ("Caution Area: Marine mammals habitat") and 5 ("Protected habitat – stay clear") on the two lines, start 6 July 00:00, duration 60 minutes, followed by a circle sub-area at 69.8° W, 42.0° N (line 46) or 42.25° N (line 47), with a zero radius — the test pattern of a whale-notice AtoN, with lon/lat in the 28/27-bit M.1371 widths rather than Circ.289's 25/24. The two stray bits are almost certainly the application having included the message's own 2-bit spare field in the BBM data (bits 38–39 and 40–41 are both `00`). A decoder that keys strictly on bits 40–55 sees garbage; a human sees a byte-alignment bug. We decode what the spec says and let `data` carry the evidence.
* **Reverse-engineered formats drift.** The Tampa test of 3 October 2008 sent zone type 0 with 24-hour durations; the Boston script of 15 October sent type 1. Same DAC/FI, same length, different semantics two weeks apart. When you reconstruct a format, record the date range of the samples it was derived from.
* **One record or many?** The 2008 whale format has no record count; the receiver infers it from the length. With byte padding (lines 12 and 16) the remainder is 8 bits, well short of 112, so the loop stops cleanly — but a format with 8-bit records would be ambiguous. Circ.289 avoids this for the area notice by fixing sub-areas at 87 bits and saying "total number of sub-areas is determined by the length of the message" [Circ.289 Table 11.1]: a 5-sub-area message has 56 + 55 + 435 = 546 bits, which the transponder pads to 552, leaving 6 bits — again short of a record. The design rule: make the record length large compared with 7.
* **Not available is not zero.** FI 11 uses all-ones; FI 31 uses a specific value per field, sometimes at the top of the range (127, 255, 511), sometimes just above the valid range (101, 24, 60, 360, 4001, 501), sometimes *zero* (day). A decoder that treats 0 as "missing" throws away a valid midnight hour and a valid wind direction of 000°; one that treats the maximum as missing on FI 31 loses "≥ 126 knots". Carry the sentinel from the table, field by field.
* **Latitude first.** Circ.236 FI 11 puts latitude before longitude; Circ.289 FI 31 and every ITU message put longitude first. A decoder that reuses the FI 31 position code for FI 11 will place the Portsmouth station at 70.7° S.
* **Offset versus two's complement.** FI 11's air temperature field is "−60.0 to +60.0 in 0.1 °C" with 2047 as not available. If it were two's complement, 2047 would be −1 and the range would not fit 11 bits symmetrically; the working interpretation, used by the free decoders, is an unsigned value with −60 added [AIVDM "Type 8(1/11)"]. FI 31 says "as per 2's complement" explicitly [Circ.289 Table 1.1]. When a table does not say, the message is ambiguous and both encoders and decoders will have guessed.
* **Visibility's hidden flag.** FI 31's 8-bit visibility is really a 1-bit flag and a 7-bit value: "the most significant bit indicates that the maximum range of the visibility equipment was reached" [Circ.289 Table 1.1]. Decode `raw & 0x7F` and `raw >> 7` separately, and note that the not-available value 127 lives in the 7-bit part.
* **Pressure's two encodings.** FI 11: `raw + 800`. FI 31: `raw + 799`, with 0 and 402 as "≤ 799" and "≥ 1201". Off-by-one between editions of the same message.
* **Encrypted traffic dominates the counts.** In every US recording after 2007 the most frequent message 8s are FI 56/57. A pipeline that decodes "all ASMs" will spend most of its time on bits it cannot read; filter on `(dac, fi)` before decoding, and count the rest.
* **Noise with a valid DAC/FI.** Line 41 is a 120-bit message 8 with DAC 0 FI 11 from a ship; `tide1` also has 8/1/41 (448 bits) and dozens of 6/x/y with random DACs, all from the noisy `nhjel` receiver. Validate the length against the table *and* the plausibility of the source (a ship does not broadcast met/hydro; a base station does not broadcast at 11 bits per field) before believing a decode.

## 9.7 Exercises

1. **(Pencil)** Decode the second record of line 14 by hand (bits 168–279) and confirm zone 2 at 42.340320° N 70.566315° W. How many minutes after the first record's start does it begin?
2. **(Pencil)** Encode a Circ.289 FI 31 message for a station at 42.3601° N 71.0589° W with wind 25 kn gusting 40 from 045°, pressure 998 hPa falling, water level −0.45 m, everything else not available. Give the 60 payload characters and fill bits; check by decoding.
3. **(Python)** The L-3 messages of §9.6 decode as a Circ.289-style area notice if you start two bits late. Write `tools/shifted_asm.py` that, for any message 8 whose DAC is not in a list of known DACs, tries offsets 40 ± 1, ± 2 and reports any that yield a known `(dac, fi)`. Run it on `sample-data/2010-11-l3-aton/ccom-sr162g-2010-11-02`.
4. **(C++)** Add a decoder for the Circ.289 area notice (DAC 1 FI 22) covering the header and the circle sub-area only [Circ.289 Tables 11.1, 11.5]; reject other shapes with `"error"`. Build a synthetic test sentence and add it to a scratch data file (not `data/`).
5. **(Rust)** `decode_whale_notice` trusts every record it finds. Add range checks — zone id 1–10, zone type 0–1, radius 9 260, latitude within 41–43° N — and set a per-record `"plausible": false` instead of rejecting. Which records in `data/ch09-asm.nmea` fail?
6. **(TypeScript)** Write a function that takes the decoded `notices` of all whale messages in a log and produces a per-zone timeline of `present`/`not observed` intervals. Run it over the 5 060 messages in `sample-data/RightwhaleAisProject/20081013.log` (two fragments are rare; one is common) and report how many hours each zone was "present" on 13 October 2008.
7. **(Any)** Measure the slot cost: for each `(dac, fi)` in `20081013.log`, sum `ceil((payload_bits + 32·k) / 256)` slots per message under the Table 58 rule and compare the encrypted traffic's share of the channel with the whale notices'.

## 9.8 References

* [M.1371-5] Annex 5 §2.1 Table 23 (application identifier), §2.1.1 (DAC 0), §2.1.2 Table 24 (IFMs), §4 (drafting rules, byte boundaries), §5.1 Tables 25/26 (IFM 0 text telegram); Annex 8 Tables 54, 57 (containers, chapter 8).
* [Circ.236] IMO SN/Circ.236 — trial message 1 (FI 11) and the other FI 11–17 layouts; cited through [AIVDM "Type 8(1/11)"].
* [Circ.289] Annex §1.2, Table 1 (FI allocation), Table 1.1 (FI 31), Table 1.2 (Beaufort), §11 and Tables 11.1, 11.4, 11.5, 11.11 (area notice), §11.7 (slot count).
* [Circ.290] presentation of ASMs.
* [IHR 2010] history of the met/hydro messages.
* [AtoN form v4] the 2010 registration of the whale-notice station (FI 22, every 5 minutes, ten zones, MMSI 3669739 replaced).
* [RTCM SC121] the 2007 proposal behind the whale notices and XML message definitions.
* [UNH RFC] the 2006 water-level experiment and the extended-FID proposal.
* [Seaway R41] §1.1 (DAC 316/366, FI 1/2/32), §1.2, §3–9.
* [EAIS IDD] encrypted AIS (not public).
* [AIVDM] "Type 6", "Type 8", "Type 8(1/11)", "Type 8(1/31)", "Inland AIS", "Byte Alignment".
* [M.585-9] MMSI formats.
