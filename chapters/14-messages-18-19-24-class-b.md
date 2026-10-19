# 14. Messages 18, 19 and 24: Class B

> **What you will learn**
> * What a Class B transponder is, why it exists, and what it deliberately
>   leaves out compared with the Class A equipment of chapters 3 and 7: no
>   navigational status, no rate of turn, no destination, ETA, draught or IMO
>   number — and a name that arrives only in a separate message.
> * The two Class B families — "CS" units that *listen before they talk*
>   (carrier-sense TDMA) and "SO" units that join the self-organising slot
>   map like a Class A — and how the fixed bit pattern `1100000000000000110`
>   in every CS report follows from that difference.
> * Every field of message 18 [M.1371-5 Annex 8 §3.16 Table 70], message 19
>   [§3.17 Table 71] and message 24 parts A and B [§3.22 Tables 78, 79, 79A],
>   including the vendor-ID field whose meaning changed between editions of
>   the Recommendation and the 30 bits of part B that hold *either* the
>   ship's dimensions *or* a mother ship's MMSI.
> * How static data gets *into* a Class B unit: the `$AISSD` and `$AIVSD`
>   sentences recorded while a UNH research boat's transponder was being
>   programmed, and what the unit said about itself before and after.
> * Code that shares helpers between message modules, decodes one message
>   that comes in two shapes (`part_number`), and — the first time in this
>   tutorial — copes with a field that is one thing or another depending on
>   the MMSI.
> * A stateful exercise for a stateless decoder: pairing message 24 part A
>   with part B by MMSI.

## 14.1 Where this fits

Everything about AIS that we have met so far was designed for ships that
*must* carry it — the SOLAS fleet with its Class A transponders
(12.5 W, 2–10 s position reports, a minimum keyboard and display, a
password-protected static-data menu [M.1371-5 Annex 1 §2.1.1.1],
[IALA G1082 §4.1.1.1 Table 3]). That left out almost every vessel on the
water: fishing boats, yachts, workboats, tenders, research launches. The
**Class B** shipborne mobile station was defined for them — "facilities
not necessarily in full accordance with IMO AIS carriage requirement"
[M.1371-5 Annex 1 §2.1.1.2] — and it comes in two flavours
[M.1371-5 Annex 1 §2.1.1.2], [IALA G1082 §4.1.1.2]:

| | Class B "SO" | Class B "CS" |
|---|---|---|
| Access scheme | SOTDMA, like a Class A: announces and reserves slots [M.1371-5 Annex 2] | **CSTDMA**, carrier-sense: listens and transmits only in a slot nobody is using [M.1371-5 Annex 7 §1] |
| Transmit power | 5 W nominal | 2 W [IALA G1082 Table 3] |
| Equipment standard | [IEC 62287-1/-2] part 2 | [IEC 62287-1/-2] part 1 |
| Position report | message 18, unit flag 0 | message 18, unit flag 1 |
| Static data | message 24 A + B (19 optional) | message 24 A + B; 19 only on interrogation [M.1371-5 Annex 7 Table 42] |
| Reporting interval | 3 min / 30 s / 15 s / 5 s by speed | 3 min / 30 s [M.1371-5 Annex 1 Table 2] |
| Can be sync source for others | yes | no — reports sync state 3 [M.1371-5 Annex 7 §4.3.3.5 note 20] |

The CS variant was the one that reached the market first (2006–2008) and
is by far the more common; everything in this chapter's recordings is CS.
SO units were still "under development" when IALA wrote its overview
[IALA G1082 Table 3]. The USCG announced the first type-approved Class B
devices in October 2008 and, in the same breath, warned that they transmit
at 2 W instead of 12 W, report at a fixed 30 s instead of 2–10 s, and that
older Class A displays would not show them until their software was
updated [Safety Alerts, Alert 10-08].

**What a Class B unit sends.** Three messages carry everything
[M.1371-5 Annex 7 §2.1.2.1, Table 42]:

* **Message 18**, the *standard Class B position report*, "output
  periodically and autonomously instead of Messages 1, 2, or 3"
  [M.1371-5 Annex 8 §3.16]. One slot, 168 bits — the same size as message
  1, but with the navigational status and rate of turn replaced by five
  flags describing the unit's capabilities.
* **Message 24**, the *static data report*, in two parts: **A** carries
  the name (160 bits), **B** the ship type, vendor ID, call sign and
  dimensions (168 bits). Both are sent every 6 minutes, B within one minute
  of A [M.1371-5 Annex 8 §3.22].
* **Message 19**, the *extended Class B position report*: message 18 plus
  the name, ship type, dimensions and EPFD in one 312-bit, two-slot
  message. Obsolete: "for future equipment this message is not needed and
  should not be used" [M.1371-5 Annex 8 §3.17]; a CS unit may transmit it
  only in answer to a base station's interrogation
  [M.1371-5 Annex 7 Table 42].

Why two parts for the static data when a Class A fits it all in one
message 5? Because a CS unit's transmissions "should not exceed one nominal
time period" [M.1371-5 Annex 7 §2.1.1] — one slot, 256 bits including
overhead, about 168 bits of payload. Message 5 is 424 bits; message 19 is
312. Splitting the static data into two single-slot messages is what lets
a carrier-sense unit send it at all, and it is why message 19 is on its
way out. The Class A side was given the same message so that *any* station
can "associate a MMSI with a name" with part A alone
[M.1371-5 Annex 8 §3.22] — SAR aircraft, for example, send message 5 *and*
24A/24B [M.1371-5 Annex 1 §2.1.4].

**What a Class B unit does not send.** Compared with messages 1–3 and 5
the Class B set has no navigational status, no rate of turn, no IMO
number, no destination, no ETA, no draught, no AIS version indicator, and
heading is optional [M.1371-5 Annex 7 §3.5.1]. The name is *not* in the
position report. A chart display that receives only message 18 shows an
anonymous target for up to six minutes — and a decoder that handles one
sentence at a time, like ours, never knows the name of a message 18 at all.
Section 14.7 makes a stateful join of 18 and 24 the main exercise.

**Reporting intervals** [M.1371-5 Annex 1 §4.2.1 Table 2]:

| Platform condition | Nominal interval | Increased interval (SO only, heavy channel load) |
|---|---|---|
| Class B SO, not moving faster than 2 knots | 3 min | 3 min |
| Class B SO, 2–14 knots | 30 s | 30 s |
| Class B SO, 14–23 knots | 15 s | 30 s |
| Class B SO, > 23 knots | 5 s | 15 s |
| Class B CS, not moving faster than 2 knots | 3 min | — |
| Class B CS, moving faster than 2 knots | 30 s | — |

A SO unit drops to the increased interval only when the last four frames
each had fewer than 50 % free slots and returns when 65 % are free
[M.1371-5 Annex 1 Table 2 note 3]. A CS unit has no such rule because it
simply fails to find a free slot — "Class B 'CS' reporting rate will be
impacted in high VDL loading situations" [IALA G1082 Table 4]. Static data
goes out every 6 minutes in both cases, and part B is repeated at once when
the dimensions or EPFD type change [M.1371-5 Annex 8 §3.22].

### Carrier sense: how a CS unit decides when to talk

A Class A or Class B SO station runs the SOTDMA state machine of Annex 2:
it listens to the slot map, announces which slot it will use next in the
communication state of each report, and other stations avoid that slot
(chapter 3). A CS unit does none of this. Instead
[M.1371-5 Annex 7 §4.3.3.1 Table 41, Fig. 40]:

1. It computes its nominal transmission time (NTT) from the reporting
   interval, and a **transmission interval** (TI) around it — RI/3 or
   10 s, whichever is less.
2. Inside the TI it picks **ten candidate periods** (CP) at random.
3. At each candidate slot it measures the received signal level during a
   window from 833 µs to 1 979 µs after the slot starts
   [M.1371-5 Annex 7 §4.3.1.2 Fig. 35]. If the level is above the
   **CS detection threshold** — the background noise measured over a
   rolling 60 s plus 10 dB, never below −107 dBm
   [M.1371-5 Annex 7 §4.3.1.3] — someone else is using the slot; wait for
   the next candidate.
4. If the slot is quiet it keys the transmitter 20 bits (2 083 µs) after
   the slot start [M.1371-5 Annex 7 §4.3.1.2], late enough that a Class A
   that started on time has already been detected.
5. "Transmission should be abandoned if all 10 CPs are 'used'"
   [M.1371-5 Annex 7 §4.3.3.1].

It also honours slots reserved by a base station's message 20
[M.1371-5 Annex 7 §1, §2.1.2.2], and it gets its slot timing by
listening: when it hears position reports (messages 1–4, 18) from Annex 2
stations it aligns its time periods to them ("sync mode 1"); when it
hears nothing but other CS units it falls back to its internal clock
("sync mode 2") [M.1371-5 Annex 7 §4.3.1.1]. The whole design is "polite
operation" — a CS unit "will not interfere with equipment that complies
with Annex 2" [M.1371-5 Annex 7 §1] — at the price of being the first to
go silent on a crowded channel.

This is why the communication state of a CS unit carries no information.
Annex 7 says so: "Because Class B 'CS' does not use any Communication
state information, the communication state field in Message 18 should be
filled with the default value `1100000000000000110` and the communication
state selector flag field filled with '1'" [M.1371-5 Annex 7 §4.3.3.5],
[M.1371-5 Annex 8 Table 70]. Read as the ITDMA state that the selector
announces (chapter 3), the constant means sync state 3 ("synchronized to
another station"), slot increment 0, number-of-slots code 3 (four slots)
and keep flag 0 — none of which the unit does. The footnote explains the
sync state: a CS unit "by default reports sync state 3 and does not report
'number of received stations'. Therefore it will not be used as sync source
for other stations" [M.1371-5 Annex 7 §4.3.3.5 note 20]. Our decoder
decodes the 19 bits like any ITDMA state and adds `cs_fixed_comm_state:
true` when they match the constant, so that a consumer can tell "this is a
CS unit saying nothing" from a SO unit's real slot reservation.

The transponder log used in this chapter shows the algorithm's failure
modes in plain text. Indoors, with the GPS antenna on a desk, the SRT unit
in `sample-data/AISmisc/200905-Cocheco-classb-config/transceiver2009-05-01`
printed `$AITXT,01,01,62,TX attempt failed (msg 18 no pos'n)` (line 293)
and, after its `$AIALR` alarms `Noise threshold exceeded Chan A` / `Chan B`
had raised the CS threshold, `$AITXT,01,01,62,Max number of CP exceeded
(msg 18)` (line 339) — all ten candidate periods were judged busy.

### Operational use

* **Small-craft visibility.** The point of Class B is that a 10-metre
  fishing boat or sailing yacht appears on the radar overlay of the ship
  that would otherwise run it down, with a name (after message 24A) and a
  course and speed. The USCG's note that Class B units "have the same
  ability to acquire and display targets not visible to radar (around the
  bend, in sea clutter, or during foul weather)" [Safety Alerts,
  Alert 10-08] cuts both ways: the big ship sees the small one too.
