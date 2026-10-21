"""Unit tests for msg_21.py."""

import unittest

from aistut.bits import Bits
from aistut.common import DecodeError
from aistut.msg_21 import decode, decode_status


class TestMsg21(unittest.TestCase):
    def test_whale_notices_station(self) -> None:
        # sample-data/2010-11-l3-aton/ccom-sr162g-2010-09.aivdm line 1
        bits = Bits.from_payload(
            "E03Ovrh77PPh;T0V2h77b4QRaP0MM`Lp<Em<000000V@20", 4
        )
        self.assertEqual(len(bits), 272)
        out = decode(bits)
        self.assertEqual(out["mmsi"], 3669739)
        self.assertEqual(out["name"], "NOAA WHALE NOTICES")
        self.assertTrue(out["off_position"])
        self.assertEqual(out["name_extension_bits"], 0)

    def test_status_page_7(self) -> None:
        obj = decode_status(0b1111_0111)
        self.assertEqual(obj["aton_status_page"], 7)
        self.assertEqual(obj["racon_status_text"], "RACON operational")
        self.assertEqual(obj["light_status"], 3)
        self.assertTrue(obj["health_alarm"])
        self.assertNotIn("racon_status", decode_status(1))

    def test_too_short(self) -> None:
        short = Bits.from_payload(
            "E03Ovrh77PPh;T0V2h77b4QRaP0MM`Lp<Em<000000", 0
        )
        with self.assertRaises(DecodeError) as ctx:
            decode(short)
        self.assertEqual(str(ctx.exception), "message 21 too short: 252 bits < 272")


if __name__ == "__main__":
    unittest.main()
