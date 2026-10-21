"""Unit tests for msg_19.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_19 import decode, decode_dimensions


class TestMsg19(unittest.TestCase):
    def test_synthetic_cocheco_and_too_short(self) -> None:
        # data/ch14-class-b.nmea lines 37-38 (synthetic, see chapter 14).
        payload = "C5Pm;j0006g4rwV:ELUFGwr0UNe06N6@:6N000000000L0L21PRP"
        bits = Bits.from_payload(payload, 0)
        self.assertEqual(len(bits), 312)
        out = decode(bits)
        self.assertEqual(out["name"], "R/V COCHECO")
        self.assertEqual(out["dim_bow_m"], 7)
        self.assertEqual(out["length_m"], 11)
        self.assertTrue(out["reference_point_available"])
        self.assertEqual(out["epfd_text"], "GPS")

        short = Bits.from_payload("C5Pm;j0006g4rwV", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 19 too short: 90 bits < 312")

    def test_all_zero_dimensions_have_no_reference_point(self) -> None:
        # Figure 41: A = B = C = D = 0 means neither dimensions nor
        # reference point are available.
        bits = Bits.from_payload("000000", 0)
        out = decode_dimensions(bits, 0)
        self.assertFalse(out["dimensions_available"])
        self.assertFalse(out["reference_point_available"])


if __name__ == "__main__":
    unittest.main()
