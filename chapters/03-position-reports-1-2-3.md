# 3. Messages 1, 2 and 3: Class A position reports

> **What you will learn**
>
> * Who sends a position report, how often (3 minutes at anchor down to
>   2 seconds for a fast ship turning), and why there are three message IDs
>   for one 168-bit layout: scheduled (1), assigned (2) and special (3).
> * Every field of the layout and every value it can take: MMSI formats and
>   the maritime identification digits; the 16 navigational statuses; the
>   square-root coding of rate of turn; tenth-of-a-knot speed; the position
>   accuracy and RAIM flags; 1/10 000-minute two's-complement latitude and
>   longitude with their 181°/91° sentinels; course, heading and the
>   60–63 time-stamp codes; and the 19-bit SOTDMA/ITDMA communication state
>   that most decoders throw away and we do not.
> * How to decode one report with pencil and paper, bit by bit.
> * The `msg_1_2_3` module in Python, C++, Rust and TypeScript: enumeration
>   tables, two's complement, nested objects and fixed-decimal output.
> * What fourteen carefully chosen real sentences reveal: stale statuses,
>   ships going astern, a default MMSI shared by two different boats, a
>   transceiver one leap second out of step, and a "message 2" that is
>   really a corrupted base-station report.

## 3.1 Where this fits

Messages 1, 2 and 3 are the heart of AIS. A Class A shipborne transceiver —
the kind carried by every SOLAS ship over 300 GT on an international voyage
and, in the United States, by most commercial vessels over 65 feet
[33 CFR 164.46] — broadcasts one of them between every two seconds and
every three minutes, for as long as it is switched on. In the N-AIS feed
that supplies most of this chapter's data, 353 372 of the 498 505 lines are
message 1 and another 29 554 are message 3; everything else put together is
a quarter of the traffic. "The position report should be output
periodically by mobile stations" [M.1371-5 Annex 8 §3.1], and this is what
"periodically" means for a Class A [M.1371-5 Annex 1 §4.2.1 Table 1]:

| Ship's dynamic conditions | Nominal reporting interval |
|---|---|
| At anchor or moored and not moving faster than 3 knots | 3 min |
| At anchor or moored and moving faster than 3 knots | 10 s |
| 0–14 knots | 10 s |
| 0–14 knots and changing course | 3 1/3 s |
| 14–23 knots | 6 s |
| 14–23 knots and changing course | 2 s |
| > 23 knots | 2 s |
| > 23 knots and changing course | 2 s |

The IMO performance standard that AIS was built to has the same shape with
slightly different numbers — 3 min at anchor, 12 s at 0–14 knots, 4 s when
turning, 3 s above 23 knots [MSC.74(69) Annex 3 §6.2 Table 1]; the ITU
figures above are the ones equipment actually implements. "Changing
course" is defined precisely: the mean heading over the last 30 s differs
from the present heading by more than 5° [M.1371-5 Annex 2 §4.3.1.2].
"Moving faster than 3 knots" is judged on speed over ground, and the slow
3-minute rate applies only when the *navigational status* says at anchor
or moored [M.1371-5 Annex 2 §4.3.1.3] — a point we will come back to when
we meet a "moored" tug doing 5.8 knots. Speed increases take effect at
once; a reduction in rate is applied only after the slower condition has
persisted for three minutes [M.1371-5 Annex 2 §4.3.1.1].

**Three message IDs, one layout.** Table 46 of [M.1371-5 Annex 8] lists
them as:

| ID | Name | Description | Access scheme | Communication state |
|---|---|---|---|---|
| 1 | Position report | Scheduled position report (Class A shipborne mobile equipment) | SOTDMA, RATDMA, ITDMA | SOTDMA |
| 2 | Position report | Assigned scheduled position report (Class A shipborne mobile equipment) | SOTDMA | SOTDMA |
| 3 | Position report | Special position report, response to interrogation (Class A shipborne mobile equipment) | RATDMA | ITDMA |

The acronyms are the four ways of claiming a time slot on the VHF data
link (VDL). The link is divided into one-minute *frames* of 2 250 slots of
26.667 ms each, aligned to the UTC minute [M.1371-5 Annex 2 §3.1.2]:

* **SOTDMA** (self-organising TDMA) is the steady state: a station
  reserves a slot for 3–7 frames ahead, tells everyone how long it will
  keep it, and picks a replacement before it expires [M.1371-5 Annex 2
  §3.3.4.4, Table 16 (TMO_MIN, TMO_MAX)].
* **ITDMA** (incremental TDMA) pre-announces the *next* slot explicitly
  ("my next transmission is 248 slots from now"). It is used "on three
  occasions: data link network entry; temporary changes and transitions in
  periodical reporting intervals; pre-announcement of safety related
  messages" [M.1371-5 Annex 2 §3.3.4.1].
* **RATDMA** (random access) grabs an unannounced free slot — for the very
  first transmission after power-on and for one-off messages
  [M.1371-5 Annex 2 §3.3.4.2].
* **FATDMA** (fixed access) is slots reserved by a base station for itself
  (chapter 6).

So the message ID tells you *how the sender got its slot and how it will
get the next one*, which is why the last 19 bits of the message differ:

* **Message 1** is the normal, autonomous, scheduled report, carrying an
  SOTDMA communication state [M.1371-5 Annex 8 Table 49].
* **Message 2** is what a Class A sends *instead of message 1* while a
  competent authority has put it in *assigned mode* with message 16 or 23
  (chapters 11 and 13): "When operating in the assigned mode, the Class A
  shipborne mobile station should use Message 2 for transmission of
  position reports instead of Message 1" [M.1371-5 Annex 2 §3.3.6]. Same
  SOTDMA state. Genuine message 2s are rare — we have found no
  convincing one in our recordings, and the single "message 2" in this
  chapter's data turns out to be something else (§3.5).
* **Message 3** is the "special position report" with an ITDMA state. It
  is sent (a) as the very first transmissions after power-on — "The first
  transmission of a Class A mobile station should always be the special
  position report (Message 3)" and the whole first frame is message 3 via
  ITDMA [M.1371-5 Annex 2 §3.3.5.2–§3.3.5.3]; (b) as the *extra* reports
  inserted between scheduled ones when a ship starts turning and needs a
  shorter interval — "the higher Rr should be maintained by using ITDMA to
  complement SOTDMA scheduled transmissions" [M.1371-5 Annex 2 §4.3.1.2];
  (c) as the 3-minute report of a ship at anchor or moored
  [M.1371-5 Annex 2 §4.3.1.3]; and (d) as the answer when a Class A is
  interrogated with message 15 [M.1371-5 Annex 8 §3.13 Table 65]. Most of
  the message 3s in a harbour log are (b) and (c).

The 149 bits before the communication state are identical in all three.
Every statement in §3.2 applies to all of them.

### Operational use

* **Collision avoidance.** The position report is what turns an AIS target
  on a radar or ECDIS display into a vector: position, course and speed
  over ground, heading, and — uniquely among sensors — the other ship's
  own rate of turn and declared navigational status. A watch officer sees
  "NORTHERN CANYON, restricted in her ability to manoeuvre, 0.6 kn,
  turning left" long before radar could resolve any of it. IMO's guidance
  on operational use is explicit that AIS *supplements* radar and visual
  observation; AIS information "is only as good as the information
  input", and the manually entered fields — navigational status above
  all — are to be treated accordingly [A.1106(29)].
* **Vessel traffic services.** A VTS plots every message 1/2/3 in its area
  and uses the reporting-rate rules in reverse: a ship that reports every
  2 s is fast or turning; one that reports every 3 min claims to be at
  anchor. Message 2 is the VTS reaching into the ship: an authority that
  wants a 2-second track of a tanker in a narrow channel assigns the rate
  with message 16, and the ship's reports change ID to 2.
* **Pilots.** A pilot's portable unit plugged into the pilot plug sees the
  own-ship `!AIVDO` (chapter 2) and every neighbour's `!AIVDM`; the
  heading, ROT and special-manoeuvre fields are what make it useful in
  a bend.
* **Search and rescue.** Navigational status 14 is reserved for AIS-SART,
  MOB and EPIRB-AIS devices, whose position reports (message 1 with an
  MMSI starting 970, 972 or 974, §3.2) must be shown and alarmed by every
  receiving display [M.1371-5 Annex 8 Table 48; M.585-9 Annex 2 Section 2].

### Unattended / automated uses

Almost every position report is consumed by a program, not a person. The
fields were designed for a bridge display, and machines read more into
them than the designers intended — sometimes legitimately, sometimes not.

* **Track databases.** The basic shore-side product is a time series of
  (MMSI, time, lat, lon, SOG, COG, heading, status) built by appending
  each decoded report. Three things make that harder than it sounds: the
  report carries only the UTC *second* of its position fix (the time stamp
  field), so the date and minute come from the receiver's clock (chapter
  5); the same transmission arrives from several receivers and must be
  de-duplicated (chapter 6 shows one base-station report logged three
  times); and MMSIs are reused, mistyped and defaulted, so "one MMSI =
  one ship" is an approximation. §3.5 shows MMSI 1193046 used by a pilot
  boat in New England and a tug in Virginia a year apart.
* **Geofencing and speed-rule enforcement.** Given a track, a polygon and
  a clock, a program can tell whether a ship entered an area, how fast it
  was going and for how long. The right-whale project that produced our
  data (`sample-data/RightwhaleAisProject`) was built around this: NOAA's
  seasonal management areas off the US east coast impose a 10-knot limit
  on vessels of 65 feet and over, and compliance is measured from AIS
  speed over ground. The same mechanism serves traffic-separation-scheme
  monitoring, marine-protected-area enforcement and emission-control
  zones. Two cautions for the implementer: SOG is the *vessel's* GNSS
  value with 0.1 kn resolution and no accuracy statement, and a geofence
  decision made on a single report is fragile — interpolate between
  reports and require persistence.
* **Port arrival prediction.** Terminal operators and pilots estimate
  arrival by projecting position, COG and SOG along the usual approach
  route, refined with the destination and ETA fields of message 5
  (chapter 7). The reporting rate itself is a signal: a ship whose
  interval drops from 10 s to 3 1/3 s has started a turn.
* **Anomaly and spoofing detection.** The position report contains
  several quantities that must agree with each other and with the
  receiver, and a counterfeit or faulty transmitter rarely gets them all
  right:
  * *Time stamp vs. receive time.* The UTC second in bits 137–142 should
    be within a second or two of the receiver's time of arrival. In §3.5,
    every ship but one agrees with the receiver to the second; the
    exception is consistently 2 s behind.
  * *Slot number vs. time of arrival.* When the SOTDMA sub-message carries
    a slot number, slot × 26.667 ms is the transmit time within the
    minute. A receiver that timestamps to the millisecond (the USCG `T`
    field, chapter 5) can check it. The same ship that is 2 s behind in
    its time stamp is 2 s behind in its slot numbers: its whole frame is
    shifted. That is the signature of a transceiver whose GNSS applied the
    wrong number of leap seconds, a fault the USCG warned about in 2005
    when one manufacturer's units began transmitting in the middle of
    slots instead of at the start [Safety Alerts, Alert 5-05].
  * *ITDMA promises.* A message 3 says "my next slot is *n* slots from
    now". If nothing arrives in that slot from that MMSI, either the
    receiver missed it or the sender is not a real transceiver. §3.5
    checks one promise and finds it kept to the millisecond.
  * *Kinematics.* Successive positions imply a speed; it should match SOG.
    Heading and COG should agree unless the ship is drifting, crabbing or
    going astern. ROT should match the change in heading. Position jumps
    of kilometres between consecutive reports, or a position on land, are
    the classic spoofing and GNSS-jamming indicators.
  * *Sync state.* A Class A with a working GNSS is "UTC direct" (sync
    state 0). A ship reporting sync state 3 has lost UTC; a whole harbour
    reporting sync state 1–3 at once suggests GNSS interference.
* **Slot maps.** The communication state exists so that *other
  transceivers* can build a map of which slots are taken; a shore receiver
  can build the same map and measure channel loading, which is how
  authorities decide whether an area needs a base station managing the
  link with messages 20 and 22. This is the "why a decoder bothers" answer:
  the 19 bits are useless to a chart plotter and invaluable to anyone
  studying the health of the VDL.
