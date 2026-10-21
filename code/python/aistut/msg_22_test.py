"""Unit tests for msg_22.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_22 import channel_frequency_khz, decode


class TestMsg22(unittest.TestCase):
    def test_frequencies(self) -> None:
        self.assertEqual(channel_frequency_khz(2087), 161_975)
        self.assertEqual(channel_frequency_khz(2088), 162_025)
        self.assertEqual(channel_frequency_khz(1075), 156_775)
        self.assertEqual(channel_frequency_khz(87), 157_375)
        self.assertEqual(channel_frequency_khz(16), 156_800)
        self.assertEqual(channel_frequency_khz(2016), 156_800)
        self.assertIsNone(channel_frequency_khz(0))
        self.assertIsNone(channel_frequency_khz(4087))

    def test_broadcast_defaults(self) -> None:
        # sample-data/2007hydro/norfolk-log.ais line 115 (own-ship VDO)
        bits = Bits.from_payload("F52HH<j2N2P3D73EB6`>6bT20000", 0)
        out = decode(bits)
        self.assertEqual(out["channel_a"], 2087)
        self.assertEqual(out["channel_b"], 2088)
        self.assertFalse(out["addressed"])
        self.assertFalse(out["region_available"])
        self.assertEqual(out["zone_size_nm"], 5)

    def test_addressed_and_too_short(self) -> None:
        # Synthetic sentence from chapter 13.
        bits = Bits.from_payload("F03OviQvevh2fwrs85MwmnPB0000", 0)
        out = decode(bits)
        self.assertTrue(out["addressed"])
        self.assertEqual(out["dest_mmsi_1"], 366999001)
        self.assertEqual(out["dest_mmsi_2"], 366999002)
        self.assertNotIn("ne_lon", out)

        short = Bits.from_payload("F03OviQvevh2", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 22 too short: 72 bits < 168")


if __name__ == "__main__":
    unittest.main()
