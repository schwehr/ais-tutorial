"""msg_21.py — message 21: aids-to-navigation (AtoN) report (chapter 15).

Spec: ITU-R M.1371-5 Annex 8 §3.19, Table 73 (field layout, 272–360 bits);
Table 74 (the 32 AtoN types); Figure 41/41bis and §3.3.3 (dimension /
reference-point encoding, shared with message 5); Table 51 (EPFD types,
shared with message 4). The eight "AtoN status" bits are only *reserved*
by ITU; their page-7 layout (RACON / light / health) comes from IALA
Recommendation R0126 (A-126) ed. 2 §4.8.4 Figure 3.

Bit layout (zero-based offsets; lengths from Table 73):

     0- 5    6  message id (21)
     6- 7    2  repeat indicator
     8- 37  30  user id (MMSI; AtoN format 99MIDxxxx per ITU-R M.585)
    38- 42   5  type of aid to navigation (Table 74); 0 = not available
    43-162 120  name, 20 six-bit characters; all '@' = not available
   163       1  position accuracy (1 = high, ≤10 m)
   164-191  28  longitude, signed, 1/10 000 minute; 181° = n/a
   192-218  27  latitude,  signed, 1/10 000 minute;  91° = n/a
   219-227   9  dimension A (reference point to "bow" / true north), m
   228-236   9  dimension B (to "stern"), m
   237-242   6  dimension C (to port), m
   243-248   6  dimension D (to starboard), m
   249-252   4  type of EPFD (Table 51); 7 = surveyed is normal for fixed/virtual aids
   253-258   6  UTC second; 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
   259       1  off-position indicator (floating AtoN only; valid only if second <= 59)
   260-267   8  AtoN status — "reserved for the indication of the AtoN status"
   268       1  RAIM flag
   269       1  virtual AtoN flag (1 = does not physically exist)
   270       1  assigned mode flag
   271       1  spare
   272-      0..84  name extension: 0–14 more six-bit characters, then
                0, 2, 4 or 6 spare bits to reach a byte boundary

Variable length rule used here: everything after bit 272 is the name
extension. We decode floor((len - 272) / 6) whole characters (at most 14)
and report the raw number of extension bits as `name_extension_bits`. The
spec says the extension never contains '@'; if one is present anyway
(six padding bits look exactly like a '@'), `Bits.text` cuts the string
there, as for every other text field.
"""

from __future__ import annotations

from .bits import Bits
from .common import epfd_text, require_bits
from .jsonout import Fixed

# Table 74. Codes 5–19 are listed under "Fixed AtoN", 20–31 under "Floating
# AtoN"; 1–4 are general. Code 4 was "spare" in M.1371-4 and became
# "Emergency Wreck Marking Buoy" in M.1371-5.
ATON_TYPES = {
    0: "not specified (default)",
    1: "reference point",
    2: "RACON",
    3: "fixed structure off-shore",
    4: "emergency wreck marking buoy",
    5: "light, without sectors",
    6: "light, with sectors",
    7: "leading light front",
    8: "leading light rear",
    9: "beacon, cardinal N",
    10: "beacon, cardinal E",
    11: "beacon, cardinal S",
    12: "beacon, cardinal W",
    13: "beacon, port hand",
    14: "beacon, starboard hand",
    15: "beacon, preferred channel port hand",
    16: "beacon, preferred channel starboard hand",
    17: "beacon, isolated danger",
    18: "beacon, safe water",
    19: "beacon, special mark",
    20: "cardinal mark N",
    21: "cardinal mark E",
    22: "cardinal mark S",
    23: "cardinal mark W",
    24: "port hand mark",
    25: "starboard hand mark",
    26: "preferred channel port hand",
    27: "preferred channel starboard hand",
    28: "isolated danger",
    29: "safe water",
    30: "special mark",
    31: "light vessel / LANBY / rig",
}

