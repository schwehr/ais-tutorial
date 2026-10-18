#!/usr/bin/env bash
# Run every language's decoder over every data/*.nmea file and compare the
# output with tests/expected/<name>.jsonl.
#
#   tests/run_tests.sh                 # all four languages
#   tests/run_tests.sh python rust     # a subset
#
# Exit status is non-zero if any build or comparison fails.

set -u
cd "$(dirname "$0")/.."
ROOT=$(pwd)

LANGS=("$@")
if [ ${#LANGS[@]} -eq 0 ]; then
  LANGS=(python cpp rust typescript)
fi

build() {
  case "$1" in
    python)     : ;;
    cpp)        make -s -C code/cpp ;;
    rust)       (cd code/rust && cargo build --quiet --release) ;;
    typescript) (cd code/typescript && tsc -p tsconfig.json) ;;
  esac
}

run() {   # run <lang> <file>
  case "$1" in
    python)     python3 code/python/decode.py "$2" ;;
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
