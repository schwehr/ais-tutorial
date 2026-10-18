"""tagblock.py — NMEA 0183 v4.10 TAG blocks (chapter 4).

A TAG block is metadata that *precedes* a sentence on the same line:

    \\s:r003669945,c:1224006993*2B\\!AIVDM,1,1,,A,15N4cJ`005Jrek0H@9n`DW5608EP,0*13
    ^                            ^ ^
    backslash  parameter list   checksum of the text between the backslashes

Defined in IEC 61162-1 ed. 4 / NMEA 0183 v4.10 §7.1.2 "TAG block".
Parameters are `code:value` pairs separated by commas:

    c:  UNIX time (seconds, or milliseconds if > 10^10 or configured so)
    d:  destination identification (≤ 15 chars)
    g:  sentence grouping "n-total-group_id" for multi-sentence groups
    n:  line count (an incrementing serial number)
    r:  relative time
    s:  source identification (≤ 15 chars) — the receiving station
    t:  free text (≤ 15 chars)

The checksum is computed exactly like a sentence checksum: XOR of all
characters between the two backslashes, excluding the `*hh`.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional

from .nmea import checksum


@dataclass
class TagBlock:
    params: dict[str, str] = field(default_factory=dict)
    checksum_ok: bool = False
    raw: str = ""

    # Typed views of the common parameters. Missing -> None.
    @property
    def time(self) -> Optional[int]:
        c = self.params.get("c")
        return int(c) if c and c.isdigit() else None

    @property
    def source(self) -> Optional[str]:
        return self.params.get("s")

    @property
    def group(self) -> Optional[tuple[int, int, int]]:
        """(sentence number, total sentences, group id) from the g: parameter."""
        g = self.params.get("g")
        if not g:
            return None
        parts = g.split("-")
        if len(parts) != 3 or not all(p.isdigit() for p in parts):
            return None
        return int(parts[0]), int(parts[1]), int(parts[2])

    def to_dict(self) -> dict:
        d: dict = {"params": dict(self.params), "checksum_ok": self.checksum_ok}
        if self.time is not None:
            d["time"] = self.time
        if self.group is not None:
            d["group"] = list(self.group)
        return d


def split_tag_block(line: str) -> tuple[Optional[TagBlock], str]:
    """Separate a leading TAG block from the sentence that follows it.

    Returns (TagBlock or None, remainder). Lines without a leading
    backslash are returned unchanged.
    """
    line = line.strip()
    if not line.startswith("\\"):
        return None, line
    end = line.find("\\", 1)
    if end < 0:
        raise ValueError("unterminated TAG block")
    inner = line[1:end]
    body, star, hexpart = inner.partition("*")
    ok = False
    if star and len(hexpart) >= 2:
        try:
            ok = int(hexpart[:2], 16) == checksum(body)
        except ValueError:
            ok = False
    params: dict[str, str] = {}
    for item in body.split(","):
        code, colon, value = item.partition(":")
        if colon:
            params[code] = value
    return TagBlock(params, ok, inner), line[end + 1:]


def format_tag_block(params: dict[str, str]) -> str:
    """Build a TAG block string from parameters (used in chapter 4 to
    convert USCG-format lines into TAG block lines)."""
    body = ",".join(f"{k}:{v}" for k, v in params.items())
    return f"\\{body}*{checksum(body):02X}\\"
