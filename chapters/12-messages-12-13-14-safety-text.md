# 12. Messages 7, 12, 13 and 14: safety-related text and acknowledgements

> **What you will learn**
> * How AIS carries free text: the addressed safety-related message 12 and the broadcast message 14, their headers, and the variable-length six-bit text that follows.
> * Why a text message that is 180 bits long arrives as 184 bits — byte alignment on the radio side, six-bit alignment in the sentence — and how a decoder tells text from padding.
> * How addressed messages are acknowledged on the VHF data link: message 13 (for 12) and message 7 (for 6), sequence numbers, the retransmit flag and the retry rules.
> * The fixed texts that AIS-SART, MOB-AIS and EPIRB-AIS beacons send in message 14, and how a watch-keeping program recognises them.
> * New language constructs: loops over repeated groups producing JSON arrays, a text helper shared between two modules, and string-keyed lookup tables that may return "nothing".

## 12.1 Where this fits

Most AIS messages are numbers: positions, courses, dimensions, codes. Four messages are different. Messages 12 and 14 carry **human-readable text** — the only free-form channel the protocol has — and messages 7 and 13 are the tiny **acknowledgements** that close the loop on addressed messages. ITU-R M.1371 calls 12 the "addressed safety related message" and 14 the "safety related broadcast message" [M.1371-5 Annex 8 §3.10, §3.12]; message 13 is the "safety related acknowledge" and message 7 the "binary acknowledge", and both share one layout [M.1371-5 Annex 8 §3.5 Table 56].

The word *safety* in the names is a statement of intent. IMO's operational guidelines say the content "should be relevant to the safety of navigation, e.g. an iceberg sighted or a buoy not on station", that messages "should be kept as short as possible", and that at present "these messages are not further regulated, to keep all possibilities open" [A.1106(29) ¶15]. The last clause explains most of what you will see on the air.

### Operational use

* **Bridge to bridge.** A watch officer types a short message on the transponder's minimum keyboard and display (MKD) or on a connected ECDIS and sends it either to one MMSI (message 12) or to everyone in range (message 14). The receiving transponder shows it on its MKD, usually with an audible alert, and the operator "should ensure that he displays and considers incoming safety-related messages" [A.1106(29) ¶18]. IMO is explicit that text does not replace voice VHF or GMDSS [A.1106(29) ¶17]; the US Coast Guard repeated the warning in Safety Alert 05-10, *AIS text messaging concerns*, after seeing text used for passing arrangements and even for distress traffic [Safety Alerts].
* **VTS to ship.** A vessel traffic service can address local navigational warnings, traffic or port information to one ship or broadcast them, and may ask the ship to acknowledge by text [A.1106(29) ¶46–47]. The VTS software sends the text through the base station's presentation interface using the `ABM` (addressed) or `BBM` (broadcast) sentence and learns the outcome from `ABK` — the same sentences chapter 8 used for binary messages 6 and 8, with message id 12 or 14 instead of 6 or 8 [IEC 61162-1 §8 ABM, BBM, ABK].
* **Distress beacons.** An AIS-SART (search and rescue transmitter), an AIS man-overboard device or an AIS EPIRB transmits bursts of message 1 and message 14; the text is fixed — `SART ACTIVE` / `SART TEST`, `MOB ACTIVE` / `MOB TEST`, `EPIRB ACTIVE` / `EPIRB TEST` — and the MMSI starts with 970, 972 or 974 [M.1371-5 Annex 1 §2.1.6–2.1.8, Annex 8 §3.12]. Class A equipment raises an alarm when it sees one.
* **Acknowledgement.** When a transponder receives a message 12 addressed to it, its link layer answers with message 13 on the same channel; message 7 does the same for message 6 [M.1371-5 Annex 8 §3.5]. These acknowledgements "are applicable only to the VHF data link" — they say *the radio got it*, not *a person read it* [M.1371-5 Annex 2 §5.3.1].

### Unattended / automated uses

* **Automated broadcasts.** Shore systems use message 14 to push text that nobody types: scheduled weather, water-level or lock-status lines, bridge-opening schedules, "channel closed" notices. Because the text is free-form, every port invents its own layout; a logger that wants to parse "WIND 230/18 GUST 25" has to be told the layout per source. IMO's answer was the structured binary application-specific messages of chapter 9 [Circ.289], but text broadcasts persist because every MKD can display them and no binary decoder is needed.
* **Beacon watch-keeping.** The six prescribed beacon strings are machine-matchable. A monitoring program can compare the text with the MMSI prefix (970/972/974 [M.585-9]) and the navigational status (14 when active, 15 under test) of the beacon's message 1 [M.1371-5 Annex 1 §2.1.6] and page a rescue coordination centre; a mismatch (a `SART ACTIVE` from a ship MMSI) is a sign of a test, a mistake or a spoof.
* **Keyword monitoring and data mining.** Coast guards and researchers scan message 12/14 text for words such as PIRACY, MAYDAY, MOB, TOW or for the names of restricted areas, and keep statistics on who texts whom. The USCG's AIS Miner tool, for instance, lets an analyst filter the national feed by message type and content [AISMiner].
* **Misuse.** Message 14 is the easiest AIS message to abuse: there is no destination, no acknowledgement and no content check. Reports from busy harbours include advertising, greetings, chat between crews, and texts so long that they occupy five slots several times a minute. The protocol's only defences are the slot limits — a mobile station may not spend more than 20 slots per frame on messages 6, 8, 12, 14 and 25 together, and a single message may not exceed 3 slots without reserved slots or 5 with them [M.1371-5 Annex 2 §5.2.1] — and the Class B restrictions: a Class B "CS" transponder may only send message 14 with predefined text in a single slot, and may not send message 12 at all [M.1371-5 Annex 7 §4.3.3.6 Table 42, §4.3.3.7 Table 43].
* **What acknowledgements do and do not tell you.** A logging station that hears a message 13 learns that *somebody's radio* received a message 12 — even when, as in our own recordings below, the message 12 itself was never heard. There is no end-to-end delivery guarantee beyond message 13; an application that needs one must ask for a reply in the text [A.1106(29) ¶16, ¶47].

