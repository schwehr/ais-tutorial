# 11. Messages 10, 15 and 16: UTC inquiry, interrogation and assignment

> **What you will learn**
> * The three messages with which one AIS station tells another what to do: *ask me the time* (10), *send me message X* (15) and *transmit on this schedule* (16) — and which message answers each (11; one of 3/4/5/9/18/21/24 according to Table 65; message 2 or the assigned-mode flag).
> * The 72-bit layout of message 10 [M.1371-5 Annex 8 §3.8 Table 60], the 88/110/112/160-bit shapes of message 15 [§3.13 Table 66] and the 96/144-bit shapes of message 16 [§3.14 Table 67], including the famous byte-alignment slip in the two-request interrogation.
> * How the same 22 bits of message 16 mean "so many reports per ten minutes" when the increment is zero and "this slot, then every *n* slots" when it is not [Table 67 note 1].
> * What the recordings show: a ship at anchor polling its neighbours for UTC and getting answers in a quarter of a second; a Class A transponder that interrogates every new MMSI it hears for its static data, including MMSIs that cannot exist; a USCG base station polling for "message 0"; and no valid message 16 at all.
> * Language notes: building a JSON array from a variable number of repeated groups with a running bit offset (the same pattern chapter 12 uses for message 7), and optional values that print as `null`.

## 11.1 Where this fits

Most AIS traffic is *autonomous*: every station broadcasts on its own schedule and nobody asks for anything. M.1371 nevertheless defines three ways for a station to address a request or an order to a specific other station, and this chapter covers all three because they share a design — a header, a destination MMSI (maritime mobile service identity) and a few parameters — and because a decoder that handles one handles the others.

| Message | Name | Bits | Sent by | Answered by |
|---|---|---|---|---|
| 10 | UTC and date inquiry | 72 | any station that needs the time | message 11 (chapter 6) from the addressed station, on the same channel [M.1371-5 Annex 8 §3.2] |
| 15 | Interrogation | 88–160 | Class A, SAR aircraft, base station (not Class B, not AtoN) | the requested message type(s) — 3, 5, 24 from a Class A; 18, 24 from Class B; 9, 24 from a SAR aircraft; 21 from an aid to navigation (AtoN); 4, 24 from a base station [Table 65] |
| 16 | Assignment mode command | 96 or 144 | a base station "when operating as a controlling entity" | nothing explicit: the mobile changes its schedule and raises its *assigned mode flag* (or, for Class A, switches from message 1 to message 2) [§3.14; Annex 2 §3.3.6] |

Sources: [M.1371-5 Annex 8 Table 46] (priority 3 for 10, 11 and 15, priority 1 for 16; all three are sent with RATDMA or FATDMA access — 10 and 15 also ITDMA — never in a scheduled slot of their own; the acronyms are the random, fixed and incremental access TDMA schemes of chapter 3), [Table 60], [Table 65], [Table 66], [Table 67].

**Message 10 → 11.** A mobile station's clock comes from its own GNSS receiver. If that fails, the station can still keep its slots synchronised by listening to others (chapter 3's *sync state* 1–3), and it can ask a specific neighbour for the full UTC date and time with message 10; the neighbour replies with message 11, which is a message 4 with a different type number [M.1371-5 Annex 8 §3.2, §3.9]. "For interrogation of UTC and date, message identifier 10 should be used" — not message 15 [Table 46 note 4]. The reply is "transmitted on the channel where the UTC request message was received" [§3.2].

**Message 15 → the requested message.** "This message should be used for interrogations via the TDMA (not DSC) VHF data link except for requests for UTC and date" [§3.13]. Up to two stations and up to three message types per interrogation; Table 65 is the matrix of who may ask whom for what, and the interrogated station "should automatically respond … on the channel where the interrogation was received" [Annex 2 §3.3.2.3; Annex 7 §4.3.3.3.3]. A base station may additionally say *in which slot* to answer (the slot offset); a mobile interrogator must leave the offset at zero and let the responder pick a slot [Table 66 notes]. Message 3 exists for exactly this: it is the "special position report, response to interrogation" of a Class A [Table 46], which is why chapter 3 showed message 3 carrying ITDMA state.

**Message 16 → assigned mode.** "Assignment should be transmitted by a base station when operating as a controlling entity. Other stations can be assigned a transmission schedule, other than the currently used one. If a station is assigned a schedule, it will also enter assigned mode" [§3.14]. Two kinds of assignment share one pair of fields: a *reporting rate* (so many reports per 10 minutes) or *exact slots* (an offset to the first slot and an increment between slots). The assignment times out "randomly selected between 4 and 8 min after the first transmission" unless renewed [§3.14], and the Class A keeps whichever is *faster* of the assigned rate and the rate its own speed and course would demand autonomously [§3.14; Annex 2 §3.3.6]. Message 23 (chapter 13) is the group version — the same idea addressed to everyone in a region.

### Operational use

* **UTC inquiry (10/11)** is invisible to operators. It happens between transponders when one has lost its time source; a bridge display shows nothing. The only operational trace is the reply's effect: a ship whose message 11 reports a time well off the receiver's clock has a clock problem (chapter 6 §6.6 audits exactly that).
* **Interrogation (15)** is a vessel traffic service (VTS) tool. A port's base station can poll a ship for its message 5 the moment it enters coverage instead of waiting up to six minutes for the scheduled broadcast; it can ask a Class B for message 24 to get the name of an unknown target; it can request a position report (3/18/9) from a specific vessel on demand — for instance to check whether a target that stopped reporting is still there. On a Class A ship the operator normally cannot send an interrogation by hand, but the transponder may do so by itself (§11.6: the L-3 ProTec at the 2007 Norfolk conference asked every new MMSI for its message 5).
* **Assignment (16)** is a traffic-management tool for a competent authority: a base station can raise the reporting rate of ships in a narrow channel to one report every two seconds, lower the rate of anchored ships to relieve a congested link, or move a station onto fixed slots that the base station has reserved with a message 20 so that its reports never collide [§3.14; Annex 2 §3.3.6.2]. The mobile shows it has been assigned — Class A by sending message 2 instead of 1, Class B "SO" and SAR aircraft by setting their assigned-mode flag [Annex 2 §3.3.6] — and reverts after 4–8 minutes unless re-assigned.

### Unattended / automated uses

