"""common.py — small things shared by several message decoders."""

from __future__ import annotations


class DecodeError(ValueError):
    """A message could not be decoded (usually: too few bits).

    All four language versions print the same text for this situation so
    that their outputs can be compared:  "message 5 too short: 160 bits < 424"
    """

    def __init__(self, msg_type: int, need_bits: int, have_bits: int):
        super().__init__(f"message {msg_type} too short: {have_bits} bits < {need_bits}")
        self.msg_type = msg_type
        self.need_bits = need_bits
        self.have_bits = have_bits


def require_bits(msg_type: int, bits_len: int, need: int) -> None:
    if bits_len < need:
        raise DecodeError(msg_type, need, bits_len)


# Type of electronic position fixing device (EPFD). ITU-R M.1371-5 Annex 8
# Table 51 (message 4); the same code list is used in messages 5, 9, 11,
# 19, 21 and 24.
EPFD_TYPES = {
    0: "undefined",
    1: "GPS",
    2: "GLONASS",
    3: "combined GPS/GLONASS",
    4: "Loran-C",
    5: "Chayka",
    6: "integrated navigation system",
    7: "surveyed",
    8: "Galileo",
    15: "internal GNSS",
}


def epfd_text(code: int) -> str:
    return EPFD_TYPES.get(code, "not used")


# Type of ship and cargo type. ITU-R M.1371-5 Annex 8 Table 53. The first
# digit is the vessel category, the second digit (for 2x–9x) the hazard
# category (1 = DG/HS/MP category X … 4 = Y, 5–8 reserved, 9 = no additional
# information). We spell out the common ones and derive the rest.
SHIP_TYPE_FIRST_DIGIT = {
    2: "WIG", 3: "", 4: "HSC", 5: "", 6: "passenger", 7: "cargo", 8: "tanker", 9: "other",
}
SHIP_TYPE_SPECIAL = {
    0: "not available (default)",
    30: "fishing", 31: "towing", 32: "towing, length > 200 m or breadth > 25 m",
    33: "dredging or underwater operations", 34: "diving operations",
    35: "military operations", 36: "sailing", 37: "pleasure craft",
    38: "reserved", 39: "reserved",
    50: "pilot vessel", 51: "search and rescue vessel", 52: "tug", 53: "port tender",
    54: "anti-pollution equipment", 55: "law enforcement", 56: "spare - local vessel",
    57: "spare - local vessel", 58: "medical transport", 59: "noncombatant ship (RR Res. 18)",
}
HAZARD_DIGIT = {
    0: "all ships of this type", 1: "hazardous category A", 2: "hazardous category B",
    3: "hazardous category C", 4: "hazardous category D", 5: "reserved", 6: "reserved",
    7: "reserved", 8: "reserved", 9: "no additional information",
}


def ship_type_text(code: int) -> str:
    if code in SHIP_TYPE_SPECIAL:
        return SHIP_TYPE_SPECIAL[code]
    if 1 <= code <= 19:
        return "reserved for future use"
    if 100 <= code <= 199:
        return "reserved for regional use"
    if code >= 200:
        return "reserved for future use"
    tens, ones = divmod(code, 10)
    if tens in SHIP_TYPE_FIRST_DIGIT and tens not in (3, 5):
        return f"{SHIP_TYPE_FIRST_DIGIT[tens]}, {HAZARD_DIGIT[ones]}"
    return "reserved"
