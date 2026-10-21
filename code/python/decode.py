#!/usr/bin/env python3
"""Run the tutorial decoder:  python3 decode.py FILE [FILE...]

Requires Python 3.14 or newer (see aistut/__init__.py). If your `python3` is
older, run `python3.14 decode.py ...` instead.
"""
import sys

if sys.version_info < (3, 14):
    sys.exit("This tutorial's Python code requires Python 3.14 or newer")

from aistut.decode import main   # noqa: E402  (after the version guard)

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
