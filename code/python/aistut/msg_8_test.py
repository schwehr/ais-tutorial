"""Unit tests for msg_8.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_8 import decode


class TestMsg8(unittest.TestCase):
    def test_whale_and_short(self) -> None:
        # data/ch08-binary-framing.nmea line 8.
        bits = Bits.from_payload("803OvqQK`PP7fb0Fg7GI63do4T;0", 0)
        out = decode(bits)
        self.assertEqual(out["dac"], 366)
        self.assertEqual(out["fi"], 34)
        self.assertEqual(out["data_bits"], 112)
        self.assertIn("application", out)

        # data/ch08-binary-framing.nmea line 49: 32 bits.
        short = Bits.from_payload("8>@q3h", 4)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 8 too short: 32 bits < 56")


if __name__ == "__main__":
    unittest.main()
