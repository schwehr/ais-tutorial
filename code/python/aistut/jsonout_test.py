"""Unit tests for jsonout.py."""

import math
import unittest

from aistut.jsonout import Fixed, dumps


class TestJsonOut(unittest.TestCase):
    def test_fixed_numbers(self) -> None:
        self.assertEqual(str(Fixed(42.0740401, 6)), "42.074040")
        self.assertEqual(str(Fixed(-0.0, 1)), "0.0")
        self.assertEqual(str(Fixed(-0.00001, 2)), "0.00")
        self.assertEqual(str(Fixed(-0.5, 1)), "-0.5")
        self.assertEqual(str(Fixed(1179170462.74, 2)), "1179170462.74")

    def test_fixed_non_finite_and_huge(self) -> None:
        self.assertEqual(str(Fixed(math.nan, 2)), "null")
        self.assertEqual(str(Fixed(math.inf, 2)), "null")
        self.assertEqual(str(Fixed(-math.inf, 2)), "null")
        self.assertEqual(str(Fixed(1e21, 2)), "null")
        self.assertEqual(str(Fixed(-1e21, 2)), "null")

    def test_escaping_and_sorting(self) -> None:
        obj = {
            "b": 'q"\\\n\r\t\x01',
            "a": None,
            "c": [True, False, 2, Fixed(1.5, 1)],
        }
        self.assertEqual(
            dumps(obj),
            r'{"a":null,"b":"q\"\\\n\r\t\u0001","c":[true,false,2,1.5]}',
        )

    def test_raw_float_and_unsupported_rejected(self) -> None:
        with self.assertRaises(TypeError):
            dumps(1.5)
        with self.assertRaises(TypeError):
            dumps(object())


if __name__ == "__main__":
    unittest.main()