* **Fishing fleets and workboats.** In the 2011 recording from the UNH
  pier used below, Class B outnumbers Class A: of the single-fragment
  `!AIVDM` lines in `sample-data/tide3/ais/ais-2011-05-24`, 706 are message
  18 and 368 are message 24 against 237 message 1 and 42 message 3. The US
  carriage rule permits Class B on some categories of commercial vessel
  [33 CFR 164.46], and the fishing vessel *Amy Philbrick* (14.5) is one.
* **Harbour and marina management, race committees.** A Class B unit is
  the cheapest way to know where a boat is. Pilot boats, line handlers,
  survey launches and research boats — *R/V Gulf Challenger*, *R/V Coastal
  Surveyor* and *R/V Cocheco* in this chapter's data — carry it so that the
  ship they serve and the office ashore can see them.
* **Collision-avoidance limits.** No navigational status, no heading on
  most units (`true_heading` 511 throughout our data), and a 30 s interval
  mean a Class B target is a *position*, not a *vector you can trust*. The
  USCG recommends Class A for vessels that "are highly maneuverable, travel
  at high speed, or routinely transit congested waters" [Safety Alerts,
  Alert 10-08].

### Unattended / automated uses

Class B is cheap, needs no keyboard, draws little power and is sold as a
module. That makes it the AIS of choice for things that are not ships, and
the message set in this chapter is what a machine will see from them.

1. **Fleet databases and the name problem.** A shore system that keys on
   MMSI gets a position every 30 s and a name every 6 min, in a different
   message, with no guarantee that part A and part B arrive together or in
   order — GPSD warns that "in the real world they may … be separated by
   other sentences or even interleaved with different Type 24 pairs"
   [AIVDM "Type 24"]. The standard only promises B "within 1 min following
   Message 24A" [M.1371-5 Annex 8 §3.22]. Every consumer therefore keeps a
   per-MMSI cache of the last part A and the last part B (exercise 5) and
   should expire it: a name cached for a day will survive a transponder
   being moved to another boat.
2. **Fishing-gear markers and "AIS net buoys".** Low-cost transmitters
   built from Class B CS chipsets are sold to mark nets, long-lines and
   pots. They send message 18 with SOG 0 and a drifting position, and
   message 24A with whatever the manufacturer chose to put in the 20
   characters — typically a gear number and a battery reading, something
   of the form `NET 3 12.4V` (an illustration, not a sentence from our
   recordings, which end in 2011 before these devices spread). Many are
   not type-approved, many share one MMSI across a whole fleet of buoys,
   and some use MMSIs that belong to real vessels. The USCG safety alerts
   in our reference set cover the general duty to transmit accurate data
   and the civil penalty for not doing so [Safety Alerts, Alerts 05-10,
   10-08]; the specific USCG Marine Safety Information Bulletins on
   non-type-approved fishing-gear AIS devices are **not** in Appendix A, so
   we do not cite them. For a decoder the lesson is the same as in
   chapter 7: treat the name as untrusted text, expect `<word> <number>`
   patterns to be gear more often than vessels, and look at the vendor ID
   (14.2) — a hundred "vessels" with the same manufacturer ID and
   consecutive serial numbers are a fleet of buoys.
