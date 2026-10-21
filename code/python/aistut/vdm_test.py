"""Unit tests for vdm.py."""

import unittest

from aistut.nmea import parse_sentence
from aistut.vdm import Assembler, Vdm, parse_vdm


def _vdm(line: str) -> Vdm:
    return parse_vdm(parse_sentence(line))


class TestVdm(unittest.TestCase):
    def test_single_fragment_and_vdo(self) -> None:
        asm = Assembler()
        frag = _vdm("!AIVDO,1,1,,A,15N8?>PP01JRVP6E7aLLkOvt0HAO,0*62")
        self.assertTrue(frag.own_ship)
        msg = asm.add(frag)
        self.assertIsNotNone(msg)
        assert msg is not None
        self.assertTrue(msg.own_ship)
        self.assertEqual(msg.first, frag)
        self.assertEqual(msg.last, frag)

    def test_two_fragment_message(self) -> None:
        asm = Assembler()
        a = _vdm(
            "!AIVDM,2,1,3,B,55P5TL01VIaAL@7WKO@mBplU@<PDhh000000001S;AJ::4A80?4i@E53,0*3E"
        )
        b = _vdm("!AIVDM,2,2,3,B,1@0000000000000,2*55")
        self.assertIsNone(asm.add(a))
        msg = asm.add(b)
        self.assertIsNotNone(msg)
        assert msg is not None
        self.assertEqual(len(msg.fragments), 2)
        self.assertEqual(msg.fill_bits, 2)
        self.assertTrue(msg.payload.startswith("55P5TL01"))
        self.assertTrue(msg.payload.endswith("1@0000000000000"))

    def test_out_of_order_fragment_is_dropped(self) -> None:
        asm = Assembler()
        b = _vdm("!AIVDM,2,2,3,B,1@0000000000000,2*55")
        self.assertIsNone(asm.add(b))

    def test_errors(self) -> None:
        s_gga = parse_sentence("$GPGGA,1*00")
        with self.assertRaises(ValueError) as ctx:
            parse_vdm(s_gga)
        self.assertEqual(str(ctx.exception), "not a VDM/VDO sentence: GPGGA")

        s_short = parse_sentence("!AIVDM,1,1,,B*2A")
        with self.assertRaises(ValueError) as ctx:
            parse_vdm(s_short)
        self.assertEqual(str(ctx.exception), "VDM needs 6 fields, got 4")

        s_bad_int = parse_sentence("!AIVDM,x,1,,B,1,0*00")
        with self.assertRaises(ValueError) as ctx:
            parse_vdm(s_bad_int)
        self.assertEqual(
            str(ctx.exception), "invalid literal for int() with base 10: 'x'"
        )


if __name__ == "__main__":
    unittest.main()
