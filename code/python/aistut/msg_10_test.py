"""Unit tests for msg_10.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_10 import decode


class TestMsg10(unittest.TestCase):
    def test_recorded_inquiry_and_too_short(self) -> None:
        # tide3/ais/ais-2010-03-09 line 131: 354741000 asks 367315470.
        bits = Bits.from_payload(":5BCc21GTjPp", 0)
        self.assertEqual(len(bits), 72)
        out = decode(bits)
        self.assertEqual(out["mmsi"], 354741000)
        self.assertEqual(out["dest_mmsi"], 367315470)

        short = Bits.from_payload(":5BCc21GTjP", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 10 too short: 66 bits < 72")


if __name__ == "__main__":
    unittest.main()
