// msg_10.hpp — message 10: UTC and date inquiry.
//
// Spec: ITU-R M.1371-5 Annex 8 §3.8, Table 60 (72 bits). The answer is a
// message 11 (same layout as message 4, chapter 6), which "is only
// transmitted as a result of a UTC request message (Message 10)" and goes
// out "on the channel where the UTC request message was received"
// [Annex 8 §3.2].
//
// Bit layout (zero-based offsets; lengths from Table 60):
//
//      0- 5   6  message id (10)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI of the station that wants to know the time)
//     38-39   2  spare
//     40-69  30  destination id (MMSI of the station being asked)
//     70-71   2  spare
//
// C++ notes for readers new to the language
// -----------------------------------------
// * Nothing new: the whole decoder is one braced-init-list that builds a
//   `Json::Object`. The function is `inline` because it lives in a header
//   that several translation units may include.

#pragma once

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

// Decode a message 10. Throws DecodeError if shorter than 72 bits.
inline Json decode_msg_10(const Bits& bits) {
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 72);
    return Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
        {"dest_mmsi", bits.uint(40, 30)},
        {"spare2", bits.uint(70, 2)},
    };
}

}  // namespace aistut
