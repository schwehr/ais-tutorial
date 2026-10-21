"""Unit tests for msg_16.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_16 import decode


class TestMsg16(unittest.TestCase):
    def test_rate_and_slot_and_two_assignments(self) -> None:
        # Chapter 11 §11.3 synthetic: 60 reports per 10 min.
        rate = Bits.from_payload("@03OviQGMIWp3h00", 0)
        out_rate = decode(rate)
        self.assertEqual(out_rate["assignments"][0]["reports_per_10_min"], 60)
        self.assertIsNone(out_rate["assignments"][0]["slot_increment"])

        # Slot assignment, increment code 2 = 375 slots.
        slot = Bits.from_payload("@03OviQGMIWp1@0P", 0)
        out_slot = decode(slot)
        self.assertEqual(out_slot["assignments"][0]["slot_increment"], 375)

        # 144 bits: two assignments.
        two = Bits.from_payload("@03OviQGMIWp1@0UN6kFPN00", 0)
        self.assertEqual(len(two), 144)
        out_two = decode(two)
        self.assertEqual(len(out_two["assignments"]), 2)

        short = Bits.from_payload("@03OviQGMIWp1@0", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 16 too short: 90 bits < 96")


if __name__ == "__main__":
    unittest.main()