* **Satellite AIS.** Position reports were designed to be heard within
  ~40 NM. Satellites hear them from 600 km up, with thousands of ships in
  the footprint and their slot structures no longer aligned, so collisions
  are the norm rather than the exception [M.2169]. The fix was message 27,
  a 96-bit "position report for long-range applications" with coarser
  position and no communication state, sent every 3 minutes on separate
  channels [M.1371-5 Annex 4; Annex 8 §3.25] — chapter 16. Base stations
  can switch it off in their coverage area with the bit in message 4
  that chapter 6 describes.
* **Devices that are not ships.** Message 1 is the Class A position
  report and only Class A equipment should send it. In practice anything
  with a VHF transmitter can: the right-whale project's own shore
  transceivers (`003669739`, `003669734` in chapter 2) sent message 1
  with a coast-station MMSI and a fixed position; the Great Bay receiver
  in `sample-data/tide1` re-encoded its own GPS as a message 1 with MMSI 0;
  aid-to-navigation (AtoN) units configured as mobiles send it instead of
  message 21; and the
  cheap "fishing-gear AIS" buoys that flooded some coasts after 2015 send
  message 1 or 18 with invented MMSIs and a net number in the name field
  (chapter 7 discusses the name-field abuse; the relevant USCG bulletins
  are not in our reference set [Safety Alerts]). A shore system must
  therefore classify a *station* by the totality of what it sends — MMSI
  format, static data, movement — not by the message ID alone.

## 3.2 The specification

Messages 1, 2 and 3 are 168 bits [M.1371-5 Annex 8 §3.1 Table 48]. Offsets
below are zero-based; the ITU table gives only lengths. "sint" means a
two's-complement signed integer (explained under *Longitude and latitude*).

| Bits | Len | Field | Type / units | Not available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 1, 2 or 3 [M.1371-5 Table 48] |
| 6–7 | 2 | Repeat indicator | uint | — | 0 default; 3 = do not repeat any more [M.1371-5 Table 48; Annex 2 §4.6.1] |
| 8–37 | 30 | User ID | uint | — | MMSI, nine decimal digits [M.1371-5 Table 48; Annex 2 §3.3.7.2.1]; formats per [M.585-9 Annex 1] |
| 38–41 | 4 | Navigational status | enum | 15 | sixteen values, table below [M.1371-5 Table 48] |
| 42–49 | 8 | Rate of turn (ROT~AIS~) | sint, special | −128 (0x80) | 4.733 √ROT coding; ±127 = turning without indicator [M.1371-5 Table 48] |
| 50–59 | 10 | SOG | uint, 1/10 knot | 1023 | 0–1021 = 0–102.1 kn; 1022 = 102.2 kn or higher [M.1371-5 Table 48] |
| 60 | 1 | Position accuracy | flag | 0 default | 1 = high (≤ 10 m), 0 = low (> 10 m); determined per Table 50 [M.1371-5 Table 48] |
| 61–88 | 28 | Longitude | sint, 1/10 000 min | 181° = 0x6791AC0 | East positive, West negative [M.1371-5 Table 48] |
| 89–115 | 27 | Latitude | sint, 1/10 000 min | 91° = 0x3412140 | North positive, South negative [M.1371-5 Table 48] |
| 116–127 | 12 | COG | uint, 1/10 degree | 3600 = 0xE10 | 0–3599; 3601–4095 should not be used [M.1371-5 Table 48] |
| 128–136 | 9 | True heading | uint, degrees | 511 | 0–359 [M.1371-5 Table 48] |
| 137–142 | 6 | Time stamp | uint, UTC second | 60 | 61 manual input, 62 dead reckoning, 63 EPFS inoperative [M.1371-5 Table 48] |
| 143–144 | 2 | Special manoeuvre indicator | enum | 0 | 1 = not engaged, 2 = engaged [M.1371-5 Table 48] |
| 145–147 | 3 | Spare | uint | — | "Not used. Should be set to zero" [M.1371-5 Table 48] |
| 148 | 1 | RAIM flag | flag | 0 default | 1 = RAIM in use [M.1371-5 Table 48] |
| 149–167 | 19 | Communication state | SOTDMA (1, 2) or ITDMA (3) | — | [M.1371-5 Table 49; Annex 2 Tables 18–20] |

The GPSD description calls bits 0–148 the "Common Navigation Block" and
uses the same offsets [AIVDM "Types 1, 2 and 3: Position Report Class A"].

### Repeat indicator (bits 6–7)

"When mobile station is transmitting a message, it should always set the
repeat indicator to default = 0" [M.1371-5 Annex 2 §4.6.1.1]. A repeater
station adds one each time it retransmits a message and does not
retransmit a message that already says 3 [M.1371-5 Annex 2 §4.6.1.2]; a
base station transmitting on behalf of another entity (a virtual AtoN,
say) sets it non-zero to mark the message as a retransmission. In a
position report from a ship, anything but 0 means the message has been
through a repeater or was synthesised ashore — worth recording, because the
receiver's time of arrival then says nothing about when the ship was
there. All fourteen reports in this chapter have 0.

### User ID — the MMSI (bits 8–37)

The maritime mobile service identity is a nine-digit decimal number
carried in 30 bits (2³⁰ = 1 073 741 824, so every nine-digit number fits
with room to spare; "the first
9 digits (most significant digits) should be used only" [M.1371-5 Annex 2
§3.3.7.2.1]). Its *shape* tells you what kind of station is talking, and
the three-digit **maritime identification digits (MID)** inside it tell
you which administration issued it. The formats [M.585-9]:

| Format | Station | Where in M.585-9 | Example in our data |
|---|---|---|---|
| `MIDXXXXXX` | Ship station | Annex 1 Section 1 | 538002219 (MID 538, Marshall Islands), 367043190 (367, USA) |
| `0MIDXXXXX` | Group ship station call identity | Annex 1 Section 1 ¶4 | — (should never transmit) |
| `00MIDXXXX` | Coast station; optionally `00MID1XXX` coast, `2` port, `3` pilot, `4` AIS repeater, `5` AIS base station | Annex 1 Section 2 | 003669717 (base station, chapter 6); 003669734 (chapter 2) |
| `111MIDXXX` | SAR aircraft; optionally `111MID1XX` fixed-wing, `111MID5XX` helicopter | Annex 1 Section 3 | chapter 10 |
| `99MIDXXXX` | AIS aid to navigation; optionally `99MID1XXX` physical, `99MID6XXX` virtual, `99MID8XXX` mobile | Annex 1 Section 4 | chapter 15 |
| `98MIDXXXX` | Craft associated with a parent ship (lifeboats, tenders) | Annex 1 Section 5 | — |
| `8MIDXXXXX` | Handheld VHF with DSC and GNSS | Annex 2 Section 1 | — |
| `970XXYYYY` | AIS-SART (XX = manufacturer, YYYY = sequence) | Annex 2 Section 2 ¶1 | — |
| `972XXYYYY` | Man-overboard device | Annex 2 Section 2 ¶2 | — |
| `974XXYYYY` | EPIRB-AIS | Annex 2 Section 2 ¶3 | — |

MIDs are allocated by the ITU in blocks by region: 2xx Europe, 3xx North
and Central America and the Caribbean, 4xx Asia, 5xx Oceania, 6xx Africa,
7xx South America. The United States has 338, 366, 367, 368 and 369 (and
303 for Alaska); 311 is the Bahamas, 538 the Marshall Islands, 636
Liberia. The MID table is maintained by the ITU and is the thing to look
up when a flag matters; the formats above are what a decoder can check
without a table.

Three practical consequences:

* **Leading zeros vanish when you print an integer.** Our decoders output
  `"mmsi":3669717`, seven digits. That *is* 003669717, a coast-station
  identity; an eight-digit value is a nine-digit identity starting with a
  single 0, i.e. the group-call format, which no individual station should
  be transmitting. Always reason about the nine-digit, zero-padded form.
* **Defaults and placeholders.** Transceivers leave the factory with
  something in the MMSI field, and not every installer changes it. The
  value 1193046 — which is 0x123456 in hexadecimal, the kind of number a
  firmware engineer types as a placeholder — appears in our recordings
  from two different vessels: a pilot boat in Rhode Island in 2008 and a
  tug in Norfolk, Virginia in 2007, both with the same bogus IMO number
  in their message 5. Nauticast units (and clones) are the usual source.
  `0`, `1`, `111111111`, `123456789` and the MMSI of the ship's old
  transceiver are the other common placeholders; the full right-whale log
  also contains the six-digit 693559 (5 930 reports) and 80337. USCG
  safety alerts have repeatedly complained about wrong MMSIs and other
  inaccurate data [Safety Alerts, Alerts 05-10, 10-08].
* **Zero is special.** MMSI 0 is what a receiver's own GPS-as-message-1
  looks like (`sample-data/misc`), what a transceiver with no identity
  programmed sends, and what line 6 of this chapter's data shows. Never
  index a database on it.

### Navigational status (bits 38–41)

A four-bit field set *by hand* on the transceiver's keyboard
[M.1371-5 Annex 2 §4.3.1.3]. All sixteen values [M.1371-5 Annex 8
Table 48]:

| Code | Meaning | Code | Meaning |
|---|---|---|---|
| 0 | under way using engine | 8 | under way sailing |
| 1 | at anchor | 9 | reserved for future amendment … HSC |
| 2 | not under command | 10 | reserved for future amendment … WIG |
| 3 | restricted manoeuvrability | 11 | power-driven vessel towing astern (regional use) |
| 4 | constrained by her draught | 12 | power-driven vessel pushing ahead or towing alongside (regional use) |
| 5 | moored | 13 | reserved for future use |
| 6 | aground | 14 | AIS-SART (active), MOB-AIS, EPIRB-AIS |
| 7 | engaged in fishing | 15 | undefined = default (also used by AIS-SART, MOB-AIS and EPIRB-AIS under test) |

Values 0–8 are the COLREGS categories a ship would show with lights and
shapes. 9 and 10 are reserved for high-speed craft and wing-in-ground
craft carrying dangerous goods; 11 and 12 were added in M.1371-5 for
inland and regional use (11–14 were "reserved for future use" through
M.1371-3); 14 was added for AIS-SART in M.1371-4 and
widened to MOB and EPIRB devices in M.1371-5. Our `nav_status_text`
strings abbreviate the long reserved descriptions; the codes are what you
should test against.

Because it is manual, the status is the least reliable field in the
message. The rule that a ship "at anchor or moored" and under 3 knots
reports every 3 minutes means a crew that forgets to change it after
sailing is reporting every 3 minutes at 12 knots — invisible for long
stretches — and a crew that never sets it leaves 15. In §3.5: a tug
"moored" at 5.8 kn, a towing vessel at "15" alongside in Boston, a ferry
"at anchor" doing 3.3 kn in Provincetown harbour.

### Rate of turn (bits 42–49)

Eight bits, two's complement, but *not* linear. The transceiver takes the
rate from a turn indicator in degrees per minute (ROT~sensor~), and
encodes [M.1371-5 Annex 8 Table 48]:

> ROT~AIS~ = 4.733 × √ROT~sensor~, rounded to the nearest integer, sign
> preserved.

| ROT~AIS~ | Meaning |
|---|---|
| 0 | not turning |
| +1 … +126 | turning right, 0.04 … 708°/min (or higher) |
| −1 … −126 | turning left, 0.04 … 708°/min (or higher) |
| +127 | turning right at more than 5° per 30 s, no turn indicator available |
| −127 | turning left at more than 5° per 30 s, no turn indicator available |
| −128 (0x80) | no turn information available (default) |

The square root gives fine resolution near zero and coarse resolution at
high rates, which is what a navigator wants: 1 → 0.04°/min, 2 → 0.18, 5 →
1.1, 10 → 4.5, 20 → 17.9, 50 → 112, 100 → 446, 126 → 709. Decoding is
the inverse: divide by 4.733, square, restore the sign. Worked examples
from the data:

* Line 8 has raw −21: 21 / 4.733 = 4.437; 4.437² = 19.69; negative, so
  **−19.7°/min**, turning left. In the other direction, a sensor reading
  of 20°/min encodes as 4.733 × √20 = 21.17 → 21.
* Line 12 has raw −11: (11 / 4.733)² = 5.40 → **−5.4°/min**.
* 708°/min, the top of the range, encodes as 4.733 × √708 = 125.9 → 126,
  which is why ±126 is "708° per min or higher".

±127 means the ship has *no* rate-of-turn indicator and the transceiver
is inferring a turn from heading changes: "more than 5° per 30 s" is the
same threshold as the "changing course" rule for reporting intervals
[M.1371-5 Annex 2 §4.3.1.2]. Our decoder reports these and −128 as
`rot_deg_per_min: null` with a `rot_text` explaining which. Note the
sentence "ROT data should not be derived from COG information"
[M.1371-5 Table 48] — small vessels with no gyro produce ±127 or −128,
never a number. In our fourteen lines only three carry a value that is not
a flag: a measured 0 on line 1, −21 on line 8, and the −11 of line 12,
which turns out to be corrupt (§3.5).

