// pycompat.hpp — the handful of things Python gives you for free that C++
// does not: strip(), split(), partition(), int(), float(), repr(), UTF-8
// handling.
//
// The Python reference implementation leans on these every few lines, and
// the test-suite requires the C++ output to be *byte-identical* to Python's,
// including its error texts ("invalid literal for int() with base 10: 'x'").
// Rather than scatter approximations through every module we collect the
// small re-implementations here, each documented with the Python behaviour
// it copies. Nothing in this file knows anything about NMEA or AIS.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `#pragma once` at the top of a header tells the compiler to include it
//   at most once per translation unit. Every header in this tutorial is
//   "header-only": the whole implementation lives in the .hpp and functions
//   are marked `inline` so that including the header from several places
//   does not produce duplicate definitions.
// * `std::string_view` (C++17) is a non-owning "window" onto characters
//   that already live somewhere else. Taking a `string_view` parameter means
//   callers can pass a `std::string`, a string literal, or a slice without
//   copying. The flip side: never keep a string_view alive longer than the
//   string it points at.
// * `std::optional<T>` (C++17) is "a T, or nothing" — Python's `Optional[T]`
//   / `None`. We return `std::nullopt` where Python would raise ValueError
//   and let the caller decide whether that is an error.
// * `std::expected<T, E>` (C++23) is "a T, or an error E". Where an optional
//   only says *that* something failed, an expected also carries *why* — here
//   Python's exact ValueError text — so the caller can report it without
//   a `try`/`catch`. `std::unexpected(text)` builds the error side.
// * `[[nodiscard]]` on a function asks the compiler to warn when a caller
//   throws the result away — ignoring a parsed value is always a bug here.
// * `namespace aistut::py { ... }` nests a namespace `py` inside `aistut`
//   (C++17 nested-namespace syntax). Callers write `py::strip(s)`.

#pragma once

#include <cctype>
#include <cerrno>
#include <cstdint>
#include <cstdlib>
#include <expected>
#include <limits>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

namespace aistut::py {

// --- Whitespace and slicing -------------------------------------------------

// Python's str.isspace() for the ASCII range: besides the six C whitespace
// characters it also counts the information separators 0x1C–0x1F.
inline bool is_space(std::uint8_t c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\v' ||
           c == '\f' || (c >= 0x1c && c <= 0x1f);
}

// str.strip() with no argument: remove leading and trailing whitespace.
inline std::string strip(std::string_view s) {
    std::size_t begin = 0;
    std::size_t end = s.size();
    while (begin < end && is_space(static_cast<std::uint8_t>(s[begin]))) ++begin;
    while (end > begin && is_space(static_cast<std::uint8_t>(s[end - 1]))) --end;
    return std::string(s.substr(begin, end - begin));
}

// str.rstrip() with no argument.
inline std::string rstrip(std::string_view s) {
    std::size_t end = s.size();
    while (end > 0 && is_space(static_cast<std::uint8_t>(s[end - 1]))) --end;
    return std::string(s.substr(0, end));
}

// str.lstrip(chars) for a single character, e.g. rest.lstrip("-").
inline std::string lstrip(std::string_view s, char ch) {
    std::size_t begin = 0;
    while (begin < s.size() && s[begin] == ch) ++begin;
    return std::string(s.substr(begin));
}

// Python slicing s[start:stop] clamps out-of-range indices instead of
// throwing; std::string::substr throws std::out_of_range if start > size().
// Negative indices are not needed in this tutorial, so they are not supported.
inline std::string slice(std::string_view s, std::size_t start,
                         std::size_t stop = std::string_view::npos) {
    if (start > s.size()) start = s.size();
    if (stop > s.size()) stop = s.size();
    if (stop < start) stop = start;
    return std::string(s.substr(start, stop - start));
}

inline bool startswith(std::string_view s, std::string_view prefix) {
    return s.size() >= prefix.size() && s.substr(0, prefix.size()) == prefix;
}

// --- Splitting --------------------------------------------------------------

// str.split(sep) with an explicit separator: "" -> [""], "a,,b" -> ["a","","b"].
// (This differs from str.split() with *no* argument, which drops empties.)
inline std::vector<std::string> split(std::string_view s, char sep) {
    std::vector<std::string> parts;
    std::size_t start = 0;
    while (true) {
        std::size_t at = s.find(sep, start);
        if (at == std::string_view::npos) {
            parts.emplace_back(s.substr(start));
            return parts;
        }
        parts.emplace_back(s.substr(start, at - start));
        start = at + 1;
    }
}

// str.partition(sep): split at the *first* occurrence into head, sep, tail.
// If sep is absent, `found` is false and `head` is the whole string — the
// same as Python returning ('whole', '', '').
struct Partition {
    std::string head;
    std::string tail;
    bool found;
};

inline Partition partition(std::string_view s, char sep) {
    std::size_t at = s.find(sep);
    if (at == std::string_view::npos) return {.head = std::string(s), .tail = "", .found = false};
    return {.head = std::string(s.substr(0, at)), .tail = std::string(s.substr(at + 1)), .found = true};
}

// --- Numbers ----------------------------------------------------------------

// str.isdigit(): non-empty and every character a digit (ASCII only here).
inline bool isdigit(std::string_view s) {
    if (s.empty()) return false;
    for (char c : s) {
        if (c < '0' || c > '9') return false;
    }
    return true;
}

inline std::int32_t digit_value(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'z') return c - 'a' + 10;
    if (c >= 'A' && c <= 'Z') return c - 'A' + 10;
    return 99;
}

