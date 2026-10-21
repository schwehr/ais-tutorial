#!/usr/bin/env python3
"""Run the tutorial decoder:  python3 decode.py FILE [FILE...]"""
import sys
from aistut.decode import main

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