3. **Autonomous surface vehicles, tenders and craft with a parent ship.**
   A USV or a ship's tender carries Class B so that other traffic sees it.
   ITU gives such craft their own MMSI format, `98MIDxxxx` ("craft
   associated with a parent ship": lifeboats, rescue boats, tenders)
   [M.585-9 Annex 1 §5], and until M.1371-4 message 24B let them put the
   **mother ship's MMSI** in the 30 bits that otherwise hold the
   dimensions [M.1371-4 Annex 8 Table 76]; the call sign is `A` followed by
   the last six digits of the parent's MMSI [M.1371-5 Annex 8 Table 79]. A
   fleet system can therefore link a tender to its ship automatically — if
   it decodes those 30 bits by MMSI range, as ours does.
4. **Man-overboard beacons.** A MOB-AIS device uses an MMSI `972xxyyyy`
   [M.585-9 Annex 2 §2] and, by the Recommendation, transmits message **1**
   with navigational status 14 and message 14 with `MOB ACTIVE` in bursts
   [M.1371-5 Annex 1 §2.1.7] (chapter 12). That paragraph is new in
   M.1371-5 (2014); it does not exist in M.1371-4. Personal beacons sold
   before it, and non-standard "personal AIS" units built on Class B CS
   modules since, transmit message **18** instead, with a `972` identity or
   with whatever MMSI the owner typed in. A watch-keeping program that
   alarms on `972` MMSIs must therefore look at message 18 as well as
   message 1, and should treat a 2 W, 30 s message 18 from a `972` identity
   as a person in the water until proven otherwise.
5. **Satellite AIS.** From orbit a Class B CS unit is the hardest station
   to hear: 2 W instead of 12.5 W, one report every 30 s instead of every
   2–10 s, and — because the unit picks a random free slot instead of
   announcing one — no slot pattern a receiver can lock onto. The
   detection-probability analysis in [M.2169] is written for Class A; the
   message 27 long-range report of chapter 16 is "intended for long-range
   detection of AIS Class A and Class B SO equipped vessels"
   [M.1371-5 Annex 8 §3.25] and a CS unit does not transmit it
   [M.1371-5 Annex 7 Table 42]. Satellite feeds consequently under-count small craft, and a
   fleet of fishing vessels on Class B can be nearly invisible offshore.
6. **The reserved bits.** Bits 38–45 and 139–140 of message 18 (and 38–45,
   139–142 of message 19) were "reserved for definition by a competent
   regional or local authority … Regional applications should not use
   zero" in M.1371-1 and -2 [M.1371-1 Annex 2 Table 31],
   [M.1371-2 Annex 7 Table 59]; since M.1371-3 they are "Spare. Not used.
   Should be set to zero" [M.1371-5 Annex 8 Table 70]. Nothing standard
   ever used them, but a decoder should print them — ours does, as `spare`
   and `spare2` — because a non-zero value is either a regional experiment
   or (as in 14.5, line 34) a sign that the whole payload is garbage.
7. **Equipment fingerprinting and anomaly detection.** The five
   capability flags of message 18 and the vendor ID of message 24B are
   constants burned into the firmware. In our data every CS unit reports
   the same flag pattern (display 0, DSC 1, band 1, message 22 1), and the
   vendor strings `WESTMAR`, `COMAR`, `SIMRAD`, `FEC0001` name the maker of
   each unit. A monitoring system can use them: a message 18 whose flags
   change from report to report, a "SIMRAD" unit that becomes a "COMAR"
   unit under the same MMSI, or a message 24B with an all-`@` vendor ID
   where yesterday there was one, is a second transponder using the same
   identity. Likewise the malformed MMSIs in 14.5 — `0`, eight-digit
   numbers, a non-US MID on a New Hampshire launch — are what mis-programmed Class
   B units look like, and a database should quarantine them rather than
   merge them.

## 14.2 The specification

Zero-based bit offsets throughout; lengths from the ITU tables. All three
messages begin with the usual 38-bit header (message ID, repeat indicator,
MMSI) and all three were renumbered between editions: message 18 is
Table 31 in M.1371-1, Table 67 in M.1371-3 and -4, Table 70 in M.1371-5;
message 19 is Table 32 / 68 / 71; message 24 was introduced in M.1371-2
(Annex 7 Tables 60–61), became Tables 75–76 in M.1371-3 and -4 (with the
vendor-ID split in Table 76A of -4) and is Tables 78, 79 and 79A in
M.1371-5. We cite M.1371-5 and note the differences where they matter;
a citation such as [M.1371-4 Annex 8 Table 76] refers to the earlier
edition listed under the [M.1371-5] entry in Appendix A. Inside the field
tables, a bare [Table 70] or [Annex 7 §3.5.1.2] means M.1371-5.

### Message 18 — standard Class B position report

168 bits, one slot [M.1371-5 Annex 8 §3.16 Table 70].

| Bits | Len | Field | Type / units | Not available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 18 [Table 70] |
| 6–7 | 2 | Repeat indicator | uint | — | "should be 0 for CS transmissions" [Table 70] |
| 8–37 | 30 | User ID | uint | — | MMSI [Table 70] |
| 38–45 | 8 | Spare | — | 0 | "Not used. Should be set to zero" [Table 70]; "reserved for regional or local applications" in M.1371-1/-2 |
| 46–55 | 10 | SOG | uint, 1/10 knot | 1023 | 1022 = 102.2 knots or higher [Table 70] |
| 56 | 1 | Position accuracy | flag | 0 | 1 = high (≤ 10 m), determined as in Table 50 [Table 70] |
| 57–84 | 28 | Longitude | sint, 1/10 000 min | 181° = `0x6791AC0` | East positive, two's complement [Table 70] |
| 85–111 | 27 | Latitude | sint, 1/10 000 min | 91° = `0x3412140` | North positive [Table 70] |
| 112–123 | 12 | COG | uint, 1/10° | 3600 | 3601–4095 should not be used [Table 70] |
| 124–132 | 9 | True heading | uint, degrees | 511 | optional on Class B [Annex 7 §3.5.1.2] |
| 133–138 | 6 | Time stamp | uint, UTC second | 60 | 61/62/63 (manual, dead reckoning, inoperative) "are not used by CS AIS" [Table 70] |
| 139–140 | 2 | Spare | — | 0 | as bits 38–45 [Table 70] |
| 141 | 1 | Class B unit flag | flag | — | 0 = Class B SOTDMA unit, 1 = Class B CS unit [Table 70] |
| 142 | 1 | Class B display flag | flag | 0 | 1 = "equipped with integrated display displaying Message 12 and 14" [Table 70] |
| 143 | 1 | Class B DSC flag | flag | 0 | 1 = "equipped with DSC function (dedicated or time-shared)" [Table 70] |
| 144 | 1 | Class B band flag | flag | 0 | 0 = upper 525 kHz of the marine band only, 1 = whole marine band; "irrelevant if Class B Message 22 flag is 0" [Table 70] |
| 145 | 1 | Class B message 22 flag | flag | 0 | 1 = frequency management via message 22 (chapter 13); 0 = AIS 1 and AIS 2 only [Table 70] |
| 146 | 1 | Mode flag | flag | 0 | 0 = autonomous and continuous, 1 = assigned mode [Table 70] |
| 147 | 1 | RAIM flag | flag | 0 | [Table 70] |
| 148 | 1 | Communication state selector | flag | — | 0 = SOTDMA state follows, 1 = ITDMA state follows; "always 1 for Class B CS" [Table 70] |
| 149–167 | 19 | Communication state | per selector | — | SOTDMA [Annex 2 §3.3.7.2.2 Tables 18–19] or ITDMA [Annex 2 §3.3.7.3.2 Table 20]; CS units send `1100000000000000110` [Table 70], [Annex 7 §4.3.3.5] |

Bits 46–138 — SOG through time stamp — are laid out exactly as in message
1 except that they start at bit 46 instead of bit 50 (message 1 has the
navigational status at 38–41 and the rate of turn at 42–49; message 18 has
8 spare bits at 38–45 and no ROT). The sentinel values, the two's
complement coordinates and the 1/10 000 minute unit are the ones from
chapter 3; our code reuses nothing from the message 1 module for these
fields only because the offsets differ, but it does reuse the two
communication-state decoders.

**The five Class B flags.** They describe the *unit*, not the voyage, and
they never change while the firmware is the same: whether it is CS or SO;
whether there is a display on which a received text message (12 or 14)
could be read — without one, sending the unit a safety message is
pointless, and message 19 maps its DTE bit to this flag
[M.1371-5 Annex 8 Table 71]; whether it has a DSC receiver (a CS unit has
a receive-only DSC function for channel management, hence the
"time-shared" wording [IALA G1082 Table 3]); whether its transmitter covers the
whole marine band or only the top 525 kHz (161.500–162.025 MHz, the
minimum for a CS unit [M.1371-5 Annex 7 Table 34]); and whether it obeys
message 22 channel management at all. GPSD's note on the display flag —
"probably not reliable" [AIVDM "Type 18"] — applies to all five: there is
no way to verify them from the air, and every unit in our data reports
the same `0 1 1 1`.

**Mode flag.** 1 when the station is in *assigned mode*: a base station
has set its reporting interval or slots with a message 23 group assignment
(a CS unit ignores message 16 [M.1371-5 Annex 7 Table 42]; chapter 13).
In 14.5 you will meet a unit that sets this bit in its own status output
on the open ocean with no base station in sight — and clears it in what
it actually transmits.

### Message 19 — extended Class B position report

312 bits, two slots [M.1371-5 Annex 8 §3.17 Table 71]. Bits 0–138 are
message 18's, field for field; Table 71 even labels them "provided by
Message 18", the name "provided by Message 24A", and the ship type,
dimensions and EPFD "provided by Message 24B" — the table is the
Recommendation's own statement that the message is redundant.

| Bits | Len | Field | Type / units | Not available | Notes |
|---|---|---|---|---|---|
| 0–138 | 139 | as message 18 | | | message ID 19; 8-bit spare at 38–45 [Table 71] |
| 139–142 | 4 | Spare | — | 0 | [Table 71] |
| 143–262 | 120 | Name | 20 × six-bit ASCII | all `@` | [Table 71], alphabet in Table 47 |
| 263–270 | 8 | Type of ship and cargo type | enum, Table 53 | 0 | 100–199 regional, 200–255 future [Table 71] |
| 271–279 | 9 | Dimension A (to bow) | uint, m | 0 | [Table 71], [§3.3.3 Fig. 41] |
| 280–288 | 9 | Dimension B (to stern) | uint, m | 0 | |
| 289–294 | 6 | Dimension C (to port) | uint, m | 0 | |
| 295–300 | 6 | Dimension D (to starboard) | uint, m | 0 | |
| 301–304 | 4 | Type of EPFD | enum, Table 51 | 0 | 1 GPS … 15 internal GNSS [Table 71] |
| 305 | 1 | RAIM flag | flag | 0 | [Table 71] |
| 306 | 1 | DTE | flag | 1 | 0 = data terminal available [Table 71], [§3.3.1] |
| 307 | 1 | Assigned mode flag | flag | 0 | [Table 71] |
| 308–311 | 4 | Spare | — | 0 | [Table 71] |

The dimension block and its two special patterns (all zero = nothing
available; A = C = 0 with B, D ≠ 0 = dimensions but no reference point)
are the ones explained in chapter 7 [M.1371-5 Annex 8 §3.3.3 Fig. 41].

### Message 24 — static data report

Two different layouts share the message ID; bits 38–39, the **part
number**, say which [M.1371-5 Annex 8 §3.22 Tables 78–79]. Values 2 and 3
are not defined; our decoder returns the remaining bits raw for them.

**Part A** — 160 bits, one slot [Table 78]:

| Bits | Len | Field | Type / units | Not available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 24 |
| 6–7 | 2 | Repeat indicator | uint | — | |
| 8–37 | 30 | User ID | uint | — | MMSI |
| 38–39 | 2 | Part number | uint | — | always 0 for part A [Table 78] |
| 40–159 | 120 | Name | 20 × six-bit ASCII | all `@` | "name of the MMSI-registered vessel"; SAR aircraft: `SAR AIRCRAFT NNNNNNN` [Table 78] |

That is the whole message: 160 bits, which is 26 armoured characters and
4 bits, so a correctly built sentence has 27 characters and **2 fill
bits**. Units that count differently produce 162, 168 or (14.5, line 30)
176 bits; our decoder reports anything beyond bit 159 as `spare` with its
width in `spare_bits`.

**Part B** — 168 bits, one slot [Table 79]:

| Bits | Len | Field | Type / units | Not available | Notes |
|---|---|---|---|---|---|
| 0–37 | 38 | header | | | as part A |
| 38–39 | 2 | Part number | uint | — | always 1 for part B [Table 79] |
| 40–47 | 8 | Type of ship and cargo type | enum, Table 53 | 0 | not applicable to SAR aircraft [Table 79] |
| 48–89 | 42 | Vendor ID | see below | `@@@@@@@` | "unique identification of the unit by a number as defined by the manufacturer (option)" [Table 79]; sub-fields in Table 79A |
| 48–65 | 18 | — Manufacturer's ID | 3 × six-bit ASCII | | "NMEA mnemonic manufacturer codes should be used" [Table 79A note 1] |
| 66–69 | 4 | — Unit model code | uint | | "the first model of the manufacturer uses 1 … reverts to 1 after reaching 15. The 0 is not used" [Table 79A] |
| 70–89 | 20 | — Unit serial number | uint | | binary if the serial number is numeric; otherwise manufacturer-defined [Table 79A] |
| 90–131 | 42 | Call sign | 7 × six-bit ASCII | `@@@@@@@` | craft associated with a parent vessel: `A` + last six digits of the parent's MMSI [Table 79] |
| 132–161 | 30 | Dimension / reference point **or** mother-ship MMSI | A 9, B 9, C 6, D 6 **or** uint | 0 | dimensions as Fig. 41 [Table 79]; "for an unregistered daughter vessel, use the MMSI of the associated mother ship in this data field" [M.1371-4 Annex 8 Table 76] |
| 162–167 | 6 | Spare (M.1371-2 to -4) | — | 0 | [M.1371-4 Table 76]; **M.1371-5 redefines these as a 4-bit type of EPFD (162–165) plus 2 spare bits (166–167)** [M.1371-5 Table 79] |

**The vendor ID, three ways.** In M.1371-2 and -3 the 42 bits were one
seven-character six-bit string, "the name of the AIS equipment vendor"
[M.1371-3 Annex 8 Table 76], [AIVDM "Type 24"]. M.1371-4 split the same
42 bits into a three-character manufacturer mnemonic, a 4-bit model code
and a 20-bit serial number [M.1371-4 Annex 8 Table 76A], numbering the
bits from the MSB: bits 41…24 manufacturer, 23…20 model, 19…0 serial — in
our zero-based payload offsets 48–65, 66–69 and 70–89. The change was
"incompatible", and GPSD doubts "that field practice has caught up"
[AIVDM "Type 24"]. It had not, in 2009–2011: every unit in 14.5 sends a
readable seven-character string (`WESTMAR`, `SIMRAD`, `COMAR`, `FEC0001`),
and the model/serial numbers that fall out of the -4 split are the bit
patterns of the letters `TMAR`, `RAD@`, `AR@@` and `0001`. Our decoder does
what GPSD recommends: it outputs **both** readings — `vendor_id` (seven
characters), and `manufacturer_id` + `unit_model_code` +
`unit_serial_number` — and leaves the judgement to the reader. A rule of
thumb that works on our data: if characters four to seven of `vendor_id`
are letters, the string reading is right; if `unit_model_code` is 1–15 and
the string's tail is unprintable, the split is right.

**Dimensions or mother ship?** Nothing in part B says which of the two the
30 bits at 132–161 are. The only key is the MMSI at 8–37: an identity of
the form `98MIDxxxx` belongs to a "craft associated with a parent ship"
[M.585-9 Annex 1 §5], and for such a craft the field holds the parent's
MMSI [M.1371-4 Annex 8 Table 76], [AIVDM "Type 24"]. Our decoder tests
`980 000 000 ≤ mmsi < 990 000 000` and outputs either the four `dim_*`
fields or `mothership_mmsi`, plus `auxiliary_craft` to say which it did.
M.1371-5 quietly dropped the mother-ship sentence from Table 79; a -5 unit
with a `98` MMSI presumably sends dimensions, and a decoder cannot tell.
Neither case occurs in our recordings.

**The last six bits.** Our decoder, like GPSD, follows the M.1371-4 table
and prints bits 162–167 as a 6-bit `spare`. M.1371-5 puts a type-of-EPFD
field there [M.1371-5 Table 79], so on a unit built to -5 you should expect
`spare` to read 4 (GPS = 1 shifted left by the 2 real spare bits), 60
(internal GNSS = 15) and so on. Every part B in our data has `spare: 0`,
which is consistent with either reading. Adding an `epfd` key for -5 units
is exercise 4.

### How static data gets into the unit: `$AISSD` and `$AIVSD`

A Class B unit has no keyboard. "Class B units are not user configurable
… the units must be configured by the manufacturer or their representative
prior to use" [IALA G1082 §4.1.1.2], and the MMSI, once programmed, must
not be changeable by the user [M.1371-5 Annex 7 §3.5.4]. The configuration
goes in over the serial port with two IEC 61162-1 sentences — the same
ones a Class A uses for its message 5 data [IEC 61162-1 §8 SSD, VSD]:

```text
$--SSD,c--c,c--c,xxx,xxx,xx,xx,c,aa*hh
        │    │    │   │   │  │  │  └ source identifier (talker of the originating device)
        │    │    │   │   │  │  └ DTE indicator flag
        │    │    │   │   └──┴ C (to port), D (to starboard), 0–63 m
        │    │    └───┴ A (to bow), B (to stern), 0–511 m
        │    └ ship's name, up to 20 characters
        └ call sign, up to 7 characters

$--VSD,x.x,x.x,x.x,c--c,hhmmss.ss,xx,xx,x.x,x.x*hh
        │   │   │   │    │         │  │  │   └ regional application flags
        │   │   │   │    │         │  │  └ navigational status
        │   │   │   │    │         └──┴ ETA day, month
        │   │   │   │    └ ETA time UTC
        │   │   │   └ destination, up to 20 characters
        │   │   └ persons on board
        │   └ maximum present static draught, m
        └ type of ship and cargo type
```

`SSD` is the *ship static data* that ends up in message 24 (call sign and
dimensions in part B, name in part A); `VSD` is the *voyage static data*
that a Class A puts in message 5 — of which a Class B transmits only the
ship type (message 24B). The DTE flag of `SSD` has no home in message 24
at all; it would go into message 19 or message 5. The 1 May 2009 log of
the R/V *Cocheco*'s transponder captured the unit echoing both sentences
before and after it was programmed (data file lines 5–8, source lines 64,
66, 1505, 1507):

```text
$AISSD,@@@@@@@,@@@@@@@@@@@@@@@@@@@@,000,000,00,00,1,AI*35
$AIVSD,037,00.0,0000,@@@@@@@@@@@@@@@@@@@@,000000,00,00,00,00*4F
$AISSD,NONE@@@,R/V COCHECO@@@@@@@@@,007,004,03,01,1,AI*3B
$AIVSD,056,00.0,0000,@@@@@@@@@@@@@@@@@@@@,000000,00,00,00,00*48
```

Factory state first: call sign and name all `@`, dimensions 0/0/0/0, ship
type **37** — exactly the default Annex 7 prescribes, "the default value
for type of ship should be 37 (pleasure craft)" [M.1371-5 Annex 7
§3.5.1.1]. Five minutes later: call sign `NONE`, name `R/V COCHECO`,
A = 7 m, B = 4 m, C = 3 m, D = 1 m (an 11 m boat, 4 m wide, antenna
starboard of centre near the stern), ship type **56** ("spare — for
assignments to local vessels" in Table 53 [M.1371-5 Annex 8 Table 53]; two
of the three UNH boats in 14.5 use it). The `@` padding is NMEA's way of sending a fixed-width six-bit
text field in plain ASCII [IEC 61162-1 §7.3.3]; the voyage fields stay at
their defaults because a Class B has nowhere to send them. Two years later
the same unit, at sea, broadcast exactly these values in message 24
(14.5, lines 24–25) — the configuration survived, and so did the
"NONE" call sign.

