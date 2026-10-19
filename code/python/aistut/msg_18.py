"""msg_18.py — message 18: standard Class B equipment position report (chapter 14).

Spec: ITU-R M.1371-5 Annex 8 §3.16, Table 70 (168 bits; this was Table 67
in M.1371-3 and -4, which is why some older references number it that way).
Communication states: Annex 2 §3.3.7.2.2 Table 18/19 (SOTDMA) and
§3.3.7.3.2 Table 20 (ITDMA). Class B "CS" behaviour: Annex 7.

Bit layout (zero-based offsets; lengths from Table 70):

     0- 5   6  message id (18)
     6- 7   2  repeat indicator (should be 0 for "CS" units)
     8-37  30  user id (MMSI)
    38-45   8  spare ("regional reserved" in older editions); should be 0
    46-55  10  speed over ground, 1/10 knot; 1023 = n/a, 1022 = ≥102.2 kn
    56      1  position accuracy (1 = high, ≤10 m)
    57-84  28  longitude, signed, 1/10 000 minute; 181° = n/a
    85-111 27  latitude,  signed, 1/10 000 minute;  91° = n/a
   112-123 12  course over ground, 1/10 degree; 3600 = n/a
   124-132  9  true heading, degrees; 511 = n/a
   133-138  6  time stamp (UTC second); 60 = n/a; 61-63 not used by "CS"
   139-140  2  spare (regional reserved in older editions); should be 0
   141      1  Class B unit flag: 0 = SOTDMA ("SO") unit, 1 = "CS" unit
   142      1  Class B display flag: 1 = can display messages 12 and 14
   143      1  Class B DSC flag: 1 = has DSC function
   144      1  Class B band flag: 1 = whole marine band, 0 = upper 525 kHz only
   145      1  Class B message 22 flag: 1 = accepts channel management
   146      1  assigned mode flag: 1 = assigned mode
   147      1  RAIM flag
   148      1  communication state selector: 0 = SOTDMA follows, 1 = ITDMA follows
   149-167 19  communication state; a "CS" unit fills it with 1100000000000000110

The communication state of a "CS" unit is a constant, because carrier-sense
units do not reserve slots. Read as ITDMA (selector = 1) the constant means
sync state 3 ("synchronized to another station"), slot increment 0,
number of slots 3 (= 4 slots) and keep flag 0 — all meaningless; we decode
it like any other ITDMA state but also flag it as `cs_fixed_comm_state`.

Python notes for readers new to the language
--------------------------------------------
* We import `decode_sotdma`, `decode_itdma` and `SYNC_STATE` from the
  message 1-3 module rather than copying them: the 19-bit communication
  state is identical in every message that carries one.
"""

from __future__ import annotations

from .bits import Bits
from .common import require_bits
from .jsonout import Fixed
from .msg_1_2_3 import decode_itdma, decode_sotdma

# Table 70: the 19-bit value every Class B "CS" unit puts in its
# communication state field.
CS_FIXED_COMM_STATE = "1100000000000000110"

CLASS_B_UNIT = {0: "Class B SOTDMA unit", 1: "Class B CS unit"}
CLASS_B_DISPLAY = {0: "no display for messages 12 and 14", 1: "integrated display for messages 12 and 14"}
CLASS_B_DSC = {0: "no DSC function", 1: "DSC function (dedicated or time-shared)"}
CLASS_B_BAND = {0: "upper 525 kHz of the marine band only", 1: "whole marine band"}
CLASS_B_MSG22 = {0: "AIS 1 and AIS 2 only, no message 22 frequency management",
                 1: "frequency management via message 22"}
ASSIGNED_MODE = {0: "autonomous and continuous", 1: "assigned"}


def decode_common_position(bits: Bits, start: int) -> dict:
    """SOG…time stamp as laid out in messages 18 and 19.

    Both messages share the block that starts with the 10-bit SOG at
    `start` (46 in both cases) and ends with the 6-bit time stamp, so the
    same code reads both. Offsets below are relative to `start`.
    """
    sog_raw = bits.uint(start, 10)
    lon_raw = bits.sint(start + 11, 28)
    lat_raw = bits.sint(start + 39, 27)
    cog_raw = bits.uint(start + 66, 12)
    heading = bits.uint(start + 78, 9)
    timestamp = bits.uint(start + 87, 6)
    return {
        "sog_knots": Fixed(sog_raw / 10.0, 1),
        "sog_available": sog_raw != 1023,
        "position_accuracy": bits.uint(start + 10, 1),
        "lon": Fixed(lon_raw / 600000.0, 6),
        "lat": Fixed(lat_raw / 600000.0, 6),
        "position_available": lon_raw != 0x6791AC0 and lat_raw != 0x3412140,
        "cog_deg": Fixed(cog_raw / 10.0, 1),
        "cog_available": cog_raw != 3600,
        "true_heading": heading,
        "heading_available": heading != 511,
        "timestamp": timestamp,
    }


def decode(bits: Bits) -> dict:
    """Decode a message 18. Raises DecodeError if shorter than 168 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 168)
    unit = bits.uint(141, 1)
    display = bits.uint(142, 1)
    dsc = bits.uint(143, 1)
    band = bits.uint(144, 1)
    msg22 = bits.uint(145, 1)
    assigned = bits.uint(146, 1)
    selector = bits.uint(148, 1)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 8),
    }
    out.update(decode_common_position(bits, 46))
    out.update({
        "spare2": bits.uint(139, 2),
        "class_b_unit": unit,
        "class_b_unit_text": CLASS_B_UNIT[unit],
        "class_b_display": display,
        "class_b_display_text": CLASS_B_DISPLAY[display],
        "class_b_dsc": dsc,
        "class_b_dsc_text": CLASS_B_DSC[dsc],
        "class_b_band": band,
        "class_b_band_text": CLASS_B_BAND[band],
        "class_b_msg22": msg22,
        "class_b_msg22_text": CLASS_B_MSG22[msg22],
        "assigned_mode": assigned,
        "assigned_mode_text": ASSIGNED_MODE[assigned],
        "raim": bits.flag(147),
        "comm_state_selector": selector,
        "cs_fixed_comm_state": bits.raw(149, 19) == CS_FIXED_COMM_STATE,
    })
    if selector == 1:
        out["comm_state"] = decode_itdma(bits, 149)
    else:
        out["comm_state"] = decode_sotdma(bits, 149)
    return out
