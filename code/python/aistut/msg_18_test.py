"""Unit tests for msg_18.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_18 import decode


class TestMsg18(unittest.TestCase):
    def test_cs_unit_at_sea(self) -> None:
        # data/ch14-class-b.nmea line 20: AMY PHILBRICK, 11.2 kn.
        bits = Bits.from_payload("B5N6=MP0L6g3idV:FdDjSws5kP06", 0)
        out = decode(bits)
        self.assertEqual(out["mmsi"], 367103350)
        self.assertEqual(str(out["sog_knots"]), "11.2")
        self.assertEqual(out["class_b_unit"], 1)
        self.assertTrue(out["cs_fixed_comm_state"])

    def test_too_short(self) -> None:
        short = Bits.from_payload("B@80P@`420100P30R27095hq@h", 4)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 18 too short: 152 bits < 168")


if __name__ == "__main__":
    unittest.main()
