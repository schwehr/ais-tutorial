"""msg_4_11.py — messages 4 and 11: base station report / UTC-date response
(chapter 6).

Spec: ITU-R M.1371-5 Annex 8 §3.2, Table 51 (field layout, 168 bits). The
SOTDMA communication state is Annex 2 §3.3.7.2.2 Table 18/19 and is decoded
by `decode_sotdma` from msg_1_2_3 — a message 4 always carries SOTDMA, never
ITDMA [Table 51, last row].

Message 4 is sent by a base station every 10 s (3 1/3 s when it acts as
the synchronisation "semaphore") [Annex 1 §4.2.1 Table 2 note 1; Annex 2
§3.1.3.3.1]. Message 11 is the *same layout* sent by a mobile station in
answer to a UTC-date inquiry (message 10) [Annex 8 §3.2, §3.9].

Bit layout (zero-based offsets; lengths from Table 51):

     0- 5   6  message id (4 or 11)
     6- 7   2  repeat indicator
     8-37  30  user id (MMSI)
    38-51  14  UTC year,   1-9999; 0 = n/a
    52-55   4  UTC month,  1-12;   0 = n/a; 13-15 not used
    56-60   5  UTC day,    1-31;   0 = n/a
    61-65   5  UTC hour,   0-23;  24 = n/a; 25-31 not used
    66-71   6  UTC minute, 0-59;  60 = n/a; 61-63 not used
    72-77   6  UTC second, 0-59;  60 = n/a; 61-63 not used
    78      1  position accuracy (1 = high, <= 10 m)
    79-106 28  longitude, signed, 1/10 000 minute; 181° (0x6791AC0) = n/a
   107-133 27  latitude,  signed, 1/10 000 minute;  91° (0x3412140) = n/a
   134-137  4  type of electronic position fixing device (EPFD)
   138      1  transmission control for long-range broadcast message
   139-147  9  spare
   148      1  RAIM flag
   149-167 19  SOTDMA communication state

Python notes for readers new to the language
--------------------------------------------
* `from .msg_1_2_3 import decode_sotdma` reuses a function from a sibling
  module of the same package. We import rather than copy because the two
  messages must agree bit for bit on what a communication state means.
* Building the timestamp string uses an f-string with format specs:
  `f"{year:04d}"` pads with zeros to four digits. We only build it when
  every component is available, otherwise the field is `None` (JSON null).
"""


from .bits import Bits
from .common import epfd_text, require_bits
from .jsonout import Fixed
from .msg_1_2_3 import decode_sotdma

# Not-available sentinels for the six UTC fields, Table 51.
YEAR_NA = 0
MONTH_NA = 0
DAY_NA = 0
HOUR_NA = 24
MINUTE_NA = 60
SECOND_NA = 60

TRANSMISSION_CONTROL: dict[int, str] = {
    0: "Class A stops message 27 within base station coverage (default)",
    1: "Class A requested to transmit message 27 within base station coverage",
}


def utc_iso(year: int, month: int, day: int, hour: int, minute: int, second: int) -> str | None:
    """Join the six UTC fields into 'YYYY-MM-DDTHH:MM:SSZ', or None.

    None is returned when *any* component is its not-available value or
    outside its valid range (Table 51: month 13-15, hour 25-31, minute and
    second 61-63 are "not used"). We do not check the calendar (31 April
    passes) — that is left to the reader as an exercise.
    """
    if not 1 <= year <= 9999 or not 1 <= month <= 12 or not 1 <= day <= 31:
        return None
    if hour > 23 or minute > 59 or second > 59:
        return None
    return f"{year:04d}-{month:02d}-{day:02d}T{hour:02d}:{minute:02d}:{second:02d}Z"


def decode(bits: Bits) -> dict:
    """Decode a message 4 or 11. Raises DecodeError if shorter than 168 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 168)
    year = bits.uint(38, 14)
    month = bits.uint(52, 4)
    day = bits.uint(56, 5)
    hour = bits.uint(61, 5)
    minute = bits.uint(66, 6)
    second = bits.uint(72, 6)
    lon_raw = bits.sint(79, 28)
    lat_raw = bits.sint(107, 27)
    epfd = bits.uint(134, 4)
    tx_control = bits.uint(138, 1)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "utc_year": year,
        "utc_year_available": year != YEAR_NA,
        "utc_month": month,
        "utc_month_available": month != MONTH_NA,
        "utc_day": day,
        "utc_day_available": day != DAY_NA,
        "utc_hour": hour,
        "utc_hour_available": hour != HOUR_NA,
        "utc_minute": minute,
        "utc_minute_available": minute != MINUTE_NA,
        "utc_second": second,
        "utc_second_available": second != SECOND_NA,
        "utc_iso": utc_iso(year, month, day, hour, minute, second),
        "position_accuracy": bits.uint(78, 1),
        "lon": Fixed(lon_raw / 600000.0, 6),
        "lat": Fixed(lat_raw / 600000.0, 6),
        "position_available": lon_raw != 0x6791AC0 and lat_raw != 0x3412140,
        "epfd": epfd,
        "epfd_text": epfd_text(epfd),
        "transmission_control": tx_control,
        "transmission_control_text": TRANSMISSION_CONTROL[tx_control],
        "spare": bits.uint(139, 9),
        "raim": bits.flag(148),
        "comm_state": decode_sotdma(bits, 149),
    }
    return out
