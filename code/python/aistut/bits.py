"""bits.py — turning an AIS payload into bits, and bits into numbers and text.

Everything in an AIS message is a *bit field*: "bits 8–37 are the MMSI",
"bits 61–88 are the longitude in 1/10 000 minute". The job of this module
is therefore very small:

  1. Undo the six-bit ASCII "armouring" that NMEA uses to carry binary data
     in printable characters (see chapter 2).
  2. Give the rest of the code a tidy way to pull out unsigned integers,
     signed integers, booleans and six-bit text starting at any bit offset.

Spec references:
  * Armouring table: IEC 61162-1 (NMEA 0183) §7.3.3 "Six-bit binary field
    conversion"; ITU-R M.1371-5 Annex 8 §3 (as reproduced in AIVDM.adoc
    "AIVDM/AIVDO Payload Armoring").
  * Six-bit ASCII for text fields: ITU-R M.1371-5 Annex 8 Table 47.
  * Two's complement for signed fields: ITU-R M.1371-5 Annex 8 §3.3.7.

Python notes for readers new to the language
--------------------------------------------
* We keep the bits as a *string* of '0' and '1' characters. That is not the
  fastest representation, but it is the clearest: slicing `bits[61:89]`
  gives you exactly the characters you would circle on paper, and
  `int("0111", 2)` converts them to a number.
* `class Bits:` bundles the string with the helper methods that read it.
  `self` is the object itself — Python passes it explicitly.
* Type hints such as `-> int` are documentation; Python does not enforce
  them, but editors and `mypy` use them.
"""

from __future__ import annotations


# --- 1. Armouring ---------------------------------------------------------
#
# Each payload character carries 6 bits. The sender adds 48 to the 6-bit
# value; if the result is above 87 it adds another 8 to skip the characters
# 'X'..'_' (which were avoided because some old equipment mangled them).
# To decode we do the reverse: subtract 48, and if the result is above 40
# subtract 8 more.


def armor_char_to_value(ch: str) -> int:
    """Return the 6-bit value (0–63) carried by one payload character.

    >>> armor_char_to_value('0'), armor_char_to_value('W'), armor_char_to_value('`'), armor_char_to_value('w')
    (0, 39, 40, 63)
    """
    code = ord(ch)
    if code < 48 or code > 119 or 88 <= code <= 95:   # outside '0'..'w', or 'X'..'_'
        raise ValueError(f"illegal armour character {ch!r}")
    value = code - 48
    if value > 40:
        value -= 8
    return value


def payload_to_bits(payload: str, fill_bits: int = 0) -> str:
    """Convert an armoured payload to a string of '0'/'1'.

    `fill_bits` is field 7 of the !AIVDM sentence: how many of the low-order
    bits of the *last* character are padding and must be thrown away.

    >>> payload_to_bits('1', 0)
    '000001'
    >>> payload_to_bits('1w', 4)       # last char 'w' = 111111, drop 4 bits
    '00000111'
    """
    if not 0 <= fill_bits <= 5:
        raise ValueError(f"fill bits must be 0..5, got {fill_bits}")
    # format(v, '06b') writes v in binary, zero-padded to 6 characters.
    bits = "".join(format(armor_char_to_value(ch), "06b") for ch in payload)
    if fill_bits:
        bits = bits[:-fill_bits]
    return bits


# --- 2. Six-bit ASCII used *inside* text fields -----------------------------
#
# Do not confuse this with the armouring above! Text fields (ship name, call
# sign, destination, safety messages) pack one character into 6 bits using a
# different table: 0–31 map to '@'..'_' and 32–63 map to ' '..'?'.

SIXBIT_ASCII = "@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_ !\"#$%&'()*+,-./0123456789:;<=>?"
assert len(SIXBIT_ASCII) == 64


# --- 3. The Bits reader -----------------------------------------------------


class Bits:
    """A bit string with typed accessors.

    All offsets are *zero-based* bit positions, exactly as in the field
    tables of this tutorial (ITU-R M.1371 itself counts bits per field,
    GPSD's AIVDM.adoc and this tutorial count absolute offsets from 0).
    """

    def __init__(self, bits: str):
        if any(c not in "01" for c in bits):
            raise ValueError("Bits expects a string of '0' and '1'")
        self.bits = bits

    @classmethod
    def from_payload(cls, payload: str, fill_bits: int = 0) -> "Bits":
        """Build directly from an !AIVDM payload and its fill-bit count."""
        return cls(payload_to_bits(payload, fill_bits))

    def __len__(self) -> int:          # lets you write len(bits)
        return len(self.bits)

    def has(self, start: int, length: int) -> bool:
        """True if the field [start, start+length) lies inside the message."""
        return start + length <= len(self.bits)

    def uint(self, start: int, length: int) -> int:
        """Unsigned big-endian integer from `length` bits at `start`."""
        if not self.has(start, length):
            raise IndexError(f"field {start}+{length} beyond {len(self.bits)} bits")
        return int(self.bits[start:start + length], 2)

    def sint(self, start: int, length: int) -> int:
        """Signed (two's complement) integer. The first bit is the sign."""
        value = self.uint(start, length)
        if self.bits[start] == "1":          # negative: subtract 2**length
            value -= 1 << length
        return value

    def flag(self, start: int) -> bool:
        """A single bit as a boolean."""
        return self.uint(start, 1) == 1

    def text(self, start: int, length: int) -> str:
        """Six-bit ASCII text occupying `length` bits (a multiple of 6).

        Per ITU-R M.1371-5 Annex 8 §3.3.8, unused characters are '@'; in
        practice names are often space-padded instead. We cut at the first
        '@' and strip trailing spaces.
        """
        chars = []
        for pos in range(start, start + length - 5, 6):
            if not self.has(pos, 6):
                break                        # tolerate short (truncated) fields
            chars.append(SIXBIT_ASCII[self.uint(pos, 6)])
        s = "".join(chars)
        at = s.find("@")
        if at >= 0:
            s = s[:at]
        return s.rstrip()

    def raw(self, start: int, length: int) -> str:
        """The field as a '0'/'1' string — for binary payloads we cannot decode."""
        return self.bits[start:start + length]