The sign is in the raw two's-complement value: the top bit of the eight
set means negative. 0xEB = 235 unsigned is −21 signed (235 − 256). The
`Bits.sint` method from chapter 2 does this subtraction; the code never
sees "235".

### Speed over ground (bits 50–59)

Ten bits in tenths of a knot: 0–1021 is 0.0–102.1 kn, 1022 means
"102.2 knots or higher", 1023 means not available [M.1371-5 Table 48].
The decoder prints `sog_knots` with one decimal and a `sog_available`
boolean; for 1023 it prints `102.3` and `false` — the number is
meaningless and the boolean is the one to test, which is the convention
for every sentinel field in this tutorial (`code/CONVENTIONS.md` §3). SOG
comes from the GNSS receiver, not a log; at berth it is typically 0.0 or
0.1 kn of GPS noise.

### Position accuracy (bit 60) and RAIM (bit 148)

Two flags about the quality of the fix, set together according to
Table 50 [M.1371-5 Annex 8 Table 50]:

| RAIM process | Expected RAIM error | RAIM flag | Differential correction | PA flag |
|---|---|---|---|---|
| none available | — | 0 | uncorrected | 0 = low (> 10 m) |
| available | ≤ 10 m | 1 | uncorrected | 1 = high (≤ 10 m) |
| available | > 10 m | 1 | uncorrected | 0 = low |
| none available | — | 0 | corrected | 1 = high |
| available | ≤ 10 m | 1 | corrected | 1 = high |
| available | > 10 m | 1 | corrected | 0 = low |

Receiver autonomous integrity monitoring (RAIM) is the GNSS receiver's
own consistency check using redundant satellites; "expected RAIM error"
is the root-sum-square of its latitude and longitude error estimates
[M.1371-5 Table 50 note 1]. The practical reading: PA = 1 means either a
differential fix or a RAIM estimate under 10 m; RAIM = 1 just means the
receiver *had* a RAIM process, not that the fix was good. A plain GPS
with no RAIM output gives 0/0 — by far the most common combination in
2007–2008 data (nine of our twelve decodable lines). Line 7 has RAIM = 1
with PA = 0 (row 3 of the table: RAIM running, but estimated error over
10 m); lines 1 and 8 have PA = 1 with RAIM = 0 (row 4: a differential fix).

### Longitude and latitude (bits 61–88, 89–115)

Both are in **1/10 000 of a minute of arc**, as two's-complement signed
integers: longitude in 28 bits (±180° = ±108 000 000), latitude in 27
(±90° = ±54 000 000). To get degrees, divide by 600 000 (60 minutes ×
10 000). East and North are positive [M.1371-5 Table 48]. The resolution
is 1/10 000′ ≈ 0.185 m in latitude, which is why the decoder prints six
decimals of a degree (1 µ° ≈ 0.11 m) — enough to round-trip the integer.

**Two's complement** is the way signed integers are stored in every
computer and in AIS [M.1371-5 Annex 8 §3]: an *n*-bit field holds
0 … 2ⁿ⁻¹ − 1 as itself, and 2ⁿ⁻¹ … 2ⁿ − 1 as *value − 2ⁿ*. The most
significant bit is therefore a sign bit, and the decoding rule is "read
the field as unsigned; if the first bit is 1, subtract 2ⁿ". For the
longitude in §3.3, the 28 bits read as unsigned are 226 159 563; the first
bit is 1, so the value is 226 159 563 − 268 435 456 = −42 275 893 →
−70.459822°, west. `Bits.sint` (chapter 2) is exactly this rule.

**The sentinels** are *outside* the valid range, which is what makes
them safe: 181° = 181 × 600 000 = 108 600 000 = **0x6791AC0** for
longitude and 91° = 54 600 000 = **0x3412140** for latitude
[M.1371-5 Table 48]. Both are positive, so the sign bit is 0 and the
unsigned and signed readings agree. In binary they are:

```text
longitude 181°  (28 bits)  0110 0111 1001 0001 1010 1100 0000   = 0x6791AC0 = 108 600 000
latitude   91°  (27 bits)   011 0100 0001 0010 0001 0100 0000   = 0x3412140 =  54 600 000
```

A decoder should test the raw integers, not the floats — `lon_raw !=
0x6791AC0`, not `lon != 181.0` — and treat *either* sentinel as "no
position". The two usually travel together (line 6), but not always: some
units send a valid latitude with the longitude sentinel. Our
`position_available` is true only when both are valid.

### Course over ground (bits 116–127) and true heading (bits 128–136)

COG is twelve bits in tenths of a degree, 0–3599; 3600 (0xE10) is not
available, and "3 601–4 095 should not be used" [M.1371-5 Table 48].
Heading is nine bits in whole degrees, 0–359, with 511 meaning not
available [M.1371-5 Table 48]. COG comes from the GNSS and is the
direction the ship is *moving*; heading comes from a gyro or
transmitting magnetic compass and is the direction it is *pointing*. A
ship with no heading sensor connected sends 511 — most small commercial
vessels in our data do. When both are present and differ by 180°, the
ship is going astern (lines 5 and 10). When SOG is near zero COG is
noise; the spec does not require it to be 3600 at rest, and most
transceivers keep sending the last computed value.

### Time stamp (bits 137–142)

The UTC *second* "when the report was generated by the electronic
position system (EPFS)" [M.1371-5 Table 48] — not the minute, not the
hour. Values 0–59 are the second; four codes carry status instead:

| Value | Meaning |
|---|---|
| 0–59 | UTC second of the position fix |
| 60 | time stamp not available (default) |
| 61 | positioning system in manual input mode |
| 62 | electronic position fixing system in estimated (dead reckoning) mode |
| 63 | positioning system inoperative |

Why only the second? Because the message is meant to be used *now*, by
the receiver, which knows the minute. For a logger it means the full time
of the fix is receive time rounded to the nearest minute boundary that
makes the seconds agree — and that the field is a free check on the
sender's clock against the receiver's (§3.5). A report with time stamp
61–63 should be plotted with suspicion: the position was typed in or
dead-reckoned.

### Special manoeuvre indicator (bits 143–144) and spare (bits 145–147)

"0 = not available = default, 1 = not engaged in special manoeuvre, 2 =
engaged in special manoeuvre (i.e. regional passing arrangement on Inland
Waterway)" [M.1371-5 Table 48]; 3 is not defined and our decoders label
it "reserved". This is the European inland-waterway "blue sign"
(passing starboard-to-starboard) in disguise [AIVDM "Types 1, 2 and 3"].
These five bits have been reshuffled between editions — M.1371-1 and
M.1371-2 had a 4-bit "reserved for regional applications" field at
143–146 and a 1-bit spare at 147 [M.1371-2 Annex 2 §3.3.8.2.1 Table 15a];
the 2-bit manoeuvre indicator appeared in M.1371-3. (The GPSD document
describes the old layout as 3 + 2 bits, which does not match the ITU
text.) Everything in our data has zeros in all five, as it should
outside Europe's rivers.

### Communication state (bits 149–167)

The last nineteen bits are the slot-management bookkeeping described in
§3.1. Which of the two layouts applies is decided by the message ID
[M.1371-5 Annex 8 Table 49]: messages 1 and 2 carry **SOTDMA**, message 3
carries **ITDMA**.

**SOTDMA** [M.1371-5 Annex 2 §3.3.7.2.2 Table 18]:

| Bits (within the 19) | Len | Field | Meaning |
|---|---|---|---|
| 0–1 | 2 | Sync state | 0 = UTC direct; 1 = UTC indirect; 2 = synchronised to a base station (base direct); 3 = synchronised to another station based on the highest number of received stations, or to another mobile directly synchronised to a base station |
| 2–4 | 3 | Slot time-out | frames remaining until a new slot is selected: 0 = this was the last transmission in this slot; 1–7 = that many frames left |
| 5–18 | 14 | Sub-message | depends on the slot time-out, Table 19 |

The sub-message [M.1371-5 Annex 2 §3.3.7.2.3 Table 19]:

| Slot time-out | Sub-message | Meaning |
|---|---|---|
| 3, 5, 7 | Received stations | number of other stations the station is currently receiving, 0–16 383 |
| 2, 4, 6 | Slot number | the slot used for this transmission, 0–2 249 |
| 1 | UTC hour and minute | hour in bits 13–9 (MSB first), minute in bits 8–2; bits 1–0 unused — "if the station has access to UTC" |
| 0 | Slot offset | offset to the slot that will be used in the next frame; 0 means the slot is de-allocated after this transmission |

A station cycling through its time-outs 7, 6, 5, … 0 therefore
broadcasts, in turn, how many neighbours it hears, which slot it is in,
what time it thinks it is, and where it is going next. Each of a
station's slots (six per frame at a 10-second rate) has its own counter,
so consecutive reports from one ship do not count down in order. The
decoder emits one of `received_stations`, `slot_number`, `utc_hour` +
`utc_minute` or `slot_offset` — never more than one — so a consumer can
tell which was present. The time-out counter is also a tool for
*receivers*: when it reaches 0 the slot becomes free, and the slot-offset
sub-message says where the station will be next frame, so a listening
receiver can track a ship's slot allocation without ever transmitting.

**ITDMA** [M.1371-5 Annex 2 §3.3.7.3.2 Table 20]:

| Bits (within the 19) | Len | Field | Meaning |
|---|---|---|---|
| 0–1 | 2 | Sync state | as in SOTDMA |
| 2–14 | 13 | Slot increment | offset to the next slot to be used, or 0 if no more transmissions |
| 15–17 | 3 | Number of slots | consecutive slots to allocate: 0 = 1, 1 = 2, 2 = 3, 3 = 4, 4 = 5; 5 = 1 slot with offset = increment + 8 192; 6 = 2 slots, +8 192; 7 = 3 slots, +8 192 |
| 18 | 1 | Keep flag | 1 = the slot remains allocated for one additional frame |

The +8 192 trick extends the 13-bit increment (max 8 191 slots, 3.64
frames) to reach up to 16 383 slots — "removes the need for RATDMA
broadcast for scheduled transmissions up to 6 min intervals"
[M.1371-5 Table 20] — which is exactly what a ship at anchor needs for
its 3-minute (6 750-slot) schedule. The decoder outputs the raw
`slot_increment` and `slots_raw`, plus the interpreted `slots` and
`slot_offset` with the 8 192 added when applicable, plus `keep_flag`. A
keep flag of 1 is the network-entry signature: "during data link network
entry, ITDMA slots should be marked so that they are reserved for one
additional frame" [M.1371-5 Annex 2 §3.3.4.1]. A slot increment of 0 is
"this was my last ITDMA transmission; I am back on my SOTDMA schedule"
(line 8).

Both tables end with the same sentence: the communication state "should
apply only to the slot in the channel where the relevant transmission
occurs" — channel A and channel B have separate slot maps, and periodic
messages alternate between them [M.1371-5 Annex 2 §4.1.2]. A slot number
in a message heard on channel B says nothing about channel A.

## 3.3 Decoding one message by hand

Line 1 of [`data/ch03-position-reports.nmea`](../data/ch03-position-reports.nmea)
(source: `sample-data/RightwhaleAisProject/20081013.log` line 4, the USCG
N-AIS feed for the Boston area, 13 October 2008):

```text
!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,s34486,d-074,T25.49701619,x78124,r003669946,1223901985
```

One fragment, channel A, 28 payload characters, 0 fill bits: 168 bits
exactly. The six fields after the checksum are USCG metadata (chapter 5):
signal strength `s34486` / `d-074` dBm, time of arrival `T25.497` seconds
into the minute, sentence counter, receiver `r003669946`, and the epoch
stamp 1223901985 = 2008-10-13 12:46:25 UTC. Armour-decode each character
(chapter 2: subtract 48, then 8 more if the result is above 40):

```text
1      8      1      5      3      :      h      0      2      B      r      u      M      N
1      8      1      5      3      10     48     0      2      18     58     61     29     30
000001 001000 000001 000101 000011 001010 110000 000000 000010 010010 111010 111101 011101 011110

F      H      J      P      h      h      e      0      T      h      0      D      0      5
22     24     26     32     48     48     45     0      36     48     0      20     0      5
010110 011000 011010 100000 110000 110000 101101 000000 100100 110000 000000 010100 000000 000101
```

