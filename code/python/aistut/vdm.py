"""vdm.py — the !AIVDM / !AIVDO encapsulation sentence (chapter 2).

    !AIVDM,1,1,,B,15N8?>PP01JRVP6E7aLLkOvt0HAO,0*3C
     │  │  │ │ │ │ │                            │ └ checksum
     │  │  │ │ │ │ │                            └── fill bits (0–5)
     │  │  │ │ │ │ └─────────────────────────────── armoured payload
     │  │  │ │ │ └───────────────────────────────── radio channel A/B (or 1/2, or empty)
     │  │  │ │ └─────────────────────────────────── sequential message id (0–9) for multi-sentence groups
     │  │  │ └───────────────────────────────────── this sentence's number within the group
     │  │  └─────────────────────────────────────── total sentences in the group
     │  └────────────────────────────────────────── VDM = received from another station,
     │                                                VDO = own station's transmission
     └───────────────────────────────────────────── talker: AI mobile, AB/BS base, AN AtoN, AR receiver, …

IEC 61162-1 §8.3.90 (VDM) and §8.3.91 (VDO). An AIS message longer than
one sentence can hold (the 82-character limit leaves room for about 60
payload characters = 360 bits) is split across several VDM sentences with
the same sequential id; only the *last* fragment carries non-zero fill bits.
"""

from dataclasses import dataclass

from .nmea import Sentence


@dataclass
class Vdm:
    talker: str
    own_ship: bool          # True for VDO
    total: int
    number: int
    seq_id: int | None
    channel: str
    payload: str
    fill_bits: int
    sentence: Sentence      # the enclosing NMEA sentence, for metadata


def parse_vdm(s: Sentence) -> Vdm:
    """Interpret the six data fields of a VDM/VDO sentence."""
    if s.formatter not in ("VDM", "VDO"):
        raise ValueError(f"not a VDM/VDO sentence: {s.address}")
    if len(s.fields) < 6:
        raise ValueError(f"VDM needs 6 fields, got {len(s.fields)}")
    total, number, seq, channel, payload, fill = s.fields[:6]
    return Vdm(
        talker=s.talker,
        own_ship=(s.formatter == "VDO"),
        total=int(total),
        number=int(number),
        seq_id=int(seq) if seq else None,
        channel=channel,
        payload=payload,
        fill_bits=int(fill) if fill else 0,
        sentence=s,
    )


@dataclass
class Message:
    """A complete AIS message: the concatenated payload of all its fragments."""

    payload: str
    fill_bits: int
    channel: str
    talker: str
    own_ship: bool
    fragments: list[Vdm]

    @property
    def first(self) -> Vdm:
        return self.fragments[0]

    @property
    def last(self) -> Vdm:
        return self.fragments[-1]


class Assembler:
    """Collects VDM fragments and hands back complete messages.

    Fragments of a group share (talker, channel, sequential id). We keep
    one pending group per key; a fragment that does not continue the
    pending group replaces it (so a lost fragment only loses that message).

    Usage:
        asm = Assembler()
        for line in lines:
            msg = asm.add(parse_vdm(parse_sentence(line)))
            if msg is not None:
                ...decode msg.payload...
    """

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
