"""msg_5.py — message 5: ship static and voyage related data (chapter 7).

Spec: ITU-R M.1371-5 Annex 8 §3.3, Table 52 (field layout, 424 bits, two
slots); §3.3.1 (DTE); §3.3.2 Table 53 (type of ship and cargo type);
§3.3.3 Figure 41 (reference point and dimensions A/B/C/D); Table 47
(six-bit ASCII). The EPFD code list is Table 51 (shared via common.py).

Bit layout (zero-based offsets; lengths from Table 52):

      0-  5   6  message id (5)
      6-  7   2  repeat indicator
      8- 37  30  user id (MMSI)
     38- 39   2  AIS version indicator (0 = M.1371-1, 1 = -3, 2 = -5, 3 = future)
     40- 69  30  IMO number; 0 = n/a; 1..999999 not used;
                 1000000..9999999 IMO number; 10000000.. flag-state number
     70-111  42  call sign, 7 six-bit characters, "@@@@@@@" = n/a
    112-231 120  name, 20 six-bit characters, all '@' = n/a
    232-239   8  type of ship and cargo type (Table 53); 0 = n/a
    240-248   9  dimension A, metres from reference point to bow;   511 = ≥511 m
    249-257   9  dimension B, metres to stern;                      511 = ≥511 m
    258-263   6  dimension C, metres to port side;                   63 = ≥63 m
    264-269   6  dimension D, metres to starboard side;              63 = ≥63 m
    270-273   4  type of electronic position fixing device (EPFD)
    274-277   4  ETA month  1..12;  0 = n/a
    278-282   5  ETA day    1..31;  0 = n/a
    283-287   5  ETA hour   0..23; 24 = n/a
    288-293   6  ETA minute 0..59; 60 = n/a
    294-301   8  maximum present static draught, 1/10 m; 0 = n/a; 255 = ≥25.5 m
    302-421 120  destination, 20 six-bit characters, all '@' = n/a
    422       1  DTE: 0 = data terminal available, 1 = not available (default)
    423       1  spare

Length rule. Table 52 says 424 bits, but recordings contain 426-bit
messages (sender under-counted the fill bits by two, see AIVDM.adoc
"Fill bits and pad errors") and occasionally 420/422-bit ones. We require
the full 424 bits — the last real field, DTE, is bit 422 — and silently
ignore anything after bit 423. A 420- or 422-bit message is reported as
`message 5 too short`, which is the honest answer: we cannot tell which
field the sender dropped.
"""

from __future__ import annotations

from .bits import Bits
from .common import epfd_text, require_bits, ship_type_text
from .jsonout import Fixed

# Table 52, "AIS version indicator".
AIS_VERSION = {
    0: "ITU-R M.1371-1",
    1: "ITU-R M.1371-3 or later",
    2: "ITU-R M.1371-5 or later",
    3: "future edition",
}

# Table 52, "DTE" (data terminal equipment ready), §3.3.1.
DTE_TEXT = {0: "available", 1: "not available (default)"}


def imo_range_text(imo: int) -> str:
    """Which of the four ranges of Table 52 the 30-bit IMO field falls in."""
    if imo == 0:
        return "not available (default)"
    if imo <= 999999:
        return "not used"
    if imo <= 9999999:
        return "IMO number"
    return "official flag state number"


def imo_check_digit_ok(imo: int) -> bool:
    """True if `imo` is a seven-digit IMO number whose check digit is right.

    An IMO number (Lloyd's Register / IHS number, made mandatory by SOLAS
    V/3 and shown as "IMO 9074729" on the hull) has six digits plus one
    check digit. Multiply the six digits by 7, 6, 5, 4, 3, 2 in turn, add
    the products, and the last digit of the sum is the check digit:

        9074729:  9*7 + 0*6 + 7*5 + 4*4 + 7*3 + 2*2 = 139  ->  9  ok

    Numbers outside 1000000..9999999 are never valid IMO numbers, so the
    "official flag state number" range of Table 52 returns False here;
    look at `imo_range_text` to tell the two cases apart.
    """
    if not 1000000 <= imo <= 9999999:
        return False
    digits = [int(c) for c in str(imo)]
    total = sum(d * w for d, w in zip(digits[:6], (7, 6, 5, 4, 3, 2)))
    return total % 10 == digits[6]


def eta_text(month: int, day: int, hour: int, minute: int) -> str:
    """'MM-DD HH:MM' in UTC with '??' for each part that is not available."""
    mm = f"{month:02d}" if 1 <= month <= 12 else "??"
    dd = f"{day:02d}" if 1 <= day <= 31 else "??"
    hh = f"{hour:02d}" if hour <= 23 else "??"
    mi = f"{minute:02d}" if minute <= 59 else "??"
    return f"{mm}-{dd} {hh}:{mi}"


def decode(bits: Bits) -> dict:
    """Decode a message 5. Raises DecodeError if shorter than 424 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 424)
    ais_version = bits.uint(38, 2)
    imo = bits.uint(40, 30)
    ship_type = bits.uint(232, 8)
    dim_a = bits.uint(240, 9)      # to bow
    dim_b = bits.uint(249, 9)      # to stern
    dim_c = bits.uint(258, 6)      # to port
    dim_d = bits.uint(264, 6)      # to starboard
    epfd = bits.uint(270, 4)
    month = bits.uint(274, 4)
    day = bits.uint(278, 5)
    hour = bits.uint(283, 5)
    minute = bits.uint(288, 6)
    draught_raw = bits.uint(294, 8)
    dte = bits.uint(422, 1)

    return {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "ais_version": ais_version,
        "ais_version_text": AIS_VERSION[ais_version],
        "imo": imo,
        "imo_available": imo != 0,
        "imo_range_text": imo_range_text(imo),
        "imo_valid": imo_check_digit_ok(imo),
        "call_sign": bits.text(70, 42),
        "name": bits.text(112, 120),
        "ship_type": ship_type,
        "ship_type_text": ship_type_text(ship_type),
        "dim_bow_m": dim_a,
        "dim_stern_m": dim_b,
        "dim_port_m": dim_c,
        "dim_starboard_m": dim_d,
        # Figure 41: all four zero = neither reference point nor dimensions;
        # A = C = 0 with B, D non-zero = dimensions but no reference point.
        "dimensions_available": not (dim_a == 0 and dim_b == 0 and dim_c == 0 and dim_d == 0),
        "reference_point_available": not (dim_a == 0 and dim_c == 0),
        "length_m": dim_a + dim_b,
        "beam_m": dim_c + dim_d,
        "epfd": epfd,
        "epfd_text": epfd_text(epfd),
        "eta_month": month,
        "eta_day": day,
        "eta_hour": hour,
        "eta_minute": minute,
        "eta_text": eta_text(month, day, hour, minute),
        "eta_available": 1 <= month <= 12 and 1 <= day <= 31 and hour <= 23 and minute <= 59,
        "draught_m": Fixed(draught_raw / 10.0, 1),
        "draught_available": draught_raw != 0,
        "destination": bits.text(302, 120),
        "dte": dte,
        "dte_text": DTE_TEXT[dte],
        "spare": bits.uint(423, 1),
    }