* **Polling static data on demand.** A shore system that sees a new MMSI in a position report can issue a message 15 for message 5 (or 24 for a Class B) immediately and have the name, dimensions and destination within seconds. That is the designed machine use; Table 65 even allows a base station to interrogate another base station for its message 4.
* **Automatic interrogation by transponders.** The 2007 logs show a Class A transponder that interrogates stations for message 5, most of them within a sentence or two of first hearing them (§11.5, §11.6). Multiply that by every ship in a busy harbour and the interrogations themselves become a measurable fraction of the link; and because the transponder interrogates whatever MMSI it decodes, including MMSIs corrupted by bit errors, a share of those slots is spent asking stations that do not exist. If you build such a feature, rate-limit it and validate the MMSI first.
* **UTC resynchronisation monitoring.** Counting message 10s per MMSI in a feed identifies ships whose GNSS time is flaky: chapter 6 found one ship asking 26 times in three days; this chapter's ship asked 46 times in two days. Both were at anchor — an anchored ship with a bad antenna view, or a transponder whose GNSS is simply unreliable, is the typical case.
* **Forcing reporting rates.** A base-station network can hold every ship in a fairway at a 2-second rate for a traffic image of radar quality, or slow down ships at anchor to free slots — and, by assigning slots, de-conflict a crowded link. In practice very few administrations have used message 16; [AIVDM "Type 16"] notes that the 144-bit form "has not been observed even in long-duration samples", and our recordings contain no valid message 16 at all (§11.6).
* **Security: interrogation and assignment as denial-of-service tools.** None of these messages is authenticated. Anyone with a software-defined radio can:
  * send message 15s to every ship in range as fast as the link allows and have each of them answer with a 2-slot message 5 — the victims generate the congestion;
  * send message 16s assigning absurd reporting rates (600 reports per 10 minutes is the legal maximum [§3.14] — one report per second per ship), or assigning all ships to the *same* slots so that their reports collide; a mobile station has no way to authenticate the sender of a message 16, and each assignment lasts up to 8 minutes before it must be renewed;
  * send message 16s with increment 7, which the standard says the mobile "should disregard" — a cheap probe of which transponders implement the rule.
  IMO's MSC.140(76) recognises that "any disruption to those channels by any one AIS device could affect the operation of all AIS devices on that network" and asks administrations "to ensure the integrity of the radio channels used for AIS in their waters" [MSC.140(76)]. For a data consumer the practical defence is detection: message 15 or 16 from an MMSI that is not a base station (`00MIDxxxx` [M.585-9]), or from a base station that is not in the authority's list, or at a rate no VTS would use, is an event worth alarming on. The USCG safety alerts on AIS misuse are about static data rather than link control [Safety Alerts], but the enforcement route is the same.
* **Data mining the control plane.** Interrogations and assignments are rare, so a log of them is a compact record of what a VTS was doing: which ships it polled and when, which it rate-assigned, how it responded to incidents. Analysts who want to understand an authority's procedures read messages 15/16/20/22/23 rather than the position reports.

## 11.2 The specification

### Message 10: UTC and date inquiry

Fixed 72 bits [M.1371-5 Annex 8 §3.8 Table 60]:

| Bits | Len | Field | Type/units | Not-available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 10 [Table 60] |
| 6–7 | 2 | Repeat indicator | uint | — | 0 default, 3 = do not repeat any more [Table 60] |
| 8–37 | 30 | Source ID | MMSI | — | "MMSI number of station which inquires UTC" [Table 60] |
| 38–39 | 2 | Spare | — | 0 | [Table 60] |
| 40–69 | 30 | Destination ID | MMSI | — | "MMSI number of station which is inquired" [Table 60] |
| 70–71 | 2 | Spare | — | 0 | [Table 60] |

Nothing else: no slot offset, no channel, no reason. The response, message 11, is described in chapter 6 (it is Table 51 with message ID 11); "a mobile station should output Message 11 only in response to interrogation by Message 10" [§3.2].

### Message 15: interrogation

Variable, 88 to 160 bits [M.1371-5 Annex 8 §3.13 Table 66]:

| Bits | Len | Field | Type/units | Not-available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 15 [Table 66] |
| 6–7 | 2 | Repeat indicator | uint | — | [Table 66] |
| 8–37 | 30 | Source ID | MMSI | — | interrogating station [Table 66] |
| 38–39 | 2 | Spare | — | 0 | [Table 66] |
| 40–69 | 30 | Destination ID 1 | MMSI | — | first interrogated station [Table 66] |
| 70–75 | 6 | Message ID 1.1 | uint | — | first requested message type [Table 66] |
| 76–87 | 12 | Slot offset 1.1 | slots | 0 | "set to zero, if slot should autonomously be allocated by the responding station" [Table 66 notes] |
| 88–89 | 2 | Spare | — | 0 | present only in the longer shapes [Table 66] |
| 90–95 | 6 | Message ID 1.2 | uint | 0 | second requested type from station 1; "set to zero" when two stations get one request each [Table 66 notes] |
| 96–107 | 12 | Slot offset 1.2 | slots | 0 | [Table 66] |
| 108–109 | 2 | Spare | — | 0 | [Table 66] |
| 110–139 | 30 | Destination ID 2 | MMSI | — | second interrogated station [Table 66] |
| 140–145 | 6 | Message ID 2.1 | uint | — | [Table 66] |
| 146–157 | 12 | Slot offset 2.1 | slots | 0 | [Table 66] |
| 158–159 | 2 | Spare | — | 0 | [Table 66] |

The notes under Table 66 define four shapes:

| Case | Content | Fields present | Bits |
|---|---|---|---|
| 1 | one station, one message | through slot offset 1.1 | 88 |
| 2 | one station, two messages | through the spare after slot offset 1.2 | 110 — "see §3.3.7, Annex 2 for byte boundaries" |
| 3 | two stations, one message each | all; "message ID 1.2 and slot offset 1.2 should be set to zero" | 160 |
| 4 | station 1 two messages, station 2 one | all | 160 |

[M.1371-5 Annex 8 Table 66 notes]. The reference to byte boundaries in case 2 is the problem: 110 is not a multiple of 8, and Annex 2 says "unused bits in the last byte should be set to zero in order to preserve byte boundary" [Annex 2 §3.3.7], so a transmitter that follows the link-layer rule sends **112** bits. [AIVDM "Byte Alignment"] calls this "an apparent error in the format design" — the trailing spare should have been 4 bits — and tells decoders to accept both 110 and 112. We do (§11.4).

Other rules in the Table 66 notes: "an interrogating mobile station should always set the parameter 'slot offset' to zero. Slot assignments for the reply to an interrogation should only be used by a base station. If a slot offset is given, it should be relative to the start slot of this transmission. A mobile station should be able to process a minimum slot offset of 10 slots." Table 65 restricts who may ask whom:

