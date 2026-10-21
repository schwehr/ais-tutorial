"""Unit tests for msg_25.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_25 import decode


class TestMsg25(unittest.TestCase):
    def test_broadcast_unstructured(self) -> None:
        # sample-data/tide3/ais/ais-2011-05-26 line 1648
        bits = Bits.from_payload("I?wwwwhHS>0fgGQ0UImCTdkfB=>R", 0)
        out = decode(bits)
        self.assertEqual(out["mmsi"], 1073741823)
        self.assertFalse(out["addressed"])
        self.assertEqual(out["data_bits"], 128)

    def test_addressed_structured(self) -> None:
        # Synthetic sentence from chapter 16.
        bits = Bits.from_payload("I52MJh=GBVL0004B=5H", 2)
        out = decode(bits)
        self.assertEqual(out["dest_mmsi"], 366123456)
        self.assertEqual(out["dac"], 0)
        self.assertEqual(out["fi"], 1)
        self.assertEqual(out["data_offset"], 88)
        self.assertEqual(out["data_bits"], 24)

    def test_too_short(self) -> None:
        bits = Bits.from_payload("ILPo0h", 4)
        with self.assertRaises(DecodeError) as ctx:
            decode(bits)
        self.assertEqual(str(ctx.exception), "message 25 too short: 32 bits < 40")


if __name__ == "__main__":
    unittest.main()
