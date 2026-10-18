# C++17 version of the tutorial decoder

The same decoder as `code/python/`, written in portable C++17 with no
dependencies beyond the standard library. Output is byte-identical to the
Python reference (the test-suite checks this).

## Build and run

```sh
make                      # -> build/decode  (g++ -std=c++17 -O2 -Wall -Wextra -pedantic)
build/decode FILE [FILE...]
build/decode - < log.nmea # '-' reads standard input
make clean
```

Or from the repository root: `tests/run_tests.sh cpp`.

Every module is a header (`*.hpp`) and `decode.cpp` is the only translation
unit, so there is a single compile command and no linking step to explain.

## Module map

| file | purpose | chapter | C++ features it demonstrates |
|---|---|---|---|
| `pycompat.hpp` | `strip`, `split`, `partition`, `int()`, `float()`, `repr()`, UTF-8 repair — the Python string behaviour the other modules rely on | — | `std::string_view`, `std::optional`, nested namespaces, `inline` functions in headers |
| `jsonout.hpp` | `Json` value type and deterministic writer | — | `std::variant`, recursive types via `std::vector`/`std::map`, constructor overloading, `snprintf("%.*f")`, structured bindings over a map |
| `nmea.hpp` | sentence splitting, checksum, GGA/GSA/GSV/RMC | 1 | aggregates (`struct` as dataclass), custom exception classes, `const std::map` lookup tables, function-pointer tables |
| `bits.hpp` | armour decoding, `Bits` reader, six-bit ASCII | 2 | `class` with `public`/`private`, `const` methods, static "named constructors", fixed-width integers, bit shifting/masking |
| `vdm.hpp` | `!AIVDM`/`!AIVDO` fields, multi-fragment `Assembler` | 2 | `std::optional` return values, `std::tuple` as a map key, references into a container, `std::move` |
| `msg_1_2_3.hpp` | position reports | 3 | `std::array` enumeration tables, integer vs floating division, `std::pow` |
| `tagblock.hpp` | NMEA 4.10 TAG blocks | 4 | `std::pair` + structured bindings, `std::array<long long, 3>`, string literal escaping |
| `uscg.hpp` | USCG N-AIS metadata after the checksum | 5 | optionals instead of exceptions for routine failure, `std::abs`, range-for over an initializer list |
| `common.hpp` | `DecodeError`, `NotImplemented`, EPFD and ship-type tables | — | exception hierarchies, `std::map::count`/`at` |
| `msg_*.hpp` (22 stubs) | one per message (family); each throws `NotImplemented` | 6+ | unnamed parameters to silence `-Wextra` |
| `decode.cpp` | CLI: lines in → JSON lines out | all | `main(argc, argv)`, `std::ifstream` + RAII, binary-safe line reading, `try`/`catch` ordering |

Function names follow the Python modules; the message decoders are
`decode_msg_1_2_3`, `decode_msg_4_11`, `decode_msg_5`, … `decode_msg_27`, all
taking `const Bits&` and returning `Json`.

## Conventions worth knowing

* `std::invalid_argument` plays the role of Python's `ValueError`; `NmeaError`
  and `DecodeError` derive from it so a caller can catch either level.
* Every nullable value is a `std::optional<T>`; `Json` converts an empty
  optional to `null`.
* Objects are `std::map<std::string, Json>`, so keys come out sorted without
  any extra code.
* Input is read as bytes with Python's "universal newline" rule (`\n`,
  `\r\n`, lone `\r`), repaired to valid UTF-8 (`errors="replace"`
  semantics), and stripped with Python's `str.strip()` character set.
* The only known behavioural difference from Python: integers are 64-bit, so
  a USCG field with a 20+ digit number lands in `unparsed` instead of being
  printed as a big integer.
