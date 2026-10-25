# 13. Messages 17, 20, 22 and 23: base-station control of the data link

> **What you will learn**
> * The four messages with which a *competent authority's* base station steers the AIS data link rather than describing a ship: 17 (DGNSS corrections), 20 (FATDMA slot reservations), 22 (channel management) and 23 (group assignment).
> * How a reservation of slots is encoded as *offset, number of slots, time-out, increment* and why the same physical slots show a different offset in every message 20 you receive.
> * The four-digit ITU-R M.1084 channel numbers (2087 = 161.975 MHz, 2088 = 162.025 MHz), the Tx/Rx mode, power and transitional-zone fields of message 22, and how one 70-bit span of the message is either two rectangle corners *or* two MMSIs depending on a flag that comes *after* it.
> * The 120 nautical-mile rule, the 4–8 minute time-out of a group assignment, and the quiet-time command.
> * What the recordings actually contain (thousands of message 20 from one USCG base station, hundreds of default message 22 from a bench transceiver, no valid 17 or 23) and how to build labelled synthetic examples where the air was silent.
> * Language notes: a nested JSON object for the RTCM header, optional integers for "frequency unknown", pattern matching on ranges (`matches!`, range arms), and `Set` membership tests.

## 13.1 Where this fits

Everything decoded so far was *about a station*: its position, its name, its cargo, a text. The four messages of this chapter are different in kind. They are sent only by **base stations** — Table 46 marks all four "B" in the M/B column — and they tell the mobile stations around them how to behave on the VHF data link [M.1371-5 Annex 8 Table 46]. A base station "is always operating in assigned mode using a fixed transmission schedule (FATDMA) for its periodic transmissions" and uses message 20 "to announce the base station's fixed allocation schedule" [M.1371-5 Table 46 note 3]. Annex 2 §4.5 lists the base station's tasks: provide synchronisation (message 4, chapter 6), provide slot assignments (messages 16 and 20), assign reporting rates (messages 16 and 23), transmit channel management messages (22) and "optionally provide GNSS corrections via the VDL by Message 17" [M.1371-5 Annex 2 §4.5].

| Message | Name | Length | Priority | Access | Purpose |
|---|---|---|---|---|---|
| 17 | GNSS broadcast binary message | 80–816 bits | 2 | FATDMA, RATDMA | differential GNSS corrections (ITU-R M.823 / RTCM SC-104 words) [M.1371-5 §3.15] |
| 20 | Data link management message | 72–160 bits | 1 | FATDMA, RATDMA | announce FATDMA slot reservations of one or more base stations [M.1371-5 §3.18] |
| 22 | Channel management | 168 bits | 1 | FATDMA, RATDMA | set channels A/B, Tx/Rx mode, power, for a region or two addressed stations [M.1371-5 §3.20] |
| 23 | Group assignment command | 160 bits | 1 | FATDMA, RATDMA | set Tx/Rx mode, reporting interval and quiet time for a group selected by region, station type and ship type [M.1371-5 §3.21] |

Sources: [M.1371-5 Annex 8 Table 46] for priority, access scheme and M/B; the section numbers for lengths. FATDMA and RATDMA are the fixed-access and random-access TDMA schemes of chapter 3.

Three of the four carry a **rectangle** — north-east and south-west corners in 1/10 minute (18-bit longitude, 17-bit latitude), the same compact coordinates as message 27 (chapter 16). Message 17 carries one such pair (the surveyed reference-station position); messages 22 and 23 carry two. Annex 2 §4.1.4 says a regional operating area "should be designated by a Mercator projection rectangle with two reference points (WGS-84)", NE first, SW second, to the nearest tenth of a minute [M.1371-5 Annex 2 §4.1.4].

Two rules recur and are worth stating once:

* **The 120 NM rule.** FATDMA reservations announced by message 20 "apply within a range of 120 nautical miles from the reserving base station"; beyond that "all stations may consider these slots as available" [M.1371-5 Annex 2 §3.3.4.3.1]. Message 22 likewise "should be accompanied by a Message 4 transmission for evaluation of the message within 120 NM" [M.1371-5 §3.20]. A mobile station can only apply the rule if it knows where the base station *is*, which is why "a data link management message (Message 20) without a base station report (Message 4) should be ignored" [M.1371-5 Annex 2 §3.3.4.3.1] — footnote 27 to §3.18 says the same for distance determination. Chapter 6 decoded message 4; this chapter assumes you have it.
* **Assignments time out.** A station that accepts a message 23 "should tag it with a time-out, randomly selected between 4 and 8 min after the first transmission" and then return to autonomous mode, so the authority must re-issue the command to keep it in force [M.1371-5 Annex 7 §4.3.3.3.2 and footnote 19]. Message 20 reservations carry their own time-out in minutes, refreshed by each repetition [M.1371-5 §3.18]. Regional settings received by message 22 are stored (the last eight) and erased when the region is more than 500 NM away or the setting is older than 24 hours [M.1371-5 Annex 2 §4.1.1, §4.1.8].

### Operational use

* **Slot planning (message 20).** A vessel traffic service (VTS) or coast-guard network with several base stations in Hampton Roads, say, needs each station's message 4, 17, 22 and binary broadcasts to go out reliably in a busy data link. The authority pre-configures FATDMA blocks per station ("each allocation should be pre-configured by the competent authority and not changed for the duration of the operation of the station") and the stations advertise them with message 20 so that ships keep out [M.1371-5 Annex 2 §3.3.4.3.1]. The recommended increments (2, 3, 5, 6, 9, 10, 15, 18, 25, 30, 45, 50, 75, 90, 125, 150, 225, 250, 375, 450, 750, 1125) all divide 2 250, the number of slots in a one-minute frame, so that the reservation pattern repeats identically every frame [M.1371-5 §3.18; Annex 2 §3.1.2]. Our USCG example reserves 5 slots every 150 — 15 blocks, 75 slots, 3.3 % of each channel.
* **Regional channel plans (message 22).** Where the AIS 1/AIS 2 frequencies are not available or a port wants a private channel pair, the authority defines a rectangle and the channels to use inside it. Ships "should immediately set [their] operating frequency channel numbers, [their] transmitter/receiver mode and [their] power level to the values as commanded" when inside the rectangle [M.1371-5 Annex 2 §4.1.4]. Within 5 NM (or the *transitional zone size* the message specifies) of the boundary a ship enters *transitional mode*, transmitting and receiving on the primary channel of both regions with its reporting interval doubled and shared between them [M.1371-5 Annex 2 §4.1.5]. Regions must be 20–200 NM on a side and no more than three regions may meet at a corner [M.1371-5 Annex 2 §4.1.5]. The same command can be given by DSC on channel 70, by the operator, or through the presentation interface with the IEC 61162-1 `ACA` sentence — line 9 of our data file [M.1371-5 Annex 2 §4.1.1, §4.1.6; IEC 61162-1 §8 ACA]. Power: the two nominal levels are 12.5 W and 1 W (5 W and 1 W for Class B "SO"); changing them "should only be by assignment by the approved channel management means" [M.1371-5 Annex 2 §2.12].
* **Group assignment and quiet time (message 23).** The authority selects everything inside a rectangle *and* of a station type (all / Class A / Class B / SAR aircraft / Class B "SO" / Class B "CS" / inland) *and* of a ship type, "all selector fields concurrently", and commands a Tx/Rx mode, a reporting interval from Table 77 or a quiet time of 1–15 minutes [M.1371-5 §3.21, Table 76]. The obvious application is load control: in a crowded harbour, tell all pleasure craft (ship type 37) to report every 3 minutes, or tell Class B "CS" units to be quiet for 10 minutes while a safety broadcast goes out. A Class B "CS" receiving a quiet time "should not transmit Messages 18 and 24 on either channel for the time commanded", but still answers interrogations; "the quiet time command should override a reporting rate command" [M.1371-5 Annex 7 §4.3.3.3.2]. Station type 10 has a special meaning: it defines the *base station coverage area* inside which message 4's bit 138 may switch off message 27 long-range broadcasts (chapter 16) [M.1371-5 §3.21].
* **DGNSS (message 17).** Before SBAS (WAAS/EGNOS) was universal, a base station connected to a DGNSS reference receiver could rebroadcast the RTCM correction words over AIS, saving a separate 300 kHz beacon receiver on the ship. "Transmissions of Message 17 by base stations should take into account ageing, update rate and the resulting accuracy of the DGNSS service" and "should be no more than necessary", because a full one (816 bits) costs four slots [M.1371-5 §3.15 Table 69 NOTE 3; Annex 2 §3.2.2.9 and §3.2.2.11 (168 data bits in one 256-bit slot, 256 more per additional slot)]. Few authorities ever did this; the US did not, and our recordings contain no valid message 17 (§13.5).

### Unattended / automated uses

