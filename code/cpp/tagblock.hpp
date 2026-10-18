// tagblock.hpp — NMEA 0183 v4.10 TAG blocks (chapter 4).
//
// A TAG block is metadata that *precedes* a sentence on the same line:
//
//     \s:r003669945,c:1224006993*2B\!AIVDM,1,1,,A,15N4cJ`005Jrek0H@9n`DW5608EP,0*13
//     ^                            ^ ^
//     backslash  parameter list   checksum of the text between the backslashes
//
// Defined in IEC 61162-1 ed. 4 / NMEA 0183 v4.10 §7.1.2 "TAG block".
// Parameters are `code:value` pairs separated by commas:
//
//     c:  UNIX time (seconds, or milliseconds if > 10^10 or configured so)
//     d:  destination identification (≤ 15 chars)
//     g:  sentence grouping "n-total-group_id" for multi-sentence groups
//     n:  line count (an incrementing serial number)
//     r:  relative time
//     s:  source identification (≤ 15 chars) — the receiving station
//     t:  free text (≤ 15 chars)
//
// The checksum is computed exactly like a sentence checksum: XOR of all
// characters between the two backslashes, excluding the `*hh`.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * In C++ source a backslash inside a string literal must be doubled:
//   `"\\"` is the one-character string `\`.
// * `split_tag_block` returns a `std::pair<std::optional<TagBlock>,
//   std::string>`. The caller unpacks it with a *structured binding*
//   (C++17): `auto [tag, rest] = split_tag_block(line);` — the same shape as
//   Python's `tag, rest = split_tag_block(line)`.
// * `std::array<long long, 3>` is a fixed-size array that knows its length
//   and can be returned by value, unlike a C array.
// * Python's `@property` methods become ordinary `const` member functions
//   here; the call needs parentheses: `tag.time()`.

#pragma once

#include <array>
#include <cstdio>
#include <map>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include "jsonout.hpp"
#include "nmea.hpp"
#include "pycompat.hpp"

namespace aistut {

struct TagBlock {
    std::map<std::string, std::string> params;
    bool checksum_ok = false;
    std::string raw;

    // Typed views of the common parameters. Missing -> none.
    std::optional<long long> time() const {
        auto it = params.find("c");
        if (it == params.end() || !py::isdigit(it->second)) return std::nullopt;
        return py::to_int(it->second);
    }

    std::optional<std::string> source() const {
        auto it = params.find("s");
        if (it == params.end()) return std::nullopt;
        return it->second;
    }

    // (sentence number, total sentences, group id) from the g: parameter.
    std::optional<std::array<long long, 3>> group() const {
        auto it = params.find("g");
        if (it == params.end() || it->second.empty()) return std::nullopt;
        std::vector<std::string> parts = py::split(it->second, '-');
        if (parts.size() != 3) return std::nullopt;
        std::array<long long, 3> out{};
        for (std::size_t i = 0; i < 3; ++i) {
            if (!py::isdigit(parts[i])) return std::nullopt;
            std::optional<long long> v = py::to_int(parts[i]);
            if (!v) return std::nullopt;
            out[i] = *v;
        }
        return out;
    }

    Json to_dict() const {
        Json::Object p;
        for (const auto& [code, value] : params) p[code] = value;
        Json d = Json::Object{{"params", p}, {"checksum_ok", checksum_ok}};
        if (auto t = time()) d["time"] = *t;
        if (auto g = group()) d["group"] = Json::Array{(*g)[0], (*g)[1], (*g)[2]};
        return d;
    }
};

// Separate a leading TAG block from the sentence that follows it.
//
// Returns (TagBlock or none, remainder). Lines without a leading backslash
// are returned unchanged. Throws std::invalid_argument("unterminated TAG
// block") if the closing backslash is missing.
inline std::pair<std::optional<TagBlock>, std::string> split_tag_block(std::string_view input) {
    std::string line = py::strip(input);
    if (!py::startswith(line, "\\")) return {std::nullopt, line};
    std::size_t end = line.find('\\', 1);
    if (end == std::string::npos) throw std::invalid_argument("unterminated TAG block");
    std::string inner = line.substr(1, end - 1);
    py::Partition at_star = py::partition(inner, '*');
    bool ok = false;
    if (at_star.found && at_star.tail.size() >= 2) {
        std::optional<long long> given = py::to_int(py::slice(at_star.tail, 0, 2), 16);
        ok = given.has_value() && *given == checksum(at_star.head);
    }
    TagBlock tag;
    for (const std::string& item : py::split(at_star.head, ',')) {
        py::Partition kv = py::partition(item, ':');
        if (kv.found) tag.params[kv.head] = kv.tail;
    }
    tag.checksum_ok = ok;
    tag.raw = inner;
    return {tag, line.substr(end + 1)};
}

// Build a TAG block string from parameters (used in chapter 4 to convert
// USCG-format lines into TAG block lines). Takes a vector of pairs rather
// than a map so that the caller controls the parameter order.
inline std::string format_tag_block(const std::vector<std::pair<std::string, std::string>>& params) {
    std::string body;
    for (const auto& [code, value] : params) {
        if (!body.empty()) body += ",";
        body += code + ":" + value;
    }
    char hex[16];
    std::snprintf(hex, sizeof hex, "%02X", checksum(body));
    return "\\" + body + "*" + hex + "\\";
}

}  // namespace aistut
