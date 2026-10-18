// msg_26.hpp — message 26: multiple slot binary message with communications
// state (chapter 16).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.24, Table 82 (layout) and Table 83
// (maximum data bits per flag combination and slot count); Annex 2
// §3.3.7.2.2 Tables 18/19 (SOTDMA) and §3.3.7.3.2 Table 20 (ITDMA) for the
// communication state, decoded by the same functions as messages 1–3.
//
// Bit layout (zero-based offsets):
//
//      0- 5   6  message id (26)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI)
//     38      1  destination indicator: 0 = broadcast, 1 = addressed
//     39      1  binary data flag: 0 = unstructured, 1 = application identifier follows
//     40-69  30  destination MMSI        (only if destination indicator = 1)
//     70-71   2  spare                   (only if destination indicator = 1)
//     next   16  application identifier  (only if binary data flag = 1): DAC 10 + FI 6
//     next    n  application data (Table 83: 104 … 1 000 bits)
//     next    4  spare "needed for byte alignment" (Table 82)
//     last 20     communication state: 1 selector bit (0 = SOTDMA, 1 = ITDMA)
//                 followed by the 19-bit SOTDMA or ITDMA state
//
// The communication state is found by counting back 20 bits from the end of
// the payload; the data length is "whatever is left" and includes the
// alignment spare, which a receiver cannot tell apart from data.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `constexpr std::size_t COMM_STATE_BITS = 20;` is a compile-time
//   constant with a type — preferred over `#define` because it obeys
//   scoping rules and shows up in the debugger.
// * The header's functions are reused across files just by including
//   them: `decode_binary_header` from msg_25.hpp and `decode_sotdma` /
//   `decode_itdma` from msg_1_2_3.hpp. `inline` on each of them is what
//   lets several translation units include the same definition.

#pragma once

#include <cstddef>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"
#include "msg_1_2_3.hpp"
#include "msg_25.hpp"

namespace aistut {

constexpr std::size_t COMM_STATE_BITS = 20;   // 1 selector bit + 19 bits of SOTDMA/ITDMA state

// Decode a message 26. Throws DecodeError if header or comm state is missing.
inline Json decode_msg_26(const Bits& bits) {
    auto [out, offset] = decode_binary_header(bits, 26);
    // The communication state is the *last* 20 bits, whatever the data length.
    require_bits(26, bits.size(), offset + COMM_STATE_BITS);
    std::size_t comm_start = bits.size() - COMM_STATE_BITS;
    out["data"] = bits.raw(offset, comm_start - offset);
    out["data_bits"] = comm_start - offset;
    std::uint64_t selector = bits.uint(comm_start, 1);
    out["comm_state_selector"] = selector;
    if (selector == 1) {
        out["comm_state"] = decode_itdma(bits, comm_start + 1);
    } else {
        out["comm_state"] = decode_sotdma(bits, comm_start + 1);
    }
    return out;
}

}  // namespace aistut