## 12.2 The specification

All offsets are zero-based bit positions in the assembled payload (fill bits removed).

### Message 12 — addressed safety related message

| Bits | Len | Field | Type/units | Not-available value | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | unsigned | — | always 12 [M.1371-5 Annex 8 Table 61] |
| 6–7 | 2 | Repeat indicator | unsigned | — | 0 = default, 3 = do not repeat any more [Table 61] |
| 8–37 | 30 | Source ID | MMSI | — | the sender [Table 61] |
| 38–39 | 2 | Sequence number | unsigned 0–3 | — | identifies the transaction; returned in message 13 [Table 61], [M.1371-5 Annex 2 §5.3.1] |
| 40–69 | 30 | Destination ID | MMSI | — | the addressee [Table 61] |
| 70 | 1 | Retransmit flag | boolean | — | 0 = first transmission (default), 1 = retransmitted [Table 61] |
| 71 | 1 | Spare | — | — | "should be zero" [Table 61] |
| 72– | ≤ 936 | Safety related text | six-bit ASCII | — | up to 156 characters [Table 61], [M.1371-5 Annex 8 Table 47] |

Total length 72–1008 bits, 1–5 slots [M.1371-5 Annex 8 Table 61].

### Message 14 — safety related broadcast message

| Bits | Len | Field | Type/units | Not-available value | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | unsigned | — | always 14 [M.1371-5 Annex 8 Table 63] |
| 6–7 | 2 | Repeat indicator | unsigned | — | as above [Table 63] |
| 8–37 | 30 | Source ID | MMSI | — | [Table 63] |
| 38–39 | 2 | Spare | — | — | "should be set to zero" [Table 63] |
| 40– | ≤ 968 | Safety related text | six-bit ASCII | — | up to 161 characters [Table 63], [Table 47] |

Total length 40–1008 bits, 1–5 slots [M.1371-5 Annex 8 Table 63].

### Messages 7 and 13 — acknowledgements

| Bits | Len | Field | Type/units | Not-available value | Notes |
|---|---|---|---|---|---|
| 0–5 | 6 | Message ID | unsigned | — | 7 = binary acknowledge, 13 = safety related acknowledge [M.1371-5 Annex 8 Table 56] |
| 6–7 | 2 | Repeat indicator | unsigned | — | [Table 56] |
| 8–37 | 30 | Source ID | MMSI | — | the station that acknowledges [Table 56] |
| 38–39 | 2 | Spare | — | — | "should be set to zero" [Table 56] |
| 40–69 | 30 | Destination ID 1 | MMSI | — | whose message is acknowledged [Table 56] |
| 70–71 | 2 | Sequence number for ID 1 | unsigned 0–3 | — | sequence number of the acknowledged message [Table 56] |
| 72–103 | 32 | Destination ID 2 + sequence number | | — | "should be omitted if no destination ID2" [Table 56] |
| 104–135 | 32 | Destination ID 3 + sequence number | | — | likewise [Table 56] |
| 136–167 | 32 | Destination ID 4 + sequence number | | — | likewise [Table 56] |

Total length 72–168 bits in steps of 32 [M.1371-5 Annex 8 Table 56]. One acknowledgement can cover up to four addressed messages from up to four stations [M.1371-5 Annex 8 §3.5]. The sequence-number fields were spare in editions before M.1371-5 [AIVDM "Type 7"].

### The text field

Characters are the six-bit ASCII of Table 47 — `@` is 0, `A`–`Z` are 1–26, space is 32, digits are 48–57; there are no lower-case letters [M.1371-5 Annex 8 Table 47] (the full table is in [Appendix B](appendix-b-six-bit-tables.md)). Nothing in the header says how many characters follow; the text simply runs to the end of the message, and the decoder learns the length from the payload length. Three rules decide what that end looks like:

1. **Byte alignment on the air.** The transport layer requires that "the variable messages are set on byte boundaries" [M.1371-5 Annex 2 §5.2.1]. A message 12 with 18 characters is 72 + 108 = 180 bits of content; the transmitter pads it to 184. Annex 5 spells out the consequence for its text telegram: the spare "should be either 0, 2, 4 or 6" bits, and when 6 bits are needed they "will be interpreted as a valid 6-bit character (all zeros is the '@' character)" [M.1371-5 Annex 5 §5.1 Table 25 Note 1]. The same arithmetic applies to messages 12 and 14 [AIVDM "Byte Alignment"].
2. **Six-bit alignment in the sentence.** The receiver armours the 184 bits into 31 characters (186 bits) and sets the fill-bit field to 2 (chapter 2). After the fill bits are removed we are back to 184 bits: 72 of header, 18 whole characters, and 4 bits that belong to no character. Our decoder reports them as `trailing_bits` and ignores them.
3. **`@` ends the text.** A six-bit spare that happens to be a whole character decodes as `@`, and transponders that allocate a fixed buffer pad with `@` too. Like every text field in this tutorial, the decoder cuts at the first `@` and strips trailing spaces (chapter 2).

How much text fits in how many slots is given by the standard, bit stuffing included: 10/48/85/122/156 characters for 1–5 slots of message 12 [M.1371-5 Annex 8 Table 62] and 16/53/90/128/161 for message 14 [M.1371-5 Annex 8 Table 64]. Applications "should minimise the use of slots by limiting the number of characters to the numbers given" [M.1371-5 Annex 8 §3.10].

