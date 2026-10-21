"""msg_17.py — message 17: GNSS broadcast binary message (chapter 13).

Spec: ITU-R M.1371-5 Annex 8 §3.15, Table 68 (80-bit header plus 0–736
bits of differential-correction data) and Table 69 (layout of the data:
a 40-bit RTCM SC-104 style header followed by N 24-bit data words, parity
removed). The content follows Recommendation ITU-R M.823.

Bit layout (zero-based offsets; lengths from Table 68):

     0- 5   6  message id (17)
     6- 7   2  repeat indicator
     8-37  30  source id (MMSI of the base station)
    38-39   2  spare
    40-57  18  longitude of the DGNSS reference station, signed, 1/10 min; 181° = n/a
    58-74  17  latitude of the DGNSS reference station, signed, 1/10 min; 91° = n/a
    75-79   5  spare
    80-     0–736 differential correction data (Table 69):
      80- 85   6  RTCM message type
      86- 95  10  RTCM station id
      96-108  13  Z count, units of 0.6 s (0–3599.4 s)
     109-111   3  sequence number (cyclic 0–7)
     112-116   5  N, number of 24-bit data words following this header (≤ 29)
     117-119   3  reference station health
     120-     N × 24 bits of DGNSS data words

Length rule used here: at least 80 bits (else DecodeError). If 40 or more
data bits follow the header we decode the RTCM header and hand back the
remaining bits as a string; `words_complete` says whether exactly N × 24
bits were present. Fewer than 40 data bits (but more than zero) is reported
as `rtcm: null` — nothing meaningful can be read from a partial header.
"""

from .bits import Bits
from .common import require_bits
from .jsonout import Fixed

LON_NOT_AVAILABLE = 181 * 600   # 0x1A838 — 1/10-minute units
LAT_NOT_AVAILABLE = 91 * 600    # 0x0D548

# Reference-station health, Table 69 → ITU-R M.823 (the RTCM SC-104 v2 code).
HEALTH: dict[int, str] = {
    0: "UDRE scale factor 1.0",
    1: "UDRE scale factor 0.75",
    2: "UDRE scale factor 0.5",
    3: "UDRE scale factor 0.3",
    4: "UDRE scale factor 0.2",
    5: "UDRE scale factor 0.1",
    6: "reference station transmission not monitored",
    7: "reference station not working",
}


def decode_rtcm_header(bits: Bits, start: int) -> dict:
    """The 40-bit RTCM header of Table 69 plus whatever data words follow."""
    n_words = bits.uint(start + 32, 5)
    health = bits.uint(start + 37, 3)
    z_count = bits.uint(start + 16, 13)
    words_start = start + 40
    words_bits = len(bits) - words_start
    return {
        "message_type": bits.uint(start, 6),
        "station_id": bits.uint(start + 6, 10),
        "z_count": z_count,
        "z_count_s": Fixed(z_count * 0.6, 1),
        "sequence": bits.uint(start + 22, 3),
        "n_words": n_words,
        "health": health,
        "health_text": HEALTH[health],
        "words": bits.raw(words_start, words_bits),
        "words_bits": words_bits,
        "words_complete": words_bits == n_words * 24,
    }


def decode(bits: Bits) -> dict:
    """Decode a message 17. Raises DecodeError below 80 bits."""
    require_bits(17, len(bits), 80)
    lon_raw = bits.sint(40, 18)
    lat_raw = bits.sint(58, 17)
    data_bits = len(bits) - 80
    out: dict = {
        "msg_type": bits.uint(0, 6),
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
        "lon": Fixed(lon_raw / 600.0, 6),
        "lat": Fixed(lat_raw / 600.0, 6),
        "position_available": lon_raw != LON_NOT_AVAILABLE and lat_raw != LAT_NOT_AVAILABLE,
        "spare2": bits.uint(75, 5),
        "data": bits.raw(80, data_bits),
        "data_bits": data_bits,
    }
    # Table 69: the two-word (40-bit) header is only meaningful when complete.
    out["rtcm"] = decode_rtcm_header(bits, 80) if data_bits >= 40 else None
    return out
