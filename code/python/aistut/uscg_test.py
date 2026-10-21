"""Unit tests for uscg.py."""

import unittest

from aistut.jsonout import dumps
from aistut.uscg import parse_uscg_fields, time_of_arrival_to_utc


class TestUscg(unittest.TestCase):
    def test_typical_line(self) -> None:
        out = parse_uscg_fields(
            ["s34486", "d-074", "T25.49701619", "x78124", "r003669946", "1223901985"]
        )
        self.assertEqual(
            dumps(out),
            r'{"rssi":34486,"sentence_counter":78124,"signal_dbm":-74,"station":"003669946","station_kind":"receiver","time":1223901985,"time_of_arrival_s":25.49701619}',
        )

    def test_odd_items_and_slot(self) -> None:
        out = parse_uscg_fields(["", "b42", "S2249", "1179170462.74", "T", "s--5", "zzz"])
        self.assertEqual(
            dumps(out),
            r'{"slot":2249,"station":"42","station_kind":"base","time":1179170462.74,"unparsed":["T","s--5","zzz"]}',
        )

    def test_time_of_arrival_to_utc(self) -> None:
        self.assertEqual(str(time_of_arrival_to_utc(1223901985, 25.5)), "1223901985.500")
        self.assertIsNone(time_of_arrival_to_utc(1223901985, None))


if __name__ == "__main__":
    unittest.main()