| Interrogated ↓ / Interrogator → | Class A | Class B "SO" | Class B "CS" | SAR aircraft | AtoN | Base station |
|---|---|---|---|---|---|---|
| Class A | 3, 5, 24 | — | — | 3, 5, 24 | — | 3, 5, 24 |
| Class B "SO" | 18, 24 | — | — | 18, 24 | — | 18, 24 |
| Class B "CS" | 18, 24 | — | — | 18, 24 | — | 18, 24 |
| SAR aircraft | 9, 24 | — | — | 9 | — | 9, 24 |
| AtoN | 21 | — | — | — | — | 21 |
| Base station | 4, 24 | — | — | 4, 24 | — | 4, 24 |

[M.1371-5 Annex 8 Table 65]. Notes to the table: a request for message 24 "shall be answered either with a Part A or a Part B or with both"; "some AtoN stations are not able to respond due to their operational behaviour". Class B units never interrogate ("A Class B 'CS' AIS should not interrogate other stations" [Annex 7 §2.1.2.3]). Timing for a Class B "CS" answer: within 30 s if no offset was given, with one retry after 30 s; in the specified slot if a base station gave an offset [Annex 7 §4.3.3.3.3]. "Interrogations for the same message received before own response has been transmitted may be ignored" [Annex 7 §4.3.3.3.3].

### Message 16: assignment mode command

96 or 144 bits [M.1371-5 Annex 8 §3.14 Table 67]:

| Bits | Len | Field | Type/units | Not-available | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | uint | — | 16 [Table 67] |
| 6–7 | 2 | Repeat indicator | uint | — | [Table 67] |
| 8–37 | 30 | Source ID | MMSI | — | "MMSI of assigning station" [Table 67] |
| 38–39 | 2 | Spare | — | 0 | [Table 67] |
| 40–69 | 30 | Destination ID A | MMSI | — | [Table 67] |
| 70–81 | 12 | Offset A | slots *or* reports per 10 min | — | "offset from current slot to first assigned slot" — see note 1 [Table 67] |
| 82–91 | 10 | Increment A | code | — | "increment to next assigned slot" — see note 1 [Table 67] |
| 92–121 | 30 | Destination ID B | MMSI | — | "should be omitted if there is assignment to station A, only" [Table 67] |
| 122–133 | 12 | Offset B | as A | — | omitted likewise [Table 67] |
| 134–143 | 10 | Increment B | as A | — | omitted likewise [Table 67] |
| 92–95 | 4 | Spare | — | 0 | *in the 96-bit form only*: "the number of spare bits, which should be 0 or 4, should be adjusted in order to observe byte boundaries" [Table 67] |

Note 1 of Table 67 and the paragraph after it carry the semantics:

* **Increment = 0 → reporting-rate assignment.** "The parameter offset should then be interpreted as the number of reports in a time interval of 10 min." Only multiples of 20 between 20 and 600 should be used; a mobile that receives another value below 600 rounds *up* to the next multiple of 20, and clamps values above 600 to 600 [§3.14]. 20 per 10 min is one report every 30 s; 600 is one per second. "Class B mobile AIS stations should not be assigned a reporting interval of less than 2 s" (i.e. more than 300 per 10 min) [§3.14].
* **Increment ≠ 0 → slot assignment.** `offset` is the number of slots from the slot carrying this message 16 to the first assigned slot, and `increment` is a *code*, not a slot count: 1 = 1125 slots, 2 = 375, 3 = 225, 4 = 125, 5 = 75, 6 = 45; "7 = undefined. If a station receives the value 7, the station should disregard this assignment" [§3.14]. At 2 250 slots per minute per channel [Annex 2 §3.1.2] those are intervals of 30, 10, 6, 3⅓, 2 and 1.2 seconds. The assigned slots use the SOTDMA (self-organising TDMA) communication state with the slot time-out counting down from 3–7, and the station returns to autonomous mode when it reaches zero [Annex 2 §3.3.6.2.2, §3.3.6.2.3]. Codes 8–1023 are not defined. (In M.1371-1 the 10-bit field *was* a raw slot increment; the code table appeared with M.1371-3. A 2001-era base station and a 2014-era mobile would misunderstand each other.)
* Assignments "are limited in time and will be re-issued by the competent authority as needed. The last received assignment should continue or overwrite the previous assignment. This should also be the case, when two assignments are made in the same Message 16 for the same station" [Annex 2 §3.3.6]. "If continued assignment is required, the new assignment should be transmitted before the start of the last frame of the previous assignment" [§3.14].
* Bounds: Annex 2 Table 16 gives the nominal increment range (75–1225 slots, 37.5 in rate-assigned mode) and report rates of 2 to 30 per minute (60 in assigned SOTDMA mode) [Annex 2 Table 16 and notes].

## 11.3 Decoding one message by hand

### A recorded message 10

The UNH Coastal Marine Lab receiver (`nhcml`, New Castle, NH) logged this pair on 9 March 2010 at 00:09:50 UTC — `sample-data/tide3/ais/ais-2010-03-09` lines 131 and 132, lines 4 and 5 of [`data/ch11-interrogation-assignment.nmea`](../data/ch11-interrogation-assignment.nmea):

