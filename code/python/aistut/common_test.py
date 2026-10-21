"""Unit tests for common.py."""

import unittest

from aistut.common import DecodeError, epfd_text, require_bits, ship_type_text


class TestCommon(unittest.TestCase):
    def test_error_text(self) -> None:
        err = DecodeError(5, 424, 160)
        self.assertEqual(str(err), "message 5 too short: 160 bits < 424")
        self.assertEqual(err.msg_type, 5)
        self.assertEqual(err.need_bits, 424)
        self.assertEqual(err.have_bits, 160)

        require_bits(1, 168, 168)
        with self.assertRaises(DecodeError) as ctx:
            require_bits(1, 100, 168)
        self.assertEqual(str(ctx.exception), "message 1 too short: 100 bits < 168")

    def test_epfd_text(self) -> None:
        self.assertEqual(epfd_text(0), "undefined")
        self.assertEqual(epfd_text(1), "GPS")
        self.assertEqual(epfd_text(7), "surveyed")
        self.assertEqual(epfd_text(15), "internal GNSS")
        self.assertEqual(epfd_text(9), "not used")

    def test_ship_types(self) -> None:
        self.assertEqual(ship_type_text(0), "not available (default)")
        self.assertEqual(ship_type_text(70), "cargo, all ships of this type")
        self.assertEqual(ship_type_text(81), "tanker, hazardous category A")
        self.assertEqual(ship_type_text(36), "sailing")
        self.assertEqual(ship_type_text(52), "tug")
        self.assertEqual(ship_type_text(45), "HSC, reserved")
        self.assertEqual(ship_type_text(20), "WIG, all ships of this type")
        self.assertEqual(ship_type_text(40), "HSC, all ships of this type")
        self.assertEqual(ship_type_text(60), "passenger, all ships of this type")
        self.assertEqual(ship_type_text(99), "other, no additional information")
        self.assertEqual(ship_type_text(5), "reserved for future use")
        self.assertEqual(ship_type_text(150), "reserved for regional use")
        self.assertEqual(ship_type_text(255), "reserved for future use")


if __name__ == "__main__":
    unittest.main()
