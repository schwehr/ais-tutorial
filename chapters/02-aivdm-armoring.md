# 2. AIVDM and six-bit armouring

> **What you will learn**
>
> * How an AIS message travels inside an NMEA 0183 sentence: the seven
>   fields of `!AIVDM`/`!AIVDO`, the talkers, the channel codes and the
>   fill-bit count.
> * How the "armoured" payload turns back into bits — subtract 48, then 8
>   — and why the characters `X` to `_` can never appear.
> * How a long message is split over several sentences and how a decoder
>   reassembles it (and what happens when a fragment is lost).
> * The difference between payload armouring and the six-bit ASCII used
>   *inside* text fields.
> * The 27 AIS message types, their sizes, and which chapter decodes each.
> * The `bits` and `vdm` modules and the `decode` driver in Python, C++,
>   Rust and TypeScript, including the deterministic JSON writer that lets
>   the four programs be compared byte for byte.

## 2.1 Where this fits

Chapter 1 ended with a parser that can split any NMEA 0183 sentence into
talker, formatter, data fields and checksum. For GPS sentences that was the
whole job: `$GPGGA` carries latitude and longitude as decimal text. AIS is
different. An AIS transceiver is a VHF radio that exchanges *binary*
packets with other transceivers in time slots [M.1371-5 Annex 2 §3.3]. When
a receiver hands such a packet to a computer it has to fit 168 (or 424, or
1 008) raw bits into the 82 printable characters that an NMEA sentence
allows [IEC 61162-1 §7]. The `VDM` ("VHF Data-link Message") and `VDO`
("VHF Data-link Own-vessel") sentence formatters do exactly that
[IEC 61162-1 §8 (VDM, VDO)], [AIVDM "AIVDM/AIVDO Sentence Layer"]:

```text
!AIVDM,1,1,,A,15N91b?P00JRlJ4E56Qe0?v20@14,0*2F
```

Everything an AIS decoder ever sees arrives in this envelope. This chapter
is about the envelope and about getting from the payload text to a string
of bits; the next chapters are about what the bits mean.

### Operational use

The `!AIVDM` sentence is the interface between an AIS unit and everything
else on a bridge: the radar/ARPA display that overlays AIS targets, the
ECDIS chart, the pilot's portable unit plugged into the pilot plug, and the
voyage data recorder that logs it [A.1106(29)]. Ashore, vessel traffic
services (VTS) receive `!AIVDM` streams from coastal receivers; the US Coast
Guard's national network (chapter 5) forwards tens of millions of these
sentences a day. `!AIVDO` is the transceiver reporting what it *itself*
just transmitted, which is how a bridge display knows the own-ship data
that is going out to the world.

### Unattended / automated uses

Almost no human ever reads an `!AIVDM` sentence; machines do. Shore
networks concatenate feeds from hundreds of receivers, so the same message
arrives several times with different metadata (chapter 5). Data loggers
such as the one that produced `sample-data/` just append a station id and
a timestamp after the checksum. Satellite AIS ground stations, web sites
such as AISHub and MarineTraffic, and research archives all exchange
`!AIVDM` text because it is the lowest common denominator — and because it
is the lowest common denominator, they also propagate every malformed
sentence a cheap receiver produces (§2.6). An automated consumer therefore
has to validate the envelope (field count, checksum, fill bits, payload
characters, bit length against message type) before it believes a single
field, and must assume that fragments of multi-sentence messages can be
lost, duplicated or interleaved.

## 2.2 The specification

### 2.2.1 The sentence, field by field

[IEC 61162-1 §8 (VDM, VDO)], [AIVDM "AIVDM/AIVDO Sentence Layer"]:

| # | Example | Field | Values | Notes |
|---|---|---|---|---|
| 0 | `!AIVDM` | Address: `!`, talker, formatter | `VDM` = received from another station; `VDO` = own station's transmission | `!` instead of `$` marks an *encapsulation* sentence [IEC 61162-1 §7] |
| 1 | `1` | Total number of sentences (fragments) in this message | 1–9 | 1 for the vast majority of messages |
| 2 | `1` | This sentence's number within the message | 1–total | One-based |
| 3 | *(empty)* | Sequential message identifier | 0–9 or empty | Shared by all fragments of one message; empty when the message fits in one sentence |
| 4 | `A` | AIS radio channel | `A`, `B`, `1`, `2`, or empty | `A`/`1` = AIS 1 = 161.975 MHz (channel 87B); `B`/`2` = AIS 2 = 162.025 MHz (channel 88B) |
| 5 | `15N91b…@14` | Encapsulated payload | up to ~62 characters per sentence | Six bits per character, §2.2.3 |
| 6 | `0` | Number of fill bits | 0–5 | Low-order bits of the *last* payload character that are padding, §2.2.4 |
| — | `*2F` | Checksum | two hex digits | XOR of everything between `!` and `*` [IEC 61162-1 §7.2.3] |

**Talkers.** The two letters after `!` identify the kind of equipment that
produced the sentence [AIVDM "Talker IDs"]:

| Talker | Equipment |
|---|---|
| `AI` | Mobile AIS station (Class A, Class B, SAR aircraft) — by far the most common, and used by most receivers too |
| `AB` | AIS base station (NMEA 4.0) |
| `BS` | AIS base station (older form, deprecated in NMEA 4.0) |
| `AN` | AIS aid-to-navigation station |
| `AR` | AIS receiving station (a receive-only shore unit) |
| `AX` | AIS repeater station |
| `SA` | Physical shore AIS station |
| `AD`, `AS`, `AT` | Dependent base station, limited base station, transmitting station (rare) |

The talker tells you about the *box that wrote the sentence*, not about
the ship that sent the radio message — that comes from the maritime mobile service identity (MMSI) inside the
payload. In `data/ch02-aivdm-samples.nmea` line 9 an aid-to-navigation unit
on the bench reports its own transmission as `!ANVDO`.