### Sequence numbers, retries and acknowledgements

An addressed message 6 or 12 carries a 2-bit sequence number chosen by the application that handed the text to the transponder; the message type, source, destination and sequence number together identify the transaction, and the sender "should not reuse a sequence number until it has been acknowledged or time-out has occurred" [M.1371-5 Annex 2 §5.3.1]. The destination returns the same sequence number in message 7 or 13, transmitted on the channel where the addressed message was received [M.1371-5 Annex 8 §3.5]. If no acknowledgement arrives the sender "should wait 4 s before attempting retries", set the retransmit flag, and try three times by default (configurable 0–3) [M.1371-5 Annex 2 §5.3.1]. Class B "SO" stations do not retry [M.1371-5 Annex 2 §5.3.1]. Repeater stations must all use the same number of repeats precisely so that messages 7 and 13 find their way back to the originator [M.1371-5 Annex 2 §4.6.1.2.1].

On the presentation interface the outcome reaches the application as an `ABK` sentence: `$--ABK,<destination MMSI>,<channel>,<message id>,<sequence number>,<type>*hh`, where type 0 means the addressed message was acknowledged by message 7/13, 1 that it was broadcast but never acknowledged, 2 that it could not be sent (too long), 3 that a requested broadcast (8/14) was completed, and 4 a late acknowledgement [IEC 61162-1 §8 ABK]. Chapter 8 shows `$AIABK,,,8,0,3` from a real whale-notice transmitter; for text the same sentence appears with 12 or 14 in the third field.

## 12.3 Decoding one message by hand

### A message 12

Line 14 of [`data/ch12-safety-text.nmea`](../data/ch12-safety-text.nmea) (from `sample-data/tide3/ais/ais-2010-01-30` line 1428) is the shortest real message 12 in our recordings:

```text
!AIVDM,1,1,,B,<000001H=C0<D5CD,0*54,rnhcml,1264859970.13
```

Sixteen payload characters, fill bits 0, so 96 bits:

```text
     <      0      0      0      0      0      1      H      =      C      0      <      D      5      C      D
001100 000000 000000 000000 000000 000000 000001 011000 001101 010011 000000 001100 010100 000101 010011 010100
```

| Bits | Value | Field |
|---|---|---|
| 0–5 | `001100` = 12 | message 12 |
| 6–7 | `00` = 0 | repeat indicator |
| 8–37 | thirty zero bits = **0** | source MMSI — the transponder has no MMSI programmed |
| 38–39 | `00` = 0 | sequence number |
| 40–69 | `01` + `011000 001101 010011 000000` + `0011` = `010110000011010100110000000011` = **369970179** | destination MMSI (the field starts in the middle of character `1` and ends in the middle of character `<`) |
| 70 | `0` | retransmit flag: first transmission |
| 71 | `0` | spare |
| 72–77 | `010100` = 20 | `T` |
| 78–83 | `000101` = 5 | `E` |
| 84–89 | `010011` = 19 | `S` |
| 90–95 | `010100` = 20 | `T` |

24 text bits make exactly four characters, so `trailing_bits` is 0 and the text is `TEST`. (96 bits is already a multiple of 8, so no padding was needed on the air either.) Now look at line 15, received 5.15 s later:

```text
!AIVDM,1,1,,B,<000001H=C0>D5CD,0*56,rnhcml,1264859975.28
```

One character has changed, `<` → `>`, i.e. `001100` → `001110` in bits 66–71: bit 70, the retransmit flag, is now 1. This is the first retry of an unacknowledged message, 5 s after the original (the standard says wait 4 s, then re-select a slot) [M.1371-5 Annex 2 §5.3.1]. Lines 16 and 17 are retries two and three; no message 13 from 369970179 was ever heard.

### A message 13

Line 5 (from `sample-data/tide3/ais/ais-2010-03-06` line 9990) acknowledges the message 12 on line 4:

```text
!AIVDM,1,1,,B,=5NANh1Gu2jQ,0*0E,rnhcml,1267918972.72
```

Twelve characters, 72 bits:

```text
     =      5      N      A      N      h      1      G      u      2      j      Q
001101 000101 011110 010001 011110 110000 000001 010111 111101 000010 110010 100001
```

| Bits | Value | Field |
|---|---|---|
| 0–5 | `001101` = 13 | message 13 |
| 6–7 | `00` | repeat indicator 0 |
| 8–37 | `010101111001000101111011000000` = **367288000** | source: the station that acknowledges |
| 38–39 | `00` | spare |
| 40–69 | `010101111111010000101100101000` = **368905000** | destination 1: whose message is acknowledged |
| 70–71 | `01` = 1 | sequence number of the acknowledged message |

72 bits leave no room for a second group, so `acks` has one entry: `{mmsi: 368905000, sequence_number: 1}`. Line 4, received 0.20 s earlier, is a message 12 from 368905000 to 367288000 with sequence number 1 — the transaction identifiers match [M.1371-5 Annex 2 §5.3.1].

Compare the message 13 on line 13 (`=000001H=C0<`) with the message 12 on line 14 (`<000001H=C0<D5CD`): the first eleven characters differ only in the message type, because bits 38–39 are 0 in both (sequence number in 12, spare in 13) and bits 40–69 hold 369970179 in both. Bit for bit, an acknowledgement is the addressed header turned around.

## 12.4 The code

The three modules follow the pattern of chapter 3, with two additions: message 7/13 collects a **repeated group** into a JSON array, and messages 12 and 14 share a **text helper** that reports the byte-alignment bookkeeping (`text_bits`, `text_chars`, `trailing_bits`) next to the text.

### Python

[`code/python/aistut/msg_7_13.py`](../code/python/aistut/msg_7_13.py):

