// bits.hpp — turning an AIS payload into bits, and bits into numbers and text.
//
// Everything in an AIS message is a *bit field*: "bits 8–37 are the MMSI",
// "bits 61–88 are the longitude in 1/10 000 minute". The job of this module
// is therefore very small:
//
//   1. Undo the six-bit ASCII "armouring" that NMEA uses to carry binary data
//      in printable characters (see chapter 2).
//   2. Give the rest of the code a tidy way to pull out unsigned integers,
//      signed integers, booleans and six-bit text starting at any bit offset.
//
// Spec references:
//   * Armouring table: IEC 61162-1 (NMEA 0183) §7.3.3 "Six-bit binary field
//     conversion" (as reproduced in AIVDM.adoc
//     "AIVDM/AIVDO Payload Armoring").
//   * Six-bit ASCII for text fields: ITU-R M.1371-5 Annex 8 Table 47.
//   * Two's complement for signed fields: ITU-R M.1371-5 Annex 8 §3 ("Negative
//     numbers are expressed using 2's complement"); bit order Annex 2 §3.3.7.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * We keep the bits as a `std::vector<std::uint8_t>` holding one 0 or 1 per
//   element. That is not the most compact representation (one byte per bit!)
//   but it is the clearest: `bits[61]` is exactly the bit you would circle
//   on paper. The Python version uses a string of '0'/'1' for the same
//   reason.
// * `class Bits` has a `private:` section that other code cannot touch and a
//   `public:` section of methods. Methods marked `const` promise not to
//   modify the object, which lets callers pass `const Bits&` (a read-only
//   reference, no copy).
// * `static Bits from_payload(...)` is a "named constructor": a function
//   that belongs to the class rather than to an instance (Python's
//   `@classmethod`).
// * Fixed-width integer types from <cstdint> — `std::uint8_t`,
//   `std::uint64_t`, `std::int64_t` — say exactly how many bits they hold,
//   unlike plain `int`. AIS fields are at most 30 bits wide, so 64 is plenty.
// * `std::size_t` is the unsigned type used for sizes and indices. Mixing it
//   with signed `int` provokes compiler warnings, so offsets here are
//   `std::size_t` throughout.

#pragma once

#include <cstddef>
#include <cstdint>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

#include "pycompat.hpp"

namespace aistut {

// --- 1. Armouring ---------------------------------------------------------
//
// Each payload character carries 6 bits. The sender adds 48 to the 6-bit
// value; if the result is above 87 it adds another 8 to skip the characters
// 'X'..'_' (which were avoided because some old equipment mangled them).
// To decode we do the reverse: subtract 48, and if the result is above 40
// subtract 8 more.

// Return the 6-bit value (0–63) carried by one payload character.
//
//     armor_char_to_value('0') == 0,  ('W') == 39,  ('`') == 40,  ('w') == 63
//
// The argument is a Unicode code point so that a stray non-ASCII character
// is reported the way Python reports it (as one character, not as bytes).
inline int armor_char_to_value(char32_t ch) {
    std::uint32_t code = ch;
    if (code < 48 || code > 119 || (88 <= code && code <= 95)) {   // outside '0'..'w', or 'X'..'_'
        throw std::invalid_argument("illegal armour character " + py::repr(py::utf8_encode(ch)));
    }
    int value = static_cast<int>(code) - 48;
    if (value > 40) value -= 8;
    return value;
}

// Convert an armoured payload to a vector of 0/1.
//
// `fill_bits` is field 7 of the !AIVDM sentence: how many of the low-order
// bits of the *last* character are padding and must be thrown away.
//
//     payload_to_bits("1", 0)   -> 0 0 0 0 0 1
//     payload_to_bits("1w", 4)  -> 0 0 0 0 0 1 1 1     (last char 'w' = 111111, drop 4 bits)
inline std::vector<std::uint8_t> payload_to_bits(std::string_view payload, long long fill_bits = 0) {
    if (fill_bits < 0 || fill_bits > 5) {
        throw std::invalid_argument("fill bits must be 0..5, got " + std::to_string(fill_bits));
    }
    std::vector<std::uint8_t> bits;
    bits.reserve(payload.size() * 6);
    for (char32_t ch : py::codepoints(payload)) {
        int value = armor_char_to_value(ch);
        // Emit the six bits most-significant first: shift right by 5, 4, … 0
        // and mask with 1 to isolate each bit.
        for (int shift = 5; shift >= 0; --shift) {
            bits.push_back(static_cast<std::uint8_t>((value >> shift) & 1));
        }
    }
    // Python's bits[:-fill_bits] on a short string just yields "".
    std::size_t drop = static_cast<std::size_t>(fill_bits);
    if (drop > bits.size()) drop = bits.size();
    bits.resize(bits.size() - drop);
    return bits;
}

// --- 2. Six-bit ASCII used *inside* text fields -----------------------------
//
// Do not confuse this with the armouring above! Text fields (ship name, call
// sign, destination, safety messages) pack one character into 6 bits using a
// different table: 0–31 map to '@'..'_' and 32–63 map to ' '..'?'.

inline const char SIXBIT_ASCII[65] = "@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_ !\"#$%&'()*+,-./0123456789:;<=>?";
static_assert(sizeof(SIXBIT_ASCII) == 65, "64 characters plus the terminating NUL");

// --- 3. The Bits reader -----------------------------------------------------

// A bit string with typed accessors.
//
// All offsets are *zero-based* bit positions, exactly as in the field tables
// of this tutorial (ITU-R M.1371 itself counts bits per field, GPSD's
// AIVDM.adoc and this tutorial count absolute offsets from 0).
class Bits {
public:
    explicit Bits(std::vector<std::uint8_t> bits) : bits_(std::move(bits)) {
        for (std::uint8_t b : bits_) {
            if (b != 0 && b != 1) throw std::invalid_argument("Bits expects a vector of 0 and 1");
        }
    }

