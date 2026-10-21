"""Unit tests for msg_4_11.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_4_11 import decode, utc_iso


class TestMsg411(unittest.TestCase):
    def test_utc_iso_sentinels(self) -> None:
        self.assertEqual(utc_iso(2008, 3, 27, 22, 9, 54), "2008-03-27T22:09:54Z")
        self.assertIsNone(utc_iso(0, 3, 27, 22, 9, 54))     # year n/a
        self.assertIsNone(utc_iso(2008, 3, 27, 24, 9, 54))  # hour n/a
        self.assertIsNone(utc_iso(2008, 3, 27, 22, 60, 54)) # minute n/a
        self.assertIsNone(utc_iso(2008, 13, 27, 22, 9, 54)) # month "not used"

    def test_surveyed_base_station(self) -> None:
        # data/ch06-base-station-reports.nmea line 3 (Norfolk, VA, 2007-05-17).
        bits = Bits.from_payload("403OviQuMHQG9rRO9>E6fE700@5r", 0)
        out = decode(bits)
        self.assertEqual(out["mmsi"], 3669702)
        self.assertEqual(out["utc_iso"], "2007-05-17T01:23:09Z")
        self.assertEqual(out["epfd_text"], "surveyed")
        self.assertEqual(str(out["lon"]), "-76.352362")
        self.assertEqual(str(out["lat"]), "36.883767")

    def test_no_fix_defaults(self) -> None:
        # data/ch06-base-station-reports.nmea line 16: L-3 base station on the
        # bench without a GNSS antenna.
        bits = Bits.from_payload("4h0C5GQuPun9n<tSF0l4Q@001`;f", 0)
        out = decode(bits)
        self.assertFalse(out["position_available"])
        self.assertEqual(out["epfd"], 0)
        self.assertEqual(out["repeat_indicator"], 3)

    def test_too_short(self) -> None:
        short = Bits.from_payload(";P@SGS1`i30t", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 11 too short: 72 bits < 168")


if __name__ == "__main__":
    unittest.main()