```text
!AIVDM,1,1,,A,:5BCc21GTjPp,0*11,rnhcml,1268093390.4
!AIVDM,1,1,,A,;5NC:3Qu`lP9kJtF7JHa0LA00000,0*27,rnhcml,1268093390.82
```

The first payload is 12 characters, 0 fill bits → 72 bits. Six-bit groups:

```text
:      5      B      C      c      2      1      G      T      j      P      p
001010 000101 010010 010011 101011 000010 000001 010111 100100 110010 100000 111000
```

| Bits | Value | Field |
|---|---|---|
| 0–5 | `001010` = 10 | message ID |
| 6–7 | `00` = 0 | repeat indicator |
| 8–37 | `010101001001001110101100001000` = **354741000** | source MMSI — MID 354, Panama [M.585-9] |
| 38–39 | `00` | spare |
| 40–69 | `010101111001001100101000001110` = **367315470** | destination MMSI — MID 367, USA |
| 70–71 | `00` | spare |

The second line, 0.42 s later on the same channel A, is a 168-bit message 11 from 367315470 (chapter 6 decodes it: 2010-03-09T00:09:51Z, 43.062802° N 70.703392° W). The asking ship is the Panama-flag bulk carrier *JIN QIANG*, at anchor in 43.08° N 70.76° W off Portsmouth, NH, destination BOSTON, according to its own message 5 in the same log. Over two days it sent 46 UTC inquiries — mostly to 367288000 and 367315470, also anchored nearby, once to 367301070 — and the receiver logged 34 message 11 replies from those three ships (29, 4 and 1); its own position reports show sync state 0 almost all the time, with occasional sync state 3 ("synchronized to another station"), so its GNSS time was dropping out intermittently rather than gone.

### A recorded message 15

`sample-data/2007hydro/sr162-noaa-norfolk-telcom2007-05-14` line 879 (line 16 of the data file), from an SR162 receiver at the US Hydro 2007 conference in Norfolk, VA:

```text
!AIVDM,1,1,,A,?52HH<iGMIWpD00,2*69,1179173361.13
```

15 characters are 90 bits; the fill count is 2, so the payload is 88 bits — case 1 of Table 66. Groups:

```text
?      5      2      H      H      <      i      G      M      I      W      p      D      0      0
001111 000101 000010 011000 011000 001100 110001 010111 011101 011001 100111 111000 010100 000000 0000|00
```

(the last two bits of the final group are the fill and are discarded).

| Bits | Value | Field |
|---|---|---|
| 0–5 | `001111` = 15 | message ID |
| 6–7 | `00` | repeat indicator |
| 8–37 | `010100001001100001100000110011` = **338040883** | source MMSI — MID 338, USA |
| 38–39 | `00` | spare |
| 40–69 | `010101110111010110011001111110` = **366831230** | destination ID 1 |
| 70–75 | `000101` = **5** | message ID 1.1: static and voyage related data |
| 76–87 | `000000000000` = 0 | slot offset 1.1: responder chooses the slot |

So: station 338040883 asks station 366831230 for its message 5. Who 338040883 is becomes clear from a second log recorded at the same conference from the maintenance port of an L-3 ProTec Class A transponder: the same interrogations appear there as `!AIVDO` — own-ship sentences — with the same source MMSI (lines 19 and 20 of the data file, from `sample-data/2007hydro/protec-util12007-05-14` lines 496 and 497). The ProTec was the interrogator; §11.6 looks at why.

### A synthetic message 16

No valid message 16 exists in our recordings (§11.6), so we build the examples. Line 36 of the data file is a slot assignment from the USCG base station that appears throughout the 2007 Norfolk logs (003669702) to the ship that was interrogated above:

| Field | Value | Bits |
|---|---|---|
| message ID | 16 | `010000` |
| repeat indicator | 0 | `00` |
| source ID | 003669702 | `000000001101111111111011000110` |
| spare | 0 | `00` |
| destination ID A | 366831230 | `010101110111010110011001111110` |
| offset A | 20 slots | `000000010100` |
| increment A | code 2 (= 375 slots) | `0000000010` |
| spare | 0 (96-bit form) | `0000` |

96 bits = 16 characters, no fill; armoured and checksummed:

```text
!AIVDM,1,1,,B,@03OviQGMIWp1@0P,0*12
```

```text
@      0      3      O      v      i      Q      G      M      I      W      p      1      @      0      P
010000 000000 000011 011111 111110 110001 100001 010111 011101 011001 100111 111000 000001 010000 000000 100000
```

Reading it back: bits 70–81 `000000010100` = 20, bits 82–91 `0000000010` = 2. Increment ≠ 0, so this is a *slot* assignment: 366831230 must transmit its position reports starting 20 slots (0.53 s) after the slot in which it received this command and then every 375 slots, i.e. every 10 s, until the assignment times out. Had the increment been 0, the same offset field would have meant "20 reports per 10 minutes" — one every 30 s. Line 34 of the data file is that reporting-rate form (offset 60, increment 0 → 60 reports per 10 min, one every 10 s), line 38 is the 144-bit form with a slot assignment for A and a rate for B, and line 40 has increment 7. The two-station message 15 shapes (lines 25–31) were built the same way with the same base-station and ship MMSIs; their field values are in §11.5.

## 11.4 The code

Message 10 is six fixed fields. Messages 15 and 16 contain *repeated groups* whose presence depends on the payload length, which we handle the way chapter 12 handles the acknowledgements of message 7: decode the mandatory first group, then test `bits.has(start, length)` for each optional block and advance a running `pos`. Whatever is left is reported as `trailing_bits` — zero for every legal shape except the byte-aligned 112-bit interrogation, where it is 2.

### Python

[`code/python/aistut/msg_10.py`](../code/python/aistut/msg_10.py):

```python
def decode(bits: Bits) -> dict:
    """Decode a message 10. Raises DecodeError if shorter than 72 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 72)
    return {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
        "dest_mmsi": bits.uint(40, 30),
        "spare2": bits.uint(70, 2),
    }
```

[`code/python/aistut/msg_15.py`](../code/python/aistut/msg_15.py) (the `MESSAGE_NAMES` table of Table 46 names is omitted here):

```python
def message_name(msg_id: int) -> str:
    return MESSAGE_NAMES.get(msg_id, "undefined")


def request(mmsi: int, msg_id: int, slot_offset: int) -> dict:
    """One (station, message, slot) triple of the `interrogations` array."""
    return {
        "mmsi": mmsi,
        "msg_type": msg_id,
        "msg_type_text": message_name(msg_id),
        "slot_offset": slot_offset,
    }


def decode(bits: Bits) -> dict:
    """Decode a message 15. Raises DecodeError below 88 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 88)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
    }
    mmsi1 = bits.uint(40, 30)
    interrogations = [request(mmsi1, bits.uint(70, 6), bits.uint(76, 12))]
    pos = 88

    # Second request to the first station (bits 88-107, plus its spare).
    if bits.has(88, 20):
        out["spare2"] = bits.uint(88, 2)
        msg_id_12 = bits.uint(90, 6)
        if msg_id_12 != 0:
            interrogations.append(request(mmsi1, msg_id_12, bits.uint(96, 12)))
        pos = 108

    # A second station (bits 108-157, plus the trailing spare).
    if bits.has(108, 50):
        out["spare3"] = bits.uint(108, 2)
        interrogations.append(request(bits.uint(110, 30), bits.uint(140, 6), bits.uint(146, 12)))
        pos = 158
        if bits.has(158, 2):
            out["spare4"] = bits.uint(158, 2)
            pos = 160
    elif bits.has(108, 2):
        # The 110/112-bit form: the spare after group 1.2, then 0 or 2
        # padding bits.
        out["spare3"] = bits.uint(108, 2)
        pos = 110

    out["interrogations"] = interrogations
    out["trailing_bits"] = len(bits) - pos
    return out
