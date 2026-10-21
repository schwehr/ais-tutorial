"""Unit tests for msg_27.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_27 import decode


class TestMsg27(unittest.TestCase):
    def test_synthetic_report(self) -> None:
        # Synthetic sentence from chapter 16: 12 kn / 095 deg at 42.35 N 70.6 W.
        bits = Bits.from_payload("Km2MJh8=JR36Q65t", 0)
        out = decode(bits)
        self.assertEqual(str(out["lon"]), "-70.600000")
        self.assertEqual(str(out["lat"]), "42.350000")
        self.assertEqual(out["sog_knots"], 12)
        self.assertEqual(out["cog_deg"], 95)

    def test_defaults(self) -> None:
        bits = Bits.from_payload("Km2MJh3n`>6bTOwv", 0)
        out = decode(bits)
        self.assertFalse(out["position_available"])
        self.assertFalse(out["sog_available"])
        self.assertFalse(out["cog_available"])

    def test_too_short(self) -> None:
        bits = Bits.from_payload("Kq`jfHko", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(bits)
        self.assertEqual(str(ctx.exception), "message 27 too short: 48 bits < 96")


if __name__ == "__main__":
    unittest.main()