**Channel codes.** The standard allows `A`, `B`, `1`, `2` or an empty
field [AIVDM "AIVDM/AIVDO Sentence Layer"]. Real equipment is more
creative. The L-3 AtoN unit of line 9 writes `X` (and elsewhere `Y`) in
its `!ANVDO` output; the Furuno Class A unit whose HyperTerminal capture
supplied line 8 writes `A` on the first `!AIVDO` of the capture and leaves
the field empty on the following ones (the empty-channel lines are not in
`data/`; they are lines 2–3 of the source file). A decoder must treat the
channel as an opaque string and never index a table with it. Our decoders
copy it into the output unchanged: `"channel":"X"` on line 9.

**Multi-sentence messages.** An NMEA sentence may be at most 82
characters including `!` and the final CR LF [IEC 61162-1 §7]. After
`!AIVDM,2,1,1,A,` and `,0*01` there is room for about 60 payload
characters, i.e. 360 bits. Message 5 (424 bits) therefore always needs two
sentences; message 19 (312 bits, 52 characters) just fits in one, although
some equipment splits it anyway (lines 6–7 of the sample file); and a
message 8 with the longest permitted binary payload (1 008 bits, 168
characters) needs three. All fragments of one
message carry the same total (field 1), consecutive numbers (field 2) and
the same sequential id (field 3); only the last fragment may have non-zero
fill bits. The sequential id is a counter 0–9 that the *sender of the
sentence* increments for each multi-sentence message, so two receivers
feeding the same log will happily reuse the same ids. That is why our
`Assembler` (§2.4) keys pending fragments on the triple (talker, channel,
sequential id) rather than on the id alone: it is the best approximation
of "same source" that the sentence offers. Lines 4–5 and 6–7 of the sample
file are two such pairs.

### 2.2.2 The 27 message types

The first six bits of every payload are the message identifier
[M.1371-5 Annex 8 §3]. Table 46 of [M.1371-5 Annex 8] lists 27 of them.
Sizes are from the individual message tables of Annex 8 §3; "M" means
transmitted by mobile stations, "B" by base stations; the access-scheme
acronyms (SOTDMA, self-organising TDMA, and the rest) are explained in
chapter 3.

| ID | Name [M.1371-5 Annex 8 Table 46] | Bits | M/B | Chapter |
|---|---|---|---|---|
| 1 | Position report (scheduled, SOTDMA) | 168 | M | [3](03-position-reports-1-2-3.md) |
| 2 | Position report (assigned schedule) | 168 | M | [3](03-position-reports-1-2-3.md) |
| 3 | Position report (special; response to interrogation) | 168 | M | [3](03-position-reports-1-2-3.md) |
| 4 | Base station report | 168 | B | [6](06-messages-4-11-base-station.md) |
| 5 | Static and voyage related data | 424 | M | [7](07-message-5-static-voyage.md) |
| 6 | Binary addressed message | 88–1 008 | M/B | [8](08-messages-6-8-binary.md), [9](09-application-specific-messages.md) |
| 7 | Binary acknowledgement | 72–168 | M/B | [12](12-messages-12-13-14-safety-text.md) |
| 8 | Binary broadcast message | 56–1 008 | M/B | [8](08-messages-6-8-binary.md), [9](09-application-specific-messages.md) |
| 9 | Standard SAR aircraft position report | 168 | M | [10](10-message-9-sar-aircraft.md) |
| 10 | UTC/date inquiry | 72 | M/B | [11](11-messages-10-15-16-interrogation-assignment.md) |
| 11 | UTC/date response | 168 | M | [6](06-messages-4-11-base-station.md) |
| 12 | Addressed safety related message | 72–1 008 | M/B | [12](12-messages-12-13-14-safety-text.md) |
| 13 | Safety related acknowledgement | 72–168 | M/B | [12](12-messages-12-13-14-safety-text.md) |
| 14 | Safety related broadcast message | 40–1 008 | M/B | [12](12-messages-12-13-14-safety-text.md) |
| 15 | Interrogation | 88–160 | M/B | [11](11-messages-10-15-16-interrogation-assignment.md) |
| 16 | Assignment mode command | 96 or 144 | B | [11](11-messages-10-15-16-interrogation-assignment.md) |
| 17 | DGNSS broadcast binary message | 80–816 | B | [13](13-messages-17-20-22-23-base-station-control.md) |
| 18 | Standard Class B position report | 168 | M | [14](14-messages-18-19-24-class-b.md) |
| 19 | Extended Class B position report | 312 | M | [14](14-messages-18-19-24-class-b.md) |
| 20 | Data link management message | 72–160 | B | [13](13-messages-17-20-22-23-base-station-control.md) |
| 21 | Aids-to-navigation report | 272–360 | M/B | [15](15-message-21-aton.md) |
| 22 | Channel management | 168 | B | [13](13-messages-17-20-22-23-base-station-control.md) |
| 23 | Group assignment command | 160 | B | [13](13-messages-17-20-22-23-base-station-control.md) |
| 24 | Static data report (part A / part B) | 160 / 168 | M/B | [14](14-messages-18-19-24-class-b.md) |
| 25 | Single slot binary message | ≤ 168 | M/B | [16](16-messages-25-26-27.md) |
| 26 | Multiple slot binary message with communication state | ≤ 1 064 | M/B | [16](16-messages-25-26-27.md) |
| 27 | Position report for long-range applications | 96 | M | [16](16-messages-25-26-27.md) |

Identifiers 0 and 28–63 are undefined. A payload whose first six bits
decode to one of them is noise or corruption (our driver reports
`unknown message type N`).

### 2.2.3 Payload armouring