```python
HEADER_BITS = 40      # message id .. spare
GROUP_BITS = 32       # destination id (30) + sequence number (2)
MAX_GROUPS = 4


def decode(bits: Bits) -> dict:
    """Decode a message 7 or 13. Raises DecodeError below 72 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), HEADER_BITS + GROUP_BITS)

    acks: list[dict] = []
    pos = HEADER_BITS
    # Loop over the repeated groups. `bits.has` is the guard that stops us
    # at the end of a 72-, 104-, 136- or 168-bit message.
    while len(acks) < MAX_GROUPS and bits.has(pos, GROUP_BITS):
        acks.append({
            "mmsi": bits.uint(pos, 30),
            "sequence_number": bits.uint(pos + 30, 2),
        })
        pos += GROUP_BITS

    return {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
        "acks": acks,
        "trailing_bits": len(bits) - pos,
    }
```

[`code/python/aistut/msg_12.py`](../code/python/aistut/msg_12.py):

```python
HEADER_BITS = 72


def decode_text_field(bits: Bits, start: int) -> dict:
    """Decode a variable-length six-bit text field that runs to the end."""
    text_bits = len(bits) - start
    return {
        "text": bits.text(start, text_bits),
        "text_bits": text_bits,
        "text_chars": text_bits // 6,       # characters on the air, '@' padding included
        "trailing_bits": text_bits % 6,     # byte-alignment leftovers, not text
    }


def decode(bits: Bits) -> dict:
    """Decode a message 12. Raises DecodeError below 72 bits."""
    require_bits(12, len(bits), HEADER_BITS)
    out: dict = {
        "msg_type": bits.uint(0, 6),
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "sequence_number": bits.uint(38, 2),
        "dest_mmsi": bits.uint(40, 30),
        "retransmit": bits.flag(70),
        "spare": bits.uint(71, 1),
    }
    out.update(decode_text_field(bits, HEADER_BITS))
    return out
```

[`code/python/aistut/msg_14.py`](../code/python/aistut/msg_14.py):

```python
from .msg_12 import decode_text_field

HEADER_BITS = 40

BEACON_TEXTS = {
    "SART ACTIVE": "AIS-SART active",
    "SART TEST": "AIS-SART test",
    "MOB ACTIVE": "MOB-AIS active",
    "MOB TEST": "MOB-AIS test",
    "EPIRB ACTIVE": "EPIRB-AIS active",
    "EPIRB TEST": "EPIRB-AIS test",
}


def decode(bits: Bits) -> dict:
    """Decode a message 14. Raises DecodeError below 40 bits."""
    require_bits(14, len(bits), HEADER_BITS)
    out: dict = {
        "msg_type": bits.uint(0, 6),
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
    }
    out.update(decode_text_field(bits, HEADER_BITS))
    out["beacon_text"] = BEACON_TEXTS.get(out["text"])
    return out
```

> **Language notes (Python).** `while len(acks) < MAX_GROUPS and bits.has(pos, GROUP_BITS)` is the idiom for a *tail-variable* message: keep reading fixed-size groups while there is room for another. `acks` is a `list` of `dict`s, which the JSON writer prints as an array of objects. `from .msg_12 import decode_text_field` imports one function from a sibling module in the same package — the leading dot means "relative to this package". `dict.get(key)` returns `None` when the key is absent, so `beacon_text` is `null` for ordinary text without an `if`.

### C++

[`code/cpp/msg_7_13.hpp`](../code/cpp/msg_7_13.hpp):

```cpp
namespace msg_7_13 {
constexpr std::size_t HEADER_BITS = 40;   // message id .. spare
constexpr std::size_t GROUP_BITS = 32;    // destination id (30) + sequence number (2)
constexpr std::size_t MAX_GROUPS = 4;
}  // namespace msg_7_13

// One acknowledgement group: destination MMSI (30 bits) + sequence number (2 bits).
inline Json decode_ack(const Bits& bits, std::size_t pos) {
    return Json::Object{
        {"mmsi", bits.uint(pos, 30)},
        {"sequence_number", bits.uint(pos + 30, 2)},
    };
}

inline Json decode_msg_7_13(const Bits& bits) {
    using namespace msg_7_13;
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), HEADER_BITS + GROUP_BITS);

    Json::Array acks;
    std::size_t pos = HEADER_BITS;
    while (acks.size() < MAX_GROUPS && bits.has(pos, GROUP_BITS)) {
        acks.push_back(decode_ack(bits, pos));
        pos += GROUP_BITS;
    }

    return Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
        {"acks", acks},
        {"trailing_bits", bits.size() - pos},
    };
}
```

[`code/cpp/msg_12.hpp`](../code/cpp/msg_12.hpp) and [`msg_14.hpp`](../code/cpp/msg_14.hpp) (the header parts mirror Python exactly; only the shared helper and the beacon lookup are shown):

```cpp
inline Json decode_text_field(const Bits& bits, std::size_t start) {
    std::size_t text_bits = bits.size() - start;
    return Json::Object{
        {"text", bits.text(start, text_bits)},
        {"text_bits", text_bits},
        {"text_chars", text_bits / 6},      // characters on the air, '@' padding included
        {"trailing_bits", text_bits % 6},   // byte-alignment leftovers, not text
    };
}
```

```cpp
inline const std::map<std::string, std::string> BEACON_TEXTS = {
    {"SART ACTIVE", "AIS-SART active"},
    // ... four more ...
    {"EPIRB TEST", "EPIRB-AIS test"},
};

inline Json decode_msg_14(const Bits& bits) {
    constexpr std::size_t HEADER_BITS = 40;
    require_bits(14, bits.size(), HEADER_BITS);
    Json out = Json::Object{ /* msg_type, repeat_indicator, mmsi, spare */ };
    out.update(decode_text_field(bits, HEADER_BITS));
    std::string s = bits.text(HEADER_BITS, bits.size() - HEADER_BITS);
    std::optional<std::string> beacon;
    auto it = BEACON_TEXTS.find(s);
    if (it != BEACON_TEXTS.end()) beacon = it->second;
    out["beacon_text"] = beacon;
    return out;
}
```

