"""msg_25.py — message 25: single slot binary message (chapter 16).

Spec: ITU-R M.1371-5 Annex 8 §3.23, Table 80 (layout) and Table 81
(maximum data bits per flag combination); Annex 5 §1.1 for the 16-bit
application identifier (10-bit DAC + 6-bit FI).

Bit layout (zero-based offsets). Unlike messages 1–24, two flags decide
which fields exist, so the offsets after bit 39 are *variable*:

      0- 5   6  message id (25)
      6- 7   2  repeat indicator
      8-37  30  source id (MMSI)
     38      1  destination indicator: 0 = broadcast, 1 = addressed
     39      1  binary data flag: 0 = unstructured, 1 = application identifier follows
     40-69  30  destination MMSI        (only if destination indicator = 1)
     70-71   2  spare                   (only if destination indicator = 1)
     next   16  application identifier  (only if binary data flag = 1): DAC 10 + FI 6
     rest    n  application data, to the end of the payload
                (Table 81: at most 128 / 112 / 96 / 80 bits so that the
                whole message fits one 168-bit slot)

There is no communication state. The message is never acknowledged
(§3.23). Class B "CS" stations must not transmit it (Table 80).
"""

from __future__ import annotations

from .bits import Bits
from .common import require_bits


def decode_binary_header(bits: Bits, msg_type: int) -> tuple[dict, int]:
    """The part of messages 25 and 26 that precedes the application data.

    Returns the decoded fields and the offset of the first data bit. Raises
    DecodeError if the optional fields announced by the flags are missing.
    """
    require_bits(msg_type, len(bits), 40)
    addressed = bits.flag(38)
    structured = bits.flag(39)
    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "addressed": addressed,      # destination indicator, bit 38
        "structured": structured,    # binary data flag, bit 39
        "dest_mmsi": None,
        "dac": None,
        "fi": None,
    }
    offset = 40
    if addressed:
        require_bits(msg_type, len(bits), offset + 32)
        out["dest_mmsi"] = bits.uint(offset, 30)
        out["spare"] = bits.uint(offset + 30, 2)
        offset += 32
    if structured:
        require_bits(msg_type, len(bits), offset + 16)
        out["dac"] = bits.uint(offset, 10)
        out["fi"] = bits.uint(offset + 10, 6)
        offset += 16
    out["data_offset"] = offset
    return out, offset


def decode(bits: Bits) -> dict:
    """Decode a message 25. Raises DecodeError if a required part is missing."""
    out, offset = decode_binary_header(bits, 25)
    # Everything after the header is application data; we cannot know its
    # internal structure, so we hand it on as a bit string (CONVENTIONS §3).
    out["data"] = bits.raw(offset, len(bits) - offset)
    out["data_bits"] = len(bits) - offset
    return out