(`:` is ASCII 58 → 10. `h` is 104 → 56 → 48. `r` is 114 → 66 → 58. `u`
is 117 → 69 → 61.) Now walk Table 48 from bit 0:

| Bits | Binary | Value | Meaning |
|---|---|---|---|
| 0–5 | `000001` | 1 | message 1, scheduled position report |
| 6–7 | `00` | 0 | repeat indicator: original transmission |
| 8–37 | `100000000100010100001100101011` | 538002219 | MMSI; nine digits, MID 538 = Marshall Islands, ship-station format |
| 38–41 | `0000` | 0 | navigational status: under way using engine |
| 42–49 | `00000000` | 0 | rate of turn: not turning (a real 0, not the −128 sentinel) |
| 50–59 | `0010010010` | 146 | SOG 14.6 knots |
| 60 | `1` | 1 | position accuracy high (≤ 10 m) |
| 61–88 | `1101011110101110101111001011` | −42 275 893 | longitude: first bit 1, so 226 159 563 − 2²⁸ = −42 275 893; ÷ 600 000 = **−70.459822°**, i.e. 70° 27.5893′ W |
| 89–115 | `001100001101010000011000011` | 25 600 195 | latitude: first bit 0; ÷ 600 000 = **42.666992°**, 42° 40.0195′ N |
| 116–127 | `000010110100` | 180 | COG 18.0° |
| 128–136 | `000010010` | 18 | true heading 18° |
| 137–142 | `011000` | 24 | time stamp: fix taken at second 24 |
| 143–144 | `00` | 0 | special manoeuvre: not available |
| 145–147 | `000` | 0 | spare |
| 148 | `0` | 0 | RAIM not in use |
| 149–150 | `00` | 0 | sync state: UTC direct |
| 151–153 | `101` | 5 | slot time-out 5 → sub-message is "received stations" [M.1371-5 Annex 2 Table 19] |
| 154–167 | `00000000000101` | 5 | receiving 5 other stations |

Cross-checks that cost nothing:

* 42.667° N, 70.460° W is in the Gulf of Maine about 6 NM east of Cape
  Ann, Massachusetts; COG 018° at 14.6 kn is a ship bound north-east
  along the coast. (Her message 5 in the same log, decoded with chapter
  7's code, says she is the tanker *NOR'EASTER*, destination PORTLAND.)
* Heading 18° and COG 18.0° agree: a ship making way with no leeway or
  set. ROT 0 and PA 1 with RAIM 0: a differentially corrected fix, not
  turning.
* Time stamp 24, receiver time of arrival 25.497 s: the fix was taken
  about a second and a half before the slot in which it was transmitted,
  the normal latency.
* At 14.6 kn the nominal interval is 6 s [M.1371-5 Annex 1 Table 1]. In
  the first 20 000 lines of the source file, receiver 003669946 heard
  this ship 330 more times; the gap was 6 s 207 times and 5 or 7 s most
  of the rest (the epoch stamps are whole seconds).

Compare the JSON in §3.5: every number above appears in it.

## 3.4 The code

The module is the first *message* decoder of the tutorial, and it sets
the pattern that chapters 6–16 follow: a module-level table for each
enumeration; one small function per odd field (`decode_rot`) or shared
sub-structure (`decode_sotdma`, `decode_itdma`); and one public `decode`
that checks the length with `require_bits`, reads every field in table
order, and returns a nested object. Chapter 2 already explained the
`Bits` reader and the JSON writer; this section shows the module in each
language and explains only what is new.

The error contract is also new. `require_bits(msg_type, have, need)` is
the *one* place a message decoder can fail on bad input, and it fails
with the shared `DecodeError` whose text is fixed by
`code/CONVENTIONS.md` §2: `message 1 too short: 56 bits < 168`. After it
passes, every `uint`/`sint` call is in range by construction, so the
field reads need no further checks — which is why the Rust version uses
plain `bits.uint(...)` rather than a `Result` at every line.

### Python

[`code/python/aistut/msg_1_2_3.py`](../code/python/aistut/msg_1_2_3.py):

```python
from .bits import Bits
from .common import require_bits
from .jsonout import Fixed

NAV_STATUS: dict[int, str] = {
    0: "under way using engine",
    1: "at anchor",
    2: "not under command",
    3: "restricted manoeuvrability",
    4: "constrained by her draught",
    5: "moored",
    6: "aground",
    7: "engaged in fishing",
    8: "under way sailing",
    9: "reserved for HSC",
    10: "reserved for WIG",
    11: "power-driven vessel towing astern",
    12: "power-driven vessel pushing ahead or towing alongside",
    13: "reserved for future use",
    14: "AIS-SART / MOB-AIS / EPIRB-AIS active",
    15: "undefined (default)",
}

SYNC_STATE: dict[int, str] = {
    0: "UTC direct",
    1: "UTC indirect",
    2: "synchronized to base station",
    3: "synchronized to another station",
}

SPECIAL_MANOEUVRE: dict[int, str] = {0: "not available", 1: "not engaged", 2: "engaged", 3: "reserved"}


def decode_rot(raw: int) -> dict:
    """Interpret the 8-bit rate-of-turn field (Table 48).

    The sensor value is compressed with a square root:
        ROT_AIS = 4.733 * sqrt(ROT_sensor)   (sign preserved)
    so to recover degrees per minute we square: (raw / 4.733)**2.
    """
    out: dict = {"rot_raw": raw}
    if raw == -128:
        out["rot_deg_per_min"] = None
        out["rot_text"] = "not available"
    elif raw == 127:
        out["rot_deg_per_min"] = None
        out["rot_text"] = "turning right > 5 deg per 30 s (no turn indicator)"
    elif raw == -127:
        out["rot_deg_per_min"] = None
        out["rot_text"] = "turning left > 5 deg per 30 s (no turn indicator)"
    else:
        value = (raw / 4.733) ** 2
        if raw < 0:
            value = -value
        out["rot_deg_per_min"] = Fixed(value, 1)
        out["rot_text"] = "turning right" if raw > 0 else "turning left" if raw < 0 else "not turning"
    return out


def decode_sotdma(bits: Bits, start: int) -> dict:
    """SOTDMA communication state, 19 bits (Annex 2 Table 18 and 19)."""
    sync = bits.uint(start, 2)
    timeout = bits.uint(start + 2, 3)
    sub = bits.uint(start + 5, 14)
    out: dict = {
        "scheme": "SOTDMA",
        "sync_state": sync,
        "sync_state_text": SYNC_STATE[sync],
        "slot_timeout": timeout,
    }
    # The meaning of the 14-bit sub-message depends on the slot time-out.
    if timeout in (3, 5, 7):
        out["received_stations"] = sub
    elif timeout in (2, 4, 6):
        out["slot_number"] = sub
    elif timeout == 1:
        out["utc_hour"] = sub >> 9            # bits 13..9
        out["utc_minute"] = (sub >> 2) & 0x7F  # bits 8..2 (7 bits)
    else:  # timeout == 0
        out["slot_offset"] = sub
    return out


def decode_itdma(bits: Bits, start: int) -> dict:
    """ITDMA communication state, 19 bits (Annex 2 Table 20)."""
    sync = bits.uint(start, 2)
    increment = bits.uint(start + 2, 13)
    nslots = bits.uint(start + 15, 3)
    keep = bits.flag(start + 18)
    out: dict = {
        "scheme": "ITDMA",
        "sync_state": sync,
        "sync_state_text": SYNC_STATE[sync],
        "slot_increment": increment,
        "slots_raw": nslots,
        "keep_flag": keep,
    }
    # Table 20: 0..4 mean 1..5 slots; 5..7 mean 1..3 slots with +8192 offset.
    if nslots <= 4:
        out["slots"] = nslots + 1
        out["slot_offset"] = increment
    else:
        out["slots"] = nslots - 4
        out["slot_offset"] = increment + 8192
    return out


def decode(bits: Bits) -> dict:
    """Decode a message 1, 2 or 3. Raises DecodeError if too short."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 168)
    nav = bits.uint(38, 4)
    sog_raw = bits.uint(50, 10)
    lon_raw = bits.sint(61, 28)
    lat_raw = bits.sint(89, 27)
    cog_raw = bits.uint(116, 12)
    heading = bits.uint(128, 9)
    timestamp = bits.uint(137, 6)
    manoeuvre = bits.uint(143, 2)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "nav_status": nav,
        "nav_status_text": NAV_STATUS[nav],
        "sog_knots": Fixed(sog_raw / 10.0, 1),
        "sog_available": sog_raw != 1023,
        "position_accuracy": bits.uint(60, 1),
        "lon": Fixed(lon_raw / 600000.0, 6),
        "lat": Fixed(lat_raw / 600000.0, 6),
        "position_available": lon_raw != 0x6791AC0 and lat_raw != 0x3412140,
        "cog_deg": Fixed(cog_raw / 10.0, 1),
        "cog_available": cog_raw != 3600,
        "true_heading": heading,
        "heading_available": heading != 511,
        "timestamp": timestamp,
        "special_manoeuvre": manoeuvre,
        "special_manoeuvre_text": SPECIAL_MANOEUVRE[manoeuvre],
        "spare": bits.uint(145, 3),
        "raim": bits.flag(148),
    }
    out.update(decode_rot(bits.sint(42, 8)))
    if msg_type == 3:
        out["comm_state"] = decode_itdma(bits, 149)
    else:
        out["comm_state"] = decode_sotdma(bits, 149)
    return out
```

Two design points. The sentinels are compared on the *raw integer*
(`lon_raw != 0x6791AC0`), never on the divided float, and the raw
rate-of-turn value is kept in the output next to the interpreted one so a
consumer can tell 0 ("not turning") from −128 ("no sensor") without
parsing a string. And `decode_sotdma` / `decode_itdma` take a `start`
offset rather than assuming 149, because messages 4, 9, 11, 18 and 26
carry the same 19 bits at other positions and reuse these functions
(chapter 6 does).

> **Language notes (Python).** An *enumeration table* is simply a `dict`
> from integer to string, defined once at module level in capitals by
> convention; `NAV_STATUS[nav]` is a lookup that raises `KeyError` on a
> missing key, which cannot happen here because a 4-bit field is 0–15 and
> all sixteen keys exist. `**` is exponentiation (`x ** 2` squares).
> `a if cond else b if cond2 else c` is a chained *conditional
> expression*, read left to right. `out.update(other)` merges another
> dict's keys into `out` — that is how `decode_rot`'s three keys land in
> the top-level object. A dict value may itself be a dict (`"comm_state":
> {...}`); the JSON writer recurses, producing the *nested object* you see
> in the output. `None` becomes JSON `null`. `Fixed(value, 1)` is the
> chapter 2 wrapper that forces one decimal place.

### C++

[`code/cpp/msg_1_2_3.hpp`](../code/cpp/msg_1_2_3.hpp):

