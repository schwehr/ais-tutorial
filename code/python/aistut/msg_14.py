"""msg_14.py — message 14: safety related broadcast message.

Spec: ITU-R M.1371-5 Annex 8 §3.12, Table 63 (layout), Table 64 (how
many characters fit in 1–5 slots), Table 47 (six-bit ASCII). Annex 1
§2.1.6–2.1.8 prescribe the fixed texts sent by AIS-SART, MOB-AIS and
EPIRB-AIS devices ("SART ACTIVE", "SART TEST", "MOB ACTIVE", "MOB TEST",
"EPIRB ACTIVE", "EPIRB TEST"); §3.12 repeats the list.

Bit layout (zero-based offsets; lengths from Table 63):

     0- 5   6  message id (14)
     6- 7   2  repeat indicator
     8-37  30  source id (MMSI)
    38-39   2  spare
    40-     ≤968  safety related text, six-bit ASCII, 0-161 characters

Length rule: the 40-bit header is required (DecodeError otherwise); the
text handling is shared with message 12 (see msg_12.decode_text_field).
"""

from .bits import Bits
from .common import require_bits
from .msg_12 import decode_text_field

HEADER_BITS = 40

# The six texts that distress beacons are required to send (Annex 1
# §2.1.6-2.1.8). A watch-keeping application matches on these.
BEACON_TEXTS: dict[str, str] = {
    "SART ACTIVE": "AIS-SART active",
    "SART TEST": "AIS-SART test",
    "MOB ACTIVE": "MOB-AIS active",
    "MOB TEST": "MOB-AIS test",
    "EPIRB ACTIVE": "EPIRB-AIS active",
    "EPIRB TEST": "EPIRB-AIS test",
}


def decode(bits: Bits) -> dict:
    """Decode a message 14. Raises DecodeError below 40 bits."""
    require_bits(14, len(bits), HEADER_BITS)
    out: dict = {
        "msg_type": bits.uint(0, 6),
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
    }
    out.update(decode_text_field(bits, HEADER_BITS))
    # `dict.get(key)` returns None (JSON null) when the text is not one of
    # the six prescribed beacon strings.
    out["beacon_text"] = BEACON_TEXTS.get(out["text"])
    return out
