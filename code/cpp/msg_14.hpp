// msg_14.hpp — message 14: safety related broadcast message.
//
// Spec: ITU-R M.1371-5 Annex 8 §3.12, Table 63 (layout), Table 64 (how
// many characters fit in 1–5 slots), Table 47 (six-bit ASCII). Annex 1
// §2.1.6–2.1.8 prescribe the fixed texts sent by AIS-SART, MOB-AIS and
// EPIRB-AIS devices ("SART ACTIVE", "SART TEST", "MOB ACTIVE", "MOB TEST",
// "EPIRB ACTIVE", "EPIRB TEST"); §3.12 repeats the list.
//
// Bit layout (zero-based offsets; lengths from Table 63):
//
//      0- 5   6  message id (14)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI)
//     38-39   2  spare
//     40-     ≤968  safety related text, six-bit ASCII, 0-161 characters
//
// Length rule: the 40-bit header is required (DecodeError otherwise); the
// text handling is shared with message 12 (decode_text_field in msg_12.hpp).
//
// C++ notes for readers new to the language
// -----------------------------------------
// * A `std::map<std::string, std::string>` keyed by the text plays the role
//   of Python's `BEACON_TEXTS.get(text)`. `find` returns an iterator; comparing
//   it with `end()` is the "key not present" test, and we then store a
//   `std::optional<std::string>` which `Json` prints as null when empty.

#pragma once

#include <cstddef>
#include <map>
#include <optional>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"
#include "msg_12.hpp"

namespace aistut {

// The six texts that distress beacons are required to send (Annex 1
// §2.1.6-2.1.8). A watch-keeping application matches on these.
inline const std::map<std::string, std::string> BEACON_TEXTS = {
    {"SART ACTIVE", "AIS-SART active"},
    {"SART TEST", "AIS-SART test"},
    {"MOB ACTIVE", "MOB-AIS active"},
    {"MOB TEST", "MOB-AIS test"},
    {"EPIRB ACTIVE", "EPIRB-AIS active"},
    {"EPIRB TEST", "EPIRB-AIS test"},
};

// Decode a message 14. Throws DecodeError below 40 bits.
inline Json decode_msg_14(const Bits& bits) {
    constexpr std::size_t HEADER_BITS = 40;
    require_bits(14, bits.size(), HEADER_BITS);
    Json out = Json::Object{
        {"msg_type", bits.uint(0, 6)},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
    };
    out.update(decode_text_field(bits, HEADER_BITS));
    // Python read the text back out of the dict (`out["text"]`); our Json
    // class has no string accessor, so we decode the (short) field again.
    std::string s = bits.text(HEADER_BITS, bits.size() - HEADER_BITS);
    std::optional<std::string> beacon;
    auto it = BEACON_TEXTS.find(s);
    if (it != BEACON_TEXTS.end()) beacon = it->second;
    out["beacon_text"] = beacon;
    return out;
}

}  // namespace aistut
