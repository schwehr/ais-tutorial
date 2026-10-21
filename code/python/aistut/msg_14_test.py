"""Unit tests for msg_14.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_14 import decode


class TestMsg14(unittest.TestCase):
    def test_sart_active_and_too_short(self) -> None:
        # Synthetic sentence from data/ch12-safety-text.nmea.
        bits = Bits.from_payload(">>M4eTA<59B04=@UHD0", 2)
        self.assertEqual(len(bits), 112)
        out = decode(bits)
        self.assertEqual(out["mmsi"], 970010001)
        self.assertEqual(out["text"], "SART ACTIVE")
        self.assertEqual(out["beacon_text"], "AIS-SART active")

        short = Bits.from_payload(">>M4eT", 0)  # 36 bits
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 14 too short: 36 bits < 40")


if __name__ == "__main__":
    unittest.main()
