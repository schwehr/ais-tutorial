// msg_7_13.hpp — messages 7 and 13: binary / safety related acknowledgement.
//
// Spec: ITU-R M.1371-5 Annex 8 §3.5 (message 7, and by reference §3.11
// message 13), Table 56. Annex 2 §5.3.1 describes when the acknowledgement
// is sent (after an addressed message 6 or 12) and what the sequence number
// means.
//
// Bit layout (zero-based offsets; lengths from Table 56):
//
//      0- 5   6  message id (7 = binary ack, 13 = safety related ack)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI of the station that acknowledges)
//     38-39   2  spare
//     40-69  30  destination id 1 (MMSI whose message is acknowledged)
//     70-71   2  sequence number of the acknowledged message, 0-3
//     72-101 30  destination id 2       } "should be omitted if no
//    102-103  2  sequence number 2      }  destination ID2"
//    104-135 32  destination id 3 + sequence number 3
//    136-167 32  destination id 4 + sequence number 4
//
// Total 72, 104, 136 or 168 bits. The repeated 32-bit groups are output as
// a JSON array `acks`, one object per acknowledged station.
//
// Length rule: at least 72 bits (one destination) are required; otherwise
// DecodeError. After the first group we take as many complete 32-bit groups
// as are present, up to four. Bits left over after the last complete group
// are counted in `trailing_bits` and never interpreted.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `Json::Array` is `std::vector<Json>`; `push_back` appends, exactly like
//   Python's `list.append`. A `Json::Object` literal inside the push_back
//   call builds the per-station object in place.
// * `constexpr std::size_t` is a compile-time constant with a type; prefer
//   it to `#define` for named numbers.
// * `acks.size()` returns `std::size_t` (unsigned), which is why the loop
//   bound `MAX_GROUPS` is also `std::size_t` — mixing signed and unsigned in
//   a comparison draws a warning under -Wextra.

#pragma once

#include <cstddef>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

namespace msg_7_13 {
constexpr std::size_t HEADER_BITS = 40;   // message id .. spare
constexpr std::size_t GROUP_BITS = 32;    // destination id (30) + sequence number (2)
constexpr std::size_t MAX_GROUPS = 4;
}  // namespace msg_7_13

// Decode a message 7 or 13. Throws DecodeError below 72 bits.
inline Json decode_msg_7_13(const Bits& bits) {
    using namespace msg_7_13;
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), HEADER_BITS + GROUP_BITS);

    Json::Array acks;
    std::size_t pos = HEADER_BITS;
    // Loop over the repeated groups. `bits.has` is the guard that stops us
    // at the end of a 72-, 104-, 136- or 168-bit message.
    while (acks.size() < MAX_GROUPS && bits.has(pos, GROUP_BITS)) {
        acks.push_back(Json::Object{
            {"mmsi", bits.uint(pos, 30)},
            {"sequence_number", bits.uint(pos + 30, 2)},
        });
        pos += GROUP_BITS;
    }

    return Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
        {"acks", acks},
        {"trailing_bits", bits.size() - pos},
    };
}

}  // namespace aistut