## 14.3 Decoding one message by hand

Lines 23 and 24 of `data/ch14-class-b.nmea` are a message 18 and the
message 24 part A that followed it four seconds later, both from the
R/V *Cocheco*'s transponder as heard by the `nhcml` receiver at the UNH
pier in New Castle, New Hampshire, on 24 May 2011
(`sample-data/tide3/ais/ais-2011-05-24` lines 1161 and 1162):

```text
!AIVDM,1,1,,B,B5Pm;j0006g4rwV:ELUFGwr5kP06,0*52,rnhcml,1306256812.5
!AIVDM,1,1,,B,H5Pm;j1:uJ0<t<PD<t000000000,2*2A,rnhcml,1306256816.44
```

### Message 18

The payload has 28 characters and 0 fill bits: 28 × 6 = **168 bits**, the
length of Table 70. The first character `B` is six-bit 18, the message ID
(chapter 2: `B` − 48 = 18). The groups:

```text
B      5      P      m      ;      j      0      0      0      6      g      4
010010 000101 100000 110101 001011 110010 000000 000000 000000 000110 101111 000100
bits 0-71

r      w      V      :      E      L      U      F      G      w      r      5
111010 111111 100110 001010 010101 011100 100101 010110 010111 111111 111010 000101
bits 72-143

k      P      0      6
110011 100000 000000 000110
bits 144-167
```

Cutting at Table 70's offsets:

| Bits | Field | Binary | Value |
|---|---|---|---|
| 0–5 | message ID | `010010` | 18 |
| 6–7 | repeat | `00` | 0 |
| 8–37 | MMSI | `010110000011010100101111001000` | 369970120 — MID 369, United States |
| 38–45 | spare | `00000000` | 0 |
| 46–55 | SOG | `0000000000` | 0 → 0.0 knots |
| 56 | position accuracy | `0` | low |
| 57–84 | longitude | `1101011110001001110101111111` | sign bit set: 226 008 447 − 2²⁸ = −42 427 009 → ÷ 600 000 = **−70.711682°** |
| 85–111 | latitude | `001100010100101010111001001` | 25 843 145 ÷ 600 000 = **43.071908°** |
| 112–123 | COG | `010101100101` | 1381 → 138.1° |
| 124–132 | heading | `111111111` | 511, not available |
| 133–138 | time stamp | `110100` | 52 |
| 139–140 | spare | `00` | 0 |
| 141 | unit flag | `1` | Class B **CS** |
| 142 | display flag | `0` | no display |
| 143 | DSC flag | `1` | has DSC |
| 144 | band flag | `1` | whole marine band |
| 145 | message 22 flag | `1` | accepts message 22 |
| 146 | mode flag | `0` | autonomous |
| 147 | RAIM | `0` | not in use |
| 148 | selector | `1` | ITDMA state follows |
| 149–167 | communication state | `1100000000000000110` | the CS constant |

43.0719° N, 70.7117° W is the UNH pier; speed 0.0 with a course of 138.1°
is a boat alongside, its GPS reporting the last course it had. The
receiver logged the sentence at 1306256812.5 = 2011-05-24 17:06:**52**.5
UTC and the time stamp says the fix was taken at second **52** — the
unit's clock and the logger's agree to the second (compare the AtoN in
chapter 15, which was 15 s off). Taking the 19 comm-state bits apart as
ITDMA [M.1371-5 Annex 2 Table 20]: sync state `11` = 3, slot increment
`0000000000000` = 0, number of slots `011` = 3 (meaning four), keep flag
`0`. Meaningless for a CS unit, as 14.1 explained, and the decoder marks
it `cs_fixed_comm_state: true`.

### Message 24 part A

27 characters with **2 fill bits**: 27 × 6 − 2 = **160 bits**. `H` is
six-bit 24.

```text
H      5      P      m      ;      j      1      :      u      J      0      <
011000 000101 100000 110101 001011 110010 000001 001010 111101 011010 000000 001100
bits 0-71

t      <      P      D      <      t      0      0      0      0      0      0
111100 001100 100000 010100 001100 111100 000000 000000 000000 000000 000000 000000
bits 72-143

0      0      0
000000 000000 0000      (the last two bits of the final character are fill)
bits 144-159
```

| Bits | Field | Binary | Value |
|---|---|---|---|
| 0–5 | message ID | `011000` | 24 |
| 6–7 | repeat | `00` | 0 |
| 8–37 | MMSI | `010110000011010100101111001000` | 369970120 — the same bits as in the message 18 |
| 38–39 | part number | `00` | 0 = part A |
| 40–159 | name | `010010 101111 010110 100000 000011 001111 000011 001000 000101 000011 001111` then nine `000000` | 18 47 22 32 3 15 3 8 5 3 15 → `R / V ␠ C O C H E C O` + 9 × `@` → **`R/V COCHECO`** |

Note how the MMSI straddles characters: it begins two bits into character
1 (`5` = `000101`: `00` is the repeat indicator, `0101` starts the MMSI)
and ends two bits into character 6 (`j` = `110010`, of which `11` ends the
MMSI and `00` is bits 38–39). This is why characters 1–5 of every message
from this unit are `5Pm;j` whatever the message type — a useful thing to
recognise when scanning a log by eye: `B5Pm;j…` and `H5Pm;j…` are the same
ship.
The slash in the name is six-bit 47 (`101111`), a legal character of
Table 47; the space is 32 (`100000`); the padding is `@` (`000000`), as
the standard asks.

## 14.4 The code

Three modules, one per message, with two helpers shared between them:

* `decode_common_position(bits, 46)` reads SOG through time stamp for
  messages 18 and 19 (same layout, same offset);
* `decode_dimensions(bits, start)` reads the 30-bit A/B/C/D block for
  message 19 (offset 271) and message 24 part B (offset 132).

Message 18 also imports `decode_sotdma` and `decode_itdma` from the
message 1–3 module of chapter 3 rather than copying them: the 19-bit
communication state is the same in every message that carries one. The
module dependency order is therefore `msg_1_2_3 → msg_18 → msg_19 →
msg_24`, in all four languages.

### Python

`code/python/aistut/msg_18.py`:

```python
from .bits import Bits
from .common import require_bits
from .jsonout import Fixed
from .msg_1_2_3 import decode_itdma, decode_sotdma

# Table 70: the 19-bit value every Class B "CS" unit puts in its
# communication state field.
CS_FIXED_COMM_STATE = "1100000000000000110"

CLASS_B_UNIT = {0: "Class B SOTDMA unit", 1: "Class B CS unit"}
CLASS_B_DISPLAY = {0: "no display for messages 12 and 14", 1: "integrated display for messages 12 and 14"}
CLASS_B_DSC = {0: "no DSC function", 1: "DSC function (dedicated or time-shared)"}
CLASS_B_BAND = {0: "upper 525 kHz of the marine band only", 1: "whole marine band"}
CLASS_B_MSG22 = {0: "AIS 1 and AIS 2 only, no message 22 frequency management",
                 1: "frequency management via message 22"}
ASSIGNED_MODE = {0: "autonomous and continuous", 1: "assigned"}


def decode_common_position(bits: Bits, start: int) -> dict:
    """SOG…time stamp as laid out in messages 18 and 19."""
    sog_raw = bits.uint(start, 10)
    lon_raw = bits.sint(start + 11, 28)
    lat_raw = bits.sint(start + 39, 27)
    cog_raw = bits.uint(start + 66, 12)
    heading = bits.uint(start + 78, 9)
    timestamp = bits.uint(start + 87, 6)
    return {
        "sog_knots": Fixed(sog_raw / 10.0, 1),
        "sog_available": sog_raw != 1023,
        "position_accuracy": bits.uint(start + 10, 1),
        "lon": Fixed(lon_raw / 600000.0, 6),
        "lat": Fixed(lat_raw / 600000.0, 6),
        "position_available": lon_raw != 0x6791AC0 and lat_raw != 0x3412140,
        "cog_deg": Fixed(cog_raw / 10.0, 1),
        "cog_available": cog_raw != 3600,
        "true_heading": heading,
        "heading_available": heading != 511,
        "timestamp": timestamp,
    }


def decode(bits: Bits) -> dict:
    """Decode a message 18. Raises DecodeError if shorter than 168 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 168)
    unit = bits.uint(141, 1)
    display = bits.uint(142, 1)
    dsc = bits.uint(143, 1)
    band = bits.uint(144, 1)
    msg22 = bits.uint(145, 1)
    assigned = bits.uint(146, 1)
    selector = bits.uint(148, 1)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 8),
    }
    out.update(decode_common_position(bits, 46))
    out.update({
        "spare2": bits.uint(139, 2),
        "class_b_unit": unit,
        "class_b_unit_text": CLASS_B_UNIT[unit],
        "class_b_display": display,
        "class_b_display_text": CLASS_B_DISPLAY[display],
        "class_b_dsc": dsc,
        "class_b_dsc_text": CLASS_B_DSC[dsc],
        "class_b_band": band,
        "class_b_band_text": CLASS_B_BAND[band],
        "class_b_msg22": msg22,
        "class_b_msg22_text": CLASS_B_MSG22[msg22],
        "assigned_mode": assigned,
        "assigned_mode_text": ASSIGNED_MODE[assigned],
        "raim": bits.flag(147),
        "comm_state_selector": selector,
        "cs_fixed_comm_state": bits.raw(149, 19) == CS_FIXED_COMM_STATE,
    })
    if selector == 1:
        out["comm_state"] = decode_itdma(bits, 149)
    else:
        out["comm_state"] = decode_sotdma(bits, 149)
    return out
```

