"""msg_8.py — message 8: binary broadcast message (chapter 8).

Spec: ITU-R M.1371-5 Annex 8 §3.6, Table 57 (layout), Table 58 (slot
budget); Annex 5 §2.1 Table 23 (application identifier).

Bit layout (zero-based offsets; lengths from Table 57):

     0- 5   6  message id (8)
     6- 7   2  repeat indicator
     8-37  30  source id (MMSI)
    38-39   2  spare
    40-49  10  designated area code (DAC)          } 16-bit application
    50-55   6  function identifier (FI)             } identifier, Annex 5 Table 23
    56-    ≤952 application data

Total 56-1 008 bits (1-5 slots; Table 58 recommends at most 12/40/68/96/121
binary data *bytes* for 1-5 slots, counting the application identifier).

Length rule: at least the 56-bit header (through the application
identifier) is required; otherwise DecodeError. The rest is `data` /
`data_bits`, plus `application` when asm.py knows the (dac, fi).
"""

from __future__ import annotations

from . import asm
from .bits import Bits
from .common import require_bits

HEADER_BITS = 56   # message id .. function identifier


def decode(bits: Bits) -> dict:
    """Decode a message 8. Raises DecodeError below 56 bits."""
    require_bits(8, len(bits), HEADER_BITS)
    dac = bits.uint(40, 10)
    fi = bits.uint(50, 6)
    out: dict = {
        "msg_type": 8,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
        "dac": dac,
        "fi": fi,
        "data": bits.raw(HEADER_BITS, len(bits) - HEADER_BITS),
        "data_bits": len(bits) - HEADER_BITS,
    }
    application = asm.decode(dac, fi, bits, HEADER_BITS)
    if application is not None:
        out["application"] = application
    return out