Each payload character carries six bits. The sender takes the 6-bit value
*v* (0–63), adds 48, and if the result is greater than 87 adds another 8;
the receiver reverses it: subtract 48 from the ASCII code, and if the
result is greater than 40 subtract 8 more [IEC 61162-1 §7.3.3],
[AIVDM "AIVDM/AIVDO Payload Armoring"]. The full table is in
[Appendix B.1](appendix-b-six-bit-tables.md#b1-payload-armouring-sentence-payload-character--6-bit-value);
the shape of it is:

```text
value   0 ... 39  ->  ASCII 48 ... 87   '0' '1' ... '9' ':' ';' '<' '=' '>' '?' '@' 'A' ... 'W'
value  40 ... 63  ->  ASCII 96 ... 119  '`' 'a' ... 'w'
```

The eight ASCII codes 88–95 (`X Y Z [ \ ] ^ _`) are skipped. The gap exists
because some early equipment could not pass those characters cleanly; the
consequence for us is that a payload containing any of them — or anything
below `0` or above `w` — is corrupt, and our decoders report
`illegal armour character 'X'` rather than guessing. The six bits of each
character are concatenated most-significant bit first to form the message.

Two remarks that save hours of confusion:

* `@` is a *valid* payload character (value 16, `010000`). It has nothing
  to do with the `@` padding of text fields (§2.2.5).
* Payload characters are case-sensitive: `a` is 41, `A` is 17.

### 2.2.4 Fill bits and byte alignment

Six does not divide most message lengths, so the last character usually
carries a few bits that are not part of the message. Field 6 says how many
(0–5); the decoder drops that many low-order bits of the last character.
A 168-bit message is exactly 28 characters with 0 fill bits. Message 21
in line 9 has 272 bits: 46 characters carry 276 bits, so the sentence says
`4` fill bits.

The subtlety is that there are *two* layers of padding. The radio layer
already pads every transmitted message to a multiple of eight bits
[M.1371-5 Annex 2 §3.3.7] ("unused bits in the last byte should be set to
zero in order to preserve byte boundary"); the sentence layer then pads
that to a multiple of six. For fixed-length messages that are multiples of
8 (168, 424, 312…) you never notice. For variable-length text and binary
messages (6, 8, 12, 14, 25, 26) and for the odd 110-bit variant of message
15, the receiver may legitimately hand you up to seven bits of zeros after
the last real field [AIVDM "AIS Payload Byte Alignment, Padding, and Bit
Stuffing"], [Toils]. And receivers get the fill count wrong: the most
common error seen on public feeds is a count two too small, which makes a
message 5 appear as 426 bits instead of 424 [AIVDM "Interaction with AIVDM
padding"]. Our decoders therefore check only a *minimum* length per
message (`require_bits`) and ignore trailing bits.

Bit stuffing (a zero inserted after five consecutive ones on the air
[M.1371-5 Annex 2 §3.2.2.1]) is removed by the receiver and is never
visible in a payload. If you ever see it, the receiver firmware is broken.

### 2.2.5 Six-bit ASCII inside text fields

Ship names, call signs, destinations, safety text and AtoN names are
packed six bits per character using a *different* table
[M.1371-5 Annex 8 Table 47]: values 0–31 are `@ A B … Z [ \ ] ^ _` (ASCII
64–95) and values 32–63 are space, `! " # … 9 : ; < = > ?` (ASCII 32–63).
There are no lower-case letters. `@` (value 0) is the padding character and
"should be placed at the end of the string" [M.1371-5 Annex 2 §3.3.7];
many encoders pad with spaces instead, so a decoder strips trailing `@`
*and* trailing spaces. The full table is
[Appendix B.2](appendix-b-six-bit-tables.md#b2-six-bit-ascii-for-text-fields-6-bit-value--character).
Keep the two tables apart in your head: armouring is how the *whole
message* rides in the sentence; six-bit ASCII is how *some fields* inside
the message spell words. The `Bits.text()` method applies the second table
to bits that the first table has already produced.

## 2.3 Decoding one message by hand

Line 1 of `data/ch02-aivdm-samples.nmea` (from
`sample-data/2007hydro/norfolk-log.ais` line 2, recorded in Norfolk,
Virginia on 14 May 2007):

```text
!AIVDM,1,1,,A,15N91b?P00JRlJ4E56Qe0?v20@14,0*2F,1179170462.87
```

Fields: one sentence of one, no sequential id, channel A, payload
`15N91b?P00JRlJ4E56Qe0?v20@14`, 0 fill bits, checksum `2F`. The trailing
`,1179170462.87` after the checksum is the logger's receive time (UNIX
seconds), chapter 5.

Twenty-eight characters. Apply "subtract 48, then 8 if above 40" to each
and write the value as six binary digits:

```text
1      5      N      9      1      b      ?      P      0      0      J      R      l      J
1      5      30     9      1      42     15     32     0      0      26     34     52     26
000001 000101 011110 001001 000001 101010 001111 100000 000000 000000 011010 100010 110100 011010

4      E      5      6      Q      e      0      ?      v      2      0      @      1      4
4      21     5      6      33     45     0      15     62     2      0      16     1      4
000100 010101 000101 000110 100001 101101 000000 001111 111110 000010 000000 010000 000001 000100
```

(`N` is ASCII 78: 78 − 48 = 30. `b` is ASCII 98: 98 − 48 = 50, which is
above 40, so 50 − 8 = 42. `?` is ASCII 63: 63 − 48 = 15. `@` is ASCII 64:
16.)

Concatenated, that is a 168-bit string. The first three fields, with
zero-based bit offsets [M.1371-5 Annex 8 Table 48]:

```text
bits 0-5   000001                          message type 1
bits 6-7   00                              repeat indicator 0
bits 8-37  010101111000100100000110101000  MMSI 367 149 480
```

Bits 0–5 are the whole of character `1`; bits 6–7 are the top two bits of
`5` (`00`); the MMSI is the remaining four bits of `5` (`0101`), then
`N 9 1 b` in full (`011110 001001 000001 101010`), then the top two bits
of `?` (`00`). Fields do not respect character boundaries, which is
exactly why we convert to a bit string first and index into that.
0b010101111000100100000110101000 = 367 149 480, a US MMSI (maritime
identification digits 367; chapter 3). The rest of the fields — latitude
36.839477°N, longitude 76.279783°W, a vessel at the Norfolk waterfront
with navigational status 15 — are decoded in chapter 3.

## 2.4 The code

Two modules are new in this chapter and a third is the program that ties
everything together:

* `bits` — armour decoding and the `Bits` reader with `uint`, `sint`,
  `flag`, `text` and `raw` accessors;
* `vdm` — the `!AIVDM`/`!AIVDO` fields (`Vdm`), the assembled `Message`,
  and the `Assembler` that keeps state across lines;
* `decode` — the driver: for each input line, TAG block → sentence → VDM
  → assembler → message decoder → JSON.

A line flows through the driver like this (Python names; the other three
are identical in structure):

```text
line ──split_tag_block──> (tagblock, rest)            chapter 4
     ──parse_sentence────> Sentence (talker, formatter, fields, checksum, after_checksum)   chapter 1
     ──parse_uscg_fields─> "uscg": {...}  from after_checksum                             chapter 5
     ──parse_vdm─────────> Vdm (total, number, seq_id, channel, payload, fill_bits)
     ──Assembler.add─────> None (wait for more fragments)  |  Message (payload, fill_bits, fragments)
     ──Bits.from_payload─> Bits
     ──DECODERS[type]────> dict          (msg_1_2_3.decode, msg_5.decode, ...)
     ──jsonout.dumps─────> one line of JSON
```

Every stage either returns a value or raises/returns an error whose text
is fixed by `code/CONVENTIONS.md` §2, so the four programs print the same
`{"line":N,"error":"...","raw":"..."}` objects.

### Python

`code/python/aistut/bits.py`, the armouring part:

```python
def armor_char_to_value(ch: str) -> int:
    """Return the 6-bit value (0–63) carried by one payload character."""
    code = ord(ch)
    if code < 48 or code > 119 or 88 <= code <= 95:   # outside '0'..'w', or 'X'..'_'
        raise ValueError(f"illegal armour character {ch!r}")
    value = code - 48
    if value > 40:
        value -= 8
    return value


def payload_to_bits(payload: str, fill_bits: int = 0) -> str:
    """Convert an armoured payload to a string of '0'/'1'."""
    if not 0 <= fill_bits <= 5:
        raise ValueError(f"fill bits must be 0..5, got {fill_bits}")
    # format(v, '06b') writes v in binary, zero-padded to 6 characters.
    bits = "".join(format(armor_char_to_value(ch), "06b") for ch in payload)
    if fill_bits:
        bits = bits[:-fill_bits]
    return bits
```

The bit string is kept as literally that — a Python `str` of `'0'` and
`'1'` — because it makes the hand decode above and the code identical:
`bits[8:38]` is the MMSI. The reader class:

```python
SIXBIT_ASCII = "@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_ !\"#$%&'()*+,-./0123456789:;<=>?"


class Bits:
    def __init__(self, bits: str):
        if any(c not in "01" for c in bits):
            raise ValueError("Bits expects a string of '0' and '1'")
        self.bits = bits

    @classmethod
    def from_payload(cls, payload: str, fill_bits: int = 0) -> Bits:
        return cls(payload_to_bits(payload, fill_bits))

    def __len__(self) -> int:          # lets you write len(bits)
        return len(self.bits)

    def has(self, start: int, length: int) -> bool:
        return start + length <= len(self.bits)

    def uint(self, start: int, length: int) -> int:
        if not self.has(start, length):
            raise IndexError(f"field {start}+{length} beyond {len(self.bits)} bits")
        return int(self.bits[start:start + length], 2)

    def sint(self, start: int, length: int) -> int:
        value = self.uint(start, length)
        if self.bits[start] == "1":          # negative: subtract 2**length
            value -= 1 << length
        return value

    def flag(self, start: int) -> bool:
        return self.uint(start, 1) == 1

    def text(self, start: int, length: int) -> str:
        chars = []
        for pos in range(start, start + length - 5, 6):
            if not self.has(pos, 6):
                break                        # tolerate short (truncated) fields
            chars.append(SIXBIT_ASCII[self.uint(pos, 6)])
        s = "".join(chars)
        at = s.find("@")
        if at >= 0:
            s = s[:at]
        return s.rstrip()

    def raw(self, start: int, length: int) -> str:
        return self.bits[start:start + length]
```

`sint` is explained in chapter 3 (two's complement). The `vdm` module is
a `Vdm` dataclass filled by `parse_vdm` (field by field, with `int()` on
the numeric ones and `None` for an empty sequential id), a `Message`
dataclass, and the `Assembler`:

```python
class Assembler:
    def __init__(self) -> None:
        self._pending: dict[tuple, list[Vdm]] = {}

    def add(self, frag: Vdm) -> Message | None:
        if frag.total == 1:
            return Message(frag.payload, frag.fill_bits, frag.channel,
                           frag.talker, frag.own_ship, [frag])
        key = (frag.talker, frag.channel, frag.seq_id)
        if frag.number == 1:
            self._pending[key] = [frag]
        else:
            group = self._pending.get(key)
            if group is None or group[-1].number != frag.number - 1:
                self._pending.pop(key, None)        # out of order: drop
                return None
            group.append(frag)
        group = self._pending[key]
        if frag.number == frag.total:
            del self._pending[key]
            payload = "".join(g.payload for g in group)
            return Message(payload, frag.fill_bits, frag.channel,
                           frag.talker, frag.own_ship, group)
        return None
```

The policy is deliberately simple: a fragment 1 always starts a fresh
group (replacing any half-finished one with the same key), a fragment
*n* > 1 must follow fragment *n* − 1 of the same key or the whole group is
discarded, and the message is returned when fragment *total* arrives. A
lost fragment costs exactly one message. The fill bits of the *last*
fragment apply to the concatenated payload.

The driver's two functions, `decode_ais(msg)` and
`process_line(line, lineno, asm)`, are shown in the flow diagram above;
`DECODERS` is a dictionary from message type to decode function, which is
how `decode.py` stays ignorant of what any message means. The JSON writer
`jsonout.dumps` prints keys sorted, no whitespace, and refuses a bare
`float`: every non-integer must be wrapped as `Fixed(value, decimals)` so
that it is printed with exactly that many decimals. This is the single
rule that makes four languages print identical text — `repr(42.07404)` in
Python, `std::to_string` in C++, `{}` in Rust and `String(x)` in
JavaScript all format doubles differently, but `%.6f` is `%.6f`
everywhere.

> **Language notes (Python).** `class Bits:` bundles data (`self.bits`)
> with the methods that read it; `self` is the object, passed explicitly.
> `@classmethod` marks a method that receives the class (`cls`) instead of
> an instance — a "named constructor" (`Bits.from_payload(...)`). Its
> return hint `-> Bits` names the class that is still being defined; that
> works because Python 3.14 evaluates annotations lazily (PEP 649), where
> older versions needed `-> "Bits"` or `from __future__ import annotations`.
> Defining `__len__` lets `len(bits)` work on your own class ("dunder"
> methods hook into built-in syntax). `int(s, 2)` parses a string of binary
> digits; `format(v, "06b")` is its inverse, producing six zero-padded
> binary digits. `1 << length` is 2 to the power `length` (shift left).
> `dict[tuple, list[Vdm]]` is a dictionary keyed by a tuple — tuples are
> immutable, so they can be keys; lists cannot. `Message | None` means
> "a `Message` or `None`", and `None` is how `add` says "not yet". A
> generator expression inside `"".join(...)` builds the string without an
> intermediate list.

### C++

`code/cpp/bits.hpp` keeps the bits as a `std::vector<std::uint8_t>` of 0s
and 1s. The armouring loop emits one element per bit:

```cpp
inline std::int32_t armor_char_to_value(char32_t ch) {
    std::uint32_t code = ch;
    if (code < 48 || code > 119 || (88 <= code && code <= 95)) {   // outside '0'..'w', or 'X'..'_'
        throw std::invalid_argument("illegal armour character " + py::repr(py::utf8_encode(ch)));
    }
    std::int32_t value = static_cast<std::int32_t>(code) - 48;
    if (value > 40) value -= 8;
    return value;
}

inline std::vector<std::uint8_t> payload_to_bits(std::string_view payload, std::int64_t fill_bits = 0) {
    if (fill_bits < 0 || fill_bits > 5) {
        throw std::invalid_argument(std::format("fill bits must be 0..5, got {}", fill_bits));
    }
    std::vector<std::uint8_t> bits;
    bits.reserve(payload.size() * 6);
    for (char32_t ch : py::codepoints(payload)) {
        std::int32_t value = armor_char_to_value(ch);
        // Emit the six bits most-significant first: shift right by 5, 4, … 0
        // and mask with 1 to isolate each bit.
        for (std::int32_t shift = 5; shift >= 0; --shift) {
            bits.push_back(static_cast<std::uint8_t>((value >> shift) & 1));
        }
    }
    std::size_t drop = static_cast<std::size_t>(fill_bits);
    if (drop > bits.size()) drop = bits.size();
    bits.resize(bits.size() - drop);
    return bits;
}
```

and `Bits::uint` rebuilds a number by shifting in one bit at a time:

```cpp
    std::uint64_t uint(std::size_t start, std::size_t length) const {
        if (!has(start, length)) {
            throw std::out_of_range(std::format("field {}+{} beyond {} bits", start, length, bits_.size()));
        }
        std::uint64_t value = 0;
        for (std::size_t i = start; i < start + length; ++i) {
            value = (value << 1) | bits_[i];       // shift left, bring in the next bit
        }
        return value;
    }
```

The `Assembler` in `vdm.hpp` is the Python one with types written out:
the pending table is `std::map<Key, std::vector<Vdm>>` where `Key` is
`std::tuple<std::string, std::string, std::optional<std::int64_t>>`, and
`add` returns `std::optional<Message>`. `parse_vdm` returns a
`std::expected<Vdm, std::string>` — either the parsed fragment or the
error text that Python would have raised as a `ValueError` — so the
driver checks `if (!frag)` and reports `frag.error()` instead of catching
an exception. `decode.cpp` is the only `.cpp`
file; all modules are headers (`#pragma once`, `inline` functions), so the
whole program is one translation unit and the `Makefile` has one compile
line. Its `DECODERS` is a `std::map<std::int32_t, Decoder>` where `Decoder` is a
function-pointer type `Json (*)(const Bits&)`. `jsonout.hpp` builds JSON
from a `Json` class wrapping a `std::variant`; because a `std::map` keeps
its keys sorted, the sorted-keys rule costs nothing, and `Fixed{value,
decimals}` is printed with `std::format("{:.{}f}", value, decimals)`.

> **Language notes (C++).** A `class` has `public:` members (the
> interface) and `private:` members (`bits_`, by convention with a trailing
> underscore). Methods marked `const` promise not to modify the object, so
> they can be called through a `const Bits&`. `static Bits from_payload`
> is a function that belongs to the class, not to an instance — Python's
> `@classmethod`. `<cstdint>` gives fixed-width integers: `std::uint8_t`
> holds exactly 8 bits, `std::uint64_t` 64; plain `int` is only guaranteed
> 16. `std::size_t` is the unsigned type for sizes and indices, and mixing
> it with signed integers provokes warnings, so offsets are `std::size_t` throughout.
> `(value >> shift) & 1` isolates one bit; `(value << 1) | bit` appends
> one. `std::tuple` compares element by element, so it can be a `std::map`
> key with no extra code; `std::optional<T>` is "a T or nothing" and
> replaces both Python's `None` return and its `int | None` field.
> `char32_t` and `py::codepoints` exist only so that a stray non-ASCII
> byte in a log is reported the way Python reports it — the test-suite
> compares the error text.

> **Language notes (C++20/23).** **`std::expected<T, E>`** (C++23,
> `<expected>`) is "a T, or an error E" — `std::optional` with a reason
> attached. `parse_vdm` returns `std::unexpected(text)` for a bad field;
> the caller tests `if (!frag)`, reads `frag.error()`, or dereferences
> `*frag` for the value. Use it where failure is *expected data*, as a bad
> log line is, and keep exceptions for programming errors. A **designated
> initialiser** (C++20) names each member when building a struct —
> `Message{.payload = …, .fill_bits = …, .channel = …}` — so that two
> strings or two integers cannot be swapped silently; `Vdm`, `Message`
> and `py::Partition` are built that way. `std::format("{:.{}f}", value,
> decimals)` is the fixed-decimal format with the precision taken from the
> next argument (C's `"%.*f"`); both are correctly rounded, which is why
> `Fixed` prints the same digits as Python's `format(x, '.6f')`.

### Rust

`code/rust/src/bits.rs` uses a `Vec<u8>`:

```rust
pub fn armor_char_to_value(ch: char) -> Result<u8, BitsError> {
    let code = ch as u32; // the Unicode code point, like Python's ord()
    if !(48..=119).contains(&code) || (88..=95).contains(&code) {
        // outside '0'..'w', or inside 'X'..'_'
        return Err(BitsError::IllegalChar(ch));
    }
    let mut value = code - 48;
    if value > 40 {
        value -= 8;
    }
    // `as u8` is safe here: value is at most 63. We checked, so the cast
    // cannot truncate.
    Ok(value as u8)
}

pub fn payload_to_bits(payload: &str, fill_bits: i64) -> Result<Vec<u8>, BitsError> {
    if !(0..=5).contains(&fill_bits) {
        return Err(BitsError::BadFillBits(fill_bits));
    }
    let mut bits = Vec::with_capacity(payload.len() * 6);
    for ch in payload.chars() {
        let value = armor_char_to_value(ch)?; // `?`: stop at the first bad char
        // Most significant bit first: shift right by 5, 4, ... 0 and mask.
        for shift in (0..6).rev() {
            bits.push((value >> shift) & 1);
        }
    }
    let keep = bits.len().saturating_sub(fill_bits as usize);
    bits.truncate(keep);
    Ok(bits)
}
```

```rust
    pub fn uint(&self, start: usize, length: usize) -> u64 {
        assert!(self.has(start, length), "field {start}+{length} beyond {} bits", self.bits.len());
        assert!(length <= 64, "uint() supports at most 64 bits");
        let mut value: u64 = 0;
        for &bit in &self.bits[start..start + length] {
            value = (value << 1) | u64::from(bit);
        }
        value
    }
```

Errors that depend on *input* (`IllegalChar`, `BadFillBits`) are an
`enum BitsError` returned in a `Result`; errors that can only be
*programming mistakes* (asking for bits past the end after `require_bits`
already checked) are `assert!`s, mirroring Python's `IndexError`. The
`Assembler` in `vdm.rs` holds a `HashMap<(String, String, Option<i64>),
Vec<Vdm>>` and `add(&mut self, frag: Vdm) -> Option<Message>`; it takes
the fragment *by value* because the table must own fragments that outlive
the line they came from. `decode.rs` replaces the dictionary of decoders
with a `match` that returns a function pointer
(`fn(&Bits) -> Result<Json, DecodeError>`), and `jsonout.rs` uses a
`BTreeMap<String, Json>` — an ordered map, so keys come out sorted — with
`Json::Fixed(f64, usize)` printed by `format!("{:.*}", decimals, value)`.

> **Language notes (Rust).** `Vec<u8>` is a growable array of bytes; `&self.bits[a..b]`
> borrows a slice of it without copying. `ch as u32` and `value as u8` are
> explicit numeric casts — Rust never converts integer types silently, and
> `as` to a narrower type truncates, so the comment proves it cannot here.
> `(0..=5).contains(&x)` is an inclusive range test; `(0..6).rev()` counts
> 5, 4, … 0. `Result<T, E>` is "a T or an error E"; the `?` operator
> returns the error to the caller immediately, which is what makes
> `armor_char_to_value(ch)?` read like a plain call. `impl fmt::Display
> for BitsError` is where the error's text lives (`to_string()` uses it).
> `saturating_sub` subtracts but stops at zero instead of panicking on
> underflow. `&mut self` on `Assembler::add` says the method mutates the
> struct; the borrow checker then guarantees nobody reads the table while
> it changes. A tuple of `String`s is hashable, so it can key a `HashMap`
> exactly like the Python tuple.

### TypeScript

`code/typescript/src/bits.ts` uses a `Uint8Array`, and it is the one
version where the *obvious* code is wrong:

```typescript
export function armorCharToValue(ch: string): number {
  const code = ch.codePointAt(0) ?? -1;
  if (code < 48 || code > 119 || (88 <= code && code <= 95)) { // outside '0'..'w', or 'X'..'_'
    throw new ValueError(`illegal armour character ${pyRepr(ch)}`);
  }
  let value = code - 48;
  if (value > 40) {
    value -= 8;
  }
  return value;
}

export function payloadToBits(payload: string, fillBits = 0): Uint8Array {
  if (!(0 <= fillBits && fillBits <= 5)) {
    throw new ValueError(`fill bits must be 0..5, got ${fillBits}`);
  }
  // Array.from(string) splits into Unicode code points, like Python's `for ch in s`.
  const chars = Array.from(payload);
  const bits = new Uint8Array(chars.length * 6);
  for (let i = 0; i < chars.length; i++) {
    const value = armorCharToValue(chars[i]);
    for (let b = 0; b < 6; b++) {
      // These are 6-bit values, so the 32-bit operators are safe here.
      bits[i * 6 + b] = (value >> (5 - b)) & 1;
    }
  }
  return fillBits ? bits.slice(0, Math.max(0, bits.length - fillBits)) : bits;
}
```

```typescript
  uint(start: number, length: number): number {
    if (!this.has(start, length)) {
      throw new RangeError(`field ${start}+${length} beyond ${this.bits.length} bits`);
    }
    let value = 0;
    for (let i = start; i < start + length; i++) {
      value = value * 2 + this.bits[i]; // NOT `value << 1 | bit` — see the 32-bit pitfall
    }
    return value;
  }
```

The `Assembler` in `vdm.ts` is a class with a `private readonly pending =
new Map<string, Vdm[]>()`; because a JavaScript `Map` compares object keys
by identity, two equal arrays would be different keys, so the triple is
serialised with `JSON.stringify([talker, channel, seqId])` to make a
string key. `decode.ts` catches errors with `instanceof` checks in a fixed
order (`DecodeError`, then `NotImplementedError`, …). `jsonout.ts` sorts
keys explicitly in its own `dumps` and prints `Fixed` with `toFixed`, plus
a small correction because `toFixed` rounds exact ties away from zero
where C, Python and Rust round to even.

> **Language notes (TypeScript).** **The 32-bit pitfall:** JavaScript's
> bitwise operators `<< >> | &` convert their operands to *signed 32-bit*
> integers first. `1 << 31` is −2 147 483 648 and `x << 30` silently loses
> bits. AIS fields are up to 30 bits wide (the MMSI) and the
> two's-complement correction needs `2 ** 28`, so `uint()` accumulates with
> `value * 2 + bit`, which is exact up to 2⁵³ (`Number.MAX_SAFE_INTEGER`).
> Shifting a value you *know* is six bits wide (`value >> (5 - b)`) is
> fine. `Uint8Array` is a typed array — the one JavaScript container that
> stores raw bytes compactly. `static fromPayload(...)` is a factory
> method; `get length()` is a getter, so callers write `bits.length`
> without parentheses. `??` is nullish coalescing: use the right-hand side
> only when the left is `null`/`undefined`. `class Assembler` with
> `private readonly` fields and a method `add(frag: Vdm): Message | null`
> shows a union return type — the caller must test `=== null` before using
> the result. Template literals (`` `...${x}...` ``) are the f-strings.

## 2.5 Sample data and results

`data/ch02-aivdm-samples.nmea` has nine lines:

| Line | Source | What it shows |
|---|---|---|
| 1 | `sample-data/2007hydro/norfolk-log.ais` line 2 | Single-sentence message 1, channel A, 2007-style logger suffix (timestamp only) |
| 2 | `sample-data/2007hydro/norfolk-log.ais` line 7 | Message 3, channel B |
| 3 | `sample-data/RightwhaleAisProject/20081005-capecod-visitor/20081005-capecod-visitor-center-basement` line 8 | Message 3 with `rcapecod-before,…` station suffix |
| 4–5 | same file, lines 9–10 | Two-fragment message 5, sequential id 1, fill bits 2 on the last fragment |
| 6–7 | `sample-data/AISmisc/classb.uscg.19.nmea` lines 3–4 | Two-fragment message 19, sequential id 6, USCG `s`/`r`/time fields |
| 8 | `sample-data/RightwhaleAisProject/2008-10-01-auk-test-cape-cod/hyperterminal/2008_10_1.txt` line 1 | `!AIVDO` — a Class A unit echoing its own message 1 |
| 9 | `sample-data/2010-11-l3-aton/ccom-l3-4-2010-11-02` line 7 | `!ANVDO`, channel `X`, message 21 with 4 fill bits |

Run any of the four:

```sh
python3 code/python/decode.py data/ch02-aivdm-samples.nmea
code/cpp/build/decode data/ch02-aivdm-samples.nmea
code/rust/target/release/decode data/ch02-aivdm-samples.nmea
node code/typescript/dist/decode.js data/ch02-aivdm-samples.nmea
```

All four print the seven lines of `tests/expected/ch02-aivdm-samples.jsonl`.
Seven, not nine: line 4 and line 6 are first fragments and print nothing
(the assembler is waiting). Line 1 in full:

```json
{"ais":{"cog_available":true,"cog_deg":332.8,"comm_state":{"scheme":"SOTDMA","slot_number":68,"slot_timeout":4,"sync_state":0,"sync_state_text":"UTC direct"},"heading_available":false,"lat":36.839477,"lon":-76.279783,"mmsi":367149480,"msg_type":1,"nav_status":15,"nav_status_text":"undefined (default)","position_accuracy":0,"position_available":true,"raim":false,"repeat_indicator":0,"rot_deg_per_min":null,"rot_raw":-128,"rot_text":"not available","sog_available":true,"sog_knots":0.0,"spare":0,"special_manoeuvre":0,"special_manoeuvre_text":"not available","timestamp":1,"true_heading":511},"channel":"A","fragments":1,"line":1,"own_ship":false,"payload_bits":168,"talker":"AI","uscg":{"time":1179170462.87}}
```

The envelope-level keys are the ones this chapter is about: `"line":1`,
`"talker":"AI"`, `"channel":"A"`, `"own_ship":false` (VDM, not VDO),
`"payload_bits":168` (28 characters × 6 − 0 fill bits) and
`"fragments":1`. `"mmsi":367149480` and `"msg_type":1` are the fields we
decoded by hand in §2.3; the rest of `"ais"` is chapter 3.

Selected fields of the other six objects:

| Line | `talker` | `channel` | `own_ship` | `fragments` | `payload_bits` | `ais` |
|---|---|---|---|---|---|---|
| 2 | `AI` | `B` | false | 1 | 168 | message 3, MMSI 212521000, at anchor |
| 3 | `AI` | `A` | false | 1 | 168 | message 3, MMSI 3669734, at anchor |
| 5 | `AI` | `A` | false | 2 | 424 | message 5, MMSI 3669734, name `NOAA TEST`, destination `NOT A VESSEL` (chapter 7) |
| 7 | `AI` | `A` | false | 2 | 312 | message 19, MMSI 523135452, 88.7 kn and a name of punctuation — a corrupt payload (chapter 14) |
| 8 | `AI` | `A` | **true** | 1 | 168 | message 1, MMSI 369871000, 19.5 kn |
| 9 | `AN` | `X` | **true** | 1 | 272 | message 21, MMSI 3669739, name `NOAA WHALE NOTICES` (chapter 15) |

Points worth checking against the sentences:

* Line 5 is reported as `"line":5` — the line of the *last* fragment —
  with `"fragments":2`. 56 + 15 = 71 characters × 6 = 426 bits, minus the
  2 fill bits of the second fragment, gives the 424 bits of a message 5.
  Lines 6–7: 43 + 9 = 52 characters × 6 = 312, fill bits 0, the exact size
  of message 19.
* Line 9: 46 characters × 6 = 276, minus 4 fill bits = 272, the minimum
  size of message 21 (the name-extension field is absent).
* Lines 5, 7 and 9 are decoded in full by the modules of chapters 7, 14
  and 15; here only the envelope matters. Line 7 is a reminder that a
  well-formed envelope says nothing about the bits inside: the message 19
  it carries is structurally valid and semantically garbage.
* Line 3's MMSI, 3669734, prints with seven digits because it is the
  integer value of 003669734 — a coast-station format identity
  (`00` + MID 366, chapter 3) transmitting a Class A position report. It
  is the right-whale project's own shore transceiver at the Cape Cod
  National Seashore visitor centre, configured with a shore-station style
  identity. Do not assume that a message type implies a kind of station.
* The `uscg` objects differ in shape from line to line because the
  loggers did: a bare timestamp (lines 1–2), a station id plus timestamp
  (3, 5, 9), and the full `s`/`r`/time N-AIS form (7). Chapter 5.

## 2.6 Pitfalls and real-world deviations

* **Sentences longer than 82 characters.** Some receivers and most shore
  networks re-join fragments before forwarding and emit a message 5 as a
  single `!AIVDM,1,1,1,A,<71 characters>,2*5C` sentence of about 140
  characters — every line of `sample-data/sbnms/2008/data/*.ais.unique` is
  one. Note the sequential id `1` that survives from the original
  two-sentence form. Parsers that allocate an 82-byte buffer truncate
  these silently. Ours does not enforce the limit and decodes them.
* **Wrong fill bits.** A fill count two too small makes a 424-bit message
  5 look like 426 bits [AIVDM "Interaction with AIVDM padding"]. Accept
  fixed-size messages that are a few bits long; reject ones that are
  short. The `require_bits` check in every decoder does exactly this.
* **Lost first fragments.** A second fragment with no first one is common
  on radio links and in logs that were started mid-stream. The assembler
  drops it. Equally common is a first fragment whose partner never comes;
  it sits in the pending table until a new first fragment with the same
  key replaces it. A long-running process should also age pending entries
  out (exercise 5).
* **Interleaved groups.** Two receivers feeding one pipe can interleave
  `2,1,3` from one with `2,1,3` from the other. Keying on (talker,
  channel, sequential id) separates them only if they differ in talker or
  channel; the sentence layer offers nothing better. NMEA 4.10 TAG blocks
  (chapter 4) add a group field `g:1-2-1234` that solves this properly.
* **Corrupted-but-checksummed payloads.** The `nhjel` receiver in
  `sample-data/tide1/` computed valid checksums over garbage: truncated
  payloads (`1fTDO<hLWP,4` is a 56-bit "message 1") and random bits whose
  first six happen to spell a legal type (`@3<QIqp,2` is a 40-bit "message
  16"). Both appear in chapter 3's data. A valid checksum means the serial
  link is clean, not that the radio decode was.
* **Illegal characters.** Anything outside `0`–`W` and `` ` ``–`w` is
  corruption. Binary junk (`\x00`, boot banners) occurs in
  `sample-data/2007hydro/protec-*`; open logs with a byte-tolerant decoder
  (`errors="replace"` in Python; the other three repair UTF-8 the same
  way) so that one bad line does not stop the file.
* **Channel field oddities.** `X`, `Y` and empty (§2.2.1). Treat it as a
  string.
* **Empty fields.** The sequential id is legitimately empty for
  single-sentence messages; the channel and the fill-bit count are
  sometimes empty too. Our parsers treat an empty fill count as 0 and an
  empty channel as the empty string.
* **`@` in two tables.** The payload character `@` is value 16. The text
  character `@` is value 0 and means "padding". If your ship names come
  out as `@@@@…` you have applied the armouring table twice.

## 2.7 Exercises

1. *(pencil)* Armour-decode the nine characters `15Pg8V003` from line 8 and
   read off the message type, repeat indicator and MMSI. Check against
   `tests/expected/ch02-aivdm-samples.jsonl` line for `"line":8`.
2. *(Python)* Write `bits_to_payload(bits: str) -> tuple[str, int]` — the
   inverse of `payload_to_bits` — returning the armoured text and the fill
   count. Verify that it round-trips every payload in
   `data/ch02-aivdm-samples.nmea`.
3. *(C++)* `Bits::uint` throws `std::out_of_range` when a field runs past
   the end. Change `Bits::text` so that a text field that is cut off
   mid-character (e.g. 13 bits requested from a 10-bit remainder) returns
   what it can, and add a `static_assert` or a small test that proves it.
   (The Python version already does this; compare the loop bounds.)
4. *(Rust)* Replace the `assert!` in `Bits::uint` with a
   `Result<u64, BitsError>` and propagate it with `?` through
   `msg_1_2_3::decode`. How many call sites change, and is the result more
   or less readable? Write a sentence defending the choice the tutorial
   made.
5. *(TypeScript)* Give `Assembler` an age limit: store the input line
   number with each pending group and drop groups older than *N* lines
   when a new fragment arrives. Feed it the two pairs in the sample file
   with a stray `!AIVDM,2,2,1,A,…` inserted between them and check that
   the right things are dropped.
6. *(any language, harder)* The sequential id is 0–9, so a receiver that
   emits more than ten multi-sentence messages between two fragments of
   one message produces an ambiguity. Using
   `sample-data/RightwhaleAisProject/20081013.log` (two receivers
   multiplexed), count how often a fragment 1 arrives while a group with
   the same (talker, channel, id) is still pending, and whether the
   replaced group's partner ever shows up later.

## 2.8 References

* [IEC 61162-1] §7 (sentence structure, 82-character limit), §7.2.3
  (checksum), §7.3.3 (six-bit field conversion), §8 (VDM and VDO
  formatters).
* [AIVDM] "AIVDM/AIVDO Sentence Layer", "Talker IDs", "AIVDM/AIVDO Payload
  Armoring", "AIS Payload Data Types", "AIS Payload Byte Alignment,
  Padding, and Bit Stuffing" (incl. "Interaction with AIVDM padding").
* [M.1371-5] Annex 2 §3.2.2.1 (bit stuffing), §3.3 (TDMA access), §3.3.7
  (message structure, byte alignment, `@` padding); Annex 8 §3
  (bit ordering, two's complement), Table 46 (message summary), Table 47
  (six-bit ASCII), Table 48 (message 1 layout used in the hand decode).
* [Toils] on padding and fill-bit ambiguities.
* [A.1106(29)] for the bridge equipment that consumes `!AIVDM`.
* [NMEA 0183] for the TAG-block grouping mechanism previewed in §2.6.
