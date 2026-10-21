"""Unit tests for nmea.py."""

import unittest

from aistut.jsonout import dumps
from aistut.nmea import (
    NmeaError,
    checksum,
    parse_date,
    parse_gga,
    parse_gsa,
    parse_gsv,
    parse_latitude,
    parse_longitude,
    parse_rmc,
    parse_sentence,
    parse_utc_time,
)


class TestNmea(unittest.TestCase):
    def test_checksum_matches_spec_example(self) -> None:
        self.assertEqual(checksum("GPGSV,1,1,00"), 0x79)
        s = parse_sentence("$GPGSV,1,1,00*79\r\n")
        self.assertTrue(s.checksum_ok)
        self.assertEqual(s.checksum_given, 0x79)
        self.assertEqual(s.address, "GPGSV")
        self.assertEqual(s.fields, ["1", "1", "00"])

    def test_uscg_extras_and_prefix(self) -> None:
        s = parse_sentence("foo.log:!AIVDM,1,1,,A,1,0*5C,r003,1223901985")
        self.assertEqual(s.start, "!")
        self.assertEqual(s.talker, "AI")
        self.assertEqual(s.formatter, "VDM")
        self.assertEqual(s.after_checksum, ["r003", "1223901985"])
        self.assertEqual(s.raw, "!AIVDM,1,1,,A,1,0*5C,r003,1223901985")

        p = parse_sentence("$PMTK010,001*2E")
        self.assertEqual(p.talker, "P")
        self.assertEqual(p.formatter, "MTK010")

    def test_errors(self) -> None:
        with self.assertRaises(NmeaError) as ctx:
            parse_sentence("hello")
        self.assertEqual(str(ctx.exception), "no '$' or '!' in line")

        with self.assertRaises(NmeaError) as ctx:
            parse_sentence("$GP,1*00")
        self.assertEqual(str(ctx.exception), "address field too short: 'GP'")

        s = parse_sentence("$GPGGA,1")
        self.assertFalse(s.checksum_ok)
        self.assertIsNone(s.checksum_given)

        bad_hex = parse_sentence("$GPGGA,1*ZZ")
        self.assertFalse(bad_hex.checksum_ok)
        self.assertIsNone(bad_hex.checksum_given)

    def test_angles_and_times(self) -> None:
        self.assertAlmostEqual(parse_latitude("4312.2805", "N") or 0.0, 43.204675)
        self.assertAlmostEqual(parse_latitude("4312.2805", "S") or 0.0, -43.204675)
        self.assertAlmostEqual(parse_longitude("07052.8474", "W") or 0.0, -70.88079)
        self.assertAlmostEqual(parse_longitude("07052.8474", "E") or 0.0, 70.88079)
        self.assertIsNone(parse_latitude("", "N"))
        self.assertIsNone(parse_longitude("", "W"))
        self.assertEqual(parse_utc_time("000429.000"), "00:04:29.000")
        self.assertIsNone(parse_utc_time("12345"))
        self.assertEqual(parse_date("081207"), "2007-12-08")
        self.assertIsNone(parse_date(""))

    def test_gps_parsers(self) -> None:
        gga = parse_gga(
            parse_sentence(
                "$GPGGA,000429.000,3601.5522,N,10947.9359,W,1,7,4.21,12491.0,M,-20.5,M,,*6D"
            )
        )
        self.assertEqual(gga["sentence"], "GGA")
        self.assertEqual(gga["fix_quality"], 1)
        self.assertEqual(gga["fix_quality_text"], "GPS fix (SPS)")
        self.assertEqual(gga["satellites_used"], 7)
        self.assertEqual(dumps(gga["lat"]), "36.025870")
        self.assertEqual(dumps(gga["lon"]), "-109.798932")

        gsa = parse_gsa(
            parse_sentence("$GPGSA,A,3,20,17,11,04,23,32,13,,,,,,6.53,4.21,4.99*04")
        )
        self.assertEqual(gsa["sentence"], "GSA")
        self.assertEqual(gsa["fix_mode"], 3)
        self.assertEqual(gsa["fix_mode_text"], "3D")
        self.assertEqual(gsa["satellites_used"], [20, 17, 11, 4, 23, 32, 13])

        gsv = parse_gsv(
            parse_sentence(
                "$GPGSV,3,1,12,20,78,031,42,23,59,308,42,17,56,143,39,11,46,058,41*7C"
            )
        )
        self.assertEqual(gsv["sentence"], "GSV")
        self.assertEqual(gsv["satellites_in_view"], 12)
        self.assertEqual(len(gsv["satellites"]), 4)
        self.assertEqual(gsv["satellites"][0]["prn"], 20)

        rmc = parse_rmc(
            parse_sentence(
                "$GPRMC,000429.000,A,3601.5522,N,10947.9359,W,466.09,278.57,081207,11.5,W,A*19"
            )
        )
        self.assertEqual(rmc["sentence"], "RMC")
        self.assertTrue(rmc["valid"])
        self.assertEqual(rmc["date"], "2007-12-08")
        self.assertEqual(dumps(rmc["magnetic_variation_deg"]), "-11.5")
        self.assertEqual(rmc["mode_text"], "autonomous")


if __name__ == "__main__":
    unittest.main()