> **Language notes (C++).** `Json::Array` is a `std::vector<Json>`; `push_back` is `list.append`, and a `Json::Object{...}` brace-initialiser builds the per-station object in place. The constants are `constexpr std::size_t` inside a small `namespace msg_7_13` so that other headers can have their own `HEADER_BITS` without a clash — header-only code shares one global namespace, so names must be scoped. `acks.size()` is unsigned, which is why `MAX_GROUPS` is `std::size_t` too: comparing signed with unsigned draws a warning under `-Wextra`. `std::map::find` returns an iterator; comparing it with `end()` is the "key not found" test, and an empty `std::optional<std::string>` prints as `null`. Our `Json` class has no string accessor, so message 14 decodes the short text a second time instead of reading it back from the object.

### Rust

[`code/rust/src/msg_7_13.rs`](../code/rust/src/msg_7_13.rs):

```rust
const HEADER_BITS: usize = 40; // message id .. spare
const GROUP_BITS: usize = 32; // destination id (30) + sequence number (2)
const MAX_GROUPS: usize = 4;

pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), HEADER_BITS + GROUP_BITS)?;

    let mut acks: Vec<Json> = Vec::new();
    let mut pos = HEADER_BITS;
    while acks.len() < MAX_GROUPS && bits.has(pos, GROUP_BITS) {
        let mut ack = Obj::new();
        put(&mut ack, "mmsi", bits.uint(pos, 30));
        put(&mut ack, "sequence_number", bits.uint(pos + 30, 2));
        acks.push(Json::Obj(ack));
        pos += GROUP_BITS;
    }

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    put(&mut out, "acks", acks);
    put(&mut out, "trailing_bits", bits.len() - pos);
    Ok(Json::Obj(out))
}
```

[`code/rust/src/msg_12.rs`](../code/rust/src/msg_12.rs) exports the helper; [`msg_14.rs`](../code/rust/src/msg_14.rs) imports it and matches the beacon strings:

```rust
pub fn decode_text_field(bits: &Bits, start: usize) -> Obj {
    let text_bits = bits.len() - start;
    let mut out = Obj::new();
    put(&mut out, "text", bits.text(start, text_bits));
    put(&mut out, "text_bits", text_bits);
    put(&mut out, "text_chars", text_bits / 6); // characters on the air, '@' padding included
    put(&mut out, "trailing_bits", text_bits % 6); // byte-alignment leftovers, not text
    out
}
```

```rust
use crate::msg_12::decode_text_field;

pub fn beacon_text(text: &str) -> Option<&'static str> {
    match text {
        "SART ACTIVE" => Some("AIS-SART active"),
        "SART TEST" => Some("AIS-SART test"),
        "MOB ACTIVE" => Some("MOB-AIS active"),
        "MOB TEST" => Some("MOB-AIS test"),
        "EPIRB ACTIVE" => Some("EPIRB-AIS active"),
        "EPIRB TEST" => Some("EPIRB-AIS test"),
        _ => None,
    }
}

pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(14, bits.len(), HEADER_BITS)?;
    let mut out = Obj::new();
    // ... msg_type, repeat_indicator, mmsi, spare ...
    out.extend(decode_text_field(bits, HEADER_BITS));
    let text = bits.text(HEADER_BITS, bits.len() - HEADER_BITS);
    put(&mut out, "beacon_text", beacon_text(&text));
    Ok(Json::Obj(out))
}
```

> **Language notes (Rust).** `Vec<Json>` is the growable array; it must be declared `let mut` because we `push` into it, and so must `pos` because we add to it — every binding is immutable unless you say otherwise. `const NAME: usize = …;` is a typed compile-time constant; `usize` is what `bits.len()` and `bits.has()` work in, so no casts are needed in the loop. A function is invisible outside its module unless it is `pub`, which is why `decode_text_field` is `pub fn` and `msg_14.rs` can `use crate::msg_12::decode_text_field`. `match` works on string slices as well as numbers; the arms return `Option<&'static str>` and the `From<Option<T>>` conversion in `jsonout.rs` prints `None` as `null`.

### TypeScript

[`code/typescript/src/msg_7_13.ts`](../code/typescript/src/msg_7_13.ts):

```typescript
const HEADER_BITS = 40; // message id .. spare
const GROUP_BITS = 32;  // destination id (30) + sequence number (2)
const MAX_GROUPS = 4;

export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, HEADER_BITS + GROUP_BITS);

  const acks: JsonObject[] = [];
  let pos = HEADER_BITS;
  while (acks.length < MAX_GROUPS && bits.has(pos, GROUP_BITS)) {
    acks.push({
      mmsi: bits.uint(pos, 30),
      sequence_number: bits.uint(pos + 30, 2),
    });
    pos += GROUP_BITS;
  }

  return {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
    acks,
    trailing_bits: bits.length - pos,
  };
}
```

[`code/typescript/src/msg_12.ts`](../code/typescript/src/msg_12.ts) and [`msg_14.ts`](../code/typescript/src/msg_14.ts):

```typescript
export function decodeTextField(bits: Bits, start: number): JsonObject {
  const textBits = bits.length - start;
  return {
    text: bits.text(start, textBits),
    text_bits: textBits,
    text_chars: Math.floor(textBits / 6), // characters on the air, '@' padding included
    trailing_bits: textBits % 6,          // byte-alignment leftovers, not text
  };
}
```

