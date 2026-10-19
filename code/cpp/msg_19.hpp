// msg_19.hpp — message 19: extended Class B equipment position report (chapter 14).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.17, Table 71 (312 bits, two slots; Table
// 69 in M.1371-4). Dimension/reference-point field: §3.3.3 Fig. 41. Ship
// type: §3.3.2 Table 53. EPFD: Table 51. DTE: §3.3.1.
//
// M.1371-5 says this message "is not needed and should not be used" by new
// equipment — a Class B unit should send message 18 plus message 24 A/B
// instead — but legacy units still send it once every 6 minutes.
//
// Bit layout (zero-based offsets; lengths from Table 71). Bits 0-138 are
// laid out exactly as in message 18, so we reuse that header's helper.
//
//      0- 5   6  message id (19)
//      6- 7   2  repeat indicator
//      8-37  30  user id (MMSI)
//     38-45   8  spare
//     46-55  10  speed over ground, 1/10 knot; 1023 = n/a
//     56      1  position accuracy
//     57-84  28  longitude, signed, 1/10 000 minute; 181° = n/a
//     85-111 27  latitude,  signed, 1/10 000 minute;  91° = n/a
//    112-123 12  course over ground, 1/10 degree; 3600 = n/a
//    124-132  9  true heading; 511 = n/a
//    133-138  6  time stamp (UTC second); 60 = n/a
//    139-142  4  spare
//    143-262 120 name, 20 six-bit characters; all '@' = n/a
//    263-270  8  type of ship and cargo type; 0 = n/a
//    271-300 30  dimensions / reference point: A 9, B 9, C 6, D 6 (Fig. 41)
//    301-304  4  type of EPFD; 0 = undefined
//    305      1  RAIM flag
//    306      1  DTE: 0 = data terminal available, 1 = not available (default)
//    307      1  assigned mode flag
//    308-311  4  spare
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `decode_dimensions` is shared with message 24 part B via `#include
//   "msg_19.hpp"`. Because every function here is `inline`, including the
//   header from two places does not produce duplicate definitions.

#pragma once

#include <array>
#include <cstdint>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"
#include "msg_18.hpp"

namespace aistut {

inline const std::array<const char*, 2> DTE = {
    "data terminal available", "data terminal not available"};

// The 30-bit dimension / reference-point field (M.1371-5 §3.3.3 Fig. 41).
//
// A = distance from the reference point (the GNSS antenna) to the bow,
// B = to the stern, C = to the port side, D = to the starboard side, all in
// metres. A and B saturate at 511, C and D at 63. All four zero means
// "nothing available"; A = C = 0 with B and D non-zero means "dimensions
// known but reference point not".
inline Json decode_dimensions(const Bits& bits, std::size_t start) {
    std::uint64_t a = bits.uint(start, 9);
    std::uint64_t b = bits.uint(start + 9, 9);
    std::uint64_t c = bits.uint(start + 18, 6);
    std::uint64_t d = bits.uint(start + 24, 6);
    return Json::Object{
        {"dim_to_bow_m", a},
        {"dim_to_stern_m", b},
        {"dim_to_port_m", c},
        {"dim_to_starboard_m", d},
        {"dimensions_available", !(a == 0 && b == 0 && c == 0 && d == 0)},
        {"reference_point_available", !(a == 0 && c == 0 && b != 0 && d != 0)},
    };
}

// Decode a message 19. Throws DecodeError if shorter than 312 bits.
inline Json decode_msg_19(const Bits& bits) {
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 312);
    std::uint64_t ship_type = bits.uint(263, 8);
    std::uint64_t epfd = bits.uint(301, 4);
    std::uint64_t dte = bits.uint(306, 1);
    std::uint64_t assigned = bits.uint(307, 1);

    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 8)},
    };
    out.update(decode_common_position(bits, 46));
    out.update(Json::Object{
        {"spare2", bits.uint(139, 4)},
        {"name", bits.text(143, 120)},
        {"ship_type", ship_type},
        {"ship_type_text", ship_type_text(static_cast<int>(ship_type))},
    });
    out.update(decode_dimensions(bits, 271));
    out.update(Json::Object{
        {"epfd", epfd},
        {"epfd_text", epfd_text(static_cast<int>(epfd))},
        {"raim", bits.flag(305)},
        {"dte", dte},
        {"dte_text", DTE[dte]},
        {"assigned_mode", assigned},
        {"assigned_mode_text", ASSIGNED_MODE[assigned]},
        {"spare3", bits.uint(308, 4)},
    });
    return out;
}

}  // namespace aistut
