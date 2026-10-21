"""Unit tests for msg_24.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_24 import decode, is_auxiliary_craft


class TestMsg24(unittest.TestCase):
    def test_part_a_and_b(self) -> None:
        # data/ch14-class-b.nmea lines 21-22: AMY PHILBRICK.
        a = Bits.from_payload("H5N6=MP4mV10PTh98T<d0000000", 2)
        out_a = decode(a)
        self.assertEqual(out_a["name"], "AMY PHILBRICK")
        self.assertEqual(out_a["spare_bits"], 0)

        b = Bits.from_payload("H5N6=MTNC9=B140G3Glipq1@D520", 0)
        out_b = decode(b)
        self.assertEqual(out_b["vendor_id"], "SIMRAD")
        self.assertEqual(out_b["manufacturer_id"], "SIM")
        self.assertEqual(out_b["call_sign"], "WCW4189")
        self.assertEqual(out_b["dim_stern_m"], 20)
        self.assertEqual(out_b["length_m"], 30)
        self.assertEqual(out_b["epfd"], 0)
        self.assertEqual(out_b["spare"], 0)

    def test_auxiliary_craft_range_and_too_short(self) -> None:
        self.assertTrue(is_auxiliary_craft(983380001))
        self.assertFalse(is_auxiliary_craft(979000001))
        self.assertFalse(is_auxiliary_craft(990000000))

        short = Bits.from_payload("H5N6=M", 0)  # 36 bits < 40
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 24 too short: 36 bits < 40")


if __name__ == "__main__":
    unittest.main()
