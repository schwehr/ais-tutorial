// msg_25.hpp — message 25: single slot binary message (chapter 16).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.23, Table 80 (layout) and Table 81
// (maximum data bits per flag combination); Annex 5 §1.1 for the 16-bit
// application identifier (10-bit DAC + 6-bit FI).
//
// Bit layout (zero-based offsets). Two flags decide which fields exist, so
// the offsets after bit 39 are *variable*:
//
//      0- 5   6  message id (25)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI)
//     38      1  destination indicator: 0 = broadcast, 1 = addressed
//     39      1  binary data flag: 0 = unstructured, 1 = application identifier follows
//     40-69  30  destination MMSI        (only if destination indicator = 1)
//     70-71   2  spare                   (only if destination indicator = 1)
//     next   16  application identifier  (only if binary data flag = 1): DAC 10 + FI 6
//     rest    n  application data, to the end of the payload (Table 81: ≤ 128/112/96/80)
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `decode_binary_header` must hand back two things: the partly filled
//   object and the offset where the data starts. Python returned a tuple;
//   C++ returns a `std::pair<Json, std::size_t>` and the caller unpacks it
//   with a structured binding: `auto [out, offset] = ...`.
// * `Json out = Json::Object{{"k", v}, ...}` builds a map from a braced
//   list of pairs; `nullptr` becomes JSON `null`.
// * `bits.size() - offset` is unsigned arithmetic on `std::size_t`. It is
//   safe here only because `require_bits` has already guaranteed that
//   `offset <= bits.size()`; an unchecked subtraction could wrap around to
//   a huge number instead of going negative.

#pragma once

#include <cstddef>
#include <utility>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

// The part of messages 25 and 26 that precedes the application data.
// Returns the decoded fields and the offset of the first data bit. Throws
// DecodeError if the optional fields announced by the flags are missing.
inline std::pair<Json, std::size_t> decode_binary_header(const Bits& bits, int msg_type) {
    require_bits(msg_type, bits.size(), 40);
    bool addressed = bits.flag(38);
    bool structured = bits.flag(39);
    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"addressed", addressed},      // destination indicator, bit 38
        {"structured", structured},    // binary data flag, bit 39
        {"dest_mmsi", nullptr},
        {"dac", nullptr},
        {"fi", nullptr},
    };
    std::size_t offset = 40;
    if (addressed) {
        require_bits(msg_type, bits.size(), offset + 32);
        out["dest_mmsi"] = bits.uint(offset, 30);
        out["spare"] = bits.uint(offset + 30, 2);
        offset += 32;
    }
    if (structured) {
        require_bits(msg_type, bits.size(), offset + 16);
        out["dac"] = bits.uint(offset, 10);
        out["fi"] = bits.uint(offset + 10, 6);
        offset += 16;
    }
    out["data_offset"] = offset;
    return {out, offset};
}

// Decode a message 25. Throws DecodeError if a required part is missing.
inline Json decode_msg_25(const Bits& bits) {
    auto [out, offset] = decode_binary_header(bits, 25);
    // Everything after the header is application data; we cannot know its
    // internal structure, so we hand it on as a bit string (CONVENTIONS §3).
    out["data"] = bits.raw(offset, bits.size() - offset);
    out["data_bits"] = bits.size() - offset;
    return out;
}

}  // namespace aistut
