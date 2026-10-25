# C++23 version of the tutorial decoder

The same decoder as `code/python/`, written in portable C++23 with no
dependencies beyond the standard library. Output is byte-identical to the
Python reference (the test-suite checks this).

## Build and run

```sh
make                      # -> build/decode  (g++ -std=c++23 -O2 -Wall -Wextra -pedantic)
make CXX=clang++          # the same with clang
build/decode FILE [FILE...]
build/decode - < log.nmea # '-' reads standard input
make clean
```

Or from the repository root: `tests/run_tests.sh cpp`.

Every module is a header (`*.hpp`) and `decode.cpp` is the only translation
unit, so there is a single compile command and no linking step to explain.

### Compiler requirements

The code uses the C++23 standard library — `<print>`, `<expected>`,
`std::views::chunk`, the monadic `std::optional` operations — so it needs

* **GCC 14 or newer** (`g++ -std=c++23`; tested with GCC 15.2), or
* **Clang 18 or newer** using **libstdc++ 14 or newer** (`clang++
  -std=c++23`; tested with Clang 21 on libstdc++ 15). Clang's own libc++
  gained these headers piecemeal across LLVM 17–19; the tutorial is only
  verified against libstdc++.

Both compilers build the program without a single warning under
`-Wall -Wextra -pedantic`.

## Module map

| file | purpose | chapter | C++ features it demonstrates |
|---|---|---|---|
| `pycompat.hpp` | `strip`, `split`, `partition`, `int()`, `float()`, `repr()`, UTF-8 repair — the Python string behaviour the other modules rely on | — | `std::string_view`, `std::optional`, `std::expected` (`int_expected`), `[[nodiscard]]`, designated initialisers, nested namespaces, `inline` functions in headers |
| `jsonout.hpp` | `Json` value type and deterministic writer | — | `std::variant`, recursive types via `std::vector`/`std::map`, constructor overloading, `std::format("{:.{}f}")`, structured bindings over a map |
| `nmea.hpp` | sentence splitting, checksum, GGA/GSA/GSV/RMC | 1 | aggregates (`struct` as dataclass), custom exception classes, `const std::map` lookup tables, function-pointer tables, `std::optional::transform` |
| `bits.hpp` | armour decoding, `Bits` reader, six-bit ASCII | 2 | `class` with `public`/`private`, `const` methods, static "named constructors", fixed-width integers, bit shifting/masking, `inline constexpr` tables, `std::format` error texts |
| `vdm.hpp` | `!AIVDM`/`!AIVDO` fields, multi-fragment `Assembler` | 2 | `std::expected<Vdm, std::string>` return, designated initialisers, `std::optional` return values, `std::tuple` as a map key, references into a container, `std::move` |
| `msg_1_2_3.hpp` | position reports | 3 | `std::array` enumeration tables, integer vs floating division, `std::pow` |
| `tagblock.hpp` | NMEA 4.10 TAG blocks | 4 | `std::pair` + structured bindings, `std::array<std::int64_t, 3>`, string literal escaping, `std::format("{:02X}")` |
| `uscg.hpp` | USCG N-AIS metadata after the checksum | 5 | optionals instead of exceptions for routine failure, `std::span<const std::string>`, `std::abs`, range-for over an initializer list |
| `commstate.hpp` | TDMA communication-state schedule and UTC clock helpers | 17 | default `std::optional<std::int32_t>` parameters, Euclidean modulo normalisation |
| `common.hpp` | `DecodeError`, `NotImplemented`, EPFD and ship-type tables | — | exception hierarchies, `std::map::contains`, `std::format` |
| `msg_4_11.hpp`, `msg_5.hpp` | base station report, static and voyage data | 6, 7 | `std::format` with `{:04}`/`{:02}` zero padding, lambdas |
| `msg_7_13.hpp`, `msg_20.hpp` | acknowledgements, data link management | 12, 13 | range pipelines: `std::views::iota \| std::views::chunk \| std::views::take` over repeated bit groups |
| `msg_22.hpp` | channel management | 13 | `std::set::contains`, `std::optional<std::int32_t>` |
| other `msg_*.hpp` | one per message (family) | 6+ | `constexpr` sentinels, `std::array` text tables, helper functions shared between messages |
| `decode.cpp` | CLI: lines in → JSON lines out | all | `main(argc, argv)`, `std::ifstream` + RAII, binary-safe line reading, `try`/`catch` ordering, `std::expected` at the call site, `std::println` |

