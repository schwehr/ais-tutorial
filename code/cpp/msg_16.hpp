// msg_16.hpp — message 16: assignment mode command.
//
// Spec: ITU-R M.1371-5 Annex 8 §3.14, Table 67 (96 or 144 bits) and the
// note (1) and paragraph below it that give the meaning of offset and
// increment. Annex 2 §3.3.6 describes what the assigned station does.
//
// Bit layout (zero-based offsets; lengths from Table 67):
//
//      0- 5   6  message id (16)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI of the assigning base station)
//     38-39   2  spare
//     40-69  30  destination id A
//     70-81  12  offset A
//     82-91  10  increment A
//     ------- then EITHER
//     92-95   4  spare (padding to a byte boundary)   -> 96 bits
//     ------- OR
//     92-121 30  destination id B
//    122-133 12  offset B
//    134-143 10  increment B                          -> 144 bits
//
// Meaning of (offset, increment) [Table 67 note 1]: increment 0 = reporting
// rate assignment, offset = reports per 10 min; increment 1-6 = slot
// assignment with 1125/375/225/125/75/45 slots between transmissions;
// 7 = undefined (disregard); 8-1023 not defined in M.1371-5.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `std::optional<long long>` is "an integer or nothing"; `Json` has a
//   constructor for it that prints `null` when empty, which is how the
//   Python version's `None` comes out identically.
// * Local variables `reports` and `slots` start empty (`std::nullopt`) and
//   are assigned in exactly one branch each.

#pragma once

#include <cstddef>
#include <cstdint>
#include <map>
#include <optional>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

namespace msg_16 {

// Table 67 text under note (1): increment code -> slots between assigned slots.
inline const std::map<std::uint64_t, std::int64_t> SLOT_INCREMENTS = {
    {1, 1125}, {2, 375}, {3, 225}, {4, 125}, {5, 75}, {6, 45},
};

// Interpret one (destination, offset, increment) triple.
inline Json assignment(std::uint64_t mmsi, std::uint64_t offset, std::uint64_t increment) {
    std::optional<std::int64_t> reports;
    std::optional<std::int64_t> slots;
    std::string text;
    auto it = SLOT_INCREMENTS.find(increment);
    if (increment == 0) {
        text = "reporting rate";
        reports = static_cast<std::int64_t>(offset);
    } else if (it != SLOT_INCREMENTS.end()) {
        text = "slot assignment";
        slots = it->second;
    } else if (increment == 7) {
        text = "undefined increment (disregard)";
    } else {
        text = "increment not defined in M.1371-5";
    }
    return Json::Object{
        {"mmsi", mmsi},
        {"offset", offset},
        {"increment", increment},
        {"assignment_text", text},
        {"reports_per_10_min", reports},
        {"slot_increment", slots},
    };
}

}  // namespace msg_16

// Decode a message 16. Throws DecodeError below 96 bits.
inline Json decode_msg_16(const Bits& bits) {
    using namespace msg_16;
    std::int32_t msg_type = static_cast<std::int32_t>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 96);

    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
    };
    Json::Array assignments;
    assignments.push_back(assignment(bits.uint(40, 30), bits.uint(70, 12), bits.uint(82, 10)));
    std::size_t pos = 92;
    if (bits.has(92, 52)) {
        assignments.push_back(assignment(bits.uint(92, 30), bits.uint(122, 12), bits.uint(134, 10)));
        pos = 144;
    } else {
        out["spare2"] = bits.uint(92, 4);
        pos = 96;
    }
    out["assignments"] = assignments;
    out["trailing_bits"] = bits.size() - pos;
    return out;
}

}  // namespace aistut
