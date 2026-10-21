"""msg_1_2_3.py — Class A position reports, messages 1, 2 and 3 (chapter 3).

Spec: ITU-R M.1371-5 Annex 8 §3.1, Table 48 (field layout, 168 bits);
Table 49 (which communication state each message carries); Annex 2
§3.3.7.2.2 Table 18/19 (SOTDMA communication state) and §3.3.7.3.2
Table 20 (ITDMA communication state).

Bit layout (zero-based offsets; lengths from Table 48):

     0- 5   6  message id (1, 2 or 3)
     6- 7   2  repeat indicator
     8-37  30  user id (MMSI)
    38-41   4  navigational status
    42-49   8  rate of turn, signed, special encoding
    50-59  10  speed over ground, 1/10 knot; 1023 = n/a, 1022 = ≥102.2 kn
    60      1  position accuracy (1 = high, ≤10 m)
    61-88  28  longitude, signed, 1/10 000 minute; 181° = n/a
    89-115 27  latitude,  signed, 1/10 000 minute;  91° = n/a
   116-127 12  course over ground, 1/10 degree; 3600 = n/a
   128-136  9  true heading, degrees; 511 = n/a
   137-142  6  time stamp (UTC second); 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
   143-144  2  special manoeuvre indicator
   145-147  3  spare
   148      1  RAIM flag
   149-167 19  communication state (SOTDMA for 1 and 2, ITDMA for 3)
"""

from __future__ import annotations

from typing import Optional

from .bits import Bits
from .common import require_bits
from .jsonout import Fixed

NAV_STATUS = {
    0: "under way using engine",
    1: "at anchor",
    2: "not under command",
    3: "restricted manoeuvrability",
    4: "constrained by her draught",
    5: "moored",
    6: "aground",
    7: "engaged in fishing",
    8: "under way sailing",
    9: "reserved for HSC",
    10: "reserved for WIG",
    11: "power-driven vessel towing astern",
    12: "power-driven vessel pushing ahead or towing alongside",
    13: "reserved for future use",
    14: "AIS-SART / MOB-AIS / EPIRB-AIS active",
    15: "undefined (default)",
}

SYNC_STATE = {
    0: "UTC direct",
    1: "UTC indirect",
    2: "synchronized to base station",
    3: "synchronized to another station",
}

SPECIAL_MANOEUVRE = {0: "not available", 1: "not engaged", 2: "engaged", 3: "reserved"}


def decode_rot(raw: int) -> dict:
    """Interpret the 8-bit rate-of-turn field (Table 48).

    The sensor value is compressed with a square root:
        ROT_AIS = 4.733 * sqrt(ROT_sensor)   (sign preserved)
    so to recover degrees per minute we square: (raw / 4.733)**2.
    """
    out: dict = {"rot_raw": raw}
    if raw == -128:
        out["rot_deg_per_min"] = None
        out["rot_text"] = "not available"
    elif raw == 127:
        out["rot_deg_per_min"] = None
        out["rot_text"] = "turning right > 5 deg per 30 s (no turn indicator)"
    elif raw == -127:
        out["rot_deg_per_min"] = None
        out["rot_text"] = "turning left > 5 deg per 30 s (no turn indicator)"
    else:
        value = (raw / 4.733) ** 2
        if raw < 0:
            value = -value
        out["rot_deg_per_min"] = Fixed(value, 1)
        out["rot_text"] = "turning right" if raw > 0 else "turning left" if raw < 0 else "not turning"
    return out


def decode_sotdma(bits: Bits, start: int) -> dict:
    """SOTDMA communication state, 19 bits (Annex 2 Table 18 and 19)."""
    sync = bits.uint(start, 2)
    timeout = bits.uint(start + 2, 3)
    sub = bits.uint(start + 5, 14)
    out: dict = {
        "scheme": "SOTDMA",
        "sync_state": sync,
        "sync_state_text": SYNC_STATE[sync],
        "slot_timeout": timeout,
    }
    # The meaning of the 14-bit sub-message depends on the slot time-out.
    if timeout in (3, 5, 7):
        out["received_stations"] = sub
    elif timeout in (2, 4, 6):
        out["slot_number"] = sub
    elif timeout == 1:
        out["utc_hour"] = sub >> 9            # bits 13..9
        out["utc_minute"] = (sub >> 2) & 0x7F  # bits 8..2 (7 bits)
    else:  # timeout == 0
        out["slot_offset"] = sub
    return out


def decode_itdma(bits: Bits, start: int) -> dict:
    """ITDMA communication state, 19 bits (Annex 2 Table 20)."""
    sync = bits.uint(start, 2)
    increment = bits.uint(start + 2, 13)
    nslots = bits.uint(start + 15, 3)
    keep = bits.flag(start + 18)
    out: dict = {
        "scheme": "ITDMA",
        "sync_state": sync,
        "sync_state_text": SYNC_STATE[sync],
        "slot_increment": increment,
        "slots_raw": nslots,
        "keep_flag": keep,
    }
    # Table 20: 0..4 mean 1..5 slots; 5..7 mean 1..3 slots with +8192 offset.
    if nslots <= 4:
        out["slots"] = nslots + 1
        out["slot_offset"] = increment
    else:
        out["slots"] = nslots - 4
        out["slot_offset"] = increment + 8192
    return out


def decode(bits: Bits) -> dict:
    """Decode a message 1, 2 or 3. Raises DecodeError if too short."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 168)
    nav = bits.uint(38, 4)
    sog_raw = bits.uint(50, 10)
    lon_raw = bits.sint(61, 28)
    lat_raw = bits.sint(89, 27)
    cog_raw = bits.uint(116, 12)
    heading = bits.uint(128, 9)
    timestamp = bits.uint(137, 6)
    manoeuvre = bits.uint(143, 2)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "nav_status": nav,
        "nav_status_text": NAV_STATUS[nav],
        "sog_knots": Fixed(sog_raw / 10.0, 1),
        "sog_available": sog_raw != 1023,
        "position_accuracy": bits.uint(60, 1),
        "lon": Fixed(lon_raw / 600000.0, 6),
        "lat": Fixed(lat_raw / 600000.0, 6),
        "position_available": lon_raw != 0x6791AC0 and lat_raw != 0x3412140,
        "cog_deg": Fixed(cog_raw / 10.0, 1),
        "cog_available": cog_raw != 3600,
        "true_heading": heading,
        "heading_available": heading != 511,
        "timestamp": timestamp,
        "special_manoeuvre": manoeuvre,
        "special_manoeuvre_text": SPECIAL_MANOEUVRE[manoeuvre],
        "spare": bits.uint(145, 3),
        "raim": bits.flag(148),
    }
    out.update(decode_rot(bits.sint(42, 8)))
    if msg_type == 3:
        out["comm_state"] = decode_itdma(bits, 149)
    else:
        out["comm_state"] = decode_sotdma(bits, 149)
    return out