```cpp
inline const std::array<const char*, 16> NAV_STATUS = {
    "under way using engine",
    "at anchor",
    "not under command",
    "restricted manoeuvrability",
    "constrained by her draught",
    "moored",
    "aground",
    "engaged in fishing",
    "under way sailing",
    "reserved for HSC",
    "reserved for WIG",
    "power-driven vessel towing astern",
    "power-driven vessel pushing ahead or towing alongside",
    "reserved for future use",
    "AIS-SART / MOB-AIS / EPIRB-AIS active",
    "undefined (default)",
};

inline const std::array<const char*, 4> SYNC_STATE = {
    "UTC direct",
    "UTC indirect",
    "synchronized to base station",
    "synchronized to another station",
};

inline const std::array<const char*, 4> SPECIAL_MANOEUVRE = {
    "not available", "not engaged", "engaged", "reserved",
};

// Interpret the 8-bit rate-of-turn field (Table 48).
inline Json decode_rot(std::int64_t raw) {
    Json out = Json::Object{{"rot_raw", raw}};
    if (raw == -128) {
        out["rot_deg_per_min"] = nullptr;
        out["rot_text"] = "not available";
    } else if (raw == 127) {
        out["rot_deg_per_min"] = nullptr;
        out["rot_text"] = "turning right > 5 deg per 30 s (no turn indicator)";
    } else if (raw == -127) {
        out["rot_deg_per_min"] = nullptr;
        out["rot_text"] = "turning left > 5 deg per 30 s (no turn indicator)";
    } else {
        double value = std::pow(static_cast<double>(raw) / 4.733, 2.0);
        if (raw < 0) value = -value;
        out["rot_deg_per_min"] = Json::fixed(value, 1);
        out["rot_text"] = raw > 0 ? "turning right" : raw < 0 ? "turning left" : "not turning";
    }
    return out;
}

// SOTDMA communication state, 19 bits (Annex 2 Table 18 and 19).
inline Json decode_sotdma(const Bits& bits, std::size_t start) {
    std::uint64_t sync = bits.uint(start, 2);
    std::uint64_t timeout = bits.uint(start + 2, 3);
    std::uint64_t sub = bits.uint(start + 5, 14);
    Json out = Json::Object{
        {"scheme", "SOTDMA"},
        {"sync_state", sync},
        {"sync_state_text", SYNC_STATE[sync]},
        {"slot_timeout", timeout},
    };
    // The meaning of the 14-bit sub-message depends on the slot time-out.
    if (timeout == 3 || timeout == 5 || timeout == 7) {
        out["received_stations"] = sub;
    } else if (timeout == 2 || timeout == 4 || timeout == 6) {
        out["slot_number"] = sub;
    } else if (timeout == 1) {
        out["utc_hour"] = sub >> 9;               // bits 13..9
        out["utc_minute"] = (sub >> 2) & 0x7F;    // bits 8..2 (7 bits)
    } else {  // timeout == 0
        out["slot_offset"] = sub;
    }
    return out;
}

// ITDMA communication state, 19 bits (Annex 2 Table 20).
inline Json decode_itdma(const Bits& bits, std::size_t start) {
    std::uint64_t sync = bits.uint(start, 2);
    std::uint64_t increment = bits.uint(start + 2, 13);
    std::uint64_t nslots = bits.uint(start + 15, 3);
    bool keep = bits.flag(start + 18);
    Json out = Json::Object{
        {"scheme", "ITDMA"},
        {"sync_state", sync},
        {"sync_state_text", SYNC_STATE[sync]},
        {"slot_increment", increment},
        {"slots_raw", nslots},
        {"keep_flag", keep},
    };
    // Table 20: 0..4 mean 1..5 slots; 5..7 mean 1..3 slots with +8192 offset.
    if (nslots <= 4) {
        out["slots"] = nslots + 1;
        out["slot_offset"] = increment;
    } else {
        out["slots"] = nslots - 4;
        out["slot_offset"] = increment + 8192;
    }
    return out;
}

// Decode a message 1, 2 or 3. Throws DecodeError if too short.
inline Json decode_msg_1_2_3(const Bits& bits) {
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 168);
    std::uint64_t nav = bits.uint(38, 4);
    std::uint64_t sog_raw = bits.uint(50, 10);
    std::int64_t lon_raw = bits.sint(61, 28);
    std::int64_t lat_raw = bits.sint(89, 27);
    std::uint64_t cog_raw = bits.uint(116, 12);
    std::uint64_t heading = bits.uint(128, 9);
    std::uint64_t timestamp = bits.uint(137, 6);
    std::uint64_t manoeuvre = bits.uint(143, 2);

    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"nav_status", nav},
        {"nav_status_text", NAV_STATUS[nav]},
        {"sog_knots", Json::fixed(static_cast<double>(sog_raw) / 10.0, 1)},
        {"sog_available", sog_raw != 1023},
        {"position_accuracy", bits.uint(60, 1)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600000.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600000.0, 6)},
        {"position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140},
        {"cog_deg", Json::fixed(static_cast<double>(cog_raw) / 10.0, 1)},
        {"cog_available", cog_raw != 3600},
        {"true_heading", heading},
        {"heading_available", heading != 511},
        {"timestamp", timestamp},
        {"special_manoeuvre", manoeuvre},
        {"special_manoeuvre_text", SPECIAL_MANOEUVRE[manoeuvre]},
        {"spare", bits.uint(145, 3)},
        {"raim", bits.flag(148)},
    };
    out.update(decode_rot(bits.sint(42, 8)));
    if (msg_type == 3) {
        out["comm_state"] = decode_itdma(bits, 149);
    } else {
        out["comm_state"] = decode_sotdma(bits, 149);
    }
    return out;
}
```

> **Language notes (C++).** The enumeration tables are
> `std::array<const char*, N>` — a fixed-size array of C string literals
> — indexed directly by the field value. That is safe only because the
> field width bounds the index (a 4-bit field cannot exceed 15 and the
> array has 16 entries); for sparse tables such as ship type the code uses
> `std::map` instead. **Integer division:** in C++ `146 / 10` is `14`;
> writing `/ 10.0` (a `double` literal) after a `static_cast<double>`
> forces floating-point division, which Python's `/` does by default.
> `std::pow(x, 2.0)` is used instead of `x * x` so that the result is
> bit-for-bit what Python's `x ** 2` computes (both call the C library's
> `pow`); that matters because the test-suite compares printed decimals.
> `Json::Object{{"k", v}, …}` is *brace initialisation* of a map from a
> list of pairs, and `out["k"] = v` on a `Json` object inserts or
> overwrites a key, as in Python. `nullptr` assigned to a `Json` prints
> `null`. `std::int64_t` for the signed fields and `std::uint64_t` for the
> unsigned ones make the two's-complement intent visible in the types; a
> hexadecimal literal `0x6791AC0` compares equal to the decoded integer
> regardless of base.

### Rust

[`code/rust/src/msg_1_2_3.rs`](../code/rust/src/msg_1_2_3.rs):

```rust
use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

/// Navigational status, Table 48.
pub fn nav_status_text(code: u64) -> &'static str {
    match code {
        0 => "under way using engine",
        1 => "at anchor",
        2 => "not under command",
        3 => "restricted manoeuvrability",
        4 => "constrained by her draught",
        5 => "moored",
        6 => "aground",
        7 => "engaged in fishing",
        8 => "under way sailing",
        9 => "reserved for HSC",
        10 => "reserved for WIG",
        11 => "power-driven vessel towing astern",
        12 => "power-driven vessel pushing ahead or towing alongside",
        13 => "reserved for future use",
        14 => "AIS-SART / MOB-AIS / EPIRB-AIS active",
        _ => "undefined (default)", // 15
    }
}

/// Synchronisation state of the communication state, Annex 2 Table 18.
pub fn sync_state_text(code: u64) -> &'static str {
    match code {
        0 => "UTC direct",
        1 => "UTC indirect",
        2 => "synchronized to base station",
        _ => "synchronized to another station", // 3
    }
}

/// Special manoeuvre indicator, Table 48.
pub fn special_manoeuvre_text(code: u64) -> &'static str {
    match code {
        0 => "not available",
        1 => "not engaged",
        2 => "engaged",
        _ => "reserved", // 3
    }
}

/// Interpret the 8-bit rate-of-turn field (Table 48).
pub fn decode_rot(raw: i64) -> Obj {
    let mut out = Obj::new();
    put(&mut out, "rot_raw", raw);
    match raw {
        -128 => {
            put(&mut out, "rot_deg_per_min", Json::Null);
            put(&mut out, "rot_text", "not available");
        }
        127 => {
            put(&mut out, "rot_deg_per_min", Json::Null);
            put(&mut out, "rot_text", "turning right > 5 deg per 30 s (no turn indicator)");
        }
        -127 => {
            put(&mut out, "rot_deg_per_min", Json::Null);
            put(&mut out, "rot_text", "turning left > 5 deg per 30 s (no turn indicator)");
        }
        _ => {
            let scaled = raw as f64 / 4.733;
            // `scaled * scaled` is the correctly rounded square, identical
            // to Python's `** 2` for this exponent.
            let mut value = scaled * scaled;
            if raw < 0 {
                value = -value;
            }
            put(&mut out, "rot_deg_per_min", Json::fixed(value, 1));
            let text = if raw > 0 {
                "turning right"
            } else if raw < 0 {
                "turning left"
            } else {
                "not turning"
            };
            put(&mut out, "rot_text", text);
        }
    }
    out
}

/// SOTDMA communication state, 19 bits (Annex 2 Table 18 and 19).
pub fn decode_sotdma(bits: &Bits, start: usize) -> Obj {
    let sync = bits.uint(start, 2);
    let timeout = bits.uint(start + 2, 3);
    let sub = bits.uint(start + 5, 14);
    let mut out = Obj::new();
    put(&mut out, "scheme", "SOTDMA");
    put(&mut out, "sync_state", sync);
    put(&mut out, "sync_state_text", sync_state_text(sync));
    put(&mut out, "slot_timeout", timeout);
    // The meaning of the 14-bit sub-message depends on the slot time-out.
    match timeout {
        3 | 5 | 7 => put(&mut out, "received_stations", sub),
        2 | 4 | 6 => put(&mut out, "slot_number", sub),
        1 => {
            put(&mut out, "utc_hour", sub >> 9); // bits 13..9
            put(&mut out, "utc_minute", (sub >> 2) & 0x7F); // bits 8..2 (7 bits)
        }
        _ => put(&mut out, "slot_offset", sub), // timeout == 0
    }
    out
}

/// ITDMA communication state, 19 bits (Annex 2 Table 20).
pub fn decode_itdma(bits: &Bits, start: usize) -> Obj {
    let sync = bits.uint(start, 2);
    let increment = bits.uint(start + 2, 13);
    let nslots = bits.uint(start + 15, 3);
    let keep = bits.flag(start + 18);
    let mut out = Obj::new();
    put(&mut out, "scheme", "ITDMA");
    put(&mut out, "sync_state", sync);
    put(&mut out, "sync_state_text", sync_state_text(sync));
    put(&mut out, "slot_increment", increment);
    put(&mut out, "slots_raw", nslots);
    put(&mut out, "keep_flag", keep);
    // Table 20: 0..4 mean 1..5 slots; 5..7 mean 1..3 slots with +8192 offset.
    if nslots <= 4 {
        put(&mut out, "slots", nslots + 1);
        put(&mut out, "slot_offset", increment);
    } else {
        put(&mut out, "slots", nslots - 4);
        put(&mut out, "slot_offset", increment + 8192);
    }
    out
}

/// Decode a message 1, 2 or 3. Fails with `DecodeError::TooShort` if the
/// payload has fewer than 168 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 168)?;
    let nav = bits.uint(38, 4);
    let sog_raw = bits.uint(50, 10);
    let lon_raw = bits.sint(61, 28);
    let lat_raw = bits.sint(89, 27);
    let cog_raw = bits.uint(116, 12);
    let heading = bits.uint(128, 9);
    let timestamp = bits.uint(137, 6);
    let manoeuvre = bits.uint(143, 2);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "nav_status", nav);
    put(&mut out, "nav_status_text", nav_status_text(nav));
    put(&mut out, "sog_knots", Json::fixed(sog_raw as f64 / 10.0, 1));
    put(&mut out, "sog_available", sog_raw != 1023);
    put(&mut out, "position_accuracy", bits.uint(60, 1));
    put(&mut out, "lon", Json::fixed(lon_raw as f64 / 600000.0, 6));
    put(&mut out, "lat", Json::fixed(lat_raw as f64 / 600000.0, 6));
    put(
        &mut out,
        "position_available",
        lon_raw != 0x6791AC0 && lat_raw != 0x3412140,
    );
    put(&mut out, "cog_deg", Json::fixed(cog_raw as f64 / 10.0, 1));
    put(&mut out, "cog_available", cog_raw != 3600);
    put(&mut out, "true_heading", heading);
    put(&mut out, "heading_available", heading != 511);
    put(&mut out, "timestamp", timestamp);
    put(&mut out, "special_manoeuvre", manoeuvre);
    put(&mut out, "special_manoeuvre_text", special_manoeuvre_text(manoeuvre));
    put(&mut out, "spare", bits.uint(145, 3));
    put(&mut out, "raim", bits.flag(148));
    out.extend(decode_rot(bits.sint(42, 8)));
    let comm_state = if msg_type == 3 {
        decode_itdma(bits, 149)
    } else {
        decode_sotdma(bits, 149)
    };
    put(&mut out, "comm_state", comm_state);
    Ok(Json::Obj(out))
}
```

