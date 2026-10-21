"""Unit tests for tools/uscg_to_tagblock.py."""

import io
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import uscg_to_tagblock  # noqa: E402
from aistut.tagblock import split_tag_block  # noqa: E402


class TestUscgToTagBlock(unittest.TestCase):
    def test_single_sentence_conversion(self) -> None:
        line = "!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,s34486,d-074,T25.49701619,x78124,r003669946,1223901985"
        group_ids: dict[tuple[str, str, str], int] = {}
        out = uscg_to_tagblock.convert(line, 1, group_ids)
        tag, rest = split_tag_block(out)
        self.assertIsNotNone(tag)
        assert tag is not None
        self.assertTrue(tag.checksum_ok)
        self.assertEqual(tag.params["s"], "003669946")
        self.assertEqual(tag.params["c"], "1223901985")
        self.assertEqual(tag.params["t"], "d-74 T25.50")
        self.assertEqual(tag.params["n"], "1")
        self.assertNotIn("g", tag.params)
        self.assertEqual(rest, "!AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02")

    def test_multi_sentence_grouping_and_main(self) -> None:
        lines = (
            "# comment\n"
            "\n"
            "!AIVDM,2,1,3,B,55P5TL01VIaAL@7WKO@mBplU@<PDhh000000001S;AJ::4A80?4i@E53,0*3E,r003669946,1223901985\n"
            "!AIVDM,2,2,3,B,1@0000000000000,2*55,r003669946,1223901985\n"
        )
        fake_in = io.StringIO(lines)
        fake_out = io.StringIO()
        with patch("sys.stdin", fake_in), patch("sys.stdout", fake_out):
            uscg_to_tagblock.main()
        out_lines = fake_out.getvalue().strip().splitlines()
        self.assertEqual(len(out_lines), 2)
        tag1, _ = split_tag_block(out_lines[0])
        tag2, _ = split_tag_block(out_lines[1])
        assert tag1 is not None and tag2 is not None
        self.assertEqual(tag1.group, (1, 2, 1))
        self.assertEqual(tag2.group, (2, 2, 1))
        self.assertEqual(tag1.params["n"], "1")
        self.assertEqual(tag2.params["n"], "2")


if __name__ == "__main__":
    unittest.main()
