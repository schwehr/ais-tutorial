// msg_20.hpp — message 20: data link management message (chapter 13).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.18, Table 72. A base station announces the
// slots it has reserved for itself (FATDMA — fixed access TDMA) so that
// mobile stations within 120 NM keep out of them (Annex 2 §3.3.1.2).
//
// Bit layout (zero-based offsets; lengths from Table 72):
//
//      0- 5   6  message id (20)
//      6- 7   2  repeat indicator
//      8-37  30  source station id (MMSI of the base station)
//     38-39   2  spare
//     40-69  30  reservation 1:  offset number (12), number of slots (4),
//                                time-out in minutes (3), increment (11)
//     70-99  30  reservation 2 (optional)
//    100-129 30  reservation 3 (optional)
//    130-159 30  reservation 4 (optional)
//    then 0, 2, 4 or 6 spare bits to reach a byte boundary; 72–160 bits total.
//
// Length rule: at least 72 bits (else DecodeError). Reservation groups are
// read while 30 bits remain (at most four); whatever is left over is
// reported as `spare2` with its width in `spare2_bits` (null if wider than
// 32 bits — that is a broken message, not a spare field).
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `Json::Array` is `std::vector<Json>`; `push_back` appends. The vector
//   is then moved into the object with `Json(std::move(reservations))`.
// * A `while` loop with two conditions replaces Python's
//   `while len(...) < 4 and bits.has(...)` one for one.

#pragma once

#include <cstdint>
#include <utility>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

inline constexpr std::size_t RESERVATION_BITS = 30;
inline constexpr std::size_t MAX_RESERVATIONS = 4;

// One offset/slots/time-out/increment group (Table 72).
inline Json decode_reservation(const Bits& bits, std::size_t start) {
    std::uint64_t offset = bits.uint(start, 12);
    std::uint64_t slots = bits.uint(start + 12, 4);
    std::uint64_t timeout = bits.uint(start + 16, 3);
    std::uint64_t increment = bits.uint(start + 19, 11);
    return Json::Object{
        {"offset", offset},
        {"slots", slots},
        {"timeout_min", timeout},
        {"increment", increment},
        // Table 72 footnote: an interrogated base station with nothing to
        // announce sends one group with offset, slots and time-out all zero.
        {"available", !(offset == 0 && slots == 0 && timeout == 0)},
    };
}

// Decode a message 20. Throws DecodeError below 72 bits.
inline Json decode_msg_20(const Bits& bits) {
    require_bits(20, bits.size(), 72);
    Json::Array reservations;
    std::size_t pos = 40;
    while (reservations.size() < MAX_RESERVATIONS && bits.has(pos, RESERVATION_BITS)) {
        reservations.push_back(decode_reservation(bits, pos));
        pos += RESERVATION_BITS;
    }
    std::size_t spare_bits = bits.size() - pos;
    Json spare2 = nullptr;
    if (spare_bits == 0) {
        spare2 = 0;
    } else if (spare_bits <= 32) {
        spare2 = bits.uint(pos, spare_bits);
    }
    return Json::Object{
        {"msg_type", bits.uint(0, 6)},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
        {"reservations", Json(std::move(reservations))},
        {"spare2", spare2},
        {"spare2_bits", spare_bits},
    };
}

}  // namespace aistut
