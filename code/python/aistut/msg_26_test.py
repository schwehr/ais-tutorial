"""Unit tests for msg_26.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_26 import decode


class TestMsg26(unittest.TestCase):
    def test_broadcast_sotdma(self) -> None:
        # Synthetic sentence from chapter 16.
        bits = Bits.from_payload("J52MJh400P4SAFN9btogaJFU0<0I", 0)
        out = decode(bits)
        self.assertEqual(out["data_bits"], 92)
        self.assertEqual(out["comm_state_selector"], 0)
        self.assertEqual(out["comm_state"]["received_stations"], 25)

    def test_too_short_for_comm_state(self) -> None:
        # sample-data/RightwhaleAisProject/20081013.log line 411757 (noise)
        bits = Bits.from_payload("JNu2<;>wlUg=NuT", 2)
        with self.assertRaises(DecodeError) as ctx:
            decode(bits)
        self.assertEqual(str(ctx.exception), "message 26 too short: 88 bits < 108")


if __name__ == "__main__":
    unittest.main()
