"""Unit tests for msg_5.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_5 import decode, eta_text, imo_check_digit_ok, imo_range_text


class TestMsg5(unittest.TestCase):
    def test_imo_check_digit(self) -> None:
        self.assertTrue(imo_check_digit_ok(9074729))
        self.assertTrue(imo_check_digit_ok(9271420))  # USNS SACAGAWEA
        self.assertFalse(imo_check_digit_ok(9271421))
        self.assertFalse(imo_check_digit_ok(0))
        self.assertFalse(imo_check_digit_ok(926974000))
        self.assertEqual(imo_range_text(0), "not available (default)")
        self.assertEqual(imo_range_text(500), "not used")
        self.assertEqual(imo_range_text(9074729), "IMO number")
        self.assertEqual(imo_range_text(926974000), "official flag state number")

    def test_eta_defaults(self) -> None:
        self.assertEqual(eta_text(0, 0, 24, 60), "??-?? ??:??")
        self.assertEqual(eta_text(1, 1, 0, 26), "01-01 00:26")

    def test_capecod_test_station_and_too_short(self) -> None:
        bits = Bits.from_payload(
            "503OvqP00000ptpD000pt461@E=@00000000000W000004@PJ03Sm80H5QDliC000000000",
            2,
        )
        self.assertEqual(len(bits), 424)
        out = decode(bits)
        self.assertEqual(out["name"], "NOAA TEST")
        self.assertEqual(out["destination"], "NOT A VESSEL")
        self.assertEqual(out["mmsi"], 3669734)

        short = Bits.from_payload("503OvqP00000", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 5 too short: 72 bits < 424")


if __name__ == "__main__":
    unittest.main()