`bits.raw(149, 19)` returns the 19 bits as a string of `0`/`1`, so the
CS test is a plain string comparison against the constant from Table 70.
The flags are emitted as integers with a `_text` companion, as the
conventions require for enumerations — even two-valued ones — because the
sense of some of them is easy to get backwards.

`code/python/aistut/msg_19.py` adds the dimension helper and the tail:

```python
from .msg_18 import ASSIGNED_MODE, decode_common_position

DTE = {0: "data terminal available", 1: "data terminal not available"}


def decode_dimensions(bits: Bits, start: int) -> dict:
    """The 30-bit dimension / reference-point field (M.1371-5 §3.3.3 Fig. 41)."""
    a = bits.uint(start, 9)
    b = bits.uint(start + 9, 9)
    c = bits.uint(start + 18, 6)
    d = bits.uint(start + 24, 6)
    return {
        "dim_to_bow_m": a,
        "dim_to_stern_m": b,
        "dim_to_port_m": c,
        "dim_to_starboard_m": d,
        "dimensions_available": not (a == 0 and b == 0 and c == 0 and d == 0),
        "reference_point_available": not (a == 0 and c == 0 and b != 0 and d != 0),
    }


def decode(bits: Bits) -> dict:
    """Decode a message 19. Raises DecodeError if shorter than 312 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 312)
    ship_type = bits.uint(263, 8)
    epfd = bits.uint(301, 4)
    dte = bits.uint(306, 1)
    assigned = bits.uint(307, 1)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 8),
    }
    out.update(decode_common_position(bits, 46))
    out.update({
        "spare2": bits.uint(139, 4),
        "name": bits.text(143, 120),
        "ship_type": ship_type,
        "ship_type_text": ship_type_text(ship_type),
    })
    out.update(decode_dimensions(bits, 271))
    out.update({
        "epfd": epfd,
        "epfd_text": epfd_text(epfd),
        "raim": bits.flag(305),
        "dte": dte,
        "dte_text": DTE[dte],
        "assigned_mode": assigned,
        "assigned_mode_text": ASSIGNED_MODE[assigned],
        "spare3": bits.uint(308, 4),
    })
    return out
```

Two things to note about `decode_dimensions`. First, the key names:
`dim_to_bow_m`, `dim_to_stern_m`, `dim_to_port_m`, `dim_to_starboard_m`.
The message 5 decoder of chapter 7 calls the same four distances
`dim_bow_m`, `dim_stern_m`, `dim_port_m`, `dim_starboard_m` and also
derives `length_m` and `beam_m`; the Class B modules do not. A program
that joins Class A and Class B static data has to map one set onto the
other (exercise 5 does). Second, `reference_point_available` is computed
as "not the A = C = 0, B ≠ 0, D ≠ 0 pattern", which is literally the
second special case of Figure 41 — but for the *first* special case, all
four zero, it comes out `true`, while chapter 7's `not (a == 0 and c == 0)`
comes out `false` for the same bits. Figure 41 says all-zero means
"neither reference point nor dimensions available", so chapter 7 has it
right; the Class B version is wrong only when `dimensions_available` is
already `false`, which never happens in this chapter's data. Exercise 4
asks you to reconcile the two.

`code/python/aistut/msg_24.py` — one function, two layouts:

```python
from .msg_19 import decode_dimensions

PART_TEXT = {0: "part A", 1: "part B", 2: "reserved", 3: "reserved"}


def is_auxiliary_craft(mmsi: int) -> bool:
    """True for MMSIs of the form 98MIDxxxx (craft associated with a parent ship)."""
    return 980_000_000 <= mmsi < 990_000_000


def decode(bits: Bits) -> dict:
    """Decode a message 24 part A or part B. Raises DecodeError if too short."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 40)
    mmsi = bits.uint(8, 30)
    part = bits.uint(38, 2)
    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": mmsi,
        "part_number": part,
        "part_text": PART_TEXT[part],
    }

    if part == 0:
        require_bits(msg_type, len(bits), 160)
        out["name"] = bits.text(40, 120)
        spare_bits = len(bits) - 160
        out["spare_bits"] = spare_bits
        out["spare"] = bits.uint(160, min(spare_bits, 8)) if spare_bits > 0 else 0
    elif part == 1:
        require_bits(msg_type, len(bits), 168)
        ship_type = bits.uint(40, 8)
        out.update({
            "ship_type": ship_type,
            "ship_type_text": ship_type_text(ship_type),
            "vendor_id": bits.text(48, 42),
            "manufacturer_id": bits.text(48, 18),
            "unit_model_code": bits.uint(66, 4),
            "unit_serial_number": bits.uint(70, 20),
            "call_sign": bits.text(90, 42),
            "auxiliary_craft": is_auxiliary_craft(mmsi),
            "spare": bits.uint(162, 6),
        })
        if is_auxiliary_craft(mmsi):
            out["mothership_mmsi"] = bits.uint(132, 30)
        else:
            out.update(decode_dimensions(bits, 132))
    else:
        out["data"] = bits.raw(40, len(bits) - 40)
        out["data_bits"] = len(bits) - 40
    return out
```

The length check happens in two stages: 40 bits to read the part number,
then 160 or 168 depending on it. The error text follows the shared
convention (`message 24 too short: H bits < M`) with whichever minimum
applied. `vendor_id` and `manufacturer_id` both start at bit 48 — the two
readings of the same field discussed in 14.2. For part A, `spare` holds at
most 8 bits even when more are present (line 30 of the data has 16), so
that the value stays small and comparable across units; `spare_bits` tells
you how many there really were.

> **Language notes (Python).** `980_000_000` — underscores inside a
> numeric literal are ignored and group the digits. `a <= x < b` is a
> chained comparison, evaluated as `a <= x and x < b`. The conditional
> expression `X if cond else Y` is Python's ternary operator. None of the
> three has a direct C++ equivalent; see below.

### C++

`code/cpp/msg_18.hpp` declares the six text tables as
`std::array<const char*, 2>` and `CS_FIXED_COMM_STATE` as a
`const char*`; `decode_common_position` returns a `Json` object that is
merged with `out.update(...)`. The decoder body is the Python dictionary
line for line, ending with:

```cpp
    out.update(Json::Object{
        // ... flags and texts as in Python ...
        {"raim", bits.flag(147)},
        {"comm_state_selector", selector},
        {"cs_fixed_comm_state", bits.raw(149, 19) == CS_FIXED_COMM_STATE},
    });
    if (selector == 1) {
        out["comm_state"] = decode_itdma(bits, 149);
    } else {
        out["comm_state"] = decode_sotdma(bits, 149);
    }
    return out;
```

`code/cpp/msg_19.hpp` includes `msg_18.hpp` for the helper and defines
`decode_dimensions`; `code/cpp/msg_24.hpp` includes `msg_19.hpp` for
*that* helper. The part-A branch of message 24 is where C++ differs:

```cpp
inline bool is_auxiliary_craft(std::uint64_t mmsi) {
    return 980000000 <= mmsi && mmsi < 990000000;
}

    if (part == 0) {
        require_bits(msg_type, bits.size(), 160);
        out["name"] = bits.text(40, 120);
        std::size_t spare_bits = bits.size() - 160;
        out["spare_bits"] = spare_bits;
        out["spare"] = spare_bits > 0 ? bits.uint(160, std::min(spare_bits, std::size_t{8})) : 0;
    } else if (part == 1) {
        // ... as Python ...
        if (is_auxiliary_craft(mmsi)) {
            out["mothership_mmsi"] = bits.uint(132, 30);
        } else {
            out.update(decode_dimensions(bits, 132));
        }
    } else {
        out["data"] = bits.raw(40, bits.size() - 40);
        out["data_bits"] = bits.size() - 40;
    }
```

> **Language notes (C++).**
> * `std::min(spare_bits, std::size_t{8})` — as in chapter 15, both
>   arguments of `std::min` must have the same type. `std::size_t{8}` is a
>   brace-initialised literal of exactly that type; `8` alone is an `int`
>   and would not compile.
> * `bits.size() - 160` is unsigned arithmetic. It is safe only because
>   `require_bits` has just proved `size() >= 160`; written the other way
>   round it would wrap to a number around 1.8 × 10¹⁹ without a warning.
> * `bits.raw(149, 19) == CS_FIXED_COMM_STATE` compares a `std::string`
>   with a `const char*`; the standard library overloads `operator==` for
>   that, so no `strcmp` is needed.
> * `cond ? a : b` is the ternary operator; `980000000 <= mmsi && mmsi <
>   990000000` needs the explicit `&&` (a chained `a <= x < b` compiles in
>   C++ and means something else — chapter 7).
> * Including `msg_19.hpp` from `msg_24.hpp` *and* from the driver is fine
>   because every function is `inline` and the header has `#pragma once`.

### Rust

`code/rust/src/msg_18.rs` writes each two-valued table as a function with
a `match`:

```rust
pub fn class_b_unit_text(code: u64) -> &'static str {
    match code {
        0 => "Class B SOTDMA unit",
        _ => "Class B CS unit",
    }
}
```

and the message 24 decoder, `code/rust/src/msg_24.rs`, branches with a
`match` whose arms are blocks:

```rust
pub fn is_auxiliary_craft(mmsi: u64) -> bool {
    (980_000_000..990_000_000).contains(&mmsi)
}

    match part {
        0 => {
            require_bits(msg_type as u32, bits.len(), 160)?;
            put(&mut out, "name", bits.text(40, 120));
            let spare_bits = bits.len() - 160;
            put(&mut out, "spare_bits", spare_bits);
            let spare = if spare_bits > 0 {
                bits.uint(160, spare_bits.min(8))
            } else {
                0
            };
            put(&mut out, "spare", spare);
        }
        1 => {
            require_bits(msg_type as u32, bits.len(), 168)?;
            // ... as Python ...
            if is_auxiliary_craft(mmsi) {
                put(&mut out, "mothership_mmsi", bits.uint(132, 30));
            } else {
                out.extend(decode_dimensions(bits, 132));
            }
        }
        _ => {
            put(&mut out, "data", bits.raw(40, bits.len() - 40));
            put(&mut out, "data_bits", bits.len() - 40);
        }
    }
    Ok(Json::Obj(out))
```

> **Language notes (Rust).**
> * `match code { 0 => …, _ => … }` — the compiler insists that every
>   `match` is exhaustive. For a one-bit value `0` and `_` cover it; the
>   catch-all is also what makes the function total for any `u64`.
> * `(980_000_000..990_000_000).contains(&mmsi)` is the idiomatic range
>   test (a half-open range, so 990 000 000 is excluded — the same as the
>   `<` in the other languages). `contains` takes a reference, hence `&mmsi`.
> * `spare_bits.min(8)` — `min` is a method on the integer types; no import
>   and no type juggling as in C++, because `8` is inferred to be a `usize`
>   from `spare_bits`.
> * `let spare = if … { … } else { … };` — `if` is an expression; both
>   branches must have the same type (`u64` here).
> * The `?` after `require_bits(...)` inside a `match` arm returns the
>   error from the whole `decode` function, exactly as it does at top
>   level. All three modules in this chapter have `#[cfg(test)]`
>   unit tests at the bottom; `cargo test msg_24` runs the pair-A-and-B
>   test on the *Amy Philbrick* sentences.

