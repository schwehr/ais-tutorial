#!/usr/bin/env bash
# Regenerate tests/expected/*.jsonl from the Python reference decoder.
# Review the diff (git diff tests/expected) before committing!
set -eu
cd "$(dirname "$0")/.."
for data in data/*.nmea; do
  name=$(basename "$data" .nmea)
  python3 code/python/decode.py "$data" > "tests/expected/$name.jsonl"
  echo "wrote tests/expected/$name.jsonl ($(wc -l < "tests/expected/$name.jsonl") lines)"
done
