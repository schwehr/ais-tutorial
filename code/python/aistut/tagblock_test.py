"""Unit tests for tagblock.py."""

import unittest

from aistut.jsonout import dumps
from aistut.tagblock import format_tag_block, split_tag_block


class TestTagBlock(unittest.TestCase):
    def test_splits_and_checks(self) -> None:
        line = r"\s:r003669945,c:1224006993*7C\!AIVDM,1,1,,A,15N4cJ`005Jrek0H@9n`DW5608EP,0*13"
        tag, rest = split_tag_block(line)
        self.assertIsNotNone(tag)
        assert tag is not None
        self.assertTrue(tag.checksum_ok)
        self.assertEqual(tag.time, 1224006993)
        self.assertEqual(tag.source, "r003669945")
        self.assertTrue(rest.startswith("!AIVDM"))
        self.assertEqual(
            dumps(tag.to_dict()),
            r'{"checksum_ok":true,"params":{"c":"1224006993","s":"r003669945"},"time":1224006993}',
        )

    def test_group_and_errors(self) -> None:
        tag, _ = split_tag_block(r"\g:1-2-7*00\$GPGGA")
        assert tag is not None
        self.assertEqual(tag.group, (1, 2, 7))

        # Non-ASCII digits (like superscript 2) must not be accepted as int.
        tag_unicode, _ = split_tag_block("\\c:12\u00b2*00\\$GPGGA")
        assert tag_unicode is not None
        self.assertIsNone(tag_unicode.time)

        with self.assertRaises(ValueError) as ctx:
            split_tag_block(r"\g:1-2-7*00")
        self.assertEqual(str(ctx.exception), "unterminated TAG block")

        none_tag, rest = split_tag_block("  $GPGGA,1*00  ")
        self.assertIsNone(none_tag)
        self.assertEqual(rest, "$GPGGA,1*00")

    def test_round_trip(self) -> None:
        params = {"c": "1224006993", "s": "r003669945"}
        text = format_tag_block(params)
        tag, _ = split_tag_block(text)
        self.assertIsNotNone(tag)
        assert tag is not None
        self.assertTrue(tag.checksum_ok)
        self.assertEqual(tag.params, params)


if __name__ == "__main__":
    unittest.main()
