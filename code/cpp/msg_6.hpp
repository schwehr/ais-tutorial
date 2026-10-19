// msg_6.hpp — message 6: addressed binary message (chapter 8).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.4, Table 54 (layout), Table 55 (slot
// budget); Annex 2 §5.3.1 (sequence number, retries, retransmit flag and the
// message 7 acknowledgement); Annex 5 §2.1 Table 23 (application identifier).
//
// Bit layout (zero-based offsets; lengths from Table 54):
//
//      0- 5   6  message id (6)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI)
//     38-39   2  sequence number 0-3 (Annex 2 §5.3.1; echoed by message 7)
//     40-69  30  destination id (MMSI)
//     70      1  retransmit flag: 0 = no retransmission (default), 1 = retransmitted
//     71      1  spare
//     72-81  10  designated area code (DAC)          } 16-bit application
//     82-87   6  function identifier (FI)             } identifier, Annex 5 Table 23
//     88-    ≤920 application data
//
// Length rule: at least the 88-bit header is required; otherwise
// DecodeError. The rest is `data`/`data_bits`, plus `application` when
// asm.hpp knows the (dac, fi).
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `if (auto app = decode_asm(...))` declares `app` and tests it in one
//   statement; an `std::optional` is "true" when it holds a value, and `*app`
//   is that value.

#pragma once

#include <cstddef>

#include "asm.hpp"
#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

namespace msg_6 {
constexpr std::size_t HEADER_BITS = 88;   // message id .. function identifier
}  // namespace msg_6

// Decode a message 6. Throws DecodeError below 88 bits.
inline Json decode_msg_6(const Bits& bits) {
    using namespace msg_6;
    require_bits(6, bits.size(), HEADER_BITS);
    int dac = static_cast<int>(bits.uint(72, 10));
    int fi = static_cast<int>(bits.uint(82, 6));
    Json out = Json::Object{
        {"msg_type", 6},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"sequence_number", bits.uint(38, 2)},
        {"dest_mmsi", bits.uint(40, 30)},
        {"retransmit", bits.flag(70)},
        {"spare", bits.uint(71, 1)},
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
