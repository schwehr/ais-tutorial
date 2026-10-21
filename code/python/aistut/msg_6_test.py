"""Unit tests for msg_6.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_6 import decode


class TestMsg6(unittest.TestCase):
    def test_text_telegram_and_short(self) -> None:
        # data/ch08-binary-framing.nmea line 21: 120 bits, DAC 1 FI 0.
        bits = Bits.from_payload("603Ovl@0>5Q@04000000", 0)
        out = decode(bits)
        self.assertEqual(out["dest_mmsi"], 923156)
        self.assertEqual(
            out["application"]["application_name"], "IFM 0 text telegram"
        )

        short = Bits.from_payload("603Ovl@0>5Q@04", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 6 too short: 84 bits < 88")


if __name__ == "__main__":
    unittest.main()
