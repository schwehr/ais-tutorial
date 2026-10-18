"""msg_24.py — message 24: static data report, parts A and B (chapter 14).

Spec: ITU-R M.1371-5 Annex 8 §3.22, Table 78 (part A, 160 bits) and
Table 79 (part B, 168 bits), Table 79A (vendor ID sub-fields). These were
Tables 75, 76 and 76A in M.1371-4, which introduced the vendor-ID split
and the mother-ship MMSI rule. Dimensions: §3.3.3 Fig. 41. Ship type:
Table 53. EPFD: Table 51. Auxiliary-craft MMSI format 98MIDxxxx: ITU-R
M.585 Annex 1.

Bit layout (zero-based offsets):

  common   0- 5   6  message id (24)
           6- 7   2  repeat indicator
           8-37  30  user id (MMSI)
          38-39   2  part number: 0 = part A, 1 = part B, 2-3 not allowed
  part A  40-159 120 name, 20 six-bit characters; all '@' = n/a
         160-167  8  (spare) — present only on units that send 168 bits
  part B  40- 47  8  type of ship and cargo type; 0 = n/a
          48- 89 42  vendor ID; since M.1371-4 split into
                        48-65 18  manufacturer's ID, 3 six-bit characters
                        66-69  4  unit model code
                        70-89 20  unit serial number
          90-131 42  call sign, 7 six-bit characters; all '@' = n/a
         132-161 30  dimensions / reference point (A 9, B 9, C 6, D 6), or
                     for an auxiliary craft (MMSI 98MIDxxxx) the MMSI of the
                     mother ship
         162-167  6  spare

Length rule: part A needs 160 bits; anything beyond 160 (eight spare bits
on units built to the pre-M.1371-4 table, or two left-over bits on units
that do not set the fill-bit count) is reported as `spare` with its width
in `spare_bits`. Part B needs 168 bits. Part numbers 2 and 3 return the
remainder of the payload undecoded as `data`/`data_bits`.

Python notes for readers new to the language
--------------------------------------------
* `if … elif … else` branches on the part number and each branch fills in
  different keys. A reader of the JSON can tell the parts apart by
  `part_number` (and `part_text`), not by which keys exist.
* `980_000_000 <= mmsi < 990_000_000`: Python allows underscores inside
  numeric literals, and chained comparisons `a <= x < b` read like maths.
"""

from __future__ import annotations

from .bits import Bits
from .common import require_bits, ship_type_text
from .msg_19 import decode_dimensions

PART_TEXT = {0: "part A", 1: "part B", 2: "reserved", 3: "reserved"}


def is_auxiliary_craft(mmsi: int) -> bool:
    """True for MMSIs of the form 98MIDxxxx (craft associated with a parent ship)."""
    return 980_000_000 <= mmsi < 990_000_000


def decode(bits: Bits) -> dict:
    """Decode a message 24 part A or part B. Raises DecodeError if too short."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 40)
    mmsi = bits.uint(8, 30)
    part = bits.uint(38, 2)
    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": mmsi,
        "part_number": part,
        "part_text": PART_TEXT[part],
    }

    if part == 0:
        require_bits(msg_type, len(bits), 160)
        out["name"] = bits.text(40, 120)
        spare_bits = len(bits) - 160
        out["spare_bits"] = spare_bits
        out["spare"] = bits.uint(160, min(spare_bits, 8)) if spare_bits > 0 else 0
    elif part == 1:
        require_bits(msg_type, len(bits), 168)
        ship_type = bits.uint(40, 8)
        out.update({
            "ship_type": ship_type,
            "ship_type_text": ship_type_text(ship_type),
            "vendor_id": bits.text(48, 42),
            "manufacturer_id": bits.text(48, 18),
            "unit_model_code": bits.uint(66, 4),
            "unit_serial_number": bits.uint(70, 20),
            "call_sign": bits.text(90, 42),
            "auxiliary_craft": is_auxiliary_craft(mmsi),
            "spare": bits.uint(162, 6),
        })
        if is_auxiliary_craft(mmsi):
            out["mothership_mmsi"] = bits.uint(132, 30)
        else:
            out.update(decode_dimensions(bits, 132))
    else:
        out["data"] = bits.raw(40, len(bits) - 40)
        out["data_bits"] = len(bits) - 40
    return out