// int(text, base) for base 10 or 16, returning nullopt where Python raises
// ValueError. Python accepts surrounding whitespace, one sign, an optional
// "0x" prefix for base 16, and single underscores *between* digits
// ("1_000"). Python integers are unbounded; we give up (nullopt) if the
// value does not fit in a `std::int64_t`, which never happens with real data.
[[nodiscard]] inline std::optional<std::int64_t> to_int(std::string_view text, std::int32_t base = 10) {
    std::string s = strip(text);
    std::size_t i = 0;
    bool negative = false;
    if (i < s.size() && (s[i] == '+' || s[i] == '-')) {
        negative = (s[i] == '-');
        ++i;
    }
    if (base == 16 && i + 1 < s.size() && s[i] == '0' && (s[i + 1] == 'x' || s[i + 1] == 'X')) {
        i += 2;
        if (i < s.size() && s[i] == '_') ++i;   // "0x_ff" is legal Python
    }
    if (i >= s.size()) return std::nullopt;
    std::int64_t value = 0;
    bool previous_was_digit = false;
    for (; i < s.size(); ++i) {
        char c = s[i];
        if (c == '_') {
            if (!previous_was_digit) return std::nullopt;
            previous_was_digit = false;
            continue;
        }
        std::int32_t d = digit_value(c);
        if (d >= base) return std::nullopt;
        // Overflow check before multiplying: value * base + d must fit.
        if (value > (std::numeric_limits<std::int64_t>::max() - d) / base) return std::nullopt;
        value = value * base + d;
        previous_was_digit = true;
    }
    if (!previous_was_digit) return std::nullopt;   // trailing underscore
    return negative ? -value : value;
}

