"""Unit tests for msg_17.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_17 import decode


class TestMsg17(unittest.TestCase):
    def test_synthetic_two_words(self) -> None:
        # Synthetic sentence from chapter 13: reference station at 36 58.5 N 076 20.0 W.
        bits = Bits.from_payload("A03OviSCAPcE80AT9aD@bbbbEEEE", 0)
        out = decode(bits)
        self.assertEqual(str(out["lon"]), "-76.333333")
        self.assertEqual(str(out["lat"]), "36.975000")
        self.assertEqual(out["data_bits"], 88)
        rtcm = out["rtcm"]
        self.assertIsNotNone(rtcm)
        self.assertEqual(rtcm["message_type"], 1)
        self.assertEqual(rtcm["station_id"], 100)
        self.assertEqual(rtcm["z_count"], 1234)
        self.assertEqual(rtcm["n_words"], 2)
        self.assertTrue(rtcm["words_complete"])

    def test_too_short(self) -> None:
        bits = Bits.from_payload("A34PBT6JT4i8", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(bits)
        self.assertEqual(str(ctx.exception), "message 17 too short: 72 bits < 80")


if __name__ == "__main__":
    unittest.main()
