"""Unit tests for decode.py."""

import io
import unittest
from unittest.mock import patch

from aistut.decode import DECODERS, main, process_line
from aistut.jsonout import dumps
from aistut.vdm import Assembler


def _run(line: str) -> str:
    asm = Assembler()
    res = process_line(line, 1, asm)
    return "" if res is None else dumps(res)


class TestDecode(unittest.TestCase):
    def test_decoders_table_complete(self) -> None:
        self.assertEqual(set(DECODERS.keys()), set(range(1, 28)))

    def test_shapes(self) -> None:
        self.assertEqual(_run("# comment"), "")
        self.assertEqual(_run("   "), "")
        self.assertEqual(
            _run("$PMTK010,001*2E"),
            r'{"address":"PMTK010","line":1,"skipped":"no parser"}',
        )
        self.assertEqual(
            _run("hello"),
            '{"error":"no \'$\' or \'!\' in line","line":1,"raw":"hello"}',
        )
        self.assertEqual(
            _run("$GPGSV,1,1,00*78"),
            r'{"error":"checksum mismatch","line":1,"raw":"$GPGSV,1,1,00*78"}',
        )
        self.assertEqual(
            _run("$GPGSV,1,1,00"),
            r'{"error":"no checksum","line":1,"raw":"$GPGSV,1,1,00"}',
        )

        # A one-character payload reaches the message 5 decoder and fails its
        # length check; the error object keeps the AIS envelope fields.
        short = _run("!AIVDM,1,1,,A,5,0*13")
        self.assertIn(r'"error":"message 5 too short: 6 bits < 424"', short)
        self.assertIn(r'"msg_type":5', short)
        self.assertIn(r'"payload_bits":6', short)

    def test_payload_too_short_and_unknown_type(self) -> None:
        # 1 char with 2 fill bits -> 4 bits (< 6 bits).
        too_short = _run("!AIVDM,1,1,,A,0,2*14")
        self.assertIn(r'"error":"payload too short: 4 bits"', too_short)

        # Message type 0 ('0' = 000000).
        unknown = _run("!AIVDM,1,1,,A,0,0*16")
        self.assertIn(r'"error":"unknown message type 0"', unknown)
        self.assertIn(r'"msg_type":0', unknown)

    def test_malformed_gps_numeric_field_captured(self) -> None:
        bad_gga = _run("$GPGGA,123519,4312.2805,N,07043.2065,W,x,08,0.9,10.0,M,-33.0,M,,*03")
        self.assertIn('"error":"invalid literal for int() with base 10: \'x\'"', bad_gga)

    def test_main_no_args_and_stdin(self) -> None:
        with patch("sys.stdout", new_callable=io.StringIO):
            self.assertEqual(main([]), 2)

        fake_in = io.StringIO("# header\n$PMTK010,001*2E\n")
        fake_out = io.StringIO()
        with patch("sys.stdin", fake_in), patch("sys.stdout", fake_out):
            self.assertEqual(main(["-"]), 0)
        self.assertEqual(
            fake_out.getvalue().strip(),
            r'{"address":"PMTK010","line":2,"skipped":"no parser"}',
        )


if __name__ == "__main__":
    unittest.main()