### TypeScript

`code/typescript/src/msg_18.ts` keeps the tables as plain string arrays
indexed by the bit (`CLASS_B_UNIT[unit]`) and merges partial objects with
`Object.assign`:

```typescript
  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 8),
  };
  Object.assign(out, decodeCommonPosition(bits, 46));
  Object.assign(out, {
    spare2: bits.uint(139, 2),
    class_b_unit: unit,
    class_b_unit_text: CLASS_B_UNIT[unit],
    // ...
    cs_fixed_comm_state: bits.raw(149, 19) === CS_FIXED_COMM_STATE,
  });
  if (selector === 1) {
    out["comm_state"] = decodeItdma(bits, 149);
  } else {
    out["comm_state"] = decodeSotdma(bits, 149);
  }
```

and `code/typescript/src/msg_24.ts`:

```typescript
export function isAuxiliaryCraft(mmsi: number): boolean {
  return 980_000_000 <= mmsi && mmsi < 990_000_000;
}

  if (part === 0) {
    requireBits(msgType, bits.length, 160);
    out["name"] = bits.text(40, 120);
    const spareBits = bits.length - 160;
    out["spare_bits"] = spareBits;
    out["spare"] = spareBits > 0 ? bits.uint(160, Math.min(spareBits, 8)) : 0;
  } else if (part === 1) {
    // ... as Python ...
  } else {
    out["data"] = bits.raw(40, bits.length - 40);
    out["data_bits"] = bits.length - 40;
  }
```

> **Language notes (TypeScript).**
> * `Object.assign(target, source)` copies the enumerable properties of
>   `source` into `target` and is the counterpart of Python's
>   `dict.update`, C++'s `Json::update` and Rust's `BTreeMap::extend`.
> * Numeric separators (`980_000_000`) are legal in JavaScript as in Python
>   and Rust.
> * `===` rather than `==` for the string comparison; `==` would also work
>   for two strings but performs type coercion elsewhere, and the strict
>   form is the habit to keep.
> * A 30-bit MMSI and a 20-bit serial number are far below the 2⁵³ limit
>   of JavaScript numbers, and `bits.uint` builds its result by
>   multiplication rather than with the 32-bit bitwise operators
>   (chapter 2), so there is nothing to watch here.

## 14.5 Sample data and results

### Where the data comes from

`data/ch14-class-b.nmea` has 38 lines: 8 comments and 30 sentences, all
but two of them real recordings. The
comment lines in the file name the sources; in detail:

| Data lines | Source | What it shows |
|---|---|---|
| 3–9 | `sample-data/AISmisc/200905-Cocheco-classb-config/transceiver2009-05-01` lines 22, 24, 64, 66, 1505, 1507, 1510 | an SRT Class B CS unit being programmed on a desk at CCOM, 1 May 2009: message 18 with MMSI 0, the `$AITXT` refusal to transmit, the factory and the programmed `$AISSD`/`$AIVSD`, and the first message 18 with the new MMSI |
| 11–13 | `sample-data/AISmisc/Sealand2007-06-27-001414.nmea` lines 57, 612, 618 | the same make of unit at sea south of Georges Bank, 27 June 2007: `!AIVDO` message 18 and its own message 24 A and B |
| 15–25 | `sample-data/tide3/ais/ais-2011-05-24` lines 7, 8, 9, 661, 662, 793, 795, 796, 1161, 1162, 1163 | four Class B units heard at the UNH pier on 24 May 2011: three research boats and a fishing vessel, each as a 24A/24B pair with a message 18 |
| 27–28 | `sample-data/tide3/ais/ais-2009-06-17` lines 294, 296 | a 24A/24B pair from an eight-digit MMSI with a Furuno vendor string |
| 30 | `sample-data/tide3/ais/ais-2009-06-20` line 3720 | a 176-bit part A |
| 31 | `sample-data/tide3/ais/ais-2009-06-28` line 339 | a 152-bit message 18 |
| 33–34 | `sample-data/AISmisc/classb.uscg.19.nmea` lines 3–4 | a two-fragment message 19 from the USCG N-AIS feed, 3 December 2007 — the right length, the wrong content |
| 35 | `sample-data/tide3/ais/ais-2010-09-30` line 479 | a 120-bit "message 19" |
| 37–38 | *synthetic* | a well-formed message 19 built from the R/V *Cocheco*'s message 18 and 24 fields; see below |

Message 19 deserves a word. Across every recording in `sample-data/` we
found no message 19 that decodes to a plausible vessel: the N-AIS pair on
lines 33–34 has the right length and a valid checksum and nothing else
(below), and the single-fragment "message 19s" from the `nhcml` receiver
are 120 bits long. That is consistent with the message's status —
deprecated since M.1371-3's Class B CS annex, and sent by a CS unit only
when a base station interrogates it [M.1371-5 Annex 7 Table 42]. So that
the chapter has one real-looking example, lines 37–38 were produced by a
short script that assembles Table 71 field by field: bits 0–138 copied
from the *Cocheco*'s message 18 on line 23 (with the message ID changed to
19 — compare the first 23 characters, `C5Pm;j0006g4rwV:ELUFGwr` against
`B5Pm;j0006g4rwV:ELUFGwr`), the four spare bits zero, the name from line
24, the ship type and dimensions from line 25, EPFD 1 (the unit's internal
GPS), RAIM 0, DTE 1 (no data terminal, matching the display flag 0 of its
message 18 — Table 71 says the DTE bit is "provided by Message 18
(Display Flag)"), assigned mode 0 and four spare bits. The 312 bits are 52
armoured characters with no fill, split into fragments of 36 and 16
characters with sequential message identifier 1 on channel A, each with a
freshly computed checksum. No decoder library was involved, and the file
says in its comment that the pair is synthetic.

Run any of the four decoders:

```text
python3 code/python/decode.py data/ch14-class-b.nmea
code/cpp/build/decode        data/ch14-class-b.nmea
code/rust/target/release/decode data/ch14-class-b.nmea
node code/typescript/dist/decode.js data/ch14-class-b.nmea
```

The output is identical for all four and is checked against
`tests/expected/ch14-class-b.jsonl`: 28 objects — 6 message 18, 7 message
24 part A, 6 part B, 2 message 19 (one real, one synthetic), 2 `error`
lines and 5 `skipped` lines. The hand-decoded message 18 of line 23 comes
out as:

```json
{"ais":{"assigned_mode":0,"assigned_mode_text":"autonomous and continuous","class_b_band":1,
"class_b_band_text":"whole marine band","class_b_display":0,"class_b_display_text":"no display for messages 12 and 14",
"class_b_dsc":1,"class_b_dsc_text":"DSC function (dedicated or time-shared)","class_b_msg22":1,
"class_b_msg22_text":"frequency management via message 22","class_b_unit":1,"class_b_unit_text":"Class B CS unit",
"cog_available":true,"cog_deg":138.1,"comm_state":{"keep_flag":false,"scheme":"ITDMA","slot_increment":0,
"slot_offset":0,"slots":4,"slots_raw":3,"sync_state":3,"sync_state_text":"synchronized to another station"},
"comm_state_selector":1,"cs_fixed_comm_state":true,"heading_available":false,"lat":43.071908,"lon":-70.711682,
"mmsi":369970120,"msg_type":18,"position_accuracy":0,"position_available":true,"raim":false,"repeat_indicator":0,
"sog_available":true,"sog_knots":0.0,"spare":0,"spare2":0,"timestamp":52,"true_heading":511},
"channel":"B","fragments":1,"line":23,"own_ship":false,"payload_bits":168,"talker":"AI",
"uscg":{"station":"nhcml","station_kind":"receiver","time":1306256812.50}}
```

(wrapped here; one line in the file) and its part A and part B, lines
24–25, as:

```json
{"ais":{"mmsi":369970120,"msg_type":24,"name":"R/V COCHECO","part_number":0,"part_text":"part A",
"repeat_indicator":0,"spare":0,"spare_bits":0},"channel":"B","fragments":1,"line":24,"own_ship":false,
"payload_bits":160,"talker":"AI","uscg":{"station":"nhcml","station_kind":"receiver","time":1306256816.44}}
{"ais":{"auxiliary_craft":false,"call_sign":"NONE","dim_to_bow_m":7,"dim_to_port_m":3,"dim_to_starboard_m":1,
"dim_to_stern_m":4,"dimensions_available":true,"manufacturer_id":"","mmsi":369970120,"msg_type":24,
"part_number":1,"part_text":"part B","reference_point_available":true,"repeat_indicator":0,"ship_type":56,
"ship_type_text":"spare - local vessel","spare":0,"unit_model_code":0,"unit_serial_number":0,"vendor_id":""},
"channel":"B","fragments":1,"line":25,"own_ship":false,"payload_bits":168,"talker":"AI",
"uscg":{"station":"nhcml","station_kind":"receiver","time":1306256817.43}}
```

Call sign `NONE`, name `R/V COCHECO`, 7/4/3/1, type 56: the `$AISSD` and
`$AIVSD` of 1 May 2009, bit for bit, two years and 20 km later. The vendor
ID is all `@` — the SRT firmware leaves the optional field empty, so
`vendor_id` and `manufacturer_id` are empty strings and the model and
serial read 0.

### Section A: programming the unit (lines 3–9)

| Line | What | Selected fields |
|---|---|---|
| 3 | message 18, `!AIVDO` | `mmsi: 0`, `sog_knots: 102.3` / `sog_available: false`, `lon: 181.0`, `lat: 91.0`, `position_available: false`, `cog_deg: 360.0`, `true_heading: 511`, `timestamp: 60`, `class_b_unit: 1`, `cs_fixed_comm_state: true`, `assigned_mode: 0`, `channel: ""`, `own_ship: true` |
| 4 | `$AITXT` | `skipped: "no parser"` — the text was `MMSI not programmed - TX disabled` |
| 5–8 | `$AISSD`, `$AIVSD` ×2 | `skipped: "no parser"` — see 14.2 |
| 9 | message 18, `!AIVDO` | as line 3 but `mmsi: 369970120`, `assigned_mode: 1` |

Line 3 is what a Class B unit says before anyone has touched it: MMSI 0
and every dynamic field at its not-available value — SOG 1023, longitude
`0x6791AC0`, latitude `0x3412140`, COG 3600, heading 511, time stamp 60 —
exactly the defaults of Table 70. The `!AIVDO` is the unit telling its own
serial port what it *would* send; the `$AITXT` on the next line says that
it did not, because "the unit should only transmit if an appropriate MMSI
is programmed" [M.1371-5 Annex 7 §3.4]. The channel field is empty: this
is the unit's once-a-second status report, not a transmission (Section B
below). Compare the Class A units of chapter 7
that broadcast MMSI 0 and 1193046 to the world: the CS specification is
stricter here, and this unit obeyed it. Five minutes later (line 9) the
MMSI is 369970120 and the unit is still blind — no position — and has
turned on its *assigned mode* flag, to which we return in 14.6. The whole
log is like line 3 and line 9: 1 173 `!AIVDO` message 18s, one per second,
none with a channel letter — the unit never managed a transmission
indoors (14.1 quoted its `TX attempt failed` and `Max number of CP
exceeded` texts). The USCG metadata after the
checksum (`rkurt-office2`, chapter 5) is the serial logger's own stamp, not
a network receiver's.

