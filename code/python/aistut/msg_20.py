"""msg_20.py — message 20: data link management message (chapter 13).

Spec: ITU-R M.1371-5 Annex 8 §3.18, Table 72. A base station announces the
slots it has reserved for itself (FATDMA — fixed access TDMA) so that
mobile stations within 120 NM keep out of them (Annex 2 §3.3.1.2).

Bit layout (zero-based offsets; lengths from Table 72):

     0- 5   6  message id (20)
     6- 7   2  repeat indicator
     8-37  30  source station id (MMSI of the base station)
    38-39   2  spare
    40-69  30  reservation 1:  offset number (12), number of slots (4),
                               time-out in minutes (3), increment (11)
    70-99  30  reservation 2 (optional)
   100-129 30  reservation 3 (optional)
   130-159 30  reservation 4 (optional)
   then 0, 2, 4 or 6 spare bits to reach a byte boundary; 72–160 bits total.

Length rule used here: at least 72 bits (else DecodeError). Reservation
groups are read while 30 bits remain (at most four); whatever is left over
is reported as `spare2` with its width in `spare2_bits`, so a message with
a non-standard length (e.g. 80 bits) is still decoded and the oddity is
visible in the output.
"""

from __future__ import annotations

from .bits import Bits
from .common import require_bits

RESERVATION_BITS = 30
MAX_RESERVATIONS = 4


def decode_reservation(bits: Bits, start: int) -> dict:
    """One offset/slots/time-out/increment group (Table 72)."""
    offset = bits.uint(start, 12)
    slots = bits.uint(start + 12, 4)
    timeout = bits.uint(start + 16, 3)
    increment = bits.uint(start + 19, 11)
    return {
        "offset": offset,
        "slots": slots,
        "timeout_min": timeout,
        "increment": increment,
        # Table 72 footnote: an interrogated base station with nothing to
        # announce sends one group with offset, slots and time-out all zero.
        "available": not (offset == 0 and slots == 0 and timeout == 0),
    }


def decode(bits: Bits) -> dict:
    """Decode a message 20. Raises DecodeError below 72 bits."""
    require_bits(20, len(bits), 72)
    reservations = []
    pos = 40
    while len(reservations) < MAX_RESERVATIONS and bits.has(pos, RESERVATION_BITS):
        reservations.append(decode_reservation(bits, pos))
        pos += RESERVATION_BITS
    spare_bits = len(bits) - pos
    # The spec allows 0, 2, 4 or 6 trailing bits. We print whatever is there
    # as an integer so that odd lengths show up; anything wider than 32 bits
    # is not a spare field but a broken message and is reported as null.
    spare2 = bits.uint(pos, spare_bits) if 0 < spare_bits <= 32 else (0 if spare_bits == 0 else None)
    return {
        "msg_type": bits.uint(0, 6),
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
        "reservations": reservations,
        "spare2": spare2,
        "spare2_bits": spare_bits,
    }