```

Design decisions, all of them documented in the module docstring:

* The output is an array `interrogations` of `{mmsi, msg_type, msg_type_text, slot_offset}`, one per request, in message order. The second request to station 1 repeats station 1's MMSI, so a consumer never has to know which slot of Table 66 an entry came from.
* The 1.2 group is appended only when its bits are present **and** its message ID is non-zero — zero is how case 3 says "no second request", and a request for "message 0" means nothing anyway. The spare bits are output whenever they are present (`spare`, `spare2`, `spare3`, `spare4`, in bit order), so nothing is silently dropped.
* The first group is mandatory and 88 bits are required; a 1.1 message ID of 0 (which the USCG base station sends, §11.5) is reported as is with `msg_type_text` `"none"`. The decoder describes; the consumer judges.

[`code/python/aistut/msg_16.py`](../code/python/aistut/msg_16.py):

```python
# Table 67 text under note (1): increment code -> slots between assigned slots.
SLOT_INCREMENTS: dict[int, int] = {1: 1125, 2: 375, 3: 225, 4: 125, 5: 75, 6: 45}


def assignment(mmsi: int, offset: int, increment: int) -> dict:
    """Interpret one (destination, offset, increment) triple."""
    reports: int | None = None
    slots: int | None = None
    if increment == 0:
        text = "reporting rate"
        reports = offset
    elif increment in SLOT_INCREMENTS:
        text = "slot assignment"
        slots = SLOT_INCREMENTS[increment]
    elif increment == 7:
        text = "undefined increment (disregard)"
    else:
        text = "increment not defined in M.1371-5"
    return {
        "mmsi": mmsi,
        "offset": offset,
        "increment": increment,
        "assignment_text": text,
        "reports_per_10_min": reports,
        "slot_increment": slots,
    }


