"""msg_10.py — message 10: UTC and date inquiry.

Spec: ITU-R M.1371-5 Annex 8 §3.8, Table 60 (72 bits). The answer is a
message 11 (same layout as message 4, chapter 6), which "is only
transmitted as a result of a UTC request message (Message 10)" and goes
out "on the channel where the UTC request message was received"
[Annex 8 §3.2].

Bit layout (zero-based offsets; lengths from Table 60):

     0- 5   6  message id (10)
     6- 7   2  repeat indicator
     8-37  30  source id (MMSI of the station that wants to know the time)
    38-39   2  spare
    40-69  30  destination id (MMSI of the station being asked)
    70-71   2  spare

This is the simplest addressed message in AIS: two MMSIs and four spare
bits. It is also the only interrogation that does *not* use message 15
[Annex 8 Table 46 note 4].
"""

from __future__ import annotations

from .bits import Bits
from .common import require_bits


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
