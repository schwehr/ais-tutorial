"""msg_19.py — message 19: extended Class B equipment position report (chapter 14).

Spec: ITU-R M.1371-5 Annex 8 §3.17, Table 71 (312 bits, two slots; this
was Table 68 in M.1371-4). Dimension/reference-point field: §3.3.3 Fig. 41.
Ship type: §3.3.2 Table 53. EPFD: Table 51. DTE: §3.3.1.

M.1371-5 says this message "is not needed and should not be used" by new
equipment — a Class B unit should send message 18 plus message 24 A/B
instead — but legacy units still send it once every 6 minutes, and base
stations may request it with an interrogation (message 15).

Bit layout (zero-based offsets; lengths from Table 71). Bits 0-138 are
laid out exactly as in message 18, so we reuse that decoder's helper.

     0- 5   6  message id (19)
     6- 7   2  repeat indicator
     8-37  30  user id (MMSI)
    38-45   8  spare
    46-55  10  speed over ground, 1/10 knot; 1023 = n/a
    56      1  position accuracy
    57-84  28  longitude, signed, 1/10 000 minute; 181° = n/a
    85-111 27  latitude,  signed, 1/10 000 minute;  91° = n/a
   112-123 12  course over ground, 1/10 degree; 3600 = n/a
   124-132  9  true heading; 511 = n/a
   133-138  6  time stamp (UTC second); 60 = n/a
   139-142  4  spare
   143-262 120 name, 20 six-bit characters; all '@' = n/a
   263-270  8  type of ship and cargo type; 0 = n/a
   271-300 30  dimensions / reference point: A 9, B 9, C 6, D 6 (Fig. 41)
   301-304  4  type of EPFD; 0 = undefined
   305      1  RAIM flag
   306      1  DTE: 0 = data terminal available, 1 = not available (default)
   307      1  assigned mode flag
   308-311  4  spare

Python notes for readers new to the language
--------------------------------------------
* `decode_dimensions` returns a dict that the caller merges with
  `out.update(...)`; message 24 part B imports and reuses it. Sharing a
  helper between modules is just an `import` — there is no header file.
"""

from .bits import Bits
from .common import epfd_text, require_bits, ship_type_text
from .msg_18 import ASSIGNED_MODE, decode_common_position

DTE: dict[int, str] = {0: "data terminal available", 1: "data terminal not available"}


def decode_dimensions(bits: Bits, start: int) -> dict:
    """The 30-bit dimension / reference-point field (M.1371-5 §3.3.3 Fig. 41).

    A = distance from the reference point (the GNSS antenna) to the bow,
    B = to the stern, C = to the port side, D = to the starboard side, all
    in metres. A and B saturate at 511, C and D at 63. All four zero means
    "nothing available"; A = C = 0 with B and D non-zero means "dimensions
    known but reference point not". The keys and the two availability
    rules are exactly those of message 5 (msg_5.py), so that Class A and
    Class B static data can be joined without renaming; message 21 reuses
    this helper as well.
    """
    a = bits.uint(start, 9)
    b = bits.uint(start + 9, 9)
    c = bits.uint(start + 18, 6)
    d = bits.uint(start + 24, 6)
    return {
        "dim_bow_m": a,
        "dim_stern_m": b,
        "dim_port_m": c,
        "dim_starboard_m": d,
        # Figure 41: all four zero = neither reference point nor dimensions;
        # A = C = 0 with B, D non-zero = dimensions but no reference point.
        # Either way A = C = 0 means the reference point is unknown.
        "dimensions_available": not (a == 0 and b == 0 and c == 0 and d == 0),
        "reference_point_available": not (a == 0 and c == 0),
        "length_m": a + b,
        "beam_m": c + d,
    }


def decode(bits: Bits) -> dict:
    """Decode a message 19. Raises DecodeError if shorter than 312 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 312)
    ship_type = bits.uint(263, 8)
    epfd = bits.uint(301, 4)
    dte = bits.uint(306, 1)
    assigned = bits.uint(307, 1)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 8),
    }
    out.update(decode_common_position(bits, 46))
    out.update({
        "spare2": bits.uint(139, 4),
        "name": bits.text(143, 120),
        "ship_type": ship_type,
        "ship_type_text": ship_type_text(ship_type),
    })
    out.update(decode_dimensions(bits, 271))
    out.update({
        "epfd": epfd,
        "epfd_text": epfd_text(epfd),
        "raim": bits.flag(305),
        "dte": dte,
        "dte_text": DTE[dte],
        "assigned_mode": assigned,
        "assigned_mode_text": ASSIGNED_MODE[assigned],
        "spare3": bits.uint(308, 4),
    })
    return out
