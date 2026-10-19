"""msg_16.py — message 16: assignment mode command.

Spec: ITU-R M.1371-5 Annex 8 §3.14, Table 67 (96 or 144 bits) and the
note (1) and paragraph below it that give the meaning of offset and
increment. Annex 2 §3.3.6 describes what the assigned station does.

Bit layout (zero-based offsets; lengths from Table 67):

     0- 5   6  message id (16)
     6- 7   2  repeat indicator
     8-37  30  source id (MMSI of the assigning base station)
    38-39   2  spare
    40-69  30  destination id A
    70-81  12  offset A
    82-91  10  increment A
    ------- then EITHER
    92-95   4  spare (padding to a byte boundary)   -> 96 bits
    ------- OR
    92-121 30  destination id B
   122-133 12  offset B
   134-143 10  increment B                          -> 144 bits

The (offset, increment) pair has two readings [Table 67 note 1]:

  * increment == 0  -> *reporting-rate assignment*: `offset` is the
    number of reports the station must make per 10 minutes. Only
    multiples of 20 between 20 and 600 should be used; a station rounds
    other values up to the next multiple of 20 and clamps above 600.
  * increment != 0  -> *slot assignment*: `offset` is the number of slots
    from the slot of this message 16 to the first assigned slot, and
    `increment` is a code for the spacing between assigned slots:
    1 = 1125, 2 = 375, 3 = 225, 4 = 125, 5 = 75, 6 = 45 slots;
    7 = undefined ("the station should disregard this assignment").
    Values 8-1023 are not defined in M.1371-5. (In M.1371-1 the field
    was a raw slot count, 0-1023; the code table arrived with M.1371-3.)

Output: an array `assignments` with one object per destination:
    {"mmsi", "offset", "increment", "assignment_text",
     "reports_per_10_min" (increment 0) or null,
     "slot_increment" (codes 1-6) or null}

Length rule: at least 96 bits (otherwise DecodeError). If 144 bits are
present the second assignment is decoded; otherwise bits 92-95 are
reported as `spare2`. Anything else is counted in `trailing_bits`.
"""

from __future__ import annotations

from typing import Optional

from .bits import Bits
from .common import require_bits

# Table 67 text under note (1): increment code -> slots between assigned slots.
SLOT_INCREMENTS = {1: 1125, 2: 375, 3: 225, 4: 125, 5: 75, 6: 45}

# 2250 slots per minute per channel (Annex 2 §3.1.2), so 1125 slots = 30 s,
# 375 = 10 s, 225 = 6 s, 125 = 3.33 s, 75 = 2 s, 45 = 1.2 s.


def assignment(mmsi: int, offset: int, increment: int) -> dict:
    """Interpret one (destination, offset, increment) triple."""
    reports: Optional[int] = None
    slots: Optional[int] = None
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
