"""msg_12.py — message 12: addressed safety related message.

Spec: ITU-R M.1371-5 Annex 8 §3.10, Table 61 (layout), Table 62 (how
many characters fit in 1–5 slots), Table 47 (six-bit ASCII); Annex 2
§5.3.1 (sequence numbers, retransmission, acknowledgement by message 13).

Bit layout (zero-based offsets; lengths from Table 61):

     0- 5   6  message id (12)
     6- 7   2  repeat indicator
     8-37  30  source id (MMSI)
    38-39   2  sequence number, 0-3
    40-69  30  destination id (MMSI)
    70      1  retransmit flag (0 = first transmission, 1 = retransmitted)
    71      1  spare
    72-     ≤936  safety related text, six-bit ASCII, 0-156 characters

Length rule: the 72-bit header is required (DecodeError otherwise). Every
complete six-bit group after it is one character. The 0-5 bits left over
at the end are *not* text: the radio layer pads messages to a byte
boundary (Annex 2 §5.2.1, "the variable messages are set on byte
boundaries"), and the receiver's armouring pads again to six bits, so a
real message often carries 2 or 4 stray bits after the last character.
We count them in `trailing_bits` and ignore them.
"""

from .bits import Bits
from .common import require_bits

HEADER_BITS = 72


def decode_text_field(bits: Bits, start: int) -> dict:
    """Decode a variable-length six-bit text field that runs to the end.

    Shared by messages 12 and 14 (and usable for any trailing text field).
    `Bits.text` already stops before a partial six-bit group, cuts the
    text at the first '@' (the Table 47 "padding"/not-available character)
    and strips trailing spaces; here we add the bookkeeping numbers.
    """
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
