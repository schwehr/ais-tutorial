"""Unit tests for msg_20.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_20 import decode


class TestMsg20(unittest.TestCase):
    def test_one_block(self) -> None:
        # sample-data/2007hydro/norfolk-log.ais line 22
        bits = Bits.from_payload("D03OviQsIN9H", 0)
        out = decode(bits)
        self.assertEqual(out["mmsi"], 3669702)
        res = out["reservations"]
        self.assertEqual(len(res), 1)
        self.assertEqual(res[0]["offset"], 1974)
        self.assertEqual(res[0]["slots"], 5)
        self.assertEqual(res[0]["timeout_min"], 7)
        self.assertEqual(res[0]["increment"], 150)
        self.assertEqual(out["spare2_bits"], 2)

    def test_two_blocks_and_too_short(self) -> None:
        # Synthetic sentence from chapter 13: two reservation blocks, 104 bits.
        bits = Bits.from_payload("D03OviP1QN9I7lW6D0", 4)
        out = decode(bits)
        self.assertEqual(len(out["reservations"]), 2)
        self.assertEqual(out["spare2_bits"], 4)

        short = Bits.from_payload("D03OviQsIN9", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 20 too short: 66 bits < 72")


if __name__ == "__main__":
    unittest.main()