```typescript
import { decodeTextField } from "./msg_12";

export const BEACON_TEXTS: { readonly [text: string]: string } = {
  "SART ACTIVE": "AIS-SART active",
  // ... four more ...
  "EPIRB TEST": "EPIRB-AIS test",
};

export function decode(bits: Bits): JsonObject {
  requireBits(14, bits.length, HEADER_BITS);
  const out: JsonObject = { /* msg_type, repeat_indicator, mmsi, spare */ };
  Object.assign(out, decodeTextField(bits, HEADER_BITS));
  out["beacon_text"] = BEACON_TEXTS[out["text"] as string] ?? null;
  return out;
}
```

> **Language notes (TypeScript).** `JsonObject[]` is "array of JsonObject"; `const acks` is fine even though we `push` into it, because `const` protects the binding, not the contents — but `pos` must be `let`. JavaScript has only floating-point numbers, so `textBits / 6` would be `18.666…`; integer division is spelled `Math.floor(textBits / 6)`. Looking up a missing key in a plain object yields `undefined`, and `?? null` (nullish coalescing) converts that into the `null` the other languages print. `out["text"] as string` is a *type assertion*: the index signature only promises "some JsonValue", and we tell the compiler we know it is a string.

### Where the four differ

* **Repeated groups.** All four use the same `while … has(pos, 32)` loop; the array type is `list[dict]`, `Json::Array`, `Vec<Json>` and `JsonObject[]`.
* **Integer division.** Python `//`, C++ and Rust `/` on unsigned integers, TypeScript `Math.floor(a / b)`.
* **Sharing a helper.** Python and TypeScript import a function from the sibling module; Rust needs `pub` and a `use crate::…` path; C++ `#include`s the whole header, and the helper is `inline` so that two translation units could both see it.
* **Optional text.** `dict.get` → `None`; `std::map::find` + `std::optional`; `match` → `Option`; object index + `?? null`.

## 12.5 Sample data and results

[`data/ch12-safety-text.nmea`](../data/ch12-safety-text.nmea) has five sections, each introduced by a `#` comment naming its source:

* **A** — `sample-data/tide3/ais/ais-2010-03-06` lines 9988–9998 (file lines 3–11): nine sentences in which 368905000 sends message 12 to 367288000 and 367288000 answers with message 13.
* **B** — `sample-data/tide3/ais/ais-2010-01-30` lines 1384, 1428, 1429, 1432, 1433 and 1466 (file lines 13–18): a transponder with MMSI 0 — the same unit whose all-zero messages 3 and 5 fill that log — and 369970179.
* **C** — `sample-data/tide3/ais/ais-2010-01-15` lines 208–209 (file lines 20–21): message 7 from the MMSI-0 unit acknowledging a message 6 from coast station 003669971 (a 00MIDxxxx coast-station identity with MID 366, United States [M.585-9]); the message 6 itself was not heard.
* **D** — `sample-data/RightwhaleAisProject/20081013.log` lines 306791, 462033 and 479818 (file lines 23–25): the only message 12, 14 and 7 in a full day of the USCG N-AIS feed from Boston, all three bit-corrupt.
* **E** — file lines 27–30: **synthetic** message 14 sentences. No readable message 14 exists anywhere in `sample-data/` — the 71 payloads that start with the type-14 code are all noise from the marginal `nhjel` receiver or corrupt N-AIS lines like the one in section D. We therefore built three by hand: header bits `001110 00 <30-bit MMSI> 00`, then each character's Table 47 code, then zero bits up to the next multiple of 8 (the radio-layer rule), then zero bits up to the next multiple of 6 with the count written in the fill-bit field, then the armouring of chapter 2 and the checksum of chapter 1. The MMSI 970010001 follows the AIS-SART pattern 970xxyyyy with manufacturer 01 and serial 0001 [M.1371-5 Annex 1 §2.1.6]; the 70-character broadcast from MMSI 0 is split into two sentences at 60 payload characters to exercise the assembler.

Run any decoder over it:

```sh
python3 code/python/decode.py data/ch12-safety-text.nmea
code/cpp/build/decode data/ch12-safety-text.nmea
code/rust/target/release/decode data/ch12-safety-text.nmea
node code/typescript/dist/decode.js data/ch12-safety-text.nmea
```

All four print the 23 lines of [`tests/expected/ch12-safety-text.jsonl`](../tests/expected/ch12-safety-text.jsonl). Selected objects:

**A real conversation** (file lines 4 and 5):

```json
{"ais":{"dest_mmsi":367288000,"mmsi":368905000,"msg_type":12,"repeat_indicator":0,"retransmit":false,"sequence_number":1,"spare":0,"text":"MSG FROM 368905000","text_bits":112,"text_chars":18,"trailing_bits":4},"channel":"B","fragments":1,"line":4,"own_ship":false,"payload_bits":184,"talker":"AI","uscg":{"station":"nhcml","station_kind":"receiver","time":1267918972.52}}
{"ais":{"acks":[{"mmsi":368905000,"sequence_number":1}],"mmsi":367288000,"msg_type":13,"repeat_indicator":0,"spare":0,"trailing_bits":0},"channel":"B","fragments":1,"line":5,"own_ship":false,"payload_bits":72,"talker":"AI","uscg":{"station":"nhcml","station_kind":"receiver","time":1267918972.72}}
```

`payload_bits` 184 = 72 + 18 × 6 + 4: the message was 180 bits of content, padded to a byte boundary on the air — `trailing_bits: 4` is exactly the padding predicted in §12.2. The acknowledgement came 0.20 s later on the same channel B, as required [M.1371-5 Annex 8 §3.5]. The text is an installer's test string (it names its own sender). Across section A the sequence number alternates 1, 0, 1, 0 and every message is acknowledged, yet from line 6 on every message 12 has `retransmit: true` — see §12.6.

**Retries without an answer** (file lines 14 and 15, section B):

