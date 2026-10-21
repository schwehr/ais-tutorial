"""Unit tests for msg_15.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_15 import decode


class TestMsg15(unittest.TestCase):
    def test_shapes(self) -> None:
        # 88 bits, recorded: 338040883 asks 366831230 for message 5.
        one = Bits.from_payload("?52HH<iGMIWpD00", 2)
        self.assertEqual(len(one), 88)
        out1 = decode(one)
        self.assertEqual(out1["trailing_bits"], 0)
        self.assertEqual(len(out1["interrogations"]), 1)
        self.assertEqual(out1["interrogations"][0]["msg_type"], 5)
        self.assertEqual(
            out1["interrogations"][0]["msg_type_text"],
            "static and voyage related data",
        )

        # 112 bits (byte-aligned 110): two requests, two padding bits.
        two = Bits.from_payload("?03OviQGMIWp<005000", 2)
        self.assertEqual(len(two), 112)
        out2 = decode(two)
        self.assertEqual(out2["trailing_bits"], 2)
        self.assertEqual(len(out2["interrogations"]), 2)

        # 87 bits: too short.
        short = Bits.from_payload("?52HH<iGMIWpD00", 3)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 15 too short: 87 bits < 88")


if __name__ == "__main__":
    unittest.main()
