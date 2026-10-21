"""Unit tests for msg_9.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_9 import decode


class TestMsg9(unittest.TestCase):
    def test_synthetic_sar_aircraft(self) -> None:
        # Chapter 10 §10.3: 111366101 at 457 m, 150 kn, SOTDMA.
        bits = Bits.from_payload("91b=?mAjBFrv4v@HAF98kl@00<2F", 0)
        out = decode(bits)
        self.assertEqual(out["mmsi"], 111366101)
        self.assertEqual(out["altitude_m"], 457)
        self.assertEqual(out["sog_knots"], 150)
        self.assertEqual(out["comm_state"]["scheme"], "SOTDMA")

    def test_too_short(self) -> None:
        bits = Bits.from_payload("9P=t4>pBN4:04Ak8010p", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(bits)
        self.assertEqual(str(ctx.exception), "message 9 too short: 120 bits < 168")


if __name__ == "__main__":
    unittest.main()