```json
{"ais":{"dest_mmsi":369970179,"mmsi":0,"msg_type":12,"repeat_indicator":0,"retransmit":false,"sequence_number":0,"spare":0,"text":"TEST","text_bits":24,"text_chars":4,"trailing_bits":0}, … "line":14,"payload_bits":96 … "time":1264859970.13}}
{"ais":{"dest_mmsi":369970179,"mmsi":0,"msg_type":12,"repeat_indicator":0,"retransmit":true,"sequence_number":0,"spare":0,"text":"TEST","text_bits":24,"text_chars":4,"trailing_bits":0}, … "line":15,"payload_bits":96 … "time":1264859975.28}}
```

Lines 16 and 17 repeat line 15 at 1264859980.45 and 1264859985.63 — the original plus the default three retries at roughly 5 s spacing [M.1371-5 Annex 2 §5.3.1]. Line 13, the message 13 from MMSI 0 (`acks: [{"mmsi":369970179,"sequence_number":0}]`, received seventeen minutes earlier) acknowledges a message 12 *from* 369970179 that our receiver never heard: a third-party logger sees only half of an addressed exchange.

**A binary acknowledgement** (file line 20):

```json
{"ais":{"acks":[{"mmsi":3669971,"sequence_number":0}],"mmsi":0,"msg_type":7,"repeat_indicator":0,"spare":0,"trailing_bits":0},"channel":"B","fragments":1,"line":20,"own_ship":false,"payload_bits":72,"talker":"AI","uscg":{"station":"nhcml","station_kind":"receiver","time":1263531364.24}}
```

The JSON prints the 30-bit field as the integer 3669971; written as a nine-digit MMSI it is 003669971, a coast station. Line 21 is the same acknowledgement 0.5 s later on channel A.

**Corruption that passes the checksum** (file line 25):

```json
{"ais":{"acks":[{"mmsi":1045687682,"sequence_number":0}],"mmsi":240894545,"msg_type":7,"repeat_indicator":3,"spare":1,"trailing_bits":8}, … "line":25,"payload_bits":80 …}
```

80 bits is not a legal length for message 7 (72, 104, 136 or 168) — hence `trailing_bits: 8` — the acknowledged "MMSI" 1045687682 exceeds the nine-digit maximum 999999999, and the spare is 1 where it "should be set to zero" [M.1371-5 Annex 8 Table 56]. Line 23 (message 12, text `/J_?P`, `trailing_bits: 2`) and line 24 (message 14, source 934594343, `spare: 3`) are the same kind of debris.

**The synthetic beacon and the long broadcast** (file lines 27 and 30):

```json
{"ais":{"beacon_text":"AIS-SART active","mmsi":970010001,"msg_type":14,"repeat_indicator":0,"spare":0,"text":"SART ACTIVE","text_bits":72,"text_chars":12,"trailing_bits":0},"channel":"A","fragments":1,"line":27,"own_ship":false,"payload_bits":112,"talker":"AI"}
{"ais":{"beacon_text":null,"mmsi":0,"msg_type":14,"repeat_indicator":0,"spare":0,"text":"TUTORIAL EXAMPLE - SYNTHETIC SAFETY BROADCAST - NOT TRANSMITTED ON AIR","text_bits":424,"text_chars":70,"trailing_bits":4},"channel":"A","fragments":2,"line":30,"own_ship":false,"payload_bits":464,"talker":"AI"}
```

`SART ACTIVE` is 11 characters = 66 bits; 40 + 66 = 106 is not a multiple of 8, so six zero bits were added to reach 112. Six zero bits are a whole character — `@` — which is why `text_chars` is 12 while the text has 11 characters and `trailing_bits` is 0 [M.1371-5 Annex 5 §5.1 Table 25 Note 1]. `SART TEST` on line 28 (9 characters, 94 bits → 96) gets 2 padding bits instead and reports `text_chars: 9, trailing_bits: 2`. The two-sentence broadcast shows `fragments: 2`; the assembler of chapter 2 needed no change for text.

## 12.6 Pitfalls and real-world deviations

* **Noise looks like text messages.** Message types 12, 13, 14 and 7 are rare on the air, so among the payloads that *claim* those types a large share are corrupt receptions with a valid checksum (the checksum protects the sentence, not the radio packet). Validate before you trust: `payload_bits` ≥ 72 (12, 7, 13) or 40 (14); `trailing_bits` 0 for 7/13 and 0, 2 or 4 for 12/14; spare fields 0; MMSIs ≤ 999 999 999 and of a plausible format [M.585-9]; text made of printable characters. Section D fails every one of these tests. In our recordings the only readable text messages come from a single receiver (`nhcml`, New Castle NH) in early 2010, and all of them are test traffic — which is representative of what a coastal receiver hears.
* **Retransmit flags that lie.** In section A every message 12 after the first carries `retransmit: true` although each was acknowledged 0.2 s earlier. Either the sender never processed the acknowledgements, or it re-sent the same text as a new transaction and set the flag anyway. Do not use the flag to infer link quality without also pairing the acknowledgements yourself (exercise 2).
* **Retry spacing.** The standard says wait 4 s [M.1371-5 Annex 2 §5.3.1]; the unit in section B retried every 5.15–5.18 s, the one in section A every 3.3–4.4 s. Slot selection adds jitter on top of the timer.
* **One-sided conversations.** A message 13 proves that an addressed message existed even when you did not log it (section B, line 13). Conversely, the absence of a 13 in your log does not mean the message went unacknowledged — the acknowledgement is transmitted on the channel where the message was received [M.1371-5 Annex 8 §3.5], which may be the one your receiver was not listening to at that instant.
* **MMSI 0.** The MMSI-0 station in sections B and C is a transponder that was installed but never programmed; its messages 3 and 5 are all zeros apart from the type. Such units exist in every port, and any software that keys a table by MMSI should expect several of them to collide on 0.
* **Padding that becomes a character.** When 6 padding bits are needed they decode as `@` and are counted in `text_chars` (line 27). Some transponders pad with spaces instead of `@`; `Bits.text` strips trailing spaces for that reason. Text that *legitimately* contains `@` cannot be represented — Table 47 reserves it [M.1371-5 Annex 8 Table 47].
* **Wrong fill-bit counts.** For fixed-length messages a wrong fill-bit field shows up as an odd length; for 12 and 14 "there is no way to detect that the pad character might be wrong" — it manifests as a lost last character or a spurious `@` [AIVDM "Interaction with AIVDM padding"]. If a text ends one character early, suspect the receiver.
* **No lower case, no accents.** Six-bit ASCII has 26 letters, digits and a few symbols [M.1371-5 Annex 8 Table 47]. Encoders that accept lower case silently upper-case it or, worse, map it to the symbols at codes 27–31 (`[\]^_`), which is one reason real-world texts contain "line noise" [AIVDM "Type 12"].
* **Length limits are per message, not per sentence.** A 156-character message 12 is 1008 bits and arrives as five `!AIVDM` fragments; the limits of Tables 62 and 64 are about radio slots, and a transponder that is asked for more answers with a negative `ABK` (type 2) rather than truncating [M.1371-5 Annex 2 §5.2.1], [IEC 61162-1 §8 ABK].

