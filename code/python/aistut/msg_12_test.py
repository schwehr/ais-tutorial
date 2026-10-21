"""Unit tests for msg_12.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_12 import decode


class TestMsg12(unittest.TestCase):
    def test_text_and_padding(self) -> None:
        # data/ch12-safety-text.nmea: "MSG FROM 368905000", 184 bits =
        # 72 header + 18 characters + 4 byte-alignment bits.
        bits = Bits.from_payload("<5Ol;:5GTGd0=C7P6B?=Pknpqhmhhh0", 2)
        self.assertEqual(len(bits), 184)
        out = decode(bits)
        self.assertEqual(out["text"], "MSG FROM 368905000")
        self.assertEqual(out["text_chars"], 18)
        self.assertEqual(out["trailing_bits"], 4)

    def test_too_short(self) -> None:
        bits = Bits.from_payload("<000001H=C0", 0)  # 66 bits
        with self.assertRaises(DecodeError) as ctx:
            decode(bits)
        self.assertEqual(str(ctx.exception), "message 12 too short: 66 bits < 72")


if __name__ == "__main__":
    unittest.main()
