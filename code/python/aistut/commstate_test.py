"""Unit tests for commstate.py — run with `python3.14 -m unittest`."""

import unittest

from .bits import Bits
from .commstate import (
    SLOTS_PER_FRAME,
    TMO_MAX,
    TMO_MIN,
    decode_itdma,
    decode_sotdma,
    next_itdma_slot,
    next_sotdma_slot,
    slot_to_seconds,
    slot_to_utc,
)


class TestCommState(unittest.TestCase):
    def test_constants(self) -> None:
        self.assertEqual(SLOTS_PER_FRAME, 2250)
        self.assertEqual(TMO_MIN, 3)
        self.assertEqual(TMO_MAX, 7)

    def test_slot_to_seconds(self) -> None:
        self.assertEqual(str(slot_to_seconds(0)), "0.000")
        # ais-2008-06-12 line 26: slot 453 -> 12.080 s into the UTC minute.
        self.assertEqual(str(slot_to_seconds(453)), "12.080")
        # ais-2008-06-12 line 34: slot 1177 -> 31.387 s.
        self.assertEqual(str(slot_to_seconds(1177)), "31.387")
        # ais-2008-06-12 line 113: slot 1203 -> 32.080 s.
        self.assertEqual(str(slot_to_seconds(1203)), "32.080")
        self.assertIsNone(slot_to_seconds(-1))
        self.assertIsNone(slot_to_seconds(2250))

    def test_next_sotdma_slot(self) -> None:
        # ais-2008-06-12 line 34 -> 91 -> 113: slot 1177 + offset 2276 mod 2250 = 1203 (+26 slots).
        self.assertEqual(next_sotdma_slot(1177, 2276), 1203)
        # ais-2008-06-12 line 26 -> 78 -> 128: slot 453 + offset 2231 mod 2250 = 434 (-19 slots).
        self.assertEqual(next_sotdma_slot(453, 2231), 434)
        # ais-2008-06-12 line 11369: slot_offset == 0 means de-allocate the slot.
        self.assertIsNone(next_sotdma_slot(782, 0))
        self.assertIsNone(next_sotdma_slot(-1, 100))
        self.assertIsNone(next_sotdma_slot(2250, 100))

    def test_next_itdma_slot(self) -> None:
        # ais-2008-06-12 line 8591 -> 8593: BALDER enters network on ch A at slot 779
        # with slot_increment = 754 -> next ch A slot is 779 + 754 = 1533.
        self.assertEqual(next_itdma_slot(779, 754), 1533)
        # Line 8593 -> 8607: slot 1533 + 738 = 2271 -> wraps to slot 21 in the next frame.
        self.assertEqual(next_itdma_slot(1533, 738), 21)
        # Line 8607: slot_increment = 0 (last transmission for this autonomous schedule chain).
        self.assertIsNone(next_itdma_slot(21, 0))

    def test_slot_to_utc_resolves_minute_ambiguity(self) -> None:
        # ais-2008-06-12 line 12: logger unix_time = 1213228834.26 (2008-06-12 00:00:34.26),
        # but timeout=1 sub-message reports utc_hour=23, utc_minute=59 (2008-06-11) and the
        # stream's slot is 1563 (41.680 s). Naive nearest-minute without hour/minute gives
        # 1213228841.680 (+7.42 s); with hour=23, minute=59 it recovers 1213228781.680 (-52.58 s).
        naive = slot_to_utc(1213228834.26, 1563)
        self.assertEqual(str(naive), "1213228841.680")
        exact = slot_to_utc(1213228834.26, 1563, 23, 59)
        self.assertEqual(str(exact), "1213228781.680")

        # ais-2008-06-12 line 53: logger 1213228924.66 (00:02:04.66), slot 453 (12.080 s),
        # utc_hour=0, utc_minute=1 -> 1213228872.080 (-52.58 s).
        self.assertEqual(
            str(slot_to_utc(1213228924.66, 453, 0, 1)),
            "1213228872.080",
        )

        # Fallback when utc_hour=24 (n/a): uses nearest minute if within 30 s.
        self.assertEqual(
            str(slot_to_utc(1213228924.66, 453, 24, 60)),
            "1213228932.080",
        )
        # Out-of-range slot returns None.
        self.assertIsNone(slot_to_utc(1213228924.66, 2250, 0, 1))

    def test_reexported_decoders(self) -> None:
        # ais-2008-06-12 line 91 (SOTDMA timeout=0, slot_offset=2276) and
        # line 8591 (ITDMA slot_increment=754, keep_flag=True).
        sotdma_bits = Bits.from_payload("1000000P01Jt;qDHaP?o2?vt00ST", 0)
        cs1 = decode_sotdma(sotdma_bits, 149)
        self.assertEqual(cs1["scheme"], "SOTDMA")
        self.assertEqual(cs1["slot_timeout"], 0)
        self.assertEqual(cs1["slot_offset"], 2276)

        itdma_bits = Bits.from_payload("38153r0000rsw2dHb?ndK;:T82tQ", 0)
        cs3 = decode_itdma(itdma_bits, 149)
        self.assertEqual(cs3["scheme"], "ITDMA")
        self.assertEqual(cs3["slot_increment"], 754)
        self.assertTrue(cs3["keep_flag"])


if __name__ == "__main__":
    unittest.main()