// float(text), returning nullopt where Python raises ValueError. Python
// accepts decimal notation with an optional exponent, "inf"/"infinity"/"nan"
// in any case, surrounding whitespace and underscores between digits. It
// rejects the hexadecimal floats ("0x1p3") that C's strtod would accept, so
// we check the characters ourselves before calling strtod.
[[nodiscard]] inline std::optional<double> to_float(std::string_view text) {
    std::string s = strip(text);
    if (s.empty()) return std::nullopt;
    std::string cleaned;
    cleaned.reserve(s.size());
    bool previous_was_digit = false;
    for (std::size_t i = 0; i < s.size(); ++i) {
        char c = s[i];
        bool is_digit = (c >= '0' && c <= '9');
        if (c == '_') {
            bool next_is_digit = (i + 1 < s.size() && s[i + 1] >= '0' && s[i + 1] <= '9');
            if (!previous_was_digit || !next_is_digit) return std::nullopt;
            previous_was_digit = false;
            continue;
        }
        cleaned.push_back(c);
        previous_was_digit = is_digit;
    }
    // Everything after an optional sign must be digits/./e/E/+/- or one of
    // the special words.
    std::size_t start = (cleaned[0] == '+' || cleaned[0] == '-') ? 1 : 0;
    std::string body = cleaned.substr(start);
    std::string lower;
    for (char c : body) lower.push_back(static_cast<char>(std::tolower(static_cast<std::uint8_t>(c))));
    bool special = (lower == "inf" || lower == "infinity" || lower == "nan");
    if (!special) {
        for (char c : body) {
            bool ok = (c >= '0' && c <= '9') || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-';
            if (!ok) return std::nullopt;
        }
    }
    errno = 0;
    char* end = nullptr;
    double value = std::strtod(cleaned.c_str(), &end);
    if (end != cleaned.c_str() + cleaned.size()) return std::nullopt;
    return value;
}

// --- repr() -----------------------------------------------------------------

// repr(str): the text in quotes with escapes, as Python prints it in error
// messages such as "illegal armour character 'x'". Python prefers single
// quotes and switches to double quotes only if the text contains a single
// quote but no double quote. Non-ASCII (UTF-8) bytes are passed through,
// which matches Python for printable characters.
inline std::string repr(std::string_view s) {
    bool has_single = s.find('\'') != std::string_view::npos;
    bool has_double = s.find('"') != std::string_view::npos;
    char quote = (has_single && !has_double) ? '"' : '\'';
    std::string out(1, quote);
    for (char c : s) {
        std::uint8_t u = static_cast<std::uint8_t>(c);
        if (c == quote || c == '\\') {
            out.push_back('\\');
            out.push_back(c);
        } else if (c == '\n') {
            out += "\\n";
        } else if (c == '\r') {
            out += "\\r";
        } else if (c == '\t') {
            out += "\\t";
        } else if (u < 0x20 || u == 0x7f) {
            static const char* hex = "0123456789abcdef";
            out += "\\x";
            out.push_back(hex[u >> 4]);
            out.push_back(hex[u & 0xf]);
        } else {
            out.push_back(c);
        }
    }
    out.push_back(quote);
    return out;
}

// Variants that throw with Python's exact wording, for the places where the
// Python reference lets ValueError propagate.
inline std::int64_t int_or_throw(std::string_view text) {
    auto v = to_int(text);
    if (!v) throw std::invalid_argument("invalid literal for int() with base 10: " + repr(text));
    return *v;
}

inline double float_or_throw(std::string_view text) {
    auto v = to_float(text);
    if (!v) throw std::invalid_argument("could not convert string to float: " + repr(text));
    return *v;
}

// The same conversion with the error as a *value* instead of an exception:
// `std::expected<std::int64_t, std::string>` holds either the number or
// Python's ValueError text. Used where the caller wants to return the
// complaint to its own caller rather than catch it (see vdm.hpp).
[[nodiscard]] inline std::expected<std::int64_t, std::string> int_expected(std::string_view text) {
    auto v = to_int(text);
    if (!v) return std::unexpected("invalid literal for int() with base 10: " + repr(text));
    return *v;
}

// --- UTF-8 ------------------------------------------------------------------
//
// Python reads the input as text (`encoding="utf-8", errors="replace"`) and
// works on *code points*; C++ strings are bytes. The two helpers below
// bridge that gap so that checksums and error messages agree even on lines
// containing stray non-ASCII bytes.

// Length of the UTF-8 sequence announced by a lead byte, or 0 if the byte
// cannot start a sequence.
inline std::int32_t utf8_sequence_length(std::uint8_t lead) {
    if (lead < 0x80) return 1;
    if (lead >= 0xC2 && lead <= 0xDF) return 2;
    if (lead >= 0xE0 && lead <= 0xEF) return 3;
    if (lead >= 0xF0 && lead <= 0xF4) return 4;
    return 0;
}