## 12.7 Exercises

1. **Pencil and paper.** Decode line 13 of the data file (`=000001H=C0<`) by hand as in §12.3 and confirm that it differs from line 14 only in bits 0–5. Then decode line 28 (`>>M4eTA<59B1@E=@`, fill bits 0) and explain why `text_chars` is 9 but `trailing_bits` is 2.
2. **Python — pair the conversation.** Read the JSON lines of `tests/expected/ch12-safety-text.jsonl`, and for every message 12 look for a later message 13 whose `mmsi` equals the 12's `dest_mmsi` and whose `acks` contain `{mmsi, sequence_number}` matching the 12's `mmsi` and `sequence_number`, within 10 s. Print, per message 12, the acknowledgement delay or "unacknowledged". You should find every message in section A acknowledged within 0.3 s and every message in section B unacknowledged.
3. **C++ — a validity filter.** Add a free function `bool plausible_ack(const Json&)` (in your own file, not in the tutorial's modules) that returns false when `trailing_bits` ≠ 0, `spare` ≠ 0 or any `acks[i].mmsi` > 999999999. Run it over the output for the data file; only line 25 should be rejected among the messages 7/13.
4. **Rust — beacon cross-check.** Write a unit test that feeds the six beacon strings through `msg_14::beacon_text` and asserts the six results, then extend `decode` (in a copy) to add `"beacon_mmsi_ok": true/false` when `beacon_text` is set, by checking that the MMSI begins with 970 (SART), 972 (MOB) or 974 (EPIRB) [M.585-9]. Line 27 should pass; a `SART ACTIVE` sent from 368905000 should fail.
5. **TypeScript — an encoder.** Write `encode14(mmsi: number, text: string): string` that performs the steps listed in §12.5 for section E and returns a complete `!AIVDM` sentence. Reproduce line 27 exactly (`>>M4eTA<59B04=@UHD0,2*4F`). Then encode a 17-character text and check against Table 64 that it needs two slots.
6. **Slots.** Using Tables 62 and 64, how many slots do the messages on lines 4, 14, 27 and 30 occupy? (Count characters, not bits: 18 → 2 slots for a message 12; 4 → 1; 11 → 1 for a message 14; 70 → 3.) Which of them could a Class B "CS" transponder have sent [M.1371-5 Annex 7 Table 42]?

## 12.8 References

* [M.1371-5] Annex 8 §3.5 and Table 56 (messages 7 and 13); §3.10, Tables 61–62 (message 12); §3.11 (13 refers to 7); §3.12, Tables 63–64 (message 14, beacon texts); Table 47 (six-bit ASCII). Annex 2 §4.6.1.2.1 (repeaters and acknowledgements), §5.2.1 and Table 21 (slot limits, byte boundaries), §5.3.1 (addressed messages, sequence numbers, retries). Annex 5 §5.1 Table 25 Note 1 (spare bits of 0/2/4/6 and the `@` character). Annex 1 §2.1.6–2.1.8 (AIS-SART, MOB-AIS, EPIRB-AIS identities and texts). Annex 7 §4.3.3.6 Table 42 and §4.3.3.7 Table 43 (Class B "CS" use of messages 12, 13, 14).
* [M.585-9] — MMSI formats: ship stations, coast stations 00MIDxxxx, AIS-SART 970xxyyyy, MOB 972xxyyyy, EPIRB 974xxyyyy.
* [IEC 61162-1] §8 — `ABM`, `BBM` and `ABK` sentence formatters (acknowledgement types 0–4).
* [AIVDM] "Type 7", "Type 12", "Type 13", "Type 14", "AIS Payload Byte Alignment, Padding, and Bit Stuffing", "Interaction with AIVDM padding".
* [Toils] — six-bit text as "the first serious blob of accidental complexity in AIS"; types 12/14 as common header plus packed six-bit text.
* [A.1106(29)] ¶15–19 (short safety-related messages), ¶46–47 (VTS text messages).
* [Safety Alerts] — USCG Safety Alert 05-10, *AIS text messaging concerns: usage during navigation and emergencies* (27 May 2010).
* [Circ.289] — the structured alternative to free text for shore broadcasts (chapter 9).
* [AISMiner] — filtering the N-AIS feed by message type and content.
* [Appendix B](appendix-b-six-bit-tables.md) — armouring and six-bit ASCII tables.
