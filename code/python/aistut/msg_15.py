"""msg_15.py — message 15: interrogation.

Spec: ITU-R M.1371-5 Annex 8 §3.13, Table 66 (88-160 bits) and Table 65
(which station class may ask which other class for which messages).
Annex 2 §3.3.2.3 ("polled" mode) and Annex 7 §4.3.3.3.3 describe the response:
automatic, on the channel where the interrogation was received.

Bit layout (zero-based offsets; lengths from Table 66):

     0- 5   6  message id (15)
     6- 7   2  repeat indicator
     8-37  30  source id (MMSI of the interrogating station)
    38-39   2  spare
    40-69  30  destination id 1 (first interrogated station)
    70-75   6  message id 1.1 (first message type requested from station 1)
    76-87  12  slot offset 1.1 (0 = let the responder choose a slot)
    ------- 88 bits: one station, one message -------
    88-89   2  spare
    90-95   6  message id 1.2 (second message type requested from station 1)
    96-107 12  slot offset 1.2
    108-109 2  spare
    ------- 110 bits as written in Table 66; 112 when padded to a byte -------
    110-139 30 destination id 2 (second interrogated station)
    140-145 6  message id 2.1
    146-157 12 slot offset 2.1
    158-159 2  spare
    ------- 160 bits: two stations -------

Table 66 lists four legal shapes:
  1. one station, one message            -> 88 bits
  2. one station, two messages           -> 110 bits (112 on the air, see below)
  3. two stations, one message each      -> 160 bits, message id 1.2 and
                                            slot offset 1.2 "set to zero"
  4. station 1 two messages, station 2 one -> 160 bits
Shape 2 is the famous byte-alignment slip: 110 is not a multiple of 8, so
a transmitter that obeys Annex 2 §3.3.7 ("unused bits in the last byte
should be set to zero in order to preserve byte boundary") sends 112 bits
[AIVDM "Byte Alignment"]. We accept both.

Output: the repeated (station, message, slot offset) triples become a
JSON array `interrogations`, in message order. Rules:
  * at least 88 bits, otherwise DecodeError;
  * the 1.2 group is appended when its 20 bits (88-107) are present AND
    its message id is non-zero — zero is how shape 3 says "no second
    request" (and a request for "message 0" means nothing anyway);
  * the 2.1 group is appended when its 50 bits (108-157) are present;
  * spare fields are output as `spare` (38), `spare2` (88), `spare3` (108)
    and `spare4` (158) when present; whatever is left after the last
    group we understood is counted in `trailing_bits` (2 for the 112-bit
    form).
"""

from .bits import Bits
from .common import require_bits

# Table 46 message names, used for the `_text` companion of a requested
# message type. The names are abbreviated from the "Name" column.
MESSAGE_NAMES: dict[int, str] = {
    0: "none",
    1: "position report",
    2: "position report (assigned)",
    3: "position report (interrogated)",
    4: "base station report",
    5: "static and voyage related data",
    6: "binary addressed message",
    7: "binary acknowledgement",
    8: "binary broadcast message",
    9: "standard SAR aircraft position report",
    10: "UTC/date inquiry",
    11: "UTC/date response",
    12: "addressed safety related message",
    13: "safety related acknowledgement",
    14: "safety related broadcast message",
    15: "interrogation",
    16: "assignment mode command",
    17: "DGNSS broadcast binary message",
    18: "standard Class B equipment position report",
    19: "extended Class B equipment position report",
    20: "data link management message",
    21: "aids-to-navigation report",
    22: "channel management",
    23: "group assignment command",
    24: "static data report",
    25: "single slot binary message",
    26: "multiple slot binary message with communications state",
    27: "position report for long-range applications",
}


def message_name(msg_id: int) -> str:
    return MESSAGE_NAMES.get(msg_id, "undefined")


def request(mmsi: int, msg_id: int, slot_offset: int) -> dict:
    """One (station, message, slot) triple of the `interrogations` array."""
    return {
        "mmsi": mmsi,
        "msg_type": msg_id,
        "msg_type_text": message_name(msg_id),
        "slot_offset": slot_offset,
    }


def decode(bits: Bits) -> dict:
    """Decode a message 15. Raises DecodeError below 88 bits."""
    msg_type = bits.uint(0, 6)
    require_bits(msg_type, len(bits), 88)

    out: dict = {
        "msg_type": msg_type,
        "repeat_indicator": bits.uint(6, 2),
        "mmsi": bits.uint(8, 30),
        "spare": bits.uint(38, 2),
    }
    mmsi1 = bits.uint(40, 30)
    interrogations = [request(mmsi1, bits.uint(70, 6), bits.uint(76, 12))]
    pos = 88

    # Second request to the first station (bits 88-107, plus its spare).
    if bits.has(88, 20):
        out["spare2"] = bits.uint(88, 2)
        msg_id_12 = bits.uint(90, 6)
        if msg_id_12 != 0:
            interrogations.append(request(mmsi1, msg_id_12, bits.uint(96, 12)))
        pos = 108

    # A second station (bits 108-157, plus the trailing spare).
    if bits.has(108, 50):
        out["spare3"] = bits.uint(108, 2)
        interrogations.append(request(bits.uint(110, 30), bits.uint(140, 6), bits.uint(146, 12)))
        pos = 158
        if bits.has(158, 2):
            out["spare4"] = bits.uint(158, 2)
            pos = 160
    elif bits.has(108, 2):
        # The 110/112-bit form: the spare after group 1.2, then 0 or 2
        # padding bits.
        out["spare3"] = bits.uint(108, 2)
        pos = 110

    out["interrogations"] = interrogations
    out["trailing_bits"] = len(bits) - pos
    return out