Function names follow the Python modules; the message decoders are
`decode_msg_1_2_3`, `decode_msg_4_11`, `decode_msg_5`, … `decode_msg_27`, all
taking `const Bits&` and returning `Json`.

## C++20/23 facilities and where they are used

| facility | standard | where |
|---|---|---|
| `std::format` | C++20 | `jsonout.hpp` (`Fixed::str`, `\uXXXX` escapes), `common.hpp` (`DecodeError`), `bits.hpp` (error texts), `tagblock.hpp` (`format_tag_block`), `msg_4_11.hpp` (`utc_iso`), `msg_5.hpp` (`eta_text`), `decode.cpp` |
| `std::print` / `std::println` | C++23 | `decode.cpp` — all standard output and error output |
| `std::expected<T, E>` / `std::unexpected` | C++23 | `pycompat.hpp` (`py::int_expected`), `vdm.hpp` (`parse_vdm`), `decode.cpp` (`process_line`) |
| `std::optional::transform` | C++23 | `nmea.hpp` (`fixed`) |
| `std::views::iota`, `std::views::chunk`, `std::views::take` | C++20 / C++23 (`chunk`) | `msg_7_13.hpp`, `msg_20.hpp` |
| `std::span` | C++20 | `uscg.hpp` (`parse_uscg_fields`) |
| designated initialisers | C++20 | `pycompat.hpp` (`Partition`), `vdm.hpp` (`Vdm`, `Message`), `jsonout.hpp` (`Fixed`), `nmea.hpp` |
| `std::map::contains`, `std::set::contains` | C++20 | `common.hpp`, `msg_22.hpp` |
| `std::string::starts_with` | C++20 | `jsonout.hpp` |
| `[[nodiscard]]` | C++17 | `Json::dump`, `Bits::from_payload`, `Assembler::add`, `parse_vdm`, `checksum`, `py::to_int`/`to_float`/`int_expected` |
| `inline constexpr` tables | C++17 | `bits.hpp` (`SIXBIT_ASCII`), `msg_*.hpp` sentinels |

Not used, deliberately: `std::to_underlying` and `std::unreachable` (there
are no scoped enums or unreachable branches to apply them to), `auto(x)`
decay copies, `if consteval`, concepts and modules.

## Conventions worth knowing

* `std::invalid_argument` plays the role of Python's `ValueError`; `NmeaError`
  and `DecodeError` derive from it so a caller can catch either level.
  Where the Python reference raises a `ValueError` that the caller is
  expected to catch immediately (`parse_vdm`), the C++ version returns a
  `std::expected` instead and the caller tests `if (!result)`.
* Every nullable value is a `std::optional<T>`; `Json` converts an empty
  optional to `null`.
* Objects are `std::map<std::string, Json>`, so keys come out sorted without
  any extra code.
* Input is read as bytes with Python's "universal newline" rule (`\n`,
  `\r\n`, lone `\r`), repaired to valid UTF-8 (`errors="replace"`
  semantics), and stripped with Python's `str.strip()` character set.
* Fixed-decimal numbers go through `std::format("{:.{}f}", value, decimals)`.
  libstdc++ and glibc's `printf` both emit the correctly rounded decimal
  expansion of the binary double, so the digits are identical to the
  `snprintf("%.*f")` the C++17 version used (verified over 14 million
  random and halfway-case values) and to Python's `format(x, '.Nf')`.
  Negative zero is normalised to `0.0…` and NaN/±inf print as `null`.
* The only known behavioural difference from Python: integers are 64-bit, so
  a USCG field with a 20+ digit number lands in `unparsed` instead of being
  printed as a big integer.
* All integer types use explicit `<cstdint>` fixed-width types (`std::int32_t`,
  `std::uint32_t`, `std::int64_t`, `std::uint64_t`, `std::uint8_t`, etc.),
  avoiding primitive/built-in integer types (`int`, `long`, `unsigned`) except
  for the standard `int main(int argc, char** argv)` signature.
