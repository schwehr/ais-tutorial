"""Unit tests for msg_23.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_23 import decode


class TestMsg23(unittest.TestCase):
    def test_synthetic_quiet_time(self) -> None:
        # Synthetic sentence from chapter 13: pleasure craft, TxA only, quiet 3 min.
        bits = Bits.from_payload("G03OviSCN0cG6VFQEt09@000@<0", 2)
        out = decode(bits)
        self.assertEqual(str(out["ne_lat"]), "37.000000")
        self.assertEqual(str(out["sw_lon"]), "-76.500000")
        self.assertEqual(out["ship_type"], 37)
        self.assertEqual(out["ship_type_text"], "pleasure craft")
        self.assertEqual(out["txrx_mode"], 1)
        self.assertEqual(out["quiet_time_min"], 3)

    def test_too_short(self) -> None:
        # sample-data/RightwhaleAisProject/20081013.log line 393089
        bits = Bits.from_payload("Gob<Mwf<", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(bits)
        self.assertEqual(str(ctx.exception), "message 23 too short: 48 bits < 160")


if __name__ == "__main__":
    unittest.main()
