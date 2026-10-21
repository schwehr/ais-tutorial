"""Unit tests for bits.py."""

import unittest

from aistut.bits import Bits, armor_char_to_value, payload_to_bits


class TestBits(unittest.TestCase):
    def test_armouring(self) -> None:
        self.assertEqual(armor_char_to_value("0"), 0)
        self.assertEqual(armor_char_to_value("W"), 39)
        self.assertEqual(armor_char_to_value("`"), 40)
        self.assertEqual(armor_char_to_value("w"), 63)
        for bad in ("/", "X", "_", "x"):
            with self.assertRaises(ValueError) as ctx:
                armor_char_to_value(bad)
            self.assertEqual(str(ctx.exception), f"illegal armour character {bad!r}")

    def test_payload_to_bits(self) -> None:
        self.assertEqual(payload_to_bits("1", 0), "000001")
        self.assertEqual(payload_to_bits("1w", 4), "00000111")
        with self.assertRaises(ValueError) as ctx:
            payload_to_bits("1", 7)
        self.assertEqual(str(ctx.exception), "fill bits must be 0..5, got 7")

    def test_signed_and_unsigned(self) -> None:
        # 1000 0000 = -128 signed, 128 unsigned; 0111 1111 = 127 both.
        b = Bits("1000000001111111")
        self.assertEqual(b.uint(0, 8), 128)
        self.assertEqual(b.sint(0, 8), -128)
        self.assertEqual(b.sint(8, 8), 127)
        self.assertEqual(b.sint(7, 2), 0)
        self.assertEqual(b.sint(8, 2), 1)
        self.assertEqual(b.sint(9, 2), -1)
        self.assertTrue(b.flag(0))
        self.assertFalse(b.flag(1))
        self.assertEqual(b.raw(6, 4), "0001")
        self.assertEqual(b.raw(14, 10), "11")
        self.assertEqual(len(b), 16)
        self.assertTrue(b.has(0, 16))
        self.assertFalse(b.has(1, 16))
        with self.assertRaises(IndexError):
            b.uint(1, 16)

    def test_invalid_bit_string(self) -> None:
        with self.assertRaises(ValueError):
            Bits("012")

    def test_six_bit_text(self) -> None:
        # "AB@ " -> 000001 000010 000000 100000 -> "AB"
        b = Bits("000001000010000000100000")
        self.assertEqual(b.text(0, 24), "AB")
        self.assertEqual(b.text(0, 12), "AB")
        self.assertEqual(b.text(0, 13), "AB")  # partial last character ignored
        self.assertEqual(b.text(0, 3), "")


if __name__ == "__main__":
    unittest.main()
