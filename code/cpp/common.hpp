// common.hpp — small things shared by several message decoders.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `DecodeError` derives from `std::invalid_argument` (our stand-in for
//   Python's ValueError). The base class stores the message; `what()`
//   returns it as a C string. The extra members (`msg_type`, …) let a
//   caller inspect the problem without parsing the text.
// * `NotImplemented` is a separate exception so that `decode.cpp` can tell
//   "this decoder is a stub" apart from "this message is broken".
// * The tables are `const std::map<int, std::string>` — the closest thing to
//   a Python dict literal. `.contains(key)` (C++20) asks "is key in the
//   map?" — Python's `key in d`.
// * `std::format("message {} too short: {} bits < {}", …)` (C++20) builds the
//   error text the way a Python f-string would; the `{}` placeholders are
//   filled in order.
// * `std::div`-style splitting of a number into tens and ones is just `/`
//   and `%` in C++; Python's `divmod(code, 10)` returns both at once.

#pragma once

#include <cstdint>
#include <format>
#include <map>
#include <stdexcept>
#include <string>

namespace aistut {

// A message could not be decoded (usually: too few bits).
//
// All four language versions print the same text for this situation so
// that their outputs can be compared:  "message 5 too short: 160 bits < 424"
class DecodeError : public std::invalid_argument {
public:
    DecodeError(std::int32_t msg_type, std::size_t need_bits, std::size_t have_bits)
        : std::invalid_argument(std::format("message {} too short: {} bits < {}", msg_type, have_bits, need_bits)),
          msg_type(msg_type), need_bits(need_bits), have_bits(have_bits) {}

    std::int32_t msg_type;
    std::size_t need_bits;
    std::size_t have_bits;
};

inline void require_bits(std::int32_t msg_type, std::size_t bits_len, std::size_t need) {
    if (bits_len < need) throw DecodeError(msg_type, need, bits_len);
}

// Thrown by decoders that have not been written yet (Python:
// NotImplementedError). decode.cpp reports {"not_implemented":true}.
class NotImplemented : public std::logic_error {
public:
    explicit NotImplemented(const std::string& what) : std::logic_error(what) {}
};

// Type of electronic position fixing device (EPFD). ITU-R M.1371-5 Annex 8
// Table 51 (message 4); the same code list is used in messages 5, 9, 11,
// 19, 21 and 24.
inline const std::map<std::int32_t, std::string> EPFD_TYPES = {
    {0, "undefined"},
    {1, "GPS"},
    {2, "GLONASS"},
    {3, "combined GPS/GLONASS"},
    {4, "Loran-C"},
    {5, "Chayka"},
    {6, "integrated navigation system"},
    {7, "surveyed"},
    {8, "Galileo"},
    {15, "internal GNSS"},
};

inline std::string epfd_text(std::int32_t code) {
    auto it = EPFD_TYPES.find(code);
    return it == EPFD_TYPES.end() ? "not used" : it->second;
}

// Type of ship and cargo type. ITU-R M.1371-5 Annex 8 Table 53. The first
// digit is the vessel category, the second digit (for 2x–9x) the hazard
// category (1–4 = DG/HS/MP category X, Y, Z, OS — formerly A–D, which is
// what the `_text` strings still say; 5–8 reserved, 9 = no additional
// information). We spell out the common ones and derive the rest.
inline const std::map<std::int32_t, std::string> SHIP_TYPE_FIRST_DIGIT = {
    {2, "WIG"}, {3, ""}, {4, "HSC"}, {5, ""}, {6, "passenger"}, {7, "cargo"}, {8, "tanker"}, {9, "other"},
};
inline const std::map<std::int32_t, std::string> SHIP_TYPE_SPECIAL = {
    {0, "not available (default)"},
    {30, "fishing"}, {31, "towing"}, {32, "towing, length > 200 m or breadth > 25 m"},
    {33, "dredging or underwater operations"}, {34, "diving operations"},
    {35, "military operations"}, {36, "sailing"}, {37, "pleasure craft"},
    {38, "reserved"}, {39, "reserved"},
    {50, "pilot vessel"}, {51, "search and rescue vessel"}, {52, "tug"}, {53, "port tender"},
    {54, "anti-pollution equipment"}, {55, "law enforcement"}, {56, "spare - local vessel"},
    {57, "spare - local vessel"}, {58, "medical transport"}, {59, "noncombatant ship (RR Res. 18)"},
};
inline const std::map<std::int32_t, std::string> HAZARD_DIGIT = {
    {0, "all ships of this type"}, {1, "hazardous category A"}, {2, "hazardous category B"},
    {3, "hazardous category C"}, {4, "hazardous category D"}, {5, "reserved"}, {6, "reserved"},
    {7, "reserved"}, {8, "reserved"}, {9, "no additional information"},
};

inline std::string ship_type_text(std::int32_t code) {
    auto special = SHIP_TYPE_SPECIAL.find(code);
    if (special != SHIP_TYPE_SPECIAL.end()) return special->second;
    if (1 <= code && code <= 19) return "reserved for future use";
    if (100 <= code && code <= 199) return "reserved for regional use";
    if (code >= 200) return "reserved for future use";
    std::int32_t tens = code / 10;
    std::int32_t ones = code % 10;
    if (SHIP_TYPE_FIRST_DIGIT.contains(tens) && tens != 3 && tens != 5) {
        return SHIP_TYPE_FIRST_DIGIT.at(tens) + ", " + HAZARD_DIGIT.at(ones);
    }
    return "reserved";
}

}  // namespace aistut