def decode(bits: Bits) -> dict:
    """Decode a message 16. Raises DecodeError below 96 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 96)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
    }
    assignments = [assignment(bits.uint(40, 30), bits.uint(70, 12), bits.uint(82, 10))]
    pos = 92
    if bits.has(92, 52):
        assignments.append(assignment(bits.uint(92, 30), bits.uint(122, 12), bits.uint(134, 10)))
        pos = 144
    else:
        out["spare2"] = bits.uint(92, 4)
        pos = 96
    out["assignments"] = assignments
    out["trailing_bits"] = len(bits) - pos
    return out
```

Each assignment carries the raw `offset` and `increment` *and* the interpretation: `reports_per_10_min` is set when the increment is 0, `slot_increment` (in slots) when it is a code 1–6, and both are `null` for 7 and for the undefined codes. We do not apply the mobile's rounding rule (up to the next multiple of 20, clamp at 600) — that is the receiving station's behaviour, and a VTS analyst wants to see what was actually sent.

> **Language notes (Python).** `int | None` (chapter 3) for a value that may be `None`; `dict.get(key, default)` for a table lookup with a fallback. The "decode the first group, then `while`/`if bits.has(...)`" shape is the loop chapter 12 uses for message 7, unrolled because here the groups are not all alike.

### C++

[`code/cpp/msg_15.hpp`](../code/cpp/msg_15.hpp), the parts that differ from Python:

```cpp
inline std::string message_name(int msg_id) {
    auto it = MESSAGE_NAMES.find(msg_id);
    return it == MESSAGE_NAMES.end() ? "undefined" : it->second;
}

inline Json request(std::uint64_t mmsi, std::uint64_t msg_id, std::uint64_t slot_offset) {
    return Json::Object{
        {"mmsi", mmsi},
        {"msg_type", msg_id},
        {"msg_type_text", message_name(static_cast<int>(msg_id))},
        {"slot_offset", slot_offset},
    };
}

inline Json decode_msg_15(const Bits& bits) {
    using namespace msg_15;
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 88);
    // ... header fields as in Python ...
    std::uint64_t mmsi1 = bits.uint(40, 30);
    Json::Array interrogations;
    interrogations.push_back(request(mmsi1, bits.uint(70, 6), bits.uint(76, 12)));
    std::size_t pos = 88;
    if (bits.has(88, 20)) {
        out["spare2"] = bits.uint(88, 2);
        std::uint64_t msg_id_12 = bits.uint(90, 6);
        if (msg_id_12 != 0) {
            interrogations.push_back(request(mmsi1, msg_id_12, bits.uint(96, 12)));
        }
        pos = 108;
    }
    // ... second station exactly as in Python ...
    out["interrogations"] = interrogations;
    out["trailing_bits"] = bits.size() - pos;
    return out;
}
```

[`code/cpp/msg_16.hpp`](../code/cpp/msg_16.hpp), the assignment helper:

```cpp
inline const std::map<std::uint64_t, long long> SLOT_INCREMENTS = {
    {1, 1125}, {2, 375}, {3, 225}, {4, 125}, {5, 75}, {6, 45},
};

inline Json assignment(std::uint64_t mmsi, std::uint64_t offset, std::uint64_t increment) {
    std::optional<long long> reports;
    std::optional<long long> slots;
    std::string text;
    auto it = SLOT_INCREMENTS.find(increment);
    if (increment == 0) {
        text = "reporting rate";
        reports = static_cast<long long>(offset);
    } else if (it != SLOT_INCREMENTS.end()) {
        text = "slot assignment";
        slots = it->second;
    } else if (increment == 7) {
        text = "undefined increment (disregard)";
    } else {
        text = "increment not defined in M.1371-5";
    }
    return Json::Object{
        {"mmsi", mmsi},
        {"offset", offset},
        {"increment", increment},
        {"assignment_text", text},
        {"reports_per_10_min", reports},
        {"slot_increment", slots},
    };
}
```

> **Language notes (C++).**
> * `std::optional<long long>` (chapter 1) starts empty; assigning a value fills it, and `Json` prints an empty optional as `null` — which is how the Python `None` comes out byte-identical. `std::map::find`/`end()` is the lookup idiom of chapters 1 and 6.
> * `Json::Array` is `std::vector<Json>`; `push_back` appends. We build the array *before* the object that holds it because the object initialiser list is evaluated in one go.

### Rust

[`code/rust/src/msg_16.rs`](../code/rust/src/msg_16.rs), the parts that differ:

```rust
pub fn slot_increment(code: u64) -> Option<u64> {
    match code {
        1 => Some(1125),
        2 => Some(375),
        3 => Some(225),
        4 => Some(125),
        5 => Some(75),
        6 => Some(45),
        _ => None,
    }
}

fn assignment(mmsi: u64, offset: u64, increment: u64) -> Json {
    let mut reports: Option<u64> = None;
    let slots = slot_increment(increment);
    let text = if increment == 0 {
        reports = Some(offset);
        "reporting rate"
    } else if slots.is_some() {
        "slot assignment"
    } else if increment == 7 {
        "undefined increment (disregard)"
    } else {
        "increment not defined in M.1371-5"
    };
    let mut a = Obj::new();
    put(&mut a, "mmsi", mmsi);
    put(&mut a, "offset", offset);
    put(&mut a, "increment", increment);
    put(&mut a, "assignment_text", text);
    put(&mut a, "reports_per_10_min", reports);
    put(&mut a, "slot_increment", slots);
    Json::Obj(a)
}

pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 96)?;
    // ... header ...
    let mut assignments: Vec<Json> =
        vec![assignment(bits.uint(40, 30), bits.uint(70, 12), bits.uint(82, 10))];
    let pos;
    if bits.has(92, 52) {
        assignments.push(assignment(bits.uint(92, 30), bits.uint(122, 12), bits.uint(134, 10)));
        pos = 144;
    } else {
        put(&mut out, "spare2", bits.uint(92, 4));
        pos = 96;
    }
    put(&mut out, "assignments", assignments);
    put(&mut out, "trailing_bits", bits.len() - pos);
    Ok(Json::Obj(out))
}
```

In [`code/rust/src/msg_15.rs`](../code/rust/src/msg_15.rs) `message_name` is one `match` with twenty-nine arms and a `_ => "undefined"` fallback, and `request()` builds each element of the `Vec<Json>` exactly as the Python `request()` does.

> **Language notes (Rust).**
> * `Option<u64>` goes straight into `put` because `jsonout` implements `From<Option<T>>` (chapter 6), and `let text = if … { … } else { … };` is the `if`-expression of chapter 3; the branch that also sets `reports` does so with a statement before its final expression.
> * `let pos;` declares a variable without a value; the compiler checks that every path assigns it exactly once before use (*definite initialisation*). Python would simply assign in each branch.
> * `vec![…]` creates a `Vec` with initial contents.

### TypeScript

[`code/typescript/src/msg_15.ts`](../code/typescript/src/msg_15.ts) and [`msg_16.ts`](../code/typescript/src/msg_16.ts), the parts that differ:

```typescript
export function messageName(msgId: number): string {
  return MESSAGE_NAMES[msgId] ?? "undefined";
}

function request(mmsi: number, msgId: number, slotOffset: number): JsonObject {
  return {
    mmsi,
    msg_type: msgId,
    msg_type_text: messageName(msgId),
    slot_offset: slotOffset,
  };
}
```

```typescript
export const SLOT_INCREMENTS: { readonly [code: number]: number } = {
  1: 1125, 2: 375, 3: 225, 4: 125, 5: 75, 6: 45,
};

function assignment(mmsi: number, offset: number, increment: number): JsonObject {
  let reports: number | null = null;
  const slots: number | null = SLOT_INCREMENTS[increment] ?? null;
  let text: string;
  if (increment === 0) {
    text = "reporting rate";
    reports = offset;
  } else if (slots !== null) {
    text = "slot assignment";
  } else if (increment === 7) {
    text = "undefined increment (disregard)";
  } else {
    text = "increment not defined in M.1371-5";
  }
  return {
    mmsi,
    offset,
    increment,
    assignment_text: text,
    reports_per_10_min: reports,
    slot_increment: slots,
  };
}
```

> **Language notes (TypeScript).**
> * `??` and `number | null` are the chapter 1 constructs: a missing table entry becomes `"undefined"` or `null` (`||` would also replace `0` and `""`, which is not what we want for a lookup table), and under `strict` the compiler makes you handle the `null` case.
> * `let text: string;` without an initialiser is allowed because TypeScript, like Rust, checks that `text` is assigned on every path before it is read.

### Where the four differ

| Concern | Python | C++ | Rust | TypeScript |
|---|---|---|---|---|
| Optional integer → `null` | `int \| None` / `None` | `std::optional<long long>` | `Option<u64>` | `number \| null` |
| Table lookup with default | `dict.get(k, d)` | `map.find` / `end()` | `match … _ =>` | `obj[k] ?? d` |
| Growing array of objects | `list.append` | `Json::Array::push_back` | `Vec::push` | `Array.push` |
| Variable assigned in branches | plain assignment | plain assignment | `let pos;` + definite initialisation | `let pos: number;` + definite assignment |

## 11.5 Sample data and results

[`data/ch11-interrogation-assignment.nmea`](../data/ch11-interrogation-assignment.nmea) has four parts:

* **A — message 10/11, recorded** (lines 4–9): three inquiries from 354741000 with the two replies that followed, from `sample-data/tide3/ais/ais-2010-03-09` lines 131–132, 393–394 and 524, and the first inquiry of the previous day, `sample-data/tide3/ais/ais-2010-03-08` line 5089.
* **B — message 15, recorded** (lines 12–22): two interrogations by USCG base station 003669702 sent 0.7 s apart on channels B and A (`sample-data/2007hydro/va-portsmouth-sr162-2007-05-15` lines 403 and 407); the ProTec's interrogation of 366831230 as heard by a separate receiver (`sample-data/2007hydro/sr162-noaa-norfolk-telcom2007-05-14` line 879); two of the ProTec's own interrogations as `!AIVDO` (`sample-data/2007hydro/protec-util12007-05-14` lines 496 and 497); and a 144-bit line that claims to be a message 15 (`sample-data/2007hydro/va-portsmouth-sr162-2007-05-17` line 13367).
* **C — message 15, synthetic** (lines 25–31): the three shapes the recordings lack — one station/two messages as 110 bits and as 112 bits, two stations/one message each, and station 1 two messages + station 2 one message — all from base station 003669702 to 366831230 and 367113050 (both heard in the Norfolk logs), requesting messages 3, 5 and 24 with slot offsets 0, 60, 120 and 180.
* **D — message 16, synthetic plus one recorded garbage line** (lines 34–43): reporting-rate (60 per 10 min), slot (offset 20, increment code 2), two-station (A slots, B 120 per 10 min), increment 7; then a 96-bit line from `sample-data/tide1/ais/ais-2007-11-24` line 265 whose "base station" is 809380875.

Run any decoder:

```sh
python3 code/python/decode.py data/ch11-interrogation-assignment.nmea
code/cpp/build/decode data/ch11-interrogation-assignment.nmea
code/rust/target/release/decode data/ch11-interrogation-assignment.nmea
node code/typescript/dist/decode.js data/ch11-interrogation-assignment.nmea
```

All four print the 21 lines of [`tests/expected/ch11-interrogation-assignment.jsonl`](../tests/expected/ch11-interrogation-assignment.jsonl). Selected objects (the `"ais"` part unless noted):

**Line 4 — message 10, the hand decode:**

```json
{"dest_mmsi":367315470,"mmsi":354741000,"msg_type":10,"repeat_indicator":0,"spare":0,"spare2":0}
```

Line 5 is the reply: `"msg_type":11,"mmsi":367315470,"utc_iso":"2010-03-09T00:09:51Z"`, `"lat":43.062802,"lon":-70.703392`, with `comm_state` `slot_timeout` 0 and `slot_offset` 0 — a one-off transmission in a slot that is released immediately, as chapter 6 explained. The receiver time stamps are 1268093390.4 and 1268093390.82: question to answer in **0.42 s**, on the same channel (A), as §3.2 requires. Lines 6–7 are the same exchange with 367288000 (0.25 s, channel B). Line 8 asks **250000848** — MID 250, Ireland — and no reply follows in the log; the Irish ship was presumably out of the receiver's range, or out of the asker's. Line 9 asks 367301070.

**Line 12 — the base station's interrogation:**

```json
{"interrogations":[{"mmsi":636090931,"msg_type":0,"msg_type_text":"none","slot_offset":0}],"mmsi":3669702,"msg_type":15,"repeat_indicator":0,"spare":0,"trailing_bits":0}
```

88 bits, source 003669702 (a coast station: MMSI `00` + MID 366 [M.585-9]), destination 636090931 (MID 636, Liberia) — and **message ID 0**. Line 13 is the identical request 0.67 s later on the other channel. Every one of the 497 interrogations this base station sent on 15–18 May 2007 has message ID 0 and slot offset 0; §11.6 discusses what that might mean.

**Line 16 — the ProTec's interrogation, as heard by another receiver:**

```json
{"interrogations":[{"mmsi":366831230,"msg_type":5,"msg_type_text":"static and voyage related data","slot_offset":0}],"mmsi":338040883,"msg_type":15,"repeat_indicator":0,"spare":0,"trailing_bits":0}
```

**Lines 19–20 — the same transponder's interrogations on its own port.** The outer object has `"own_ship":true` (the sentences are `!AIVDO`); the requests are for message 5 from 211711000 (Germany) and 248544000 (Malta), 60 ms apart.

**Line 22 — 144 bits is not a legal length.** The decoder reads what it can:

```json
{"interrogations":[{"mmsi":1039636045,"msg_type":20,"msg_type_text":"data link management message","slot_offset":3114},{"mmsi":1039636045,"msg_type":43,"msg_type_text":"undefined","slot_offset":4030}],"mmsi":283164725,"msg_type":15,"repeat_indicator":1,"spare":0,"spare2":3,"spare3":3,"trailing_bits":34}
```

and everything says "noise": a ten-digit destination MMSI, a request for message 43 (there are 27), spare bits set, 34 trailing bits, a repeat indicator of 1 from a "source" (MID 283, Austria) that is not a base station. `trailing_bits` 34 is the decoder's way of telling you the length matched no shape in Table 66.

**Lines 25 and 27 — one station, two messages, as 110 and as 112 bits.** The two sentences have the *same* 19 payload characters, `?03OviQGMIWp<005000`, and differ only in the fill count: `,4` for 110 bits, `,2` for 112. The decoded objects are identical except for the last field:

```json
{"interrogations":[{"mmsi":366831230,"msg_type":3,"msg_type_text":"position report (interrogated)","slot_offset":0},{"mmsi":366831230,"msg_type":5,"msg_type_text":"static and voyage related data","slot_offset":0}],"mmsi":3669702,"msg_type":15,"repeat_indicator":0,"spare":0,"spare2":0,"spare3":0,"trailing_bits":0}
```

versus `"trailing_bits":2` for the byte-aligned form. A consumer that insists on `trailing_bits == 0` would reject the form that a standards-compliant transmitter actually sends.

**Line 29 — two stations, one message each (case 3):** `interrogations` has two entries, for 366831230 (message 5, slot offset 120) and 367113050 (message 5, slot offset 180); `spare2`, `spare3`, `spare4` are all 0 and there is no entry for the zeroed 1.2 group. **Line 31 — case 4:** three entries — 366831230 message 3 at offset 60, 366831230 message 5 at offset 120, 367113050 message 24 at offset 180.

**Line 34 — reporting-rate assignment:**

```json
{"assignments":[{"assignment_text":"reporting rate","increment":0,"mmsi":366831230,"offset":60,"reports_per_10_min":60,"slot_increment":null}],"mmsi":3669702,"msg_type":16,"repeat_indicator":0,"spare":0,"spare2":0,"trailing_bits":0}
```

**Line 36 — slot assignment (the hand decode):** `"increment":2,"offset":20,"slot_increment":375,"reports_per_10_min":null,"assignment_text":"slot assignment"`. **Line 38 — 144 bits:** two entries, the first as line 36, the second `{"mmsi":367113050,"offset":120,"increment":0,"reports_per_10_min":120,…}`; no `spare2` key, because the 144-bit form has no spare. **Line 40 — increment 7:** `"assignment_text":"undefined increment (disregard)"`, both interpretations `null`.

**Line 43 — the recorded garbage:** 96 bits that decode to "base station" 809380875 (not a `00MID` identity) assigning station 1968553 (seven digits) a rate of **1216** reports per 10 minutes — twice the legal maximum — with repeat indicator 2. Structurally valid, semantically impossible; only the field-range checks catch it.

## 11.6 Pitfalls and real-world deviations

* **A base station interrogating for "message 0".** The USCG base station 003669702 in Hampton Roads sent 497 message 15s in four days of our 2007 logs, every one with message ID 1.1 = 0 and slot offset 0, each one repeated 0.7 s later on the other channel, addressed to 76 different MMSIs, about half of which never appear as a transmitter anywhere in our Norfolk logs. Message 0 does not exist, so no station could have answered (none did). Two readings are possible: a shore software that used message 15 as a "are you there?" ping to targets that had aged out of its track table, with the message-type field left at its default; or a bug. Either way it is real traffic from a real authority, and a decoder that rejected message ID 0 would hide it. Report it (`msg_type_text` `"none"`) and let the analyst decide.
* **Transponders that interrogate on their own.** The L-3 ProTec at the Norfolk conference sent 60 message 15s in about two hours, every one a request for message 5. For most of them (about 35 of the 60, depending on how strictly you count a "valid" first reception) the destination was an MMSI the transponder had heard for the *first time* one to four sentences earlier; about 15 went to stations first heard thousands of lines before — presumably ones whose message 5 had still not arrived — and a handful went to MMSIs that occur in no structurally valid message in the log at all. At least eight of the destinations are impossible identities — 1024604400 and 966656752 (ten digits), 2369708 and 919938 (seven or six) among them — which means the transponder was interrogating MMSIs that came out of bit-corrupted position reports it had accepted. No standard requires a Class A to behave this way, and nothing forbids it; the lesson for implementers is to validate an MMSI before spending a slot on it, and for analysts that a flood of message 15s for message 5 is probably a transponder feature, not an attack. (This transponder had no antenna connected and was logging `$AIALR` "Tx malfunction" alarms; the NOAA receiver elsewhere in the same building still heard its interrogations, but the ships it asked almost certainly did not — exercise 6.)
* **No valid message 16 anywhere.** Across every recording, the scan found twelve lines whose first six bits say 16 and whose length is 96 or 144 bits; all twelve come from the `nhjel` receiver, and all twelve have a non-base-station "source" MMSI, an impossible destination or an offset/increment outside any legal range (line 43 is typical). Two more had other lengths. [AIVDM "Type 16"] records the same scarcity. If your pipeline ever *does* see a real message 16, the first question is which base station sent it and whether it is one of the authority's.
* **110 or 112 bits.** The two-request interrogation is written as 110 bits in Table 66 and transmitted as 112 by any unit that pads to a byte boundary [Annex 2 §3.3.7; AIVDM "Byte Alignment"]. Accept both; our decoder reports the two padding bits as `trailing_bits` 2. Note that the armoured text is the same in both cases — only the fill count in the sentence changes (lines 25 and 27).
* **Zero means "absent" in message 15, but not everywhere.** Case 3 of Table 66 encodes "no second request to station 1" as message ID 1.2 = 0. We drop that group from the array; we do *not* drop a 1.1 or 2.1 group with message ID 0, because those positions are not optional and the base station above shows that zero is sent there in practice.
* **Increment is a code, not a count.** A message 16 with increment 375 does not mean "every 375 slots"; under M.1371-3 and later it means nothing (undefined code), and under M.1371-1 it meant 375 slots. Our decoder reports codes 8–1023 as `"increment not defined in M.1371-5"` rather than guessing which edition the sender read.
* **Rounding is the mobile's job.** A rate of 50 reports per 10 min is legal to *send*; the mobile rounds it up to 60 [§3.14]. If you simulate a mobile, implement the rounding and the 600 clamp; if you audit a base station, flag non-multiples of 20 as sloppy configuration.
* **Who may send what.** Message 16 only from a base station; message 15 never from a Class B; message 10 from anyone. Interrogation and assignment from a ship-format MMSI (first digit 2–7) is either a mis-configured unit or someone experimenting — in both cases worth noticing (§11.1, security).
* **M.1371-4 and -5 table numbers.** In M.1371-5 message 10 is Table 60, the interrogator/interrogated matrix Table 65, message 15 Table 66 and message 16 Table 67. In M.1371-4 the same tables were numbered three lower — 57, 62, 63 and 64 — so older texts and decoder comments that cite "Table 63" or "Table 64" for message 15/16 are counting from the 2010 edition.

## 11.7 Exercises

1. **(Pencil)** Decode line 31 of the data file, `?03OviQGMIWp<3h51p5N6kFV0e0` with 2 fill bits, by hand: confirm 160 bits, then read message ID 1.1, slot offset 1.1, message ID 1.2, slot offset 1.2, destination ID 2, message ID 2.1 and slot offset 2.1. Check against `tests/expected/ch11-interrogation-assignment.jsonl`.
2. **(Python)** Write a *pairing* script: read a log, keep every message 10 keyed by (destination, channel), and when a message 11 arrives from that MMSI on that channel within 10 s print the latency. Run it over `sample-data/tide3/ais/ais-2010-03-09` and report the median latency and the number of unanswered inquiries. (You will need to strip `\r` and parse the trailing receive time as chapter 5 does.)
3. **(TypeScript)** Implement the mobile's side of a reporting-rate assignment: `assignedIntervalSeconds(reportsPer10Min)` that applies the "next higher multiple of 20, clamp at 600" rule [§3.14] and returns the interval in seconds. Table it for 1, 19, 20, 21, 50, 300, 599, 600, 601, 4095. Which of those would a Class B be forbidden to accept?
4. **(C++)** Our `decode_msg_15` appends the 1.2 group only when its message ID is non-zero. Change it (in a copy) to always append it when present, and add a boolean `requested` to each entry instead. Which behaviour is friendlier to a consumer that counts "requests per interrogation"? Which is friendlier to one that wants to see exactly what was on the air?
5. **(Rust)** Write a unit test for `msg_15::decode` with a 109-bit payload (take the 112-bit synthetic sentence and use 5 fill bits instead of 2). What does the decoder report in `trailing_bits`, and which groups appear in `interrogations`? Is the result "right"? Argue for your answer from Table 66.
6. **(Any language, data)** Over `sample-data/2007hydro/protec-util12007-05-14`, list each message 15 the ProTec sent with the line number of the first message received from the destination MMSI before it, and the time between the two. Then check whether any destination answered with a message 5 within 60 s (remember: two fragments). What fraction did? What does the ProTec's `$AIALR` "Tx malfunction" alarm in the same log suggest about why?

## 11.8 References

* [M.1371-5] Annex 8 §3.8 and Table 60 (message 10); §3.2 and §3.9 (message 11 as the reply, same channel); §3.13 with Table 65 (who may interrogate whom for what) and Table 66 with its notes (message 15 layout, the four cases, slot-offset rules); §3.14 with Table 67 and note 1 (message 16 layout, reporting-rate versus slot assignment, the increment code table, 4–8 minute time-out, Class B 2-second floor); Table 46 and notes 4, 9 (priorities, access schemes, "use message 10 for UTC"); Table 52 and Table 78 (what a message 5 / 24 answer contains); Annex 2 §3.1.2 (2 250 slots per minute), §3.3.2.3 (polled mode), §3.3.6–§3.3.6.2.3 (assigned operation, entering and leaving), §3.3.7 (byte boundaries), Table 16 (bounds of assignment settings), §4.2.3 (message priorities); Annex 7 §2.1.2.3 (Class B "CS" does not interrogate) and §4.3.3.3.3 (Class B "CS" response timing).
* [M.585-9] coast-station identities `00MIDxxxx` (how to recognise a base station as sender of 15/16) and ship MIDs used above.
* [AIVDM] "Type 10", "Type 15", "Type 16", and "AIS Payload Byte Alignment, Padding, and Bit Stuffing" (the 110/112-bit note; rarity of message 16).
* [MSC.140(76)] protection of the AIS VHF data link.
* [Safety Alerts] USCG alerts on AIS misuse (context for enforcement).
* Chapter 3 (message 3 as the interrogation response; communication states), chapter 6 (messages 4 and 11, the 10/11 pair there), chapter 12 (the repeated-group pattern of message 7), chapter 13 (messages 20, 22, 23 — slot reservation and group assignment), chapter 14 (Class B response to interrogation).
* Appendix B for the six-bit armour table used in §11.3.