### Section B: the same unit type at sea (lines 11–13)

| Line | Message | Selected fields |
|---|---|---|
| 11 | 18 | `mmsi: 338022762`, `sog_knots: 7.7`, `cog_deg: 343.8`, `lat: 40.610595`, `lon: -67.721972`, `timestamp: 39`, `true_heading: 511`, `raim: false`, `position_accuracy: 0`, `assigned_mode: 1`, `channel: ""`, `own_ship: true` |
| 12 | 24A | `name: "VISIONS OF JOHANNA"`, `payload_bits: 160`, `spare_bits: 0` |
| 13 | 24B | `ship_type: 37` `pleasure craft`, `call_sign: "WDB7973"`, `dim_to_bow_m: 16`, `dim_to_stern_m: 4`, `dim_to_port_m: 4`, `dim_to_starboard_m: 1`, `vendor_id: ""`, `auxiliary_craft: false` |

A 20 m sailing yacht (MID 338, United States), about 200 nautical miles
south-east of the UNH pier, making 7.7 knots north-north-west, with an
SRT unit of the same family as the *Cocheco*'s (same empty vendor ID, same
flag pattern) but with the factory ship type 37 left in place. The
message 18 has no channel letter, the two message 24 parts have channel
`B`. The source file explains the difference: it holds 502 message 18
`!AIVDO`s with an empty channel, one per second, and 18 with a channel —
one every 30 s, the CS reporting interval for a moving vessel. The
one-per-second lines are the unit's status output, the pattern we saw with
the AtoN's `X`/`Y` channel codes in chapter 15; the lines with a channel
are the ones it transmitted. Line 11 is one of the status lines, and it
has `assigned_mode: 1`; every one of the 18 transmitted message 18s in the
file has `assigned_mode: 0`. The flag means something to the firmware's
scheduler, not to the air — see 14.6.

### Section C: four boats at the UNH pier (lines 15–25)

| Lines | MMSI | `name` (24A) | `ship_type` (24B) | `call_sign` | A/B/C/D | `vendor_id` → `manufacturer_id`, `unit_model_code`, `unit_serial_number` | message 18 |
|---|---|---|---|---|---|---|---|
| 15, 16, 17 | 366764000 | `R/V GULF CHALLENGER` | 38 `reserved` | `WDE6362` | 8/8/2/2 | `WESTMAR` → `WES`, 5, 53330 | 0.0 kn, COG 0.0°, 43.072110 N 70.711078 W, ts 37, `raim: true` |
| 18, 19 | 330029917 | `R/V COASTAL SURVEYOR` | 56 `spare - local vessel` | `NONE` | 10/3/1/3 | `COMAR` → `COM`, 0, 335872 | (not excerpted) |
| 20, 21, 22 | 367103350 | `AMY PHILBRICK` | 30 `fishing` | `WCW4189` | 10/20/5/2 | `SIMRAD` → `SIM`, 4, 528640 | 11.2 kn, COG 80.8°, 43.074035 N 70.727318 W, ts 54 |
| 23, 24, 25 | 369970120 | `R/V COCHECO` | 56 `spare - local vessel` | `NONE` | 7/4/3/1 | (empty) → (empty), 0, 0 | 0.0 kn, COG 138.1°, 43.071908 N 70.711682 W, ts 52 |

All four are Class B CS with `cs_fixed_comm_state: true`, heading 511 and
the flag pattern `0 1 1 1`; all four pairs arrived A-then-B about one
second apart, within the one-minute rule [M.1371-5 Annex 8 §3.22]. Things
to notice:

* **The vendor strings are seven-character names**, as 14.2 predicted:
  `WESTMAR` (a West Marine-branded unit), `SIMRAD`, `COMAR`. The split
  reading turns `TMAR` into model 5, serial 53330; `RAD@` into model 4,
  serial 528640; `AR@@` into model 0 — "the 0 is not used"
  [M.1371-5 Table 79A] — serial 335872. Three different makes, three
  different wrong serial numbers, and a decoder that printed only the
  M.1371-4 reading would have thrown the information away.
* **Ship type 38** on *Gulf Challenger* is "reserved for future use" in
  the 3x activity column [M.1371-5 Annex 8 Table 53]; a 16 m research
  vessel has no good code, and whoever programmed it picked the next
  unused number after 37. The dimensions 8/8/2/2 put the antenna exactly
  amidships, which is what an installer enters when nobody measured.
* **MMSI 330029917**: 330 is not a US MID (chapter 15 listed them: 338,
  366–369), the call sign is `NONE` and the vendor is COMAR — the *Coastal
  Surveyor* is a UNH survey launch whose transponder was programmed with
  the wrong country code. Nothing in the message is wrong *syntactically*;
  only a registry lookup catches it.
* **Amy Philbrick** is the one working vessel: a 30 m fishing boat
  (A + B = 30), type 30, a proper FCC call sign, under way at 11.2 knots
  past the pier. Her message 24 pair came 14 s after her message 18; the
  *Cocheco*'s came 4 s after; *Gulf Challenger*'s came 7 s *before* — the
  two messages are on independent schedules, and a receiver cannot count
  on hearing them in any order.
* **Gulf Challenger's `raim: true`** — the only RAIM-in-use report in the
  file; her receiver monitors its own integrity, or at least says so.

### Section D: deviations (lines 27–31)

| Line | Message | Selected fields |
|---|---|---|
| 27 | 24A | `mmsi: 36694810`, `name: "ELIZABETH B"` |
| 28 | 24B | `ship_type: 37`, `call_sign: "WDB7790"`, 6/8/2/2, `vendor_id: "FEC0001"` → `FEC`, 12, 199729 |
| 30 | 24A | `mmsi: 75605156`, `name: "AF"`, `payload_bits: 176`, `spare_bits: 16`, `spare: 0` |
| 31 | 18 | `error: "message 18 too short: 152 bits < 168"`, `payload_bits: 152` |

*Elizabeth B* has an **eight-digit MMSI**, 036694810 — perhaps a US
`36694810x` with its last digit lost, perhaps a number typed with a
leading zero. The
vendor string `FEC0001` is Furuno Electric Co.'s mnemonic followed by what
reads like a four-digit model or batch number; the -4 split makes it
model 12, serial 199729, which is the bit pattern of `0001`
(`110000 110000 110000 110001`) and nothing else. Line 30 is a part A
that is **176 bits** long — 30 characters with 4 fill bits — from another
malformed identity (75605156, eight digits) with the name `AF`; the extra
16 bits are reported as `spare_bits: 16` and the first eight of them as
`spare: 0` (the last eight, `11110011`, are not shown; exercise 3). Line 31
is a 26-character message 18 with 4 fill bits, 152 bits, that fails
`require_bits`: the payload decodes to repeat indicator 1 and MMSI 8396866,
neither of which a Class B unit would send, and is almost certainly a
corrupted frame that happened to pass the CRC.

### Section E: message 19 (lines 33–35)

Line 34 completes the two-fragment N-AIS message 19:

```json
"mmsi":523135452,"repeat_indicator":2,"spare":191,"sog_knots":88.7,"lat":-35.017815,"lon":-40.028855,
"cog_deg":152.9,"true_heading":244,"timestamp":37,"spare2":10,"name":"]%\"=#<>?/O-7;).]0Q1N",
"ship_type":123,"ship_type_text":"reserved for regional use","dim_to_bow_m":350,"dim_to_stern_m":233,
"dim_to_port_m":35,"dim_to_starboard_m":51,"epfd":6,"epfd_text":"integrated navigation system",
"dte":0,"assigned_mode":1,"spare3":12
```

with `"payload_bits":312,"fragments":2` and the USCG metadata
`"rssi":22983,"station":"993689982","time":1196666940`. Every field is
in range and every field is absurd: a repeat indicator of 2, a spare byte
of 191, a 583-metre vessel 86 m wide doing 88.7 knots in the South
Atlantic (35° S 40° W), received by a Coast Guard shore station in the
United States, named `]%"=#<>?/O-7;).]0Q1N`. The length is right, the
checksums of both fragments verify, and the six-bit characters are all
legal. This is what random bits look like when the first six happen to be
`010011`. A decoder cannot reject it; a consumer must — on the spare
fields, on the position relative to the receiver, on the name's character
mix. Chapter 7's `nhjel` receiver produced "message 5s" of the same kind.
Line 35, `Cg810000000000000000`, is 120 bits and fails cleanly:
`message 19 too short: 120 bits < 312`.

### Section F: the synthetic message 19 (lines 37–38)

```json
"mmsi":369970120,"msg_type":19,"name":"R/V COCHECO","ship_type":56,"dim_to_bow_m":7,"dim_to_stern_m":4,
"dim_to_port_m":3,"dim_to_starboard_m":1,"epfd":1,"epfd_text":"GPS","raim":false,"dte":1,
"dte_text":"data terminal not available","assigned_mode":0,"sog_knots":0.0,"cog_deg":138.1,
"lat":43.071908,"lon":-70.711682,"timestamp":52,"spare":0,"spare2":0,"spare3":0
```

with `"fragments":2,"payload_bits":312,"line":38`. The dynamic block is
identical to line 23 and the static block to lines 24–25 — which is the
point of the message, and the reason it is no longer needed.

## 14.6 Pitfalls and real-world deviations

* **The name is not in the position report.** A stateless decoder prints
  `mmsi` for message 18 and `name` for message 24A and never joins them;
  a display that drops message 24 because it "has no position" will show
  anonymous Class B targets forever. Keep a cache keyed on MMSI, accept
  part B before part A, expect A and B from different MMSIs to interleave
  [AIVDM "Type 24"], and expire entries. Until the first part A arrives —
  up to six minutes after the first message 18 — the target has no name;
  do not fill the gap with the vendor ID.
* **Vendor ID: string or structure?** Everything in our 2009–2011 data is
  a seven-character string; the M.1371-4 split produces plausible-looking
  but meaningless model and serial numbers (`SIMRAD` → model 4, serial
  528640). Decode both, display the string when its tail is printable,
  and never use `unit_serial_number` as a device identity without checking
  `vendor_id` first. The reverse failure also exists: a -4-compliant unit's
  binary serial number decodes as a string of unprintable six-bit garbage
  after the first three letters.
* **Part B's last six bits.** M.1371-2 to -4: six spare bits. M.1371-5:
  four bits of EPFD type and two spare [M.1371-5 Table 79]. Our decoder
  prints the six bits as `spare`; a -5 unit will show `spare: 4` (GPS) or
  `60` (internal GNSS) rather than 0, and that is not an error. Exercise 4.