    // Build directly from an !AIVDM payload and its fill-bit count.
    static Bits from_payload(std::string_view payload, long long fill_bits = 0) {
        return Bits(payload_to_bits(payload, fill_bits));
    }

    // Python's len(bits).
    std::size_t size() const { return bits_.size(); }

    // True if the field [start, start+length) lies inside the message.
    bool has(std::size_t start, std::size_t length) const {
        return start + length <= bits_.size();
    }

    // Unsigned big-endian integer from `length` bits at `start`.
    std::uint64_t uint(std::size_t start, std::size_t length) const {
        if (!has(start, length)) {
            throw std::out_of_range("field " + std::to_string(start) + "+" + std::to_string(length) +
                                    " beyond " + std::to_string(bits_.size()) + " bits");
        }
        if (length > 64) {
            // Python integers are unbounded; ours are not. No AIS field is
            // this wide, so this guards against programming errors only.
            throw std::invalid_argument("uint() supports at most 64 bits");
        }
        std::uint64_t value = 0;
        for (std::size_t i = start; i < start + length; ++i) {
            value = (value << 1) | bits_[i];       // shift left, bring in the next bit
        }
        return value;
    }

    // Signed (two's complement) integer. The first bit is the sign.
    std::int64_t sint(std::size_t start, std::size_t length) const {
        std::uint64_t raw = uint(start, length);
        if (length == 0 || length > 63) throw std::invalid_argument("sint() needs 1..63 bits");
        std::int64_t value = static_cast<std::int64_t>(raw);
        if (bits_[start] == 1) {                   // negative: subtract 2**length
            value -= static_cast<std::int64_t>(1) << length;
        }
        return value;
    }

    // A single bit as a boolean.
    bool flag(std::size_t start) const { return uint(start, 1) == 1; }

    // Six-bit ASCII text occupying `length` bits (a multiple of 6).
    //
    // Per ITU-R M.1371-5 Annex 2 §3.3.7, unused characters are '@'; in
    // practice names are often space-padded instead. We cut at the first
    // '@' and strip trailing spaces.
    std::string text(std::size_t start, std::size_t length) const {
        std::string s;
        // Python: for pos in range(start, start + length - 5, 6) — i.e. only
        // positions where a whole 6-bit character fits in `length`.
        for (std::size_t pos = start; pos + 6 <= start + length; pos += 6) {
            if (!has(pos, 6)) break;               // tolerate short (truncated) fields
            s.push_back(SIXBIT_ASCII[uint(pos, 6)]);
        }
        std::size_t at = s.find('@');
        if (at != std::string::npos) s.erase(at);
        return py::rstrip(s);
    }

    // The field as a '0'/'1' string — for binary payloads we cannot decode.
    // Like Python slicing, a range that runs past the end is clipped.
    std::string raw(std::size_t start, std::size_t length) const {
        std::string s;
        for (std::size_t i = start; i < start + length && i < bits_.size(); ++i) {
            s.push_back(bits_[i] ? '1' : '0');
        }
        return s;
    }

private:
    std::vector<std::uint8_t> bits_;
};

}  // namespace aistut