> **Language notes (Rust).** An enumeration table is a function with a
> `match` returning `&'static str` — a string slice that lives for the
> whole program (string literals do). `match` must be *exhaustive*: the
> compiler cannot know the argument is only 0–15, so a final `_ =>` arm
> is required, and the comment records which value it stands for. Arms
> can list alternatives with `|` (`3 | 5 | 7 =>`) and match negative
> literals (`-128 =>`). `if … else if … else` is an *expression* here, so
> `let text = if … { "a" } else { "b" };` assigns the chosen literal.
> `raw as f64` is the explicit integer-to-float cast — Rust never does it
> silently — and `i64` versus `u64` in the signatures documents which
> fields are two's complement. `Obj` is a `BTreeMap<String, Json>`;
> `out.extend(other)` merges one map into another (the counterpart of
> `dict.update`), and `put(&mut out, "comm_state", comm_state)` with an
> `Obj` value is how the nested object is built. `Json::Null` is the
> explicit `null`. `decode` is the only function that returns a `Result`,
> because `require_bits(...)?` is the only thing that can fail.

### TypeScript

[`code/typescript/src/msg_1_2_3.ts`](../code/typescript/src/msg_1_2_3.ts):

```typescript
import { Bits } from "./bits";
import { requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";

export const NAV_STATUS: { readonly [code: number]: string } = {
  0: "under way using engine",
  1: "at anchor",
  2: "not under command",
  3: "restricted manoeuvrability",
  4: "constrained by her draught",
  5: "moored",
  6: "aground",
  7: "engaged in fishing",
  8: "under way sailing",
  9: "reserved for HSC",
  10: "reserved for WIG",
  11: "power-driven vessel towing astern",
  12: "power-driven vessel pushing ahead or towing alongside",
  13: "reserved for future use",
  14: "AIS-SART / MOB-AIS / EPIRB-AIS active",
  15: "undefined (default)",
};

export const SYNC_STATE: { readonly [code: number]: string } = {
  0: "UTC direct",
  1: "UTC indirect",
  2: "synchronized to base station",
  3: "synchronized to another station",
};

export const SPECIAL_MANOEUVRE: { readonly [code: number]: string } = {
  0: "not available", 1: "not engaged", 2: "engaged", 3: "reserved",
};

/** Interpret the 8-bit rate-of-turn field (Table 48). */
export function decodeRot(raw: number): JsonObject {
  const out: JsonObject = { rot_raw: raw };
  if (raw === -128) {
    out["rot_deg_per_min"] = null;
    out["rot_text"] = "not available";
  } else if (raw === 127) {
    out["rot_deg_per_min"] = null;
    out["rot_text"] = "turning right > 5 deg per 30 s (no turn indicator)";
  } else if (raw === -127) {
    out["rot_deg_per_min"] = null;
    out["rot_text"] = "turning left > 5 deg per 30 s (no turn indicator)";
  } else {
    // x * x rather than x ** 2: both give the correctly rounded square, and
    // multiplication makes that obvious (C's pow() is what Python calls).
    const scaled = raw / 4.733;
    let value = scaled * scaled;
    if (raw < 0) {
      value = -value;
    }
    out["rot_deg_per_min"] = new Fixed(value, 1);
    out["rot_text"] = raw > 0 ? "turning right" : raw < 0 ? "turning left" : "not turning";
  }
  return out;
}

/** SOTDMA communication state, 19 bits (Annex 2 Table 18 and 19). */
export function decodeSotdma(bits: Bits, start: number): JsonObject {
  const sync = bits.uint(start, 2);
  const timeout = bits.uint(start + 2, 3);
  const sub = bits.uint(start + 5, 14);
  const out: JsonObject = {
    scheme: "SOTDMA",
    sync_state: sync,
    sync_state_text: SYNC_STATE[sync],
    slot_timeout: timeout,
  };
  // The meaning of the 14-bit sub-message depends on the slot time-out.
  if (timeout === 3 || timeout === 5 || timeout === 7) {
    out["received_stations"] = sub;
  } else if (timeout === 2 || timeout === 4 || timeout === 6) {
    out["slot_number"] = sub;
  } else if (timeout === 1) {
    out["utc_hour"] = sub >> 9;            // bits 13..9
    out["utc_minute"] = (sub >> 2) & 0x7f; // bits 8..2 (7 bits)
  } else { // timeout === 0
    out["slot_offset"] = sub;
  }
  return out;
}

/** ITDMA communication state, 19 bits (Annex 2 Table 20). */
export function decodeItdma(bits: Bits, start: number): JsonObject {
  const sync = bits.uint(start, 2);
  const increment = bits.uint(start + 2, 13);
  const nslots = bits.uint(start + 15, 3);
  const keep = bits.flag(start + 18);
  const out: JsonObject = {
    scheme: "ITDMA",
    sync_state: sync,
    sync_state_text: SYNC_STATE[sync],
    slot_increment: increment,
    slots_raw: nslots,
    keep_flag: keep,
  };
  // Table 20: 0..4 mean 1..5 slots; 5..7 mean 1..3 slots with +8192 offset.
  if (nslots <= 4) {
    out["slots"] = nslots + 1;
    out["slot_offset"] = increment;
  } else {
    out["slots"] = nslots - 4;
    out["slot_offset"] = increment + 8192;
  }
  return out;
}

/** Decode a message 1, 2 or 3. Throws DecodeError if too short. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 168);
  const nav = bits.uint(38, 4);
  const sogRaw = bits.uint(50, 10);
  const lonRaw = bits.sint(61, 28);
  const latRaw = bits.sint(89, 27);
  const cogRaw = bits.uint(116, 12);
  const heading = bits.uint(128, 9);
  const timestamp = bits.uint(137, 6);
  const manoeuvre = bits.uint(143, 2);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    nav_status: nav,
    nav_status_text: NAV_STATUS[nav],
    sog_knots: new Fixed(sogRaw / 10.0, 1),
    sog_available: sogRaw !== 1023,
    position_accuracy: bits.uint(60, 1),
    lon: new Fixed(lonRaw / 600000.0, 6),
    lat: new Fixed(latRaw / 600000.0, 6),
    position_available: lonRaw !== 0x6791ac0 && latRaw !== 0x3412140,
    cog_deg: new Fixed(cogRaw / 10.0, 1),
    cog_available: cogRaw !== 3600,
    true_heading: heading,
    heading_available: heading !== 511,
    timestamp,
    special_manoeuvre: manoeuvre,
    special_manoeuvre_text: SPECIAL_MANOEUVRE[manoeuvre],
    spare: bits.uint(145, 3),
    raim: bits.flag(148),
  };
  Object.assign(out, decodeRot(bits.sint(42, 8)));
  if (msgType === 3) {
    out["comm_state"] = decodeItdma(bits, 149);
  } else {
    out["comm_state"] = decodeSotdma(bits, 149);
  }
  return out;
}
```

> **Language notes (TypeScript).** `{ readonly [code: number]: string }`
> is an *index signature*: an object used as a map from number to string,
> with `readonly` forbidding assignment after construction. A plain
> object literal whose keys are numbers is the natural enumeration table
> in JavaScript (the keys become strings internally, which is invisible
> here). `===` is strict equality — always use it; `==` coerces types.
> `Object.assign(out, other)` copies `other`'s properties into `out`
> (`dict.update`). `sub >> 9` and `(sub >> 2) & 0x7f` are safe despite
> the 32-bit pitfall of chapter 2 because `sub` is only 14 bits wide; the
> lat/lon arithmetic, which is not safe, goes through `Bits.sint` and
> plain division instead. `null` is a legal `JsonValue`, so
> `rot_deg_per_min: null` needs no special case. An object literal may
> contain another object (`comm_state`), which is the nested-object
> output; and `new Fixed(x, 6)` is printed with `toFixed(6)` plus the
> round-half-even correction described in chapter 2.

**Where the four differ.** The enumeration table is a `dict`, a
`std::array` indexed by value, a `match` function, and an index-signature
object: four idioms for "small integer to string". The signed fields come
out of `sint` as `int`, `std::int64_t`, `i64` and `number`; only C++ and
Rust make the sign visible in the type. The square of a `double` is
written three ways (`** 2`, `std::pow(x, 2.0)`, `x * x`) that all produce
the same correctly rounded result — the comments in the C++ and Rust
sources say why that was checked. And the fallible step is signalled by
an exception (Python, C++, TypeScript) or a `Result` with `?` (Rust); in
all four the message text is identical, which is what the test-suite
compares on line 13 of the data file.

## 3.5 Sample data and results

[`data/ch03-position-reports.nmea`](../data/ch03-position-reports.nmea)
has fourteen lines, all real, `\r` stripped and nothing else changed. Each
was chosen to show one thing:

| Line | Source (`sample-data/…`) | Source line | What it shows |
|---|---|---|---|
| 1 | `RightwhaleAisProject/20081013.log` | 4 | Message 1, under way at 14.6 kn, every field valid, received-stations sub-message |
| 2 | same | 5 | ROT and heading unavailable; slot-number sub-message; a 2-second clock offset |
| 3 | same | 12 | Navigational status 15; slot time-out 0 with a slot offset |
| 4 | same | 35 | "Engaged in fishing" with the default MMSI 1193046; UTC hour/minute sub-message |
| 5 | same | 72 | Message 3, ITDMA with keep flag; ROT −127; heading opposite to COG |
| 6 | same | 556 | Message 3 from MMSI 0 with every field at its not-available value |
| 7 | same | 12123 | "Moored" at 5.8 kn; RAIM in use |
| 8 | same | 75842 | Message 3 with a measured ROT (−21 → −19.7°/min); PA 1; ITDMA increment 0 |
| 9 | same | 214053 | "At anchor" at 3.3 kn; ROT +127 |
| 10 | `misc/rv-auk-boston-harbor-log.2008-08-20` | 9 | Message 3, keep flag, ROT +127, from the logging vessel's own transceiver |
| 11 | `2007hydro/norfolk-log.ais` | 19 | Slot time-out 1: UTC hour/minute sub-message |
| 12 | `2007hydro/protec-util12007-05-14` | 2637 | A "message 2" that is a corrupted message 4 |
| 13 | `tide1/ais/ais-2008-11-05` | — | A 56-bit payload beginning with `1` |
| 14 | `tide1/ais/ais-2008-11-05` | — | A 40-bit payload beginning with `@` (type 16) |

Run any of the four decoders:

```sh
python3 code/python/decode.py data/ch03-position-reports.nmea
code/cpp/build/decode data/ch03-position-reports.nmea
code/rust/target/release/decode data/ch03-position-reports.nmea
node code/typescript/dist/decode.js data/ch03-position-reports.nmea
```

All four print the fourteen lines of
[`tests/expected/ch03-position-reports.jsonl`](../tests/expected/ch03-position-reports.jsonl)
byte for byte. Line 1 in full — the message decoded by hand in §3.3:

```json
{"ais":{"cog_available":true,"cog_deg":18.0,"comm_state":{"received_stations":5,"scheme":"SOTDMA","slot_timeout":5,"sync_state":0,"sync_state_text":"UTC direct"},"heading_available":true,"lat":42.666992,"lon":-70.459822,"mmsi":538002219,"msg_type":1,"nav_status":0,"nav_status_text":"under way using engine","position_accuracy":1,"position_available":true,"raim":false,"repeat_indicator":0,"rot_deg_per_min":0.0,"rot_raw":0,"rot_text":"not turning","sog_available":true,"sog_knots":14.6,"spare":0,"special_manoeuvre":0,"special_manoeuvre_text":"not available","timestamp":24,"true_heading":18},"channel":"A","fragments":1,"line":1,"own_ship":false,"payload_bits":168,"talker":"AI","uscg":{"rssi":34486,"sentence_counter":78124,"signal_dbm":-74,"station":"003669946","station_kind":"receiver","time":1223901985,"time_of_arrival_s":25.49701619}}
```

The `ais` object has 24 keys: the 15 fields of Table 48 before the
communication state, three enumerations doubled with their `_text`
(`nav_status`, `special_manoeuvre`, and the rate of turn tripled as
`rot_raw`, `rot_deg_per_min`, `rot_text`), four sentinel fields doubled
with their `_available` (SOG, position, COG, heading), and `comm_state`
nested.
The rest of the file, in groups.

### Under way, moored, anchored, undefined (lines 1, 2, 3, 7, 9)

| Line | MMSI | Type | `nav_status` | `sog_knots` | `cog_deg` | `true_heading` | `rot_raw` | `lat`, `lon` | `timestamp` | comm state |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 538002219 | 1 | 0 under way | 14.6 | 18.0 | 18 | 0 | 42.666992, −70.459822 | 24 | time-out 5, received stations 5 |
| 2 | 367043190 | 1 | 0 under way | 10.7 | 209.1 | 511 | −128 | 43.170997, −70.483103 | 23 | time-out 4, slot 883 |
| 3 | 367022790 | 1 | 15 undefined | 0.0 | 294.7 | 511 | −128 | 42.362315, −71.032353 | 26 | time-out 0, slot offset 2230 |
| 7 | 367164250 | 1 | 5 moored | 5.8 | 164.8 | 511 | −128 | 42.383257, −71.044622 | 3 | time-out 4, slot 98 |
| 9 | 369156000 | 1 | 1 at anchor | 3.3 | 37.1 | 44 | +127 | 42.034342, −70.171502 | 40 | time-out 4, slot 1499 |