* **Mother-ship MMSI.** Only the MMSI range tells a decoder to read bits
  132–161 as an MMSI, the rule was dropped from the -5 table, and a unit
  with a `98MIDxxxx` identity that was programmed with ordinary dimensions
  will produce a "mother ship" of 14 696 641 (A = 7, B = 4, C = 3,
  D = 1 read as one 30-bit number). Sanity-check `mothership_mmsi` against the
  MMSI formats of [M.585-9 Annex 1] before you link two vessels.
* **Part A length.** The table says 160 bits [M.1371-5 Table 78]; GPSD
  reports that units send 160 or 168 [AIVDM "Type 24"]; a unit that sends
  27 characters with fill 0 gives 162; our data has 176. Decode the name
  from bits 40–159 and report the remainder, as ours does with
  `spare_bits`; never require exactly 160 and never assume the surplus
  is zero.
* **Garbage with a valid checksum.** Lines 31, 34 and 35 are three ways a
  syntactically perfect sentence can carry nothing: too short, right
  length with impossible content, far too short. Filter on `payload_bits`
  first, then on the spare fields and the position, before the first six
  bits are allowed to mean anything.
* **The capability flags describe firmware, not the boat.** Display, DSC,
  band and message 22 flags are constants per model; GPSD calls the
  display flag "probably not reliable" [AIVDM "Type 18"], the band flag is
  defined as "irrelevant" unless the message 22 flag is set
  [M.1371-5 Table 70], and every unit in this chapter reports the same
  `0 1 1 1`. Use them for fingerprinting (14.1), not for deciding whether
  the skipper can read your safety message.
* **The assigned-mode flag in SRT status reports.** The 2007 SRT firmware
  (`SW Ver 10.0.7.0 Dec 11 2007` per its own `$PSRT` banner) emits a
  message 18 `!AIVDO` every second with an empty channel field, and in
  those the mode flag comes and goes: on in all 502 of *Visions of
  Johanna*'s status lines at sea, while the 18 lines with a channel letter
  — the transmissions — all have it off; on and off in stretches of
  minutes in the *Cocheco*'s indoor log (source lines 22–161 off,
  162–2205 on, 2206–2483 off, …), where nothing was transmitted at all.
  Whatever the firmware means by the bit in its status output, it is not a
  message 23 assignment, and lines 9 and 11 of the data file show it. A
  program that reads a transponder's own `!AIVDO` should not infer
  base-station control from the flag; one that reads `!AIVDM` from other
  ships has a better chance, but should still want to see the message 23
  (chapter 13). The 2011 units at the pier all report 0.
* **`spare` and `spare2` are not always zero.** The N-AIS message 19 has
  191 and 10 in them. A non-zero spare in a *plausible* message would be a
  regional application from the M.1371-1/-2 era; in an implausible one it
  is the first thing to look at.
* **MMSI 0 and short MMSIs.** `0` (line 3), `36694810` (line 27) and
  `75605156` (line 30) are what unconfigured and mis-typed Class B units
  look like. The CS standard forbids transmitting without an MMSI
  [M.1371-5 Annex 7 §3.4] and this unit obeyed; the two eight-digit
  identities are on the air. Validate the MID (chapters 3 and 7) before
  keying a database on the number.
* **Table numbers drift.** Message 18 is Table 31 (M.1371-1), 59 (Annex 7
  of M.1371-2), 67 (-3, -4) and 70 (-5); the code comments in our modules
  say "Table 68 in M.1371-4" for message 18 and "Table 69" for message 19,
  which is off by one — it is 67 and 68. When a document cites "Table 68"
  without an edition, check which message it means.
* **Field-name drift inside this tutorial.** `dim_to_bow_m` here versus
  `dim_bow_m` in message 5; `reference_point_available` computed two
  different ways. Chapter 7's naming and logic are the ones to converge
  on (exercise 4).

## 14.7 Exercises

1. *(pencil and paper)* Decode line 13, the *Visions of Johanna*'s part B
   (`H52GAJTU0000000G42oqok204410`, fill 0). Confirm ship type 37, the call
   sign `WDB7973` from the 42 bits at offset 90, and the dimensions
   16/4/4/1. Then show that bits 48–89 are all zero and explain why
   `vendor_id`, `manufacturer_id`, `unit_model_code` and
   `unit_serial_number` are all "empty" rather than "not available".
2. *(Python)* Write `vendor_reading(part_b: dict) -> str` that returns
   `"string"` when the seven-character `vendor_id` is the right reading
   and `"split"` when the M.1371-4 manufacturer/model/serial reading is,
   using the rule of thumb from 14.2 (printable tail versus model code
   1–15). Run it over `tests/expected/ch14-class-b.jsonl`; it should say
   `"string"` for every part B that has a vendor ID. Then construct, by
   hand, a 42-bit vendor field for manufacturer `SRT`, model 2, serial
   1196, and check that your function says `"split"`. (The *Cocheco*
   unit's own serial number, `0008001196` from its `$PSRT,SNO` line, does
   not fit: 8 001 196 is larger than 2²⁰ − 1 = 1 048 575. Table 79A says a
   manufacturer may then "define the coding method" — one reason the field
   is so often left empty.)
3. *(C++)* Part A's `spare` is capped at 8 bits. Change the decoder to
   output the *whole* surplus as a bit string, `spare_raw`, next to
   `spare_bits` (keep `spare` for compatibility), make the other three
   languages match, regenerate `tests/expected/` with the Python version
   and review the diff: only line 30 should change, gaining
   `"spare_raw":"0000000011110011"`.
4. *(Rust)* Two reconciliations with chapter 7. (a) Make
   `decode_dimensions` compute `reference_point_available` as
   `!(a == 0 && c == 0)`, as message 5 does, and add `length_m` and
   `beam_m`; show with a unit test that an all-zero block now reports
   `reference_point_available: false`. (b) Add an `epfd` / `epfd_text`
   pair to part B from bits 162–165 and shrink `spare` to bits 166–167,
   per M.1371-5 Table 79. Decide — and write down in the module docstring —
   what a reader should conclude when `epfd` is 0 on a unit whose vendor
   string says it was built before 2014.
5. *(TypeScript)* Write a stateful pairer: read the JSON lines of
   `tests/expected/ch14-class-b.jsonl`, keep for each MMSI the latest part
   A, the latest part B and the latest message 18, and print one line per
   MMSI with name, call sign, ship type, length and beam (A + B, C + D),
   last position and the time between the part A and the part B
   (`uscg.time` where present). Treat a part B that arrives before its
   part A as valid. For 367103350 the answer is `AMY PHILBRICK WCW4189
   fishing 30 m × 7 m 43.074035 -70.727318 1.13 s`; for 338022762 there
   are no `uscg.time` values, so print the gap as unknown. Then map the
   Class B field names onto the message 5 names so that the same table
   could include the ships of chapter 7.
6. *(any language, harder)* Using the whole
   `sample-data/tide3/ais/ais-2011-05-24` file (if you have it), measure
   the reporting interval of each Class B unit from consecutive message
   18s: histogram the gaps per MMSI and compare them with Table 2 — 30 s
   when `sog_knots > 2`, 180 s otherwise. Which units keep to it, which
   miss transmissions (gaps of 60 s, 90 s …), and does the fishing vessel
   at 11 knots ever report faster than 30 s? Then do the same for message
   24 pairs and the six-minute rule.
7. *(any language, harder)* Write a "not a ship" classifier for message
   18 + 24 streams: flag an MMSI when (a) its SOG never exceeds 0.5 knots
   over an hour but its position drifts more than 200 m, (b) its part A
   name matches `^[A-Z]+ ?[0-9]+( [0-9.]+V)?$`, (c) more than three MMSIs
   share the same `vendor_id` and consecutive `unit_serial_number`s, or
   (d) its MMSI is in the `972` range. Which of these rules would have
   fired on this chapter's data, and which would fire on a fleet of
   fishing-net buoys?

## 14.8 References

* [M.1371-5] Annex 1 §2.1.1.2 (Class B SO and CS), §2.1.4 (SAR aircraft
  send 24A/B), §2.1.7 (MOB-AIS), §4.2.1 Table 2 (Class B reporting
  intervals); Annex 2 §3.3.7.2.2 Tables 18–19 (SOTDMA state), §3.3.7.3.2
  Table 20 (ITDMA state); Annex 7 §1 (CSTDMA definition), §2.1.1 (one
  slot per transmission), §2.1.2 (modes), §3.4 (transmit only with an
  MMSI), §3.5.1 (information content, default ship type 37), §3.5.4
  (MMSI not user-changeable), §4.2.1 Table 34 (frequency range), §4.3.1.2
  Fig. 35 (carrier-sense window), §4.3.1.3 (threshold), §4.3.3.1 Table 41
  and Fig. 40 (candidate periods), §4.3.3.5 and note 20 (fixed
  communication state), Table 42 (message use by a CS unit); Annex 8 §3.16
  Table 70 (message 18), §3.17 Table 71 (message 19), §3.22 Tables 78, 79,
  79A (message 24), §3.3.1 (DTE), §3.3.2 Table 53 (ship type), §3.3.3
  Fig. 41 (dimensions), Table 47 (six-bit ASCII), Table 50 (position
  accuracy), Table 51 (EPFD).
* [M.1371-5] earlier editions, cited for the history: M.1371-1 Annex 2
  Table 31 (message 18 with regional bits); M.1371-2 Annex 7 Tables 59–61
  (CS message 18, message 24 introduced with the mother-ship rule);
  M.1371-3 Annex 8 Tables 67, 68, 75, 76 (seven-character vendor ID);
  M.1371-4 Annex 8 Tables 67, 68, 75, 76, 76A (vendor-ID split, mother-ship
  rule, six spare bits).
* [M.585-9] Annex 1 §1 (ship identities and MIDs), §5 (craft associated
  with a parent ship, `98MIDxxxx`); Annex 2 §2 (AIS-SART `970`, MOB
  `972`, EPIRB-AIS `974`).
* [IEC 62287-1/-2] Class B CS (part 1) and SO (part 2) equipment
  standards, cited for the two families; we do not quote clauses.
* [IEC 61162-1] §8 SSD and VSD sentence formatters; §7.3.3 six-bit field
  padding with `@`.
* [IALA G1082] §4.1.1.2 and Table 3 (Class A versus Class B comparison:
  power, access scheme, frequency range, DSC, display), Table 4 (reporting
  intervals and the note on CS under heavy load), §4.1.3 (MOB-AIS).
* [AIVDM] "Type 18", "Type 19", "Type 24" (field tables; the 160/168-bit
  part A; the vendor-ID "incompatible change"; the auxiliary-craft MMSI
  rule).
* [Safety Alerts] Alert 10-08 (type approval of Class B, 2 W and 30 s,
  older Class A displays, civil penalties for inaccurate information);
  Alert 05-10 (accurate AIS data). The USCG MSIBs on non-type-approved
  fishing-gear AIS devices are not in this compilation and are not cited.
* [33 CFR 164.46] US carriage rule, under which Class B is permitted on
  some commercial vessels.
* [M.2169] satellite detection of AIS, for the discussion of Class B
  visibility from orbit.
* [Toils] for the general theme — here, the vendor-ID field that changed
  meaning under the decoders' feet.