* **Autonomous base-station networks.** All four messages are generated by software, not by an operator on watch. A network controller computes a slot plan for *n* stations, hands each its FATDMA blocks, and the stations emit message 20 at a fixed cadence (our USCG station: every 10 s, alternating channels, with a 7-minute time-out). The same software decides regional channel plans and emits 22s, and can react to measured channel load — count the occupied slots per frame from message 4's slot number and the received traffic — by issuing a 23 with a longer reporting interval or a quiet time for a selected group. Everything in this chapter is therefore also the *input* to an automated base-station monitor: are all stations still announcing, do their reservations overlap, is a 22 or 23 in force that should not be.
* **A side channel for data.** Message 17 is a "binary message" with a prescribed structure, but a receiver that does not restore RTCM preamble and parity (NOTE 1) simply sees 24-bit words. Some systems have used it to push non-GNSS data to receivers that understand nothing else; treat unexpected RTCM message types (Table 69's 6-bit *message type*) as exactly that.
* **Rogue-command detection.** This is the security angle. Nothing in AIS authenticates a sender. A transmitter with a base-station-looking maritime mobile service identity (MMSI) can broadcast a message 22 moving every ship in a rectangle to an empty channel pair, or a 23 commanding a 15-minute quiet time, or a 20 reserving most of a frame; compliant transceivers will obey. IMO's resolution on the protection of the AIS VHF data link recognises that "any disruption to those channels by any one AIS device could affect the operation of all AIS devices on that network" and asks administrations to "take steps necessary to ensure the integrity of the radio channels used for AIS in their waters" [MSC.140(76)]. The spec gives you the checks a monitoring receiver can run automatically: the sender's MMSI must have the coast-station form 00MIDxxxx [M.585-9]; a 20 or 22 must be paired with a message 4 from the same MMSI [M.1371-5 Annex 2 §3.3.4.3.1; §3.20]; region sides must be 20–200 NM [M.1371-5 Annex 2 §4.1.5]; reservation blocks must not exceed 5 slots and increments should be from the recommended list [§3.18]; station type and reporting-interval codes must be defined values; and a *mobile* MMSI sending any of the four is wrong by definition [Table 46]. Our data file contains a Class A transceiver (MMSI 338040883) sending messages 20 and 22 — a bench unit in a test mode, but a detector cannot know that and should flag it.
* **Noise masquerading as rare types.** As in chapters 10 and 11, most lines that *claim* to be 17 or 23 are receiver noise with a valid checksum. Length validation (80 + 8*k* bits for 17, 72–160 and a multiple of 8 for 20, exactly 168 for 22, exactly 160 for 23) eliminates nearly all of it; field-range checks (§13.6) do the rest.

## 13.2 The specification

All four messages share the 40-bit header of every other message: message ID (6), repeat indicator (2), source MMSI (30), spare (2) [M.1371-5 Tables 68, 72, 75, 76]. Offsets below are zero-based.

### Message 17: GNSS broadcast binary message

[M.1371-5 §3.15 Table 68]:

| Bits | Len | Field | Type/units | Not-available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 17 [Table 68] |
| 6–7 | 2 | Repeat indicator | uint | — | 0 default, 3 = do not repeat [Table 68] |
| 8–37 | 30 | Source ID | MMSI | — | base station [Table 68] |
| 38–39 | 2 | Spare | — | 0 | [Table 68] |
| 40–57 | 18 | Longitude | signed, 1/10 min | 181° = 108 600 = 0x1A838 | surveyed position of the DGNSS reference station; 181° "if interrogated and differential correction service not available" [Table 68] |
| 58–74 | 17 | Latitude | signed, 1/10 min | 91° = 54 600 = 0xD548 | [Table 68] |
| 75–79 | 5 | Spare | — | 0 | [Table 68] |
| 80– | 0–736 | Data | bits | empty | "differential correction data"; empty if interrogated and no service, "interpreted by the recipient as DGNSS data words set to zero" [Table 68] |

Total 80 bits with no data, up to 816 bits with N = 29 words [Table 68]. The data section is structured per Table 69, "in accordance with Recommendation ITU-R M.823, excluding preamble and parity formatting" [M.1371-5 §3.15]:

| Bits | Len | Field | Notes |
|---|---|---|---|
| 80–85 | 6 | Message type | RTCM message type [Table 69] |
| 86–95 | 10 | Station ID | reference station identifier [Table 69] |
| 96–108 | 13 | Z count | time in units of 0.6 s, 0–3 599.4 s [Table 69] |
| 109–111 | 3 | Sequence number | cyclic 0–7 [Table 69] |
| 112–116 | 5 | N | "number of DGNSS data words following the two word header, up to a maximum of 29" [Table 69] |
| 117–119 | 3 | Health | reference station health per M.823 [Table 69] |
| 120– | N × 24 | DGNSS data words | "excluding parity" [Table 69] |

The *health* field is the three "station health" bits of the RTCM SC-104 version 2 frame header that M.823 adopts. M.823 itself defines only two codes — 111 (7) "the reference station is not working properly" and 110 (6) "the transmission is unmonitored" — and leaves the others to the service provider [M.823 Annex 1 §1.3]; the RTCM standard assigns 0–5 as scale factors of 1.0, 0.75, 0.5, 0.3, 0.2 and 0.1 on the user differential range error (UDRE), and those are the `health_text` strings our decoders print. NOTE 1 of Table 69 reminds you that "it is necessary to restore preamble and parity in accordance with Recommendation ITU-R M.823 before using this message to differentially correct GNSS positions"; NOTE 2, that with several sources "the DGNSS corrections from the nearest DGNSS reference station should be used taking into account the Z count, and the health" [M.1371-5 Table 69]. We decode the 40-bit header into a nested `rtcm` object and hand the words on as a bit string; `words_complete` tells you whether exactly N × 24 bits followed. If fewer than 40 data bits are present the header cannot be read and `rtcm` is `null`.

### Message 20: data link management message

[M.1371-5 §3.18 Table 72]:

| Bits | Len | Field | Type/units | Not-available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 20 [Table 72] |
| 6–7 | 2 | Repeat indicator | uint | — | [Table 72] |
| 8–37 | 30 | Source station ID | MMSI | — | base station [Table 72] |
| 38–39 | 2 | Spare | — | 0 | [Table 72] |
| 40–51 | 12 | Offset number 1 | slots | 0 | offset from the slot in which message 20 was received to the first reserved slot [Table 72; §3.18] |
| 52–55 | 4 | Number of slots 1 | 1–15 | 0 | consecutive slots in the block; "should not exceed 5" [Table 72; §3.18] |
| 56–58 | 3 | Time-out 1 | minutes | 0 | [Table 72] |
| 59–69 | 11 | Increment 1 | slots | — | slots between block starts; 0 = one block per frame [Table 72; §3.18] |
| 70–99 | 30 | Reservation 2 | as above | — | optional [Table 72] |
| 100–129 | 30 | Reservation 3 | as above | — | optional [Table 72] |
| 130–159 | 30 | Reservation 4 | as above | — | optional [Table 72] |
| last | 0/2/4/6 | Spare | — | 0 | "adjusted in order to observe byte boundaries" [Table 72] |

Total 72–160 bits [Table 72]. The four parameters of a block "should be treated as a unit" [§3.18]. "If interrogated and no data link management information available, only offset number 1, number of slots 1, time-out 1, and increment 1 should be sent. These fields should all be set to zero" [§3.18; repeated as Table 72 footnote 1] — our `available: false`. "This message applies only to the frequency channel in which it is transmitted" [§3.18], so a base station that reserves slots on both channels sends two different message 20s, as the USCG station in §13.5 does.

The offset is *relative to the receiving slot*. Two message 20s ten seconds (375 slots) apart that describe the same physical slots therefore show offsets that differ by 375 modulo the increment. For the USCG station (increment 150) the offsets 1974, 1224 and 474 on channel B are all ≡ 24 (mod 150), and 1624 and 874 on channel A are ≡ 124 (mod 150): one reservation pattern per channel, seen from different slots.

### Message 22: channel management

[M.1371-5 §3.20 Table 75]:

| Bits | Len | Field | Type/units | Not-available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 22 [Table 75] |
| 6–7 | 2 | Repeat indicator | uint | — | [Table 75] |
| 8–37 | 30 | Station ID | MMSI | — | base station [Table 75] |
| 38–39 | 2 | Spare | — | 0 | [Table 75] |
| 40–51 | 12 | Channel A | M.1084 channel number | — | "25 kHz simplex or simplex use of 25 kHz duplex" [Table 75] |
| 52–63 | 12 | Channel B | M.1084 channel number | — | [Table 75] |
| 64–67 | 4 | Tx/Rx mode | enum | 0 | 0 = Tx A/Tx B, Rx A/Rx B (default); 1 = Tx A, Rx A/Rx B; 2 = Tx B, Rx A/Rx B; 3–15 not used [Table 75] |
| 68 | 1 | Power | flag | 0 | 0 = high (default), 1 = low [Table 75] |
| 69–86 | 18 | Longitude 1 *or* 18 MSBs of addressed station ID 1 | signed, 1/10 min | 181° | NE corner [Table 75] |
| 87–103 | 17 | Latitude 1 *or* 12 LSBs of station ID 1 followed by 5 zero bits | signed, 1/10 min | 91° | NE corner [Table 75] |
| 104–121 | 18 | Longitude 2 *or* 18 MSBs of station ID 2 | signed, 1/10 min | 181° | SW corner [Table 75] |
| 122–138 | 17 | Latitude 2 *or* 12 LSBs of station ID 2 + 5 zero bits | signed, 1/10 min | 91° | SW corner [Table 75] |
| 139 | 1 | Addressed or broadcast message indicator | flag | 0 | 0 = broadcast geographical area (default), 1 = addressed [Table 75] |
| 140 | 1 | Channel A bandwidth | flag | 0 | 0 = default; 1 = spare, "formerly 12.5 kHz bandwidth in M.1371-1 now obsolete" [Table 75] |
| 141 | 1 | Channel B bandwidth | flag | 0 | as above [Table 75] |
| 142–144 | 3 | Transitional zone size | NM − 1 | 4 (= 5 NM) | "calculated by adding 1 to this parameter value" [Table 75; Annex 2 §4.1.5] |
| 145–167 | 23 | Spare | — | 0 | [Table 75] |

Total 168 bits [Table 75]. Two things to get right:

* **The addressed form.** When bit 139 is 1, bits 69–98 are a 30-bit MMSI (18 bits in the "longitude" field plus 12 in the "latitude" field), bits 99–103 are five zero bits, bits 104–133 a second MMSI and 134–138 five more zeros [Table 75]. The flag that decides this sits *after* the fields it governs, so a decoder reads bit 139 first. [AIVDM "Type 22"] remarks on this too. Our output has either `ne_lon/ne_lat/sw_lon/sw_lat/region_available` or `dest_mmsi_1/dest_spare_1/dest_mmsi_2/dest_spare_2`, never both.
* **Channel numbers.** The 12-bit channel fields hold the *four-digit* numbers of ITU-R M.1084 Annex 4: the two-digit RR Appendix 18 channel, prefixed with 1 for "simplex operation of the ship station frequency" or 2 for "simplex operation of the coast station frequency" [M.1084 Annex 4 §3; M.1371-5 Annex 2 §4.1.3]. AIS 1 is channel 87B = 2087 = 161.975 MHz and AIS 2 is 88B = 2088 = 162.025 MHz; the long-range channels are 1075 and 1076 (156.775/156.825 MHz) [M.1371-5 Annex 2 §4.1.1]. Ship frequencies run 156.050 + 0.050 (n − 1) MHz for channels 1–28 and 156.025 + 0.050 (n − 60) MHz for 60–88; the coast frequency of a duplex channel is 4.6 MHz higher, and channels 6, 8–17 and 67–77 are simplex (one frequency) [M.1084 Annex 4 §1]. Our decoder adds `channel_a_mhz`/`channel_b_mhz` from that arithmetic and prints `null` for anything outside those ranges (the 12.5 kHz numbers of M.1084 Annex 4 — 2xx interleaved and 4xx narrow-band — and the 6.25 kHz 1xx/3xx numbers, none of which AIS supports: Table 75 NOTES, "narrow-band channel management operation is no longer supported").

The Tx/Rx mode note matters for load: "when the dual channel transmission is suspended by Tx/Rx mode command 1 or 2, the required reporting interval should be maintained using the remaining transmission channel" [Table 75] — so a ship told to transmit on A only puts *all* its reports on A.

### Message 23: group assignment command

[M.1371-5 §3.21 Table 76]:

| Bits | Len | Field | Type/units | Not-available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 23 [Table 76] |
| 6–7 | 2 | Repeat indicator | uint | — | [Table 76] |
| 8–37 | 30 | Source ID | MMSI | — | "MMSI of assigning station" [Table 76] |
| 38–39 | 2 | Spare | — | 0 | [Table 76] |
| 40–57 | 18 | Longitude 1 | signed, 1/10 min | — | NE corner [Table 76] |
| 58–74 | 17 | Latitude 1 | signed, 1/10 min | — | NE corner [Table 76] |
| 75–92 | 18 | Longitude 2 | signed, 1/10 min | — | SW corner [Table 76] |
| 93–109 | 17 | Latitude 2 | signed, 1/10 min | — | SW corner [Table 76] |
| 110–113 | 4 | Station type | enum | 0 | table below [Table 76] |
| 114–121 | 8 | Type of ship and cargo type | enum | 0 = all types | 1–99 per Table 53, 100–199 regional, 200–255 future [Table 76] |
| 122–143 | 22 | Spare | — | 0 | [Table 76] |
| 144–145 | 2 | Tx/Rx mode | enum | 0 | 0 = TxA/TxB, RxA/RxB; 1 = TxA, RxA/RxB; 2 = TxB, RxA/RxB; 3 = reserved [Table 76] |
| 146–149 | 4 | Reporting interval | enum | 0 | Table 77, below [Table 76] |
| 150–153 | 4 | Quiet time | minutes | 0 = none | 1–15 min [Table 76] |
| 154–159 | 6 | Spare | — | 0 | [Table 76] |

Total 160 bits, "occupies one-time period" [Table 76]. Note that Table 76 gives *no* not-available value for the four coordinates (unlike Tables 68 and 75); we apply the 181°/91° sentinels anyway for `region_available`, because they are the same 1/10-minute fields and a base station answering an interrogation with "no assignment" has nothing else to send.

**Station type** [M.1371-5 Table 76]:

| Code | Meaning |
|---|---|
| 0 | all types of mobiles (default) |
| 1 | Class A mobile stations only |
| 2 | all types of Class B mobile stations |
| 3 | SAR airborne mobile station |
| 4 | Class B "SO" mobile stations only |
| 5 | Class B "CS" shipborne mobile station only |
| 6 | inland waterways |
| 7–9 | regional use |
| 10 | base station coverage area (see messages 4 and 27) |
| 11–15 | for future use |

**Reporting interval** [M.1371-5 Table 77]:

| Code | Interval | Code | Interval |
|---|---|---|---|
| 0 | as given by the autonomous mode | 8 | 5 s |
| 1 | 10 min | 9 | next shorter reporting interval (autonomous mode only) |
| 2 | 6 min | 10 | next longer reporting interval (autonomous mode only) |
| 3 | 3 min | 11 | 2 s (not applicable to Class B "CS" and "SO") |
| 4 | 1 min | 12–15 | reserved for future use |
| 5 | 30 s | | |
| 6 | 15 s | | |
| 7 | 10 s | | |

Table 77's NOTE 1 repeats the Tx/Rx rule: with dual-channel transmission suspended, the interval is maintained on the remaining channel. A Class A station obeys an assigned interval only if the autonomous mode does not require a *shorter* one, and marks its reports as message 2 while assigned; other mobiles "should not change [their] reporting interval for changing course and speed" while assigned [M.1371-5 Annex 2 §3.3.6]. Precedence: an individually addressed message 22 beats message 23 for Tx/Rx mode, but a *regional* message 22 loses to message 23 [M.1371-5 Annex 7 §4.3.3.3.2; AIVDM "Type 23"].

### The `$AIACA` sentence

Line 9 of the data file is not an AIS message but the IEC 61162-1 *AIS channel assignment* sentence with which a transceiver reports (or is given) a regional operating setting on its serial port [IEC 61162-1 §8 ACA]:

```text
$AIACA,0,,,,,,,,,5,2087,0,2088,0,0,0,I,1,000000*15
```

Its nineteen fields are: sequence number (0); NE latitude, N/S, NE longitude, E/W, SW latitude, N/S, SW longitude, E/W (all empty — no region, i.e. the default high-seas setting); transitional zone size (5 NM); channel A (2087); channel A bandwidth (0); channel B (2088); channel B bandwidth (0); Tx/Rx mode (0); power level (0 = high); information source; in-use flag (1 = in use); UTC time of the in-use change (000000). The information source letters IEC defines are A (addressed message 22), B (broadcast message 22), C (an ACA sentence), D (DSC telecommand) and M (manual input) [IEC 61162-1 §8 ACA]; the `I` this unit prints is not one of them — presumably "internal default". Our decoder reports the sentence as `"skipped":"no parser"`; decoding it is exercise 5.

## 13.3 Decoding one message by hand

Line 11 of [`data/ch13-base-station-control.nmea`](../data/ch13-base-station-control.nmea) is `sample-data/2007hydro/norfolk-log.ais` line 22, received on 14 May 2007 at 19:21:11 UTC in Norfolk, Virginia:

```text
!AIVDM,1,1,,B,D03OviQsIN9H,0*66,1179170471.01
```

Twelve characters, no fill bits: 72 bits, the shortest legal message 20. The six-bit groups (Appendix B):

```text
D      0      3      O      v      i      Q      s      I      N      9      H
010100 000000 000011 011111 111110 110001 100001 111011 011001 011110 001001 011000
```

| Bits | Value | Field |
|---|---|---|
| 0–5 | `010100` = 20 | message ID |
| 6–7 | `00` = 0 | repeat indicator |
| 8–37 | `0000 000011 011111 111110 110001 10` = `000000001101111111111011000110` = **3 669 702** | source MMSI → 003669702: coast-station format 00 + MID 366 (USA) [M.585-9] |
| 38–39 | `00` | spare |
| 40–51 | `01 111011 0110` = `011110110110` = **1974** | offset number 1 |
| 52–55 | `01 01` = **5** | number of slots 1 (the maximum block size) |
| 56–58 | `111` = **7** | time-out 1, minutes |
| 59–69 | `0 001001 0110` = `00010010110` = **150** | increment 1 |
| 70–71 | `00` | spare (72 − 70 = 2 bits, as Table 72 allows) |

Reading: starting 1974 slots after the slot this message arrived in, five consecutive slots are reserved, and again every 150 slots (15 times per frame), for the next seven minutes — on channel B only, because that is where the message was heard. 1974 slots is 52.6 s ahead; the station is announcing *next* frame's blocks.

A second, shorter walk through the 168-bit message 22 on line 19, `F52HH<j2N2P3D73EB6`>6bT20000` (the ProTec transceiver's own-ship `!AIVDO`):

```text
F      5      2      H      H      <      j      2      N      2      P      3      D      7
010110 000101 000010 011000 011000 001100 110010 000010 011110 000010 100000 000011 010100 000111

3      E      B      6      `      >      6      b      T      2      0      0      0      0
000011 010101 010010 000110 101000 001110 000110 101010 100100 000010 000000 000000 000000 000000
```

Bits 0–5 `010110` = 22; MMSI bits 8–37 = `010100001001100001100000110011` = 338 040 883 (a US *ship* MID, not a coast station — see §13.6). Bits 40–51 `100000100111` = 2087, bits 52–63 `100000101000` = 2088: AIS 1 and AIS 2. Bits 64–67 `0000` = Tx/Rx mode 0, bit 68 `0` = high power. Bits 69–86 `011010100000111000` = 108 600 = 0x1A838 = 181° and bits 87–103 `01101010101001000` = 54 600 = 0xD548 = 91°: the NE corner is "not available"; bits 104–138 repeat the pair for the SW corner. Bit 139 `0` → broadcast (so those *were* coordinates, not MMSIs); bits 140–141 `00`; bits 142–144 `100` = 4 → a 5 NM transitional zone; bits 145–167 all zero. This is exactly the "not available and/or international default settings" that §3.20 says a base station should send when it performs no channel management [M.1371-5 §3.20].

## 13.4 The code

Four small modules per language. The new constructs are: a loop over a variable number of fixed-width groups that ends on *remaining bits* rather than a count (message 20), a nested object (`rtcm` in message 17), an optional frequency (`channel_a_mhz` may be `null`) and the conditional field set of message 22.

### Python

[`code/python/aistut/msg_20.py`](../code/python/aistut/msg_20.py):

```python
RESERVATION_BITS = 30
MAX_RESERVATIONS = 4


def decode_reservation(bits: Bits, start: int) -> dict:
    """One offset/slots/time-out/increment group (Table 72)."""
    offset = bits.uint(start, 12)
    slots = bits.uint(start + 12, 4)
    timeout = bits.uint(start + 16, 3)
    increment = bits.uint(start + 19, 11)
    return {
        "offset": offset,
        "slots": slots,
        "timeout_min": timeout,
        "increment": increment,
        # Table 72 footnote: an interrogated base station with nothing to
        # announce sends one group with offset, slots and time-out all zero.
        "available": not (offset == 0 and slots == 0 and timeout == 0),
    }


def decode(bits: Bits) -> dict:
    """Decode a message 20. Raises DecodeError below 72 bits."""
    require_bits(20, len(bits), 72)
    reservations = []
    pos = 40
    while len(reservations) < MAX_RESERVATIONS and bits.has(pos, RESERVATION_BITS):
        reservations.append(decode_reservation(bits, pos))
        pos += RESERVATION_BITS
    spare_bits = len(bits) - pos
    # The spec allows 0, 2, 4 or 6 trailing bits. We print whatever is there
    # as an integer so that odd lengths show up; anything wider than 32 bits
    # is not a spare field but a broken message and is reported as null.
    spare2 = bits.uint(pos, spare_bits) if 0 < spare_bits <= 32 else (0 if spare_bits == 0 else None)
    return {
        "msg_type": bits.uint(0, 6),
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
        "reservations": reservations,
        "spare2": spare2,
        "spare2_bits": spare_bits,
    }
```

The loop is the same shape as message 7's (chapter 12) but the group is 30 bits, not 32, so a 160-bit message leaves 0 spare bits and a 72-bit one leaves 2. We deliberately print the leftover as a number: a message 20 with 10 trailing bits (line 24 of the data file) is suspicious, and `spare2_bits: 10` says so without the decoder having to judge.

[`code/python/aistut/msg_22.py`](../code/python/aistut/msg_22.py) — the frequency helper and the decoder:

```python
# RR Appendix 18 channels that are simplex (one frequency, "ship & coast"):
# 6, 8–17 and 67–77. Everything else in 1–28 / 60–88 is a duplex pair.
SIMPLEX_CHANNELS: set[int] = {6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
                              67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77}


def channel_frequency_khz(number: int) -> int | None:
    prefix, base = divmod(number, 1000)
    if prefix not in (0, 1, 2):
        return None
    if 1 <= base <= 28:
        ship = 156050 + 50 * (base - 1)
    elif 60 <= base <= 88:
        ship = 156025 + 50 * (base - 60)
    else:
        return None
    coast = ship if base in SIMPLEX_CHANNELS else ship + 4600
    return coast if prefix == 2 else ship


def channel_mhz(number: int) -> Fixed | None:
    khz = channel_frequency_khz(number)
    return None if khz is None else Fixed(khz / 1000.0, 3)


def decode(bits: Bits) -> dict:
    """Decode a message 22. Raises DecodeError below 168 bits."""
    require_bits(22, len(bits), 168)
    channel_a = bits.uint(40, 12)
    channel_b = bits.uint(52, 12)
    txrx = bits.uint(64, 4)
    power = bits.uint(68, 1)
    addressed = bits.flag(139)
    zone = bits.uint(142, 3)
    out: dict = {
        "msg_type": bits.uint(0, 6),
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
        "channel_a": channel_a,
        "channel_a_mhz": channel_mhz(channel_a),
        "channel_b": channel_b,
        "channel_b_mhz": channel_mhz(channel_b),
        "txrx_mode": txrx,
        "txrx_mode_text": txrx_mode_text(txrx),
        "power": power,
        "power_text": POWER[power],
        "addressed": addressed,
        "band_a": bits.uint(140, 1),
        "band_b": bits.uint(141, 1),
        "zone_size": zone,
        "zone_size_nm": zone + 1,
        "spare2": bits.uint(145, 23),
    }
    if addressed:
        # Table 75: the 30-bit MMSI occupies the 18-bit longitude field and
        # the first 12 bits of the 17-bit latitude field; 5 zero bits follow.
        out["dest_mmsi_1"] = bits.uint(69, 30)
        out["dest_spare_1"] = bits.uint(99, 5)
        out["dest_mmsi_2"] = bits.uint(104, 30)
        out["dest_spare_2"] = bits.uint(134, 5)
    else:
        ne_lon = bits.sint(69, 18)
        ne_lat = bits.sint(87, 17)
        sw_lon = bits.sint(104, 18)
        sw_lat = bits.sint(122, 17)
        out["ne_lon"] = Fixed(ne_lon / 600.0, 6)
        out["ne_lat"] = Fixed(ne_lat / 600.0, 6)
        out["sw_lon"] = Fixed(sw_lon / 600.0, 6)
        out["sw_lat"] = Fixed(sw_lat / 600.0, 6)
        out["region_available"] = (ne_lon != LON_NOT_AVAILABLE and ne_lat != LAT_NOT_AVAILABLE
                                   and sw_lon != LON_NOT_AVAILABLE and sw_lat != LAT_NOT_AVAILABLE)
    return out
```

Note `addressed = bits.flag(139)` is read *before* bits 69–138 are interpreted. `txrx_mode_text` is `TXRX_MODE.get(code, "not used")` because codes 3–15 are "not used" rather than enumerated.

[`code/python/aistut/msg_17.py`](../code/python/aistut/msg_17.py) — the nested header:

```python
def decode_rtcm_header(bits: Bits, start: int) -> dict:
    """The 40-bit RTCM header of Table 69 plus whatever data words follow."""
    n_words = bits.uint(start + 32, 5)
    health = bits.uint(start + 37, 3)
    z_count = bits.uint(start + 16, 13)
    words_start = start + 40
    words_bits = len(bits) - words_start
    return {
        "message_type": bits.uint(start, 6),
        "station_id": bits.uint(start + 6, 10),
        "z_count": z_count,
        "z_count_s": Fixed(z_count * 0.6, 1),
        "sequence": bits.uint(start + 22, 3),
        "n_words": n_words,
        "health": health,
        "health_text": HEALTH[health],
        "words": bits.raw(words_start, words_bits),
        "words_bits": words_bits,
        "words_complete": words_bits == n_words * 24,
    }


def decode(bits: Bits) -> dict:
    """Decode a message 17. Raises DecodeError below 80 bits."""
    require_bits(17, len(bits), 80)
    lon_raw = bits.sint(40, 18)
    lat_raw = bits.sint(58, 17)
    data_bits = len(bits) - 80
    out: dict = {
        "msg_type": bits.uint(0, 6),
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
        "lon": Fixed(lon_raw / 600.0, 6),
        "lat": Fixed(lat_raw / 600.0, 6),
        "position_available": lon_raw != LON_NOT_AVAILABLE and lat_raw != LAT_NOT_AVAILABLE,
        "spare2": bits.uint(75, 5),
        "data": bits.raw(80, data_bits),
        "data_bits": data_bits,
    }
    # Table 69: the two-word (40-bit) header is only meaningful when complete.
    out["rtcm"] = decode_rtcm_header(bits, 80) if data_bits >= 40 else None
    return out
```

[`code/python/aistut/msg_23.py`](../code/python/aistut/msg_23.py) is a flat 160-bit decode with three enumeration tables (`STATION_TYPE`, `TXRX_MODE`, `REPORTING_INTERVAL`) and one special case:

```python
        "ship_type": ship_type,
        # Table 76 redefines 0 as "all types"; 1–255 follow Table 53.
        "ship_type_text": "all types (default)" if ship_type == 0 else ship_type_text(ship_type),
```

> **Language notes (Python).** `divmod(number, 1000)` returns the quotient and remainder as a pair — the four-digit channel number splits into prefix and base in one line. `{6, 8, 9, …}` is a *set literal* and `base in SIMPLEX_CHANNELS` a constant-time membership test. `dict.get(key, default)` and `int | None` are the chapter 1 constructs.

### C++

[`code/cpp/msg_20.hpp`](../code/cpp/msg_20.hpp):

```cpp
inline Json decode_msg_20(const Bits& bits) {
    require_bits(20, bits.size(), 72);
    Json::Array reservations;
    constexpr std::size_t FIRST_GROUP = 40;
    for (auto group : std::views::iota(FIRST_GROUP, bits.size()) | std::views::chunk(RESERVATION_BITS) |
                          std::views::take(MAX_RESERVATIONS)) {
        if (group.size() < RESERVATION_BITS) break;   // the trailing spare bits
        reservations.push_back(decode_reservation(bits, group.front()));
    }
    std::size_t pos = FIRST_GROUP + reservations.size() * RESERVATION_BITS;
    std::size_t spare_bits = bits.size() - pos;
    Json spare2 = nullptr;
    if (spare_bits == 0) {
        spare2 = 0;
    } else if (spare_bits <= 32) {
        spare2 = bits.uint(pos, spare_bits);
    }
    return Json::Object{
        {"msg_type", bits.uint(0, 6)},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
        {"reservations", Json(std::move(reservations))},
        {"spare2", spare2},
        {"spare2_bits", spare_bits},
    };
}
```

[`code/cpp/msg_22.hpp`](../code/cpp/msg_22.hpp) — the frequency helper:

```cpp
inline const std::set<int> SIMPLEX_CHANNELS = {6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
                                               67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77};

inline std::optional<int> channel_frequency_khz(int number) {
    int prefix = number / 1000;
    int base = number % 1000;
    if (prefix < 0 || prefix > 2) return std::nullopt;
    int ship;
    if (1 <= base && base <= 28) {
        ship = 156050 + 50 * (base - 1);
    } else if (60 <= base && base <= 88) {
        ship = 156025 + 50 * (base - 60);
    } else {
        return std::nullopt;
    }
    int coast = SIMPLEX_CHANNELS.contains(base) ? ship : ship + 4600;
    return prefix == 2 ? coast : ship;
}

inline Json channel_mhz(int number) {
    std::optional<int> khz = channel_frequency_khz(number);
    if (!khz) return Json(nullptr);
    return Json::fixed(static_cast<double>(*khz) / 1000.0, 3);
}
```

[`code/cpp/msg_17.hpp`](../code/cpp/msg_17.hpp) — the one line that differs in spirit from Python:

```cpp
    // Table 69: the two-word (40-bit) header is only meaningful when complete.
    out["rtcm"] = data_bits >= 40 ? decode_rtcm_header(bits, 80) : Json(nullptr);
```

[`code/cpp/msg_23.hpp`](../code/cpp/msg_23.hpp) uses three `std::array<const char*, N>` tables sized to the field widths (16, 4, 16), as chapter 15 does for aid-to-navigation (AtoN) types, and

```cpp
        {"ship_type_text", ship_type == 0 ? std::string("all types (default)") : ship_type_text(ship_type)},
```

> **Language notes (C++).**
> * `std::optional<int>` is the chapter 1 optional: `std::nullopt` empty, `!khz` tests, `*khz` reads. The `Json` class converts an empty optional to `null`, so the value could also have been stored directly.
> * `std::set<int>::contains(x)` (C++20) is "is x in the set" — before C++20 one wrote `.count(x)`, which returns 0 or 1. `std::set` is an ordered tree; for 22 small integers that is plenty fast.
> * The reservation loop is a **range pipeline** (`<ranges>`): `std::views::iota(FIRST_GROUP, bits.size())` is `range(40, len(bits))` over bit positions, `std::views::chunk(RESERVATION_BITS)` (C++23) groups them in thirties, `std::views::take(MAX_RESERVATIONS)` caps the count at four. The stages are glued with `|` and evaluated lazily as the `for` loop iterates; `group.front()` is the first position of a chunk and `group.size()` tells a full group from the trailing spare. It is Python's `while len(...) < 4 and bits.has(...)` expressed as data flow — compare the Rust version below, which keeps the `while`.
> * Both branches of `?:` must have the same type. `decode_rtcm_header(...)` returns `Json`, so the other branch is written `Json(nullptr)`; `"all types (default)"` is a `const char*`, so it is wrapped in `std::string(...)` to match `ship_type_text`'s return type.
> * `Json spare2 = nullptr; … spare2 = 0; … spare2 = bits.uint(...)` — a `Json` variable can be reassigned a value of a different JSON type; that is what the `std::variant` inside it is for (chapter 1).

### Rust

[`code/rust/src/msg_20.rs`](../code/rust/src/msg_20.rs):

```rust
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(20, bits.len(), 72)?;
    let mut reservations: Vec<Json> = Vec::new();
    let mut pos = 40;
    while reservations.len() < MAX_RESERVATIONS && bits.has(pos, RESERVATION_BITS) {
        reservations.push(Json::Obj(decode_reservation(bits, pos)));
        pos += RESERVATION_BITS;
    }
    let spare_bits = bits.len() - pos;
    let spare2: Option<u64> = match spare_bits {
        0 => Some(0),
        1..=32 => Some(bits.uint(pos, spare_bits)),
        _ => None,
    };
    let mut out = Obj::new();
    put(&mut out, "msg_type", bits.uint(0, 6));
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    put(&mut out, "reservations", reservations);
    put(&mut out, "spare2", spare2);
    put(&mut out, "spare2_bits", spare_bits);
    Ok(Json::Obj(out))
}
```

[`code/rust/src/msg_22.rs`](../code/rust/src/msg_22.rs):

```rust
pub fn is_simplex_channel(base: u64) -> bool {
    matches!(base, 6 | 8..=17 | 67..=77)
}

pub fn channel_frequency_khz(number: u64) -> Option<u64> {
    let prefix = number / 1000;
    let base = number % 1000;
    if prefix > 2 {
        return None;
    }
    let ship = match base {
        1..=28 => 156_050 + 50 * (base - 1),
        60..=88 => 156_025 + 50 * (base - 60),
        _ => return None,
    };
    let coast = if is_simplex_channel(base) { ship } else { ship + 4600 };
    Some(if prefix == 2 { coast } else { ship })
}

pub fn channel_mhz(number: u64) -> Json {
    match channel_frequency_khz(number) {
        Some(khz) => Json::fixed(khz as f64 / 1000.0, 3),
        None => Json::Null,
    }
}
```

[`code/rust/src/msg_17.rs`](../code/rust/src/msg_17.rs):

```rust
    let rtcm = if data_bits >= 40 { Some(decode_rtcm_header(bits, 80)) } else { None };
    put(&mut out, "rtcm", rtcm);
```

[`code/rust/src/msg_23.rs`](../code/rust/src/msg_23.rs) — one of the three `match` tables:

```rust
pub fn station_type_text(code: u64) -> &'static str {
    match code {
        0 => "all types of mobiles (default)",
        1 => "Class A mobile stations only",
        2 => "all types of Class B mobile stations",
        3 => "SAR airborne mobile station",
        4 => "Class B SO mobile stations only",
        5 => "Class B CS shipborne mobile station only",
        6 => "inland waterways",
        7..=9 => "regional use",
        10 => "base station coverage area",
        _ => "for future use", // 11–15
    }
}
```

> **Language notes (Rust).**
> * `matches!(value, pattern)` is a macro that is `true` when the value fits the pattern; `6 | 8..=17 | 67..=77` is an *or-pattern* of a literal and two inclusive ranges. It replaces a `HashSet` for a fixed set of numbers and costs nothing at run time.
> * `let ship = match base { … _ => return None, };` — a `match` arm may `return` from the whole function; its type is then the *never* type and does not have to agree with the other arms. This is how early exits are written inside an expression.
> * `Option<u64>` and `Option<Obj>` are both accepted by `put` because jsonout.rs implements `From<Option<T>>` for any `T` that converts to `Json`; `None` becomes `null`. `?` is only for `Result`, so inside `channel_frequency_khz` (which returns `Option`) we write `return None`.
> * `1..=32 =>` and `7..=9 =>` are *range patterns* in `match`. The compiler still requires a `_` arm because it does not know that a `u64` from a 4-bit field is at most 15.
> * `156_050` — underscores in numeric literals are ignored and only aid reading.

### TypeScript

[`code/typescript/src/msg_20.ts`](../code/typescript/src/msg_20.ts):

```typescript
export function decode(bits: Bits): JsonObject {
  requireBits(20, bits.length, 72);
  const reservations: JsonObject[] = [];
  let pos = 40;
  while (reservations.length < MAX_RESERVATIONS && bits.has(pos, RESERVATION_BITS)) {
    reservations.push(decodeReservation(bits, pos));
    pos += RESERVATION_BITS;
  }
  const spareBits = bits.length - pos;
  let spare2: number | null = null;
  if (spareBits === 0) {
    spare2 = 0;
  } else if (spareBits <= 32) {
    spare2 = bits.uint(pos, spareBits);
  }
  return {
    msg_type: bits.uint(0, 6),
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
    reservations,
    spare2,
    spare2_bits: spareBits,
  };
}
```

[`code/typescript/src/msg_22.ts`](../code/typescript/src/msg_22.ts):

```typescript
export const SIMPLEX_CHANNELS = new Set<number>([
  6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
  67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77,
]);

export function channelFrequencyKhz(num: number): number | null {
  const prefix = Math.floor(num / 1000);
  const base = num % 1000;
  if (prefix < 0 || prefix > 2) {
    return null;
  }
  let ship: number;
  if (1 <= base && base <= 28) {
    ship = 156050 + 50 * (base - 1);
  } else if (60 <= base && base <= 88) {
    ship = 156025 + 50 * (base - 60);
  } else {
    return null;
  }
  const coast = SIMPLEX_CHANNELS.has(base) ? ship : ship + 4600;
  return prefix === 2 ? coast : ship;
}

export function channelMhz(num: number): Fixed | null {
  const khz = channelFrequencyKhz(num);
  return khz === null ? null : new Fixed(khz / 1000.0, 3);
}
```

[`code/typescript/src/msg_17.ts`](../code/typescript/src/msg_17.ts):

```typescript
  // Table 69: the two-word (40-bit) header is only meaningful when complete.
  out["rtcm"] = dataBits >= 40 ? decodeRtcmHeader(bits, 80) : null;
```

[`code/typescript/src/msg_23.ts`](../code/typescript/src/msg_23.ts):

```typescript
    // Table 76 redefines 0 as "all types"; 1–255 follow Table 53.
    ship_type_text: shipType === 0 ? "all types (default)" : shipTypeText(shipType),
```

> **Language notes (TypeScript).**
> * `new Set<number>([...])` with `.has(x)` is the JavaScript set; `Math.floor(num / 1000)` and `num % 1000` replace Python's `divmod` because `/` on numbers is always floating-point division.
> * `number | null` as a return type (and `let spare2: number | null = null`) is the union type of chapter 1; `khz === null ? null : …` is the standard shape.
> * `let ship: number;` declares without initialising; TypeScript's *definite assignment analysis* (chapter 11) checks that every path assigns `ship` before it is read (the `else` branch returns, so that path never reads it).
> * A nested object (`out["rtcm"] = decodeRtcmHeader(...)`) needs nothing special: `JsonObject` values may themselves be `JsonObject`s, and the sorted stringifier recurses.

### Where the four differ

| Concern | Python | C++ | Rust | TypeScript |
|---|---|---|---|---|
| Set of simplex channels | set literal + `in` | `std::set::count` | `matches!` with range patterns | `Set.has` |
| Optional frequency | `int \| None`, `None` | `std::optional<int>`, `nullopt` | `Option<u64>`, `None` | `number \| null` |
| Nested `rtcm` or null | dict or `None` | `Json(…)` vs `Json(nullptr)` | `Option<Obj>` via `From` | `JsonObject \| null` |
| Leftover bits as number | `bits.uint` of any width | `std::uint64_t` (≤ 32 bits used) | `u64` | `number` (safe ≤ 53 bits) |
| Enumeration with gaps (txrx 3–15) | `dict.get(code, default)` | `switch` with `default` | `match` with `_` | `code in TABLE ? … : default` |

## 13.5 Sample data and results

[`data/ch13-base-station-control.nmea`](../data/ch13-base-station-control.nmea) has 42 lines: one `$AIACA`, thirteen recorded AIS lines and six synthetic ones. Sources:

| Data line | Source | What it is |
|---|---|---|
| 9 | `sample-data/2007hydro/norfolk-log.ais` line 1 | `$AIACA` channel-assignment status |
| 11–14 | `norfolk-log.ais` lines 22, 51, 79, 98 | message 20 from USCG base station 003669702, every 10 s, channels B, A, B, A |
| 16 | `sample-data/2007hydro/kurts_log_BGsRoomOvernightsheraton_norfolk.ais` line 16 | the same station two days later |
| 18–19 | `norfolk-log.ais` lines 113, 115 | `!AIVDO` message 20 and 22 from the L-3 ProTec bench transceiver, MMSI 338040883 |
| 21–22 | `sample-data/2007hydro/sr162-noaa-norfolk-telcom2007-05-14` lines 402, 507 | the same two payloads received over the air by an SR162 |
| 24 | `sample-data/2007hydro/protec-util12007-05-14` line 1817 | an 80-bit message 20 — one block plus 10 bits |
| 26 | `protec-util12007-05-14` line 3656 | message 20 whose MMSI characters arrived damaged |
| 28 | `sample-data/tide1/ais/ais-2009-03-11` line 210 | 168-bit "message 17" from the `nhjel` receiver |
| 30 | `sample-data/RightwhaleAisProject/20081013.log` line 393089 | 48-bit "message 23" |
| 32–42 | synthetic | one 17, one two-block 20, a broadcast and an addressed 22, two 23 |

**What the recordings contain.** A scan of every file under `sample-data/` (≈ 650 MB, 2006–2011, six receivers) for first-six-bits 17, 20, 22 or 23, validated by length only:

* **Message 20:** 11 202 structurally valid lines. 10 881 of them are from the USCG base station 003669702 at Norfolk, all 72 bits, all "5 slots, time-out 7 min, increment 150" with the offset varying as explained in §13.2; one from the same station is 80 bits (data line 24); 310 are the ProTec bench unit's all-zero "nothing to announce" message; one is the damaged copy on data line 26. The remaining nine, from the `nhjel` receiver and the N-AIS feed, have ship or nonsense MMSIs, block sizes of 9–15 slots, increments not in the recommended list and non-zero trailing bits — noise. **No genuine multi-block message 20 was recorded**, which is why data line 34 is synthetic.
* **Message 22:** 346 valid 168-bit lines, *all* the identical default payload of the ProTec unit (241 as `!AIVDO` on its own utility port, 95 heard by the NOAA-side SR162, 10 in `norfolk-log.ais`). No regional or addressed message 22 was recorded; lines 36 and 38 are synthetic.
* **Message 17:** 33 lines of plausible length (80 + 8*k*). Every one fails validation: MMSIs that are not coast stations, reference-station positions like 95° S or 218° W, N > 29 or N inconsistent with the bits present (`words_complete` is false for all 33). Line 28 is typical: MMSI 206049936, 41.93° N **173.12° W**, "N = 22" with 48 word bits. There is no message 17 in the recordings; line 32 is synthetic.
* **Message 23:** 15 lines labelled 23, none of them 160 bits long. Line 30 (48 bits) is one of two in the N-AIS feed. Lines 40 and 42 are synthetic.

**How the synthetic lines were built.** Field values were chosen, written as bit strings of the Table widths, concatenated, padded to a multiple of six with fill bits, armoured (add 48, add 8 more above 87 — the inverse of chapter 2's table) and given an NMEA checksum, exactly as in chapters 10 and 11. All use the real USCG MMSI 003669702 so that they could sit alongside its message 4s, and a rectangle over Hampton Roads: NE 37°00.0′ N 076°00.0′ W, SW 36°40.0′ N 076°30.0′ W (20 NM × 24 NM, the minimum side Annex 2 §4.1.5 allows). The message 17 reference station is at 36°58.5′ N 076°20.0′ W with RTCM type 1, station 100, Z count 1234 (= 740.4 s), sequence 5, N = 2, health 0 and the words 0xAAAAAA, 0x555555 — visibly a test pattern, not a correction. The data-file comments list the remaining values.

Run any decoder:

```sh
python3 code/python/decode.py data/ch13-base-station-control.nmea
code/cpp/build/decode data/ch13-base-station-control.nmea
code/rust/target/release/decode data/ch13-base-station-control.nmea
node code/typescript/dist/decode.js data/ch13-base-station-control.nmea
```

All four print the twenty lines of [`tests/expected/ch13-base-station-control.jsonl`](../tests/expected/ch13-base-station-control.jsonl). Selected objects (`"ais"` part only unless noted):

**Line 9 — the ACA sentence** is reported whole:

```json
{"address":"AIACA","line":9,"skipped":"no parser","uscg":{"time":1179170462.74}}
```

**Line 11 — the hand-decoded message 20:**

```json
{"mmsi":3669702,"msg_type":20,"repeat_indicator":0,"reservations":[{"available":true,"increment":150,"offset":1974,"slots":5,"timeout_min":7}],"spare":0,"spare2":0,"spare2_bits":2}
```

Lines 12–14 and 16 are the same except for `offset` 1624, 1224, 874 and 474 and the `channel` (A, B, A, B in the outer object). Compare the time stamps in the `uscg` object: 1179170471.01, 481.02, 491.01, 501.01 — ten seconds apart to the hundredth.

**Line 18 — the ProTec's own message 20** (`"own_ship":true` in the outer object):

```json
{"mmsi":338040883,"msg_type":20,"repeat_indicator":0,"reservations":[{"available":false,"increment":0,"offset":0,"slots":0,"timeout_min":0}],"spare":0,"spare2":0,"spare2_bits":2}
```

The Table 72 footnote case: one all-zero block, `available: false`. Line 22 is the identical payload heard by a different receiver 40 minutes later.

**Line 19 — the ProTec's message 22,** decoded by hand in §13.3:

```json
{"addressed":false,"band_a":0,"band_b":0,"channel_a":2087,"channel_a_mhz":161.975,"channel_b":2088,"channel_b_mhz":162.025,"mmsi":338040883,"msg_type":22,"ne_lat":91.000000,"ne_lon":181.000000,"power":0,"power_text":"high","region_available":false,"repeat_indicator":0,"spare":0,"spare2":0,"sw_lat":91.000000,"sw_lon":181.000000,"txrx_mode":0,"txrx_mode_text":"TxA/TxB, RxA/RxB","zone_size":4,"zone_size_nm":5}
```

**Line 24 — the 80-bit message 20:**

```json
{"mmsi":3669702,"msg_type":20,"repeat_indicator":0,"reservations":[{"available":true,"increment":1692,"offset":300,"slots":4,"timeout_min":1}],"spare":0,"spare2":300,"spare2_bits":10}
```

`payload_bits` is 80. Ten trailing bits with the value 300, a block of 4 instead of the station's usual 5, a time-out of 1 and an increment of 1692 (not in the recommended list, and 2250/1692 is not an integer) — every field says this is a corrupted copy of the station's normal message, received by a transceiver with no antenna connected.

**Line 26** decodes cleanly to `"mmsi":16070` with the familiar `offset` 1224 / 5 / 7 / 150: the payload is `D000viQ<QN9H` where line 13 has `D03OviQ<QN9H`. The checksum is correct for the damaged text, so the error occurred before the sentence was built.

**Line 28 — the "message 17":**

```json
{"data":"0010010010001001110011100101000010110011100110000100100110101110000010101000010010101111","data_bits":88,"lat":41.930000,"lon":-173.118333,"mmsi":206049936,"msg_type":17,"position_available":true,"repeat_indicator":0,"rtcm":{"health":3,"health_text":"UDRE scale factor 0.3","message_type":9,"n_words":22,"sequence":4,"station_id":137,"words":"100110000100100110101110000010101000010010101111","words_bits":48,"words_complete":false,"z_count":6602,"z_count_s":3961.2},"spare":1,"spare2":2}
```

Structurally fine (80 + 88 bits), semantically absurd: `n_words` 22 would need 528 bits, 48 are present; the MMSI is a ship's; the station is in the Pacific; both spare fields are non-zero. `words_complete: false` is the machine-readable flag.

**Line 30** is `{"error":"message 23 too short: 48 bits < 160", …}` with `"msg_type":23` and `"payload_bits":48` in the outer object.

**Line 32 — synthetic message 17:**

```json
{"data":"0000010001100100001001101001010100010000101010101010101010101010010101010101010101010101","data_bits":88,"lat":36.975000,"lon":-76.333333,"mmsi":3669702,"msg_type":17,"position_available":true,"repeat_indicator":0,"rtcm":{"health":0,"health_text":"UDRE scale factor 1.0","message_type":1,"n_words":2,"sequence":5,"station_id":100,"words":"101010101010101010101010010101010101010101010101","words_bits":48,"words_complete":true,"z_count":1234,"z_count_s":740.4},"spare":0,"spare2":0}
```

**Line 34 — synthetic two-block message 20** (`payload_bits` 104):

```json
{"mmsi":3669702,"msg_type":20,"repeat_indicator":0,"reservations":[{"available":true,"increment":150,"offset":24,"slots":5,"timeout_min":7},{"available":true,"increment":1125,"offset":1149,"slots":2,"timeout_min":3}],"spare":0,"spare2":0,"spare2_bits":4}
```

40 + 2 × 30 = 100 bits plus 4 spare bits to reach 104, a byte boundary [Table 72].

**Line 36 — synthetic broadcast message 22:**

```json
{"addressed":false,"band_a":0,"band_b":0,"channel_a":2087,"channel_a_mhz":161.975,"channel_b":2088,"channel_b_mhz":162.025,"mmsi":3669702,"msg_type":22,"ne_lat":37.000000,"ne_lon":-76.000000,"power":1,"power_text":"low","region_available":true,"repeat_indicator":0,"spare":0,"spare2":0,"sw_lat":36.666667,"sw_lon":-76.500000,"txrx_mode":1,"txrx_mode_text":"TxA, RxA/RxB","zone_size":2,"zone_size_nm":3}
```

36°40.0′ prints as 36.666667 — six decimals of a value whose true resolution is 1/600°.

**Line 38 — synthetic addressed message 22:**

```json
{"addressed":true,"band_a":0,"band_b":0,"channel_a":2027,"channel_a_mhz":161.950,"channel_b":2028,"channel_b_mhz":162.000,"dest_mmsi_1":366999001,"dest_mmsi_2":366999002,"dest_spare_1":0,"dest_spare_2":0,"mmsi":3669702,"msg_type":22,"power":0,"power_text":"high","repeat_indicator":0,"spare":0,"spare2":0,"txrx_mode":0,"txrx_mode_text":"TxA/TxB, RxA/RxB","zone_size":4,"zone_size_nm":5}
```

No `ne_lon`…`region_available` keys at all. Channels 2027/2028 are the coast-station frequencies of duplex channels 27 and 28 — 157.350 + 4.6 = 161.950 MHz and 162.000 MHz [M.1084 Annex 4] — the pair just below AIS 1/AIS 2.

**Lines 40 and 42 — synthetic message 23:**

```json
{"mmsi":3669702,"msg_type":23,"ne_lat":37.000000,"ne_lon":-76.000000,"quiet_time_min":0,"region_available":true,"repeat_indicator":0,"reporting_interval":1,"reporting_interval_text":"10 min","ship_type":0,"ship_type_text":"all types (default)","spare":0,"spare2":0,"spare3":0,"station_type":2,"station_type_text":"all types of Class B mobile stations","sw_lat":36.666667,"sw_lon":-76.500000,"txrx_mode":0,"txrx_mode_text":"TxA/TxB, RxA/RxB"}
```

"All Class B stations in the rectangle: report every 10 minutes." Line 42 selects `station_type` 0 and `ship_type` 37 (`"pleasure craft"`), commands `txrx_mode` 1 (`"TxA, RxA/RxB"`), `reporting_interval` 0 and `quiet_time_min` 3: "pleasure craft of any class: be silent for three minutes, then transmit on A only." Footnote 19 to Annex 7 §4.3.3.3.2 explains why the first command is nearly useless unless repeated: a 10-minute interval never gets established before the 4–8 minute assignment time-out.

## 13.6 Pitfalls and real-world deviations

* **A mobile station sending base-station messages.** The ProTec transceiver (MMSI 338040883, a US *ship* MID, not the 00MIDxxxx form of a coast station [M.585-9]) transmitted messages 20 and 22, which Table 46 reserves for base stations, 346 times in one afternoon — and a second receiver across town heard them. It was a bench unit with no antenna and no GPS (its alarm log says so: `Antenna VSWR exceeds limit`, `AIS: UTC clock lost`) in some test or base-station emulation mode. A compliant ship would have taken its message 22 as a channel command from an unknown "base station" — harmless here only because the content was the default. This is the rogue-command case of §13.1, in the wild, by accident.
* **Offsets are relative.** Expect every message 20 from a station to carry a different offset even though the reservation never changes (§13.2). Software that keys reservations on `offset` will see thousands of distinct reservations; key on the *absolute* slot (receive slot + offset, modulo 2250, modulo increment) instead. You need the receive slot, which `!AIVDM` does not give you — a message 4 from the same station gives you the slot number of *its* transmission [M.1371-5 Table 51 (communication state)], or a receiver with UTC-stamped slots.
* **One message 20 per channel.** The reservation applies "only to the frequency channel in which it is transmitted" [§3.18]. The USCG station's channel A and channel B patterns differ (offsets ≡ 124 vs ≡ 24 mod 150). Logs that drop the channel field make this invisible.
* **Length is not validity.** All 33 "message 17s" and all nine odd "message 20s" in the recordings pass the length test. Check in addition: coast-station MMSI; for 17, `rtcm.n_words ≤ 29` and `words_complete`; for 20, `slots ≤ 5`, time-out > 0 unless all-zero, increment in the recommended list, trailing bits zero and at most 6; for 22, `txrx_mode ≤ 2`, both bandwidth bits 0, channel numbers with prefix 0/1/2 and base 1–28 or 60–88; for 23, `station_type ≠ 11–15`, `reporting_interval ≤ 11`, `txrx_mode ≠ 3`, spares zero, region sides 20–200 NM; and for all of them, a region or reference station within a few hundred miles of the receiver.
* **Power bit polarity.** M.1371-5 Table 75 says 0 = high (default), 1 = low. [AIVDM "Type 22"] prints the opposite ("Low=0, high=1"). Trust the ITU table; the bench unit's default `power: 0` → `"high"` is consistent with it.
* **Station type tables moved between editions.** [AIVDM "Type 23"] lists code 1 as "reserved" and 4 as "Aid to Navigation station" (an M.1371-3 reading); M.1371-5 Table 76 makes 1 "Class A mobile stations only" and 4 "Class B SO mobile stations only", and adds 10 = base station coverage area. Decoders built from older documentation disagree with ours on those codes.
* **The bandwidth bits are dead.** M.1371-1 allowed 12.5 kHz channels; M.1371-5 calls bit 140/141 "spare (formerly 12.5 kHz bandwidth … now obsolete)" and Annex 2 §4.1.5.1 says a competent authority "should not assign narrow bandwidths" [M.1371-5 Table 75; Annex 2 §4.1.5.1]. A 1 here is a sign of an old or confused sender.
* **Message 23's region has no sentinel.** Table 76 defines no not-available value, so a base station answering an interrogation (message 15 asking for 23) with "no assignment" has no standard way to say so. We report `region_available` using the Table 75 sentinels; a decoder that does not will print a rectangle at 181° E.
* **Message 17 words have no parity.** The AIS form strips the 6-bit RTCM parity from each 30-bit word, leaving 24 [Table 69]. If you want to feed the words to a GNSS receiver you must regenerate the parity and preamble; the words are also *not* byte-aligned in the original RTCM framing, so do not expect `words` to split into bytes meaningfully.
* **Our decoder is lenient about length by design.** Message 17 with 1–39 data bits gets `rtcm: null`; message 20 with 73–159 bits is decoded as far as whole blocks go; message 22/23 longer than 168/160 bits decode the first 168/160 and leave `payload_bits` to tell you. The strict checks above belong in a validation layer, not in the field reader.

## 13.7 Exercises

1. **(Any language)** For the five USCG message 20s on data lines 11–16, compute `offset mod increment` and group by channel. Confirm the two patterns (≡ 24 and ≡ 124 mod 150). Then compute how many slots per frame the station reserves on each channel and what fraction of 2 250 that is.
2. **(Python)** Write `validate_msg_20(obj) -> list[str]` returning the Table 72 / §3.18 rule violations of a decoded message 20 (block > 5 slots, increment not recommended, trailing bits non-zero or > 6, MMSI not of the form 00MIDxxxx). Run it over `tests/expected/ch13-base-station-control.jsonl`; lines 24 and 26 should each produce at least one finding and lines 11–16 none.
3. **(C++)** Extend `channel_frequency_khz` with a `channel_text` function that returns strings like `"AIS 1 (87B, 161.975 MHz)"`, `"AIS 2 (88B, 162.025 MHz)"`, `"ch 75 (156.775 MHz, long-range)"`, or `"ch 27 coast (161.950 MHz)"`. Use [M.1371-5 Annex 2 §4.1.1] for the four named channels. Keep the JSON output unchanged (do not add the key) so the tests still pass, or add it in all four languages and regenerate the expected file.
4. **(Rust)** Implement the point-in-rectangle test of Annex 2 §4.1.4 for a decoded message 22 or 23: `fn in_region(obj: &Obj, lat: f64, lon: f64) -> Option<bool>` returning `None` when `region_available` is false or the message is addressed. Then add the transitional-zone test: is the point within `zone_size_nm` of a boundary? (1 NM = 1/60° of latitude; scale longitude by cos(latitude).)
5. **(TypeScript)** Write a parser for the `$--ACA` sentence of §13.2 that produces an object with the same field names as our message 22 output (`channel_a`, `channel_b`, `txrx_mode`, `power`, `zone_size_nm`, `ne_lat` … or `null`, plus `source` and `in_use`). Decode data line 9. Which of its values would you expect to see in a message 22 broadcast by a base station, and which are local to the transceiver?
6. **(Any language, harder)** Build the rogue-command detector of §13.1: a filter over a decoded stream that flags any message 17/20/22/23 whose source MMSI is not a coast station, that has not been preceded within the last 10 minutes by a message 4 from the same MMSI, or whose region fails the 20–200 NM rule. Feed it the data file and, if you have it, `sample-data/2007hydro/protec-util12007-05-14`. How many distinct "base stations" does it find, and how many are real?

## 13.8 References

* [M.1371-5] Annex 8 §3.15 and Tables 68–69 (message 17); §3.18 and Table 72 (message 20); §3.20 and Table 75 (message 22); §3.21 and Tables 76–77 (message 23); Table 46 (priorities, access schemes, M/B); Table 51 (message 4 slot number); Table 53 (ship types). Annex 2 §2.12 (power levels), §3.1.2 (frame of 2 250 slots), §3.3.4.3 (FATDMA and the 120 NM rule), §3.3.6 (assigned operation), §4.1 (multi-channel operation: §4.1.1 channels, §4.1.3 regional frequencies, §4.1.4 regional areas, §4.1.5 transitional mode and region size, §4.1.6 manual input, §4.1.8 priority and clearing), §4.4.1 (slot reuse and base-station slots), §4.5 (base station operation). Annex 7 §2.1.2.2 and §4.3.3.3.2 (Class B "CS" assigned mode and quiet time).
* [M.1084] Annex 4 §1 (RR Appendix 18 channel frequencies and simplex/duplex), §3 (four-digit numbering of simplex use of duplex channels).
* [M.823] reference-station health codes carried in message 17 (RTCM SC-104 version 2 format).
* [M.585-9] coast-station MMSI format 00MIDxxxx.
* [MSC.140(76)] protection of the AIS VHF data link.
* [IEC 61162-1] §8, the `ACA` sentence (AIS channel assignment message).
* [AIVDM] "Type 17", "Type 20", "Type 22", "Type 23" — note the differences recorded in §13.6.
* [IALA G1082] for the context of base-station networks.
