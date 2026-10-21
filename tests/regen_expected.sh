#!/usr/bin/env bash
# Regenerate tests/expected/*.jsonl from the Python reference decoder.
# Review the diff (git diff tests/expected) before committing!
#
# Requires Python 3.14 or newer; set PYTHON if plain python3 is older:
#   PYTHON=python3.14 tests/regen_expected.sh
set -eu
cd "$(dirname "$0")/.."
if [ -z "${PYTHON:-}" ]; then
  if command -v python3.14 >/dev/null 2>&1; then
    PYTHON=python3.14
  else
    PYTHON=python3
  fi
fi
for data in data/*.nmea; do
  name=$(basename "$data" .nmea)
  "$PYTHON" code/python/decode.py "$data" > "tests/expected/$name.jsonl"
  echo "wrote tests/expected/$name.jsonl ($(wc -l < "tests/expected/$name.jsonl") lines)"
done
