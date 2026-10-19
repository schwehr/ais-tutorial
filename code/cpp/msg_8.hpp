// msg_8.hpp — message 8: binary broadcast message (chapter 8).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.6, Table 57 (layout), Table 58 (slot
// budget); Annex 5 §2.1 Table 23 (application identifier).
//
// Bit layout (zero-based offsets; lengths from Table 57):
//
//      0- 5   6  message id (8)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI)
//     38-39   2  spare
//     40-49  10  designated area code (DAC)          } 16-bit application
//     50-55   6  function identifier (FI)             } identifier, Annex 5 Table 23
//     56-    ≤952 application data
//
// Length rule: at least the 56-bit header is required; otherwise
// DecodeError. The rest is `data`/`data_bits`, plus `application` when
// asm.hpp knows the (dac, fi).

#pragma once

#include <cstddef>

#include "asm.hpp"
#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

namespace msg_8 {
constexpr std::size_t HEADER_BITS = 56;   // message id .. function identifier
}  // namespace msg_8

// Decode a message 8. Throws DecodeError below 56 bits.
inline Json decode_msg_8(const Bits& bits) {
    using namespace msg_8;
    require_bits(8, bits.size(), HEADER_BITS);
    int dac = static_cast<int>(bits.uint(40, 10));
    int fi = static_cast<int>(bits.uint(50, 6));
    Json out = Json::Object{
        {"msg_type", 8},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
        {"dac", dac},
        {"fi", fi},
        {"data", bits.raw(HEADER_BITS, bits.size() - HEADER_BITS)},
        {"data_bits", bits.size() - HEADER_BITS},
    };
    if (auto app = decode_asm(dac, fi, bits, HEADER_BITS)) {
        out["application"] = *app;
    }
    return out;
}

}  // namespace aistut
