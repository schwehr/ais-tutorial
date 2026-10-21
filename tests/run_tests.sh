#!/usr/bin/env bash
# Run every language's decoder over every data/*.nmea file and compare the
# output with tests/expected/<name>.jsonl.
#
#   tests/run_tests.sh                 # all four languages
#   PYTHON=python3.14 tests/run_tests.sh   # if plain python3 is older than 3.14
#   tests/run_tests.sh python rust     # a subset
#
# The Python decoder requires Python 3.14 or newer; set PYTHON to the
# interpreter to use (default: python3.14 if present, else python3).
#
# Exit status is non-zero if any build or comparison fails.

set -u
cd "$(dirname "$0")/.."
ROOT=$(pwd)

if [ -z "${PYTHON:-}" ]; then
  if command -v python3.14 >/dev/null 2>&1; then
    PYTHON=python3.14
  else
    PYTHON=python3
  fi
fi

LANGS=("$@")
if [ ${#LANGS[@]} -eq 0 ]; then
  LANGS=(python cpp rust typescript)
fi

build() {
  case "$1" in
    python)
      "${PYTHON:-python3}" -m unittest discover -q -s code/python/aistut -t code/python -p "*_test.py" &&
      "${PYTHON:-python3}" -m unittest discover -q -s code/python/tools -p "*_test.py"
      ;;
    cpp)        make -s -C code/cpp ;;
    rust)       (cd code/rust && cargo build --quiet --release) ;;
    typescript) (cd code/typescript && tsc -p tsconfig.json) ;;
  esac
}

run() {   # run <lang> <file>
  case "$1" in
    python)     "${PYTHON:-python3}" code/python/decode.py "$2" ;;
    cpp)        code/cpp/build/decode "$2" ;;
    rust)       code/rust/target/release/decode "$2" ;;
    typescript) node code/typescript/dist/decode.js "$2" ;;
  esac
}

status=0
for lang in "${LANGS[@]}"; do
  if ! build "$lang"; then
    echo "BUILD FAILED: $lang"
    status=1
    continue
  fi
  for data in data/*.nmea; do
    name=$(basename "$data" .nmea)
    expected="tests/expected/$name.jsonl"
    if [ ! -f "$expected" ]; then
      echo "MISSING expected output: $expected"
      status=1
      continue
    fi
    actual=$(mktemp)
    if run "$lang" "$data" > "$actual" 2>"$actual.err"; then
      if diff -q "$expected" "$actual" > /dev/null; then
        echo "ok   $lang $name"
      else
        echo "FAIL $lang $name"
        diff "$expected" "$actual" | head -20
        status=1
      fi
    else
      echo "CRASH $lang $name"
      head -20 "$actual.err"
      status=1
    fi
    rm -f "$actual" "$actual.err"
  done
done
exit $status
