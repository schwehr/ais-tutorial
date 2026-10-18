"""msg_26.py — message 26: multiple slot binary message with communications
state (chapter 16).

Spec: ITU-R M.1371-5 Annex 8 §3.24, Table 82 (layout) and Table 83
(maximum data bits per flag combination and slot count); Annex 2
§3.3.7.2.2 Tables 18/19 (SOTDMA) and §3.3.7.3.2 Table 20 (ITDMA) for the
communication state, decoded by the same functions as messages 1–3.

Bit layout (zero-based offsets):

      0- 5   6  message id (26)
      6- 7   2  repeat indicator
      8-37  30  source id (MMSI)
     38      1  destination indicator: 0 = broadcast, 1 = addressed
     39      1  binary data flag: 0 = unstructured, 1 = application identifier follows
     40-69  30  destination MMSI        (only if destination indicator = 1)
     70-71   2  spare                   (only if destination indicator = 1)
     next   16  application identifier  (only if binary data flag = 1): DAC 10 + FI 6
     next    n  application data (Table 83: 104 … 1 000 bits)
     next    4  spare "needed for byte alignment" (Table 82)
     last 20     communication state: 1 selector bit (0 = SOTDMA, 1 = ITDMA)
                 followed by the 19-bit SOTDMA or ITDMA state

The only way to find the communication state is to count back 20 bits
from the end of the payload — the data length is "whatever is left". We
cannot tell the 4-bit alignment spare from application data (the
sender may have omitted it, or padded more), so `data` includes it and
the chapter explains how to reason about `data_bits % 8`.
"""

from __future__ import annotations

from .bits import Bits
from .common import require_bits
from .msg_1_2_3 import decode_itdma, decode_sotdma
from .msg_25 import decode_binary_header

COMM_STATE_BITS = 20   # 1 selector bit + 19 bits of SOTDMA/ITDMA state


def decode(bits: Bits) -> dict:
    """Decode a message 26. Raises DecodeError if header or comm state is missing."""
    out, offset = decode_binary_header(bits, 26)
    # The communication state is the *last* 20 bits, whatever the data length.
    require_bits(26, len(bits), offset + COMM_STATE_BITS)
    comm_start = len(bits) - COMM_STATE_BITS
    out["data"] = bits.raw(offset, comm_start - offset)
    out["data_bits"] = comm_start - offset
    selector = bits.uint(comm_start, 1)
    out["comm_state_selector"] = selector
    if selector == 1:
        out["comm_state"] = decode_itdma(bits, comm_start + 1)
    else:
        out["comm_state"] = decode_sotdma(bits, comm_start + 1)
    return out