# IALA R0126 Figure 3, status page 7 ("111 RR LL H").
RACON_STATUS = {
    0: "no RACON installed",
    1: "RACON installed but not monitored",
    2: "RACON operational",
    3: "RACON error",
}
LIGHT_STATUS = {
    0: "no light or no monitoring",
    1: "light on",
    2: "light off",
    3: "light fail or at reduced range",
}

MIN_BITS = 272            # Table 73: the fixed part (two slots)
MAX_EXTENSION_CHARS = 14  # Table 73: "up to 14 additional 6-bit-ASCII characters"


def aton_type_text(code: int) -> str:
    return ATON_TYPES[code]


def is_floating(aton_type: int) -> bool:
    """True for the Table 74 codes that describe a floating aid (a buoy).

    Table 74 heads codes 20–31 "Floating AtoN"; code 4, the emergency wreck
    marking buoy, is listed above that heading but is a buoy as well.
    """
    return aton_type == 4 or 20 <= aton_type <= 31


def decode_status(status: int) -> dict:
    """Interpret the 8 AtoN status bits.

    ITU only reserves them (Table 73). IALA R0126 §4.8.4 splits them into a
    3-bit page id (the most significant bits) and 5 page-dependent bits;
    page 0 is "no data" (the all-zero default) and page 7 carries the
    RACON / light / health summary that monitoring systems key on.
    """
    page = status >> 5
    out: dict = {"aton_status_page": page}
    if page == 7:
        racon = (status >> 3) & 0x3
        light = (status >> 1) & 0x3
        out["racon_status"] = racon
        out["racon_status_text"] = RACON_STATUS[racon]
        out["light_status"] = light
        out["light_status_text"] = LIGHT_STATUS[light]
        out["health_alarm"] = (status & 0x1) == 1
    return out


def decode(bits: Bits) -> dict:
    """Decode a message 21. Raises DecodeError if shorter than 272 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), MIN_BITS)
    aton_type = bits.uint(38, 5)
    lon_raw = bits.sint(164, 28)
    lat_raw = bits.sint(192, 27)
    epfd = bits.uint(249, 4)
    timestamp = bits.uint(253, 6)
    status = bits.uint(260, 8)

    # --- the variable-length name extension -------------------------------
    # Table 73: "0, 6, 12, … 84" bits of six-bit characters followed by 0, 2,
    # 4 or 6 spare bits so that the message ends on a byte boundary. Only
    # whole characters are decoded; a partial character is ignored.
    extension_bits = len(bits) - MIN_BITS
    extension_chars = min(extension_bits // 6, MAX_EXTENSION_CHARS)
    name = bits.text(43, 120)
    name_extension = bits.text(MIN_BITS, extension_chars * 6)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "aton_type": aton_type,
        "aton_type_text": aton_type_text(aton_type),
        "name": name,
        "name_extension": name_extension,
        "name_extension_bits": extension_bits,
        "full_name": name + name_extension,
        "position_accuracy": bits.uint(163, 1),
        "lon": Fixed(lon_raw / 600000.0, 6),
        "lat": Fixed(lat_raw / 600000.0, 6),
        "position_available": lon_raw != 0x6791AC0 and lat_raw != 0x3412140,
        "dim_a_m": bits.uint(219, 9),
        "dim_b_m": bits.uint(228, 9),
        "dim_c_m": bits.uint(237, 6),
        "dim_d_m": bits.uint(243, 6),
        "epfd": epfd,
        "epfd_text": epfd_text(epfd),
        "timestamp": timestamp,
        "off_position": bits.flag(259),
        # Table 73 note 1: the flag means something only for a floating aid
        # whose time stamp is a real second (0–59).
        "off_position_valid": is_floating(aton_type) and timestamp <= 59,
        "aton_status": status,
        "aton_status_bits": bits.raw(260, 8),
        "raim": bits.flag(268),
        "virtual_aton": bits.flag(269),
        "assigned_mode": bits.flag(270),
        "spare": bits.uint(271, 1),
    }
    out.update(decode_status(status))
    return out
