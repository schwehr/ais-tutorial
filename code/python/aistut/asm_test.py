"""Unit tests for asm.py."""

import unittest

from aistut import asm
from aistut.bits import Bits
from aistut.jsonout import dumps


class TestAsm(unittest.TestCase):
    def test_whale_notice_record(self) -> None:
        # data/ch08-binary-framing.nmea line 8: one 112-bit record.
        bits = Bits.from_payload("803OvqQK`PP7fb0Fg7GI63do4T;0", 0)
        out = asm.decode(366, 34, bits, 56)
        self.assertIsNotNone(out)
        assert out is not None
        self.assertEqual(out["trailing_bits"], 0)
        self.assertEqual(len(out["notices"]), 1)
        rec = out["notices"][0]
        self.assertEqual(rec["zone_id"], 1)
        self.assertEqual(str(rec["lon"]), "-70.678317")
        self.assertEqual(rec["radius_m"], 9260)

    def test_met_hydro_289_and_236(self) -> None:
        # data/ch09-asm.nmea synthetic Circ.289 (1, 31) and Circ.236 (1, 11).
        b289 = Bits.from_payload(
            "803OMdP0GuvQU1>oNUsWQQ9hfH>DpC6db2<L75wwe7wvlO1A3MwwnPR4ts00", 0
        )
        out289 = asm.decode(1, 31, b289, 56)
        assert out289 is not None
        self.assertEqual(
            out289["application_name"],
            "IMO Circ.289 meteorological and hydrographic data",
        )
        self.assertEqual(out289["pressure_hpa"], 1013)

        b289_na = Bits.from_payload(
            "803OMdP0Gm;Jt2V`406??wvlFR06EuOwgwl?wnSwe7wvlOwwsAwwnSGmwvh0", 0
        )
        out289_na = asm.decode(1, 31, b289_na, 56)
        assert out289_na is not None
        self.assertIsNone(out289_na["lon"])
        self.assertIsNone(out289_na["lat"])
        self.assertIsNone(out289_na["pressure_hpa"])

        b236 = Bits.from_payload(
            "803OMdP0BjMfu=vQU3o?32CQLjjaj:=ED3Q1iOwwwwwwwhD@oOwww8r?>h0", 2
        )
        out236 = asm.decode(1, 11, b236, 56)
        assert out236 is not None
        self.assertEqual(
            out236["application_name"],
            "IMO Circ.236 meteorological and hydrological data",
        )
        self.assertEqual(out236["pressure_hpa"], 1013)

    def test_unknown_and_short(self) -> None:
        bits = Bits.from_payload("803OvqQK`PP7fb0Fg7GI63do4T;0", 0)
        self.assertIsNone(asm.decode(366, 56, bits, 56))
        short_out = asm.decode(1, 31, bits, 56)
        self.assertIsNotNone(short_out)
        self.assertIn("needs 304 data bits, have 112", dumps(short_out))


if __name__ == "__main__":
    unittest.main()
