"""jsonout.py — a tiny, deterministic JSON writer.

Why not `json.dumps`? Because the C++, Rust and TypeScript versions of this
tutorial must print *byte-identical* output so that the test-suite can diff
them. Python's `json` module prints floats with `repr`, which other
languages do not reproduce. So we follow two rules everywhere:

  1. Object keys are written in sorted order.
  2. Every non-integer number is wrapped in `Fixed(value, decimals)` and
     printed with exactly that many decimals (`format(x, '.6f')`, which
     rounds the exact binary value the same way C's printf, Rust's `{:.6}`
     and JavaScript's `toFixed` do).

Supported values: None, bool, int, Fixed, str, list, dict.

A `Fixed` whose value is NaN, ±infinity or of magnitude 1e21 or more is
printed as `null`: JSON has no `nan`/`inf` tokens, and above 1e21
JavaScript's `toFixed` switches to exponent notation, so the four languages
could not agree on the digits anyway. Such values never come from a real
receiver; they only arise from corrupted text like `hdop` = `inf`.
"""

import math
from dataclasses import dataclass

# |value| at or above this prints as null (see the module docstring).
FIXED_LIMIT = 1e21


@dataclass(frozen=True)
class Fixed:
    """A float to be printed with a fixed number of decimals."""

    value: float
    decimals: int

    def __str__(self) -> str:
        if not math.isfinite(self.value) or abs(self.value) >= FIXED_LIMIT:
            return "null"
        s = format(self.value, f".{self.decimals}f")
        if s.startswith("-") and float(s) == 0.0:   # avoid "-0.0"
            s = s[1:]
        return s


def _escape(s: str) -> str:
    out = ['"']
    for ch in s:
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\r":
            out.append("\\r")
        elif ch == "\t":
            out.append("\\t")
        elif ord(ch) < 0x20:
            out.append(f"\\u{ord(ch):04x}")
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)


def dumps(value) -> str:
    """Serialise `value` to compact JSON with sorted keys."""
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, Fixed):
        return str(value)
    if isinstance(value, float):
        raise TypeError("wrap floats in Fixed(value, decimals) for deterministic output")
    if isinstance(value, str):
        return _escape(value)
    if isinstance(value, (list, tuple)):
        return "[" + ",".join(dumps(v) for v in value) + "]"
    if isinstance(value, dict):
        items = sorted(value.items())
        return "{" + ",".join(_escape(str(k)) + ":" + dumps(v) for k, v in items) + "}"
    raise TypeError(f"cannot serialise {type(value).__name__}")