// Is `b` a valid continuation byte at position `index` (1-based) after `lead`?
// The second byte has narrower limits for a few leads (over-long forms and
// surrogates are illegal).
inline bool utf8_continuation_ok(std::uint8_t lead, std::int32_t index, std::uint8_t b) {
    if (index == 1) {
        if (lead == 0xE0) return b >= 0xA0 && b <= 0xBF;
        if (lead == 0xED) return b >= 0x80 && b <= 0x9F;
        if (lead == 0xF0) return b >= 0x90 && b <= 0xBF;
        if (lead == 0xF4) return b >= 0x80 && b <= 0x8F;
    }
    return b >= 0x80 && b <= 0xBF;
}

// Decode-with-replacement: every malformed sequence becomes U+FFFD (the
// "replacement character", bytes EF BF BD), exactly as Python's
// errors="replace" does: an invalid lead byte is replaced on its own, and a
// truncated sequence is replaced as one unit.
inline std::string utf8_replace_invalid(std::string_view bytes) {
    static const char kReplacement[] = "\xEF\xBF\xBD";
    std::string out;
    out.reserve(bytes.size());
    std::size_t i = 0;
    while (i < bytes.size()) {
        std::uint8_t lead = static_cast<std::uint8_t>(bytes[i]);
        std::int32_t length = utf8_sequence_length(lead);
        if (length == 0) {
            out += kReplacement;
            ++i;
            continue;
        }
        std::int32_t have = 1;
        while (have < length && i + have < bytes.size() &&
               utf8_continuation_ok(lead, have, static_cast<std::uint8_t>(bytes[i + have]))) {
            ++have;
        }
        if (have == length) {
            out.append(bytes.substr(i, length));
        } else {
            out += kReplacement;
        }
        i += have;
    }
    return out;
}

// Split (already valid) UTF-8 into code points. Used wherever Python does
// per-character work: checksums (`ord(ch)`) and armour decoding.
inline std::vector<char32_t> codepoints(std::string_view s) {
    std::vector<char32_t> out;
    std::size_t i = 0;
    while (i < s.size()) {
        std::uint8_t lead = static_cast<std::uint8_t>(s[i]);
        std::int32_t length = utf8_sequence_length(lead);
        if (length <= 1 || i + length > s.size()) {   // ASCII, or malformed: 1 byte
            out.push_back(lead);
            ++i;
            continue;
        }
        char32_t cp = lead & (0xFF >> (length + 1));
        for (std::int32_t k = 1; k < length; ++k) {
            cp = (cp << 6) | (static_cast<std::uint8_t>(s[i + k]) & 0x3F);
        }
        out.push_back(cp);
        i += length;
    }
    return out;
}

// Encode one code point as UTF-8 (needed to print a bad character in an
// error message).
inline std::string utf8_encode(char32_t cp) {
    std::string out;
    if (cp < 0x80) {
        out.push_back(static_cast<char>(cp));
    } else if (cp < 0x800) {
        out.push_back(static_cast<char>(0xC0 | (cp >> 6)));
        out.push_back(static_cast<char>(0x80 | (cp & 0x3F)));
    } else if (cp < 0x10000) {
        out.push_back(static_cast<char>(0xE0 | (cp >> 12)));
        out.push_back(static_cast<char>(0x80 | ((cp >> 6) & 0x3F)));
        out.push_back(static_cast<char>(0x80 | (cp & 0x3F)));
    } else {
        out.push_back(static_cast<char>(0xF0 | (cp >> 18)));
        out.push_back(static_cast<char>(0x80 | ((cp >> 12) & 0x3F)));
        out.push_back(static_cast<char>(0x80 | ((cp >> 6) & 0x3F)));
        out.push_back(static_cast<char>(0x80 | (cp & 0x3F)));
    }
    return out;
}

}  // namespace aistut::py