* **Line 2** is the typical small US commercial vessel: no heading sensor
  (`"true_heading":511`, `"heading_available":false`), no turn indicator
  (`"rot_raw":-128`, `"rot_text":"not available"`), position accuracy 0.
  Off the southern Maine coast, making 10.7 kn on 209° towards Portsmouth (her
  message 5 says `PORTSMOUTH_NH`). Her slot number is 883, which begins
  883 × 60 / 2 250 = **23.55 s** into the minute, and her time stamp says
  second 23 — consistent with each other. But the receiver's own
  time-of-arrival field says **25.55 s**. Receiver 003669946 is not the
  problem: line 1, received by the same station a few milliseconds
  earlier, has time stamp 24 and arrival 25.50, and across the first
  20 000 lines of the log the arrival time minus the slot start is
  0.00 s for most ships, 1.00 s for a handful, and 2.00 s for this one
  on every one of her 74 slot-number reports. Her transceiver's idea of
  UTC is two seconds behind everyone else's, so she transmits in the
  middle of other people's slots — the Saab leap-second fault of
  [Safety Alerts, Alert 5-05] in all but name, three years later.
  A decoder cannot *fix* this; it can detect it, because the report
  carries its own clock in two places.
* **Line 3** is a towing vessel (`QUENAMES`) stopped at the East Boston
  waterfront with status 15, "undefined (default)" — nobody has ever set
  it. Her slot time-out is 0 and the slot offset is 2230: this is the
  last use of this slot, and next frame she will transmit 2230 slots
  after it — equivalently 2250 − 2230 = **20 slots (0.53 s) earlier** in
  the frame than the current slot [M.1371-5 Annex 2 Table 19]. A
  listening receiver can now mark the old slot free and the new one
  taken.
* **Line 7** is the tug `FREEDOM` in the Mystic River, Boston, declaring
  herself **moored at 5.8 knots**, 165° COG. The status is stale; the
  transceiver dutifully applies the rule that "at anchor or moored" *and*
  under 3 knots means 3-minute reporting [M.1371-5 Annex 2 §4.3.1.3] —
  she is over 3 knots, so she still reports every 10 s, but a display
  that filters "moored" targets would hide a moving tug. `"raim":true`
  with `"position_accuracy":0` is the third row of Table 50: RAIM
  available, expected error over 10 m.
* **Line 9** is `KITTIWAKE`, a Provincetown excursion boat, "at anchor" at
  3.3 kn with ROT +127 at 00:27 UTC. Over 3 knots again, so 10-second
  reporting; the +127 says the transceiver saw the heading change more
  than 5° in 30 s and has no rate sensor to quantify it — a boat swinging
  hard at a mooring, or one that has slipped it and left the status
  behind. Slot 1499 starts at 39.97 s; time stamp 40.

### The default MMSI (line 4)

```json
{"ais":{"cog_available":true,"cog_deg":0.0,"comm_state":{"scheme":"SOTDMA","slot_timeout":1,"sync_state":0,"sync_state_text":"UTC direct","utc_hour":12,"utc_minute":46},"heading_available":false,"lat":41.575413,"lon":-71.281970,"mmsi":1193046,"msg_type":1,"nav_status":7,"nav_status_text":"engaged in fishing","position_accuracy":0,"position_available":true,"raim":false,"repeat_indicator":0,"rot_deg_per_min":null,"rot_raw":-128,"rot_text":"not available","sog_available":true,"sog_knots":0.0,"spare":0,"special_manoeuvre":0,"special_manoeuvre_text":"not available","timestamp":27,"true_heading":511},"channel":"B","fragments":1,"line":4,"own_ship":false,"payload_bits":168,"talker":"AI","uscg":{"sentence_counter":104282,"station":"003669947","station_kind":"receiver","time":1223901988}}
```

MMSI **1193046** = 0x123456, a seven-digit number that fits no format in
§3.2, stationary in Newport harbour, Rhode Island, "engaged in fishing".
The same MMSI, with the same nonsense IMO number 303174162 in its message
5, calls itself `NAUTICAST`, a pilot vessel bound for `CASABLANCA`, in
this log — and `MISS JEAN`, a tug bound for `NORF VA`, in the Norfolk log
that supplies line 12. Two transceivers of the same make, neither ever
configured. The slot time-out is 1, so the sub-message is the UTC hour
and minute: 12:46, matching the receiver's 12:46:28. (The full
right-whale log contains 229 reports from 1193046.)

### Message 3 and ITDMA (lines 5, 8, 10)

| Line | MMSI | `nav_status` | `sog_knots` | `cog_deg` | `true_heading` | `rot_raw` → `rot_deg_per_min` | `slot_increment` | `slots` | `keep_flag` | sync |
|---|---|---|---|---|---|---|---|---|---|---|
| 5 | 367137410 | 0 under way | 4.0 | 223.5 | 38 | −127 → null | 248 | 1 | true | 0 |
| 8 | 311612000 | 3 restricted manoeuvrability | 0.6 | 253.0 | 272 | −21 → **−19.7** | 0 | 1 | false | 0 |
| 10 | 369871000 | 0 under way | 2.0 | 348.3 | 182 | +127 → null | 253 | 1 | true | 0 |

* **Line 5** is the tug `ORION` in Salem Sound, heading 038° but moving
  223.5° — **going astern** at 4 knots, and turning left fast enough
  (−127) to trigger the "changing course" rule. That is why it is a
  message 3: the 3 1/3-second rate for a turning ship under 14 knots is
  achieved by inserting ITDMA reports between the SOTDMA ones
  [M.1371-5 Annex 2 §4.3.1.2]. The communication state promises the next
  transmission **248 slots** later. 248 × 26.667 ms = 6.613 s; this line
  arrived at 32.097 s, and the source log (line 137, not excerpted) has
  the next message 3 from `ORION` at **38.710 s** — the promise kept to
  the millisecond, announcing a further 243 slots. Over the first 20 000
  lines of the log her reports are 3–4 s apart 214 times and 10 s apart
  31 times: message 1 on the 10-second SOTDMA schedule with two message
  3s between.
* **Line 8** is `NORTHERN CANYON`, a cable/survey ship whose message 5
  destination reads `SUBSEA OPS 0.5 CPA` — asking passing traffic for
  half a mile. Status 3, 0.6 kn, PA 1 (differential fix), heading 272,
  COG 253, and the only genuine *non-zero* measured rate of turn in the file:
  `"rot_raw":-21` → (21 / 4.733)² = 19.69 → `"rot_deg_per_min":-19.7`,
  `"rot_text":"turning left"`. Slot increment 0 and keep flag false mean
  "no more ITDMA transmissions; back to the SOTDMA schedule"
  [M.1371-5 Annex 2 §3.3.4.1.1].
