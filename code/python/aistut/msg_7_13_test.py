"""Unit tests for msg_7_13.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_7_13 import decode


class TestMsg713(unittest.TestCase):
    def test_one_group_and_too_short(self) -> None:
        # 72 bits: one acknowledged station.
        one = Bits.from_payload("=000001H=C0<", 0)
        self.assertEqual(len(one), 72)
        out = decode(one)
        self.assertEqual(out["msg_type"], 13)
        self.assertEqual(len(out["acks"]), 1)
        self.assertEqual(out["trailing_bits"], 0)

        # 71 bits: too short.
        short = Bits.from_payload("=000001H=C0<", 1)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 13 too short: 71 bits < 72")


if __name__ == "__main__":
    unittest.main()
