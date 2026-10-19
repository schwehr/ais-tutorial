"""msg_6.py — message 6: addressed binary message (chapter 8).

Spec: ITU-R M.1371-5 Annex 8 §3.4, Table 54 (layout), Table 55 (slot
budget); Annex 2 §5.3.1 (sequence number, retries, retransmit flag and the
message 7 acknowledgement); Annex 5 §2.1 Table 23 (application identifier).

Bit layout (zero-based offsets; lengths from Table 54):

     0- 5   6  message id (6)
     6- 7   2  repeat indicator
     8-37  30  source id (MMSI)
    38-39   2  sequence number 0-3 (Annex 2 §5.3.1; echoed by message 7)
    40-69  30  destination id (MMSI)
    70      1  retransmit flag: 0 = no retransmission (default), 1 = retransmitted
    71      1  spare
    72-81  10  designated area code (DAC)          } 16-bit application
    82-87   6  function identifier (FI)             } identifier, Annex 5 Table 23
    88-    ≤920 application data

Total 88-1 008 bits (1-5 slots; Table 55 recommends at most 8/36/64/92/117
binary data *bytes* for 1-5 slots, counting the application identifier).

Length rule: at least the 88-bit header (through the application
identifier) is required; otherwise DecodeError. Whatever follows is
reported as `data`/`data_bits` and, when asm.py knows the (dac, fi), also
decoded into `application`. Trailing byte-alignment padding is part of
`data`; the receiver cannot tell it from payload (chapter 8 §8.6).
"""

from __future__ import annotations

from . import asm
from .bits import Bits
from .common import require_bits

HEADER_BITS = 88   # message id .. function identifier


def decode(bits: Bits) -> dict:
    """Decode a message 6. Raises DecodeError below 88 bits."""
    require_bits(6, len(bits), HEADER_BITS)
    dac = bits.uint(72, 10)
    fi = bits.uint(82, 6)
    out: dict = {
        "msg_type": 6,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "sequence_number": bits.uint(38, 2),
        "dest_mmsi": bits.uint(40, 30),
        "retransmit": bits.flag(70),
        "spare": bits.uint(71, 1),
        "dac": dac,
        "fi": fi,
        "data": bits.raw(HEADER_BITS, len(bits) - HEADER_BITS),
        "data_bits": len(bits) - HEADER_BITS,
    }
    application = asm.decode(dac, fi, bits, HEADER_BITS)
    if application is not None:
        out["application"] = application
    return out