* **Line 10** is the NOAA R/V *Auk* herself, moored in Scituate, heard by
  the SR162G receiver on her own deck (station `auk`; the receiver's own
  `!AIVDO` lines are its GPS re-encoded with MMSI 0, and are *not* the
  ship's Class A). Heading 182, COG 348, 2 kn, ROT +127, keep flag set,
  next slot 253 away. Everything about it says "manoeuvring at low speed
  with a GPS course that means little".

### Nothing available (line 6)

```text
!AIVDM,1,1,,B,3000005P?w<tSF0l4Q@>4?wp1P00,0*7C,x104487,r003669947,1223902043
```

```json
{"ais":{"cog_available":false,"cog_deg":360.0,"comm_state":{"keep_flag":false,"scheme":"ITDMA","slot_increment":0,"slot_offset":0,"slots":1,"slots_raw":0,"sync_state":3,"sync_state_text":"synchronized to another station"},"heading_available":false,"lat":91.000000,"lon":181.000000,"mmsi":0,"msg_type":3,"nav_status":5,"nav_status_text":"moored","position_accuracy":0,"position_available":false,"raim":false,"repeat_indicator":0,"rot_deg_per_min":null,"rot_raw":-128,"rot_text":"not available","sog_available":false,"sog_knots":102.3,"spare":0,"special_manoeuvre":0,"special_manoeuvre_text":"not available","timestamp":60,"true_heading":511},"channel":"B","fragments":1,"line":6,"own_ship":false,"payload_bits":168,"talker":"AI","uscg":{"sentence_counter":104487,"station":"003669947","station_kind":"receiver","time":1223902043}}
```

Every sentinel in one message: MMSI 0, SOG 1023 (`102.3`, `false`),
longitude 181°, latitude 91°, COG 3600 (`360.0`, `false`), heading 511,
time stamp 60, ROT −128, sync state 3 ("synchronized to another
station" — it has no UTC of its own). Only the navigational status, 5
"moored", was ever set by a human. This is a Class A transceiver with no
MMSI programmed and no position sensor, powered up alongside a Boston
pier and transmitting anyway. It is a useful test vector: a decoder that
prints `"lon":181.000000` with `"position_available":false` and does not
crash on MMSI 0 is handling the not-available cases correctly.

### UTC in the sub-message (line 11)

Line 11, from the Norfolk, Virginia log of May 2007, is a pilot boat
(`.VA.PILOT SWIFT`, MMSI 367023320) at 0.1 kn with slot time-out 1:
`"utc_hour":19,"utc_minute":21`. The logger's epoch stamp 1179170469.4
is 19:21:09.4 UTC. The hour and minute sub-message is the one place a
position report says what *time* its sender thinks it is; chapter 6
uses it to catch a base station two minutes slow.

### A message 2 that is not one (line 12)

```text
!AIVDM,1,1,,B,203OvmAuMGD@MJRW9>E9`1100HAg,0*0D,1179173790.22
```

```json
{"ais":{"cog_available":true,"cog_deg":246.4,"comm_state":{"scheme":"SOTDMA","slot_number":1135,"slot_timeout":6,"sync_state":0,"sync_state_text":"UTC direct"},"heading_available":true,"lat":68.409313,"lon":-164.176505,"mmsi":3669717,"msg_type":2,"nav_status":1,"nav_status_text":"at anchor","position_accuracy":0,"position_available":true,"raim":false,"repeat_indicator":0,"rot_deg_per_min":-5.4,"rot_raw":-11,"rot_text":"turning left","sog_available":true,"sog_knots":85.5,"spare":0,"special_manoeuvre":0,"special_manoeuvre_text":"not available","timestamp":32,"true_heading":32},"channel":"B","fragments":1,"line":12,"own_ship":false,"payload_bits":168,"talker":"AI","uscg":{"time":1179173790.22}}
```

The checksum is valid and the length is right, and the decoder has no
grounds to refuse it. But read it: a ship at **anchor doing 85.5 knots**
while turning left at 5.4°/min, at 68.4° N 164.2° W — the Chukchi Sea
coast of north-west Alaska, the far side of the continent from the
Norfolk receiver that logged it —
with MMSI **3669717**, which is 003669717, a *coast-station* identity.
Chapter 6 meets 003669717: it is a GPS-fed base station on the Hampton
Roads shore at 36.963° N, 76.325° W, reporting every 10 s. Base stations
send message 4, not message 2, and message 4 is also 168 bits with the
MMSI in the same place. Message ID 2 is `000010`; message ID 4 is
`000100`. Two adjacent bits.

Take the sentence, change the first payload character from `2` to `4`,
recompute the checksum (`0B`), and run chapter 6's decoder on it:

```text
!AIVDM,1,1,,B,403OvmAuMGD@MJRW9>E9`1100HAg,0*0B,1179173790.22
```

```json
{"ais":{"comm_state":{"scheme":"SOTDMA","slot_number":1135,"slot_timeout":6,"sync_state":0,"sync_state_text":"UTC direct"},"epfd":1,"epfd_text":"GPS","lat":36.962993,"lon":-76.325055,"mmsi":3669717,"msg_type":4,"position_accuracy":0,"position_available":true,"raim":false,"repeat_indicator":0,"spare":0,"transmission_control":0,"transmission_control_text":"Class A stops message 27 within base station coverage (default)","utc_day":14,"utc_day_available":true,"utc_hour":20,"utc_hour_available":true,"utc_iso":"2007-05-14T20:16:29Z","utc_minute":16,"utc_minute_available":true,"utc_month":5,"utc_month_available":true,"utc_second":29,"utc_second_available":true,"utc_year":2007,"utc_year_available":true},"channel":"B","fragments":1,"line":1,"own_ship":false,"payload_bits":168,"talker":"AI","uscg":{"time":1179173790.22}}
```

(This modified sentence is not in `data/`; it was made in a scratch
file.) A perfectly ordinary base-station report: 003669717 at
36.962993° N, 76.325055° W — 6 m from where chapter 6 places it — saying
2007-05-14T20:16:29Z, logged at 20:16:30.22, in slot 1135 (30.27 s into
the minute). The communication state did not change because bits
149–167 were never touched; the "at anchor" and the rate of turn were the
top twelve bits of the UTC year (bits 38–49), the "85.5 knots" were the
end of the year, the month and most of the day (bits 50–59), and the
Chukchi Sea position was the hour, minute, second and PA flag followed by
the real longitude read 18 bits early. One or two
bits of a 168-bit frame were corrupted on the air — before the CRC, which
the receiver evidently failed to check, or in the receiver — and the NMEA
checksum was then computed correctly over the corrupted payload. This is
the deepest lesson of the chapter: **a valid checksum, a legal message
type and a legal length prove only that the serial link is clean.** The
content has to pass plausibility tests too, and this one fails four
(speed at anchor, MMSI format, distance from receiver, message type for
that station). A real message 2 would come from a `MIDXXXXXX` MMSI in an
area where a base station is assigning schedules. The few dozen payloads
beginning with `2` in the whole of `sample-data/` are isolated one-off
lines, most of them from the noisy Great Bay receiver of chapter 2; we
have not found one from a ship that then keeps sending message 2 for
minutes on end, as assigned mode would require.

### Garbage with good checksums (lines 13, 14)

```text
!AIVDM,1,1,,A,1fTDO<hLWP,4*35,rnhjel,1225518541.08
!AIVDM,1,1,,A,@3<QIqp,2*72,rnhjel,1225517931.54
```

```json
{"channel":"A","error":"message 1 too short: 56 bits < 168","fragments":1,"line":13,"msg_type":1,"own_ship":false,"payload_bits":56,"talker":"AI","uscg":{"station":"nhjel","station_kind":"receiver","time":1225518541.08}}
{"channel":"A","error":"message 16 too short: 40 bits < 96","fragments":1,"line":14,"msg_type":16,"own_ship":false,"payload_bits":40,"talker":"AI","uscg":{"station":"nhjel","station_kind":"receiver","time":1225517931.54}}
```

The `nhjel` receiver at Great Bay, New Hampshire, emitted thousands of
these (chapter 2 §2.6). Line 13 is ten characters minus 4 fill bits = 56
bits whose first six happen to be `000001`; `require_bits` stops it with
the shared error text, and the driver still reports the envelope
(`payload_bits`, `msg_type`, `uscg`) so that a log analyser can count
such lines per receiver. Line 14 is seven characters minus 2 fill bits =
40 bits starting with `@` = 16, which is a legal message ID
(assignment mode command, 96 or 144 bits; chapter 11), so chapter 11's
decoder is called — and its first act is to reject the payload with
`message 16 too short: 40 bits < 96`. Nothing before the length check should be
trusted, and the length check should be the *first* thing a message
decoder does.

## 3.6 Pitfalls and real-world deviations

* **Navigational status is typed by hand and goes stale.** Moored at
  5.8 kn (line 7), at anchor at 3.3 kn (line 9), undefined forever (line
  3). Never derive "is this ship moving?" from the status; derive it from
  SOG and successive positions, and treat the status as the crew's claim.
  Remember also that the status drives the *reporting rate*: a ship that
  sails without changing "at anchor" and stays under 3 kn reports every
  3 minutes.
* **Sentinels must be tested on the raw integer.** `lon_raw == 0x6791AC0`,
  `sog_raw == 1023`, `cog_raw == 3600`, `heading == 511`, `rot_raw ==
  -128`, `timestamp >= 60`. Testing `lon == 181.0` after a floating-point
  division works today and will break the day someone changes the number
  of decimals. Test latitude *and* longitude: some units send one valid
  and one sentinel.
* **ROT 0 and ROT −128 are different.** 0 means a working indicator reads
  zero; −128 means there is no indicator. Our `rot_raw` keeps the
  distinction; a decoder that outputs only degrees per minute loses it.
  ±127 is a *flag*, not a rate — do not square it.
* **Heading 511 is the norm for small craft.** Treat COG as the only
  direction available and do not draw a heading line. When both exist and
  differ by ~180°, the vessel is going astern (lines 5 and 10), not
  broken.
* **COG at rest is noise.** Line 3 has SOG 0.0 and COG 294.7°. The spec
  does not require 3600 when stopped and most units send the last
  computed value.
* **The time stamp is a second, not a time.** Combine it with the
  receiver's clock to reconstruct the fix time, and when the two disagree
  by more than a couple of seconds, suspect the transceiver's GNSS (line
  2) or the logger's clock (chapter 6). Values 61–63 mean the position
  was not from a working GNSS.
* **Placeholder MMSIs are common and shared.** 1193046 (two different
  boats in our data), 0 (line 6 and every receiver-echo `!AIVDO`),
  111111111, 693559 and 80337 all appear in the right-whale log. Key your
  database on MMSI, but keep a list of known placeholders and never merge
  tracks across one.
* **Seven- and eight-digit MMSIs are nine-digit identities with leading
  zeros.** 3669717 is 003669717, a coast station. A coast-station or
  group identity transmitting message 1 is either a shore transceiver
  configured that way on purpose (the right-whale project's) or a
  misconfiguration.
* **Message 2 is almost never a message 2.** It requires a base station
  that assigns schedules. Outside such areas, a message ID 2 is far more
  likely to be a corrupted 4 (line 12) — or a corrupted 1 or 3 — than the
  real thing. Validate every "message 2" against the MMSI format and the
  ship's other reports before believing it.
* **Valid checksum, invalid content.** Lines 12, 13 and 14. Receivers that
  do not verify the radio-layer CRC, or that output partial frames, pass
  garbage with a perfect NMEA checksum. Enforce the length, then
  plausibility.
* **Whole-second slot offsets.** When a receiver provides a precise time
  of arrival (the USCG `T` field, or a TAG block `c:` with milliseconds),
  compare it with the SOTDMA slot number. An offset that is a whole
  number of seconds, constant per ship, is a transceiver with a
  leap-second error [Safety Alerts, Alert 5-05]; its transmissions
  straddle two slots and collide with its neighbours.
* **The communication state is only meaningful for the channel heard.**
  A slot number on channel B says nothing about channel A, and the
  *same* message received by two receivers has the same communication
  state — de-duplicate before building a slot map.
* **Edition drift in bits 143–147.** Pre-M.1371-3 decoders treat them as a
  4-bit regional field plus 1-bit spare; current ones as a 2-bit
  manoeuvre indicator plus 3-bit spare [M.1371-2 Table 15a; AIVDM "Types
  1, 2 and 3"]. In our data the five bits are always zero, so the readings
  agree.

## 3.7 Exercises

1. **(Pencil)** Decode line 9 by hand as in §3.3. Confirm MMSI 369156000,
   SOG 3.3 kn, ROT raw +127, latitude 42.034342° N, longitude 70.171502° W,
   time stamp 40, slot time-out 4 and slot number 1499. Then compute the
   slot's start time within the minute and compare with the time stamp.
2. **(Pencil)** Show that the latitude sentinel 0x3412140 is 91 × 600 000
   and write it as 27 binary digits. Then take latitude 42.666992° (line
   1), convert it back to 1/10 000 minutes, and show that it reproduces the
   27 bits `001100001101010000011000011`. Which bit would you flip to move
   the ship exactly 0.0001′ north?
3. **(Python)** Write `rot_encode(deg_per_min: float) -> int` implementing
   4.733 × √ROT with sign and rounding, and check that
   `decode_rot(rot_encode(x))` recovers *x* to within the coding's
   resolution for x in −708 … +708. At what rate does the resolution
   become worse than 1°/min? Than 10°/min?
4. **(TypeScript)** Add a `plausible(ais): string[]` function to a small
   script that reads a decoded `.jsonl` file and lists, per line, the
   checks it fails: SOG > 3 kn with status 1 or 5; SOG > 50 kn; MMSI not
   in `MIDXXXXXX` format; position more than 100 NM from a receiver
   position you supply. Run it on
   `tests/expected/ch03-position-reports.jsonl` with the Boston receiver
   at 42.36° N 71.05° W and a Norfolk receiver at 36.85° N 76.30° W; lines
   7, 9 and 12 should be flagged (and line 4's position is 50 NM away —
   decide whether that is a problem).
5. **(Rust)** `decode_sotdma` outputs one of four sub-message keys. Add a
   unit test (`#[cfg(test)]`) that builds a 19-bit communication state for
   each slot time-out value 0–7 from a bit string, decodes it, and asserts
   which key is present and that the UTC hour/minute split for time-out 1
   puts hour 23, minute 59 into the right bits (`10111 111011 00`).
6. **(C++, larger data)** Using `sample-data/RightwhaleAisProject/20081013.log`,
   for every message 1/2/3 whose comm state carries a slot number and
   whose USCG metadata carries `T`, compute `(T − slot × 60/2250)` modulo
   60 and keep a per-MMSI histogram in a `std::map<std::uint64_t,
   std::map<int, int>>` of the value rounded to 0.1 s. Which ships are
   consistently 1.0 or 2.0 s off? Then check whether those ships' time
   stamps (bits 137–142) show the same offset against the receiver's
   epoch stamp.
7. **(Any language, harder)** Build a slot map. For one receiver and one
   channel in the same log, keep a 2 250-entry array per frame and mark
   each slot used by a message 1/2/3 with a slot-number sub-message, each
   slot promised by an ITDMA increment, and each slot released by a
   time-out-0 offset. How often is a promised ITDMA slot actually used by
   the promising MMSI (as in line 5)? How often does a station appear in a
   slot that another station's time-out said was still reserved?

## 3.8 References

* [M.1371-5 Annex 8 §3.1 Table 48] — message 1/2/3 layout, every field's
  range and sentinel; [Table 49] — SOTDMA for 1 and 2, ITDMA for 3;
  [Table 50] — position accuracy and RAIM determination; [Table 46] —
  names, access schemes and priorities of the three messages;
  [§3.13 Table 65] — message 3 as the response to interrogation;
  [§3.25] and [Annex 4] — message 27 for satellite reception.
* [M.1371-5 Annex 1 §4.2.1 Table 1] — Class A reporting intervals.
* [M.1371-2 Annex 2 §3.3.8.2.1 Table 15a] — the pre-M.1371-3 layout of
  bits 143–147 (4-bit regional field, 1-bit spare); [M.1371-4 Annex 8
  Table 45] — navigational status 14 added, 11–13 still reserved.
* [M.1371-5 Annex 2 §3.1.2] — 2 250-slot frame; [§3.3.4.1–§3.3.4.4] —
  ITDMA, RATDMA, FATDMA and SOTDMA access schemes; [§3.3.5.2–§3.3.5.3] — network
  entry with message 3; [§3.3.6] — assigned mode and message 2;
  [§3.3.7.2.1–§3.3.7.2.3 Tables 18–19] — SOTDMA communication state and
  sub-messages; [§3.3.7.3.2 Table 20] — ITDMA communication state;
  [§4.1.2] — channel alternation; [§4.3.1.1–§4.3.1.3] — reporting
  interval changes with speed, course and navigational status;
  [§4.6.1] — repeat indicator.
* [M.585-9 Annex 1 Sections 1–5], [Annex 2 Sections 1–2] — MMSI formats
  for ships, groups, coast stations, SAR aircraft, AtoN, associated craft,
  handhelds, AIS-SART, MOB and EPIRB-AIS.
* [MSC.74(69) Annex 3 §6.2 Table 1] — the IMO reporting-interval table
  that M.1371 refines.
* [A.1106(29)] — operational use of AIS on the bridge; manual-input
  fields.
* [33 CFR 164.46] — US carriage requirement.
* [M.2169] — satellite detection of AIS and why message 27 exists.
* [Safety Alerts] — Alert 5-05 (leap-second slot timing fault), Alerts
  05-10 and 10-08 (inaccurate AIS data, civil penalties).
* [AIVDM "Types 1, 2 and 3: Position Report Class A"] — GPSD's field
  table, the history of bits 143–147 and the inland "blue sign" reading
  of the manoeuvre indicator.
