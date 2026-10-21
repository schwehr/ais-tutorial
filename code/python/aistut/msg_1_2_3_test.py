"""Unit tests for msg_1_2_3.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_1_2_3 import decode, decode_itdma, decode_rot, decode_sotdma


class TestMsg123(unittest.TestCase):
    def test_rot_special_values(self) -> None:
        self.assertEqual(decode_rot(-128)["rot_text"], "not available")
        self.assertIsNone(decode_rot(-128)["rot_deg_per_min"])
        self.assertEqual(
            decode_rot(127)["rot_text"],
            "turning right > 5 deg per 30 s (no turn indicator)",
        )
        self.assertEqual(
            decode_rot(-127)["rot_text"],
            "turning left > 5 deg per 30 s (no turn indicator)",
        )
        self.assertEqual(str(decode_rot(0)["rot_deg_per_min"]), "0.0")
        self.assertEqual(decode_rot(0)["rot_text"], "not turning")
        self.assertEqual(str(decode_rot(-10)["rot_deg_per_min"]), "-4.5")
        self.assertEqual(decode_rot(-10)["rot_text"], "turning left")
        self.assertEqual(decode_rot(10)["rot_text"], "turning right")

    def test_comm_state_helpers(self) -> None:
        # SOTDMA timeout=1 -> utc_hour and utc_minute (7-bit mask).
        b_utc = Bits("00" + "001" + format((12 << 9) | (34 << 2), "014b"))
        cs_utc = decode_sotdma(b_utc, 0)
        self.assertEqual(cs_utc["utc_hour"], 12)
        self.assertEqual(cs_utc["utc_minute"], 34)

        # ITDMA with nslots > 4 adds 8192 to slot_offset.
        b_itdma = Bits("00" + format(100, "013b") + "101" + "1")
        cs_itdma = decode_itdma(b_itdma, 0)
        self.assertEqual(cs_itdma["slots"], 1)
        self.assertEqual(cs_itdma["slot_offset"], 8292)
        self.assertTrue(cs_itdma["keep_flag"])

    def test_decode_and_too_short(self) -> None:
        bits = Bits.from_payload("15N8?>PP01JRVP6E7aLLkOvt0HAO", 0)
        self.assertEqual(len(bits), 168)
        out = decode(bits)
        self.assertEqual(out["msg_type"], 1)
        self.assertEqual(out["mmsi"], 367136570)
        self.assertEqual(out["comm_state"]["scheme"], "SOTDMA")

        short = Bits.from_payload("15N8?>PP01JRVP6E7aLLkOvt", 0)
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 1 too short: 144 bits < 168")


if __name__ == "__main__":
    unittest.main()
