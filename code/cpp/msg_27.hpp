// msg_27.hpp — message 27: long-range AIS broadcast message (chapter 16).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.25, Table 84 (96 bits); Annex 4 §3
// (long-range applications by broadcast: channels 75/76, 3-minute interval,
// Table 22 packet structure); Table 51 (message 4 "transmission control for
// long-range broadcast message" bit).
//
// Bit layout (zero-based offsets; lengths from Table 84):
//
//      0- 5   6  message id (27)
//      6- 7   2  repeat indicator ("always 3": do not repeat)
//      8-37  30  user id (MMSI)
//     38      1  position accuracy (as message 1)
//     39      1  RAIM flag (as message 1)
//     40-43   4  navigational status (as message 1)
//     44-61  18  longitude, signed, 1/10 minute; 181° (0x1A838) = n/a
//     62-78  17  latitude,  signed, 1/10 minute;  91° (0xD548)  = n/a
//     79-84   6  speed over ground, whole knots 0–62; 63 = n/a
//     85-93   9  course over ground, whole degrees 0–359; 511 = n/a
//     94      1  position latency: 0 = < 5 s, 1 = > 5 s (default)
//                (M.1371-4 called this bit "status of current GNSS position")
//     95      1  spare
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `constexpr std::int64_t LON_NOT_AVAILABLE = 0x1A838;` — the sentinel is
//   compared against a *signed* value from `bits.sint`, so we give the
//   constant the same signed type to avoid a signed/unsigned comparison
//   warning under -Wextra.
// * `NAV_STATUS` is the `std::array` from msg_1_2_3.hpp; indexing it with
//   a 4-bit value is always in range (0–15).

#pragma once

#include <array>
#include <cstdint>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"
#include "msg_1_2_3.hpp"

namespace aistut {

constexpr std::int64_t LON_NOT_AVAILABLE = 0x1A838;   // 181 degrees * 600
constexpr std::int64_t LAT_NOT_AVAILABLE = 0xD548;    //  91 degrees * 600

inline const std::array<const char*, 2> POSITION_LATENCY = {
    "less than 5 s", "greater than 5 s (default)",
};

// Decode a message 27. Throws DecodeError if shorter than 96 bits.
// Payloads longer than 96 bits are decoded from their first 96 bits; the
// driver's `payload_bits` tells the caller that something was odd.
inline Json decode_msg_27(const Bits& bits) {
    require_bits(27, bits.size(), 96);
    std::uint64_t nav = bits.uint(40, 4);
    std::int64_t lon_raw = bits.sint(44, 18);
    std::int64_t lat_raw = bits.sint(62, 17);
    std::uint64_t sog = bits.uint(79, 6);
    std::uint64_t cog = bits.uint(85, 9);
    std::uint64_t latency = bits.uint(94, 1);
    return Json::Object{
        {"msg_type", 27},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"position_accuracy", bits.uint(38, 1)},
        {"raim", bits.flag(39)},
        {"nav_status", nav},
        {"nav_status_text", NAV_STATUS[nav]},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600.0, 6)},
        {"position_available", lon_raw != LON_NOT_AVAILABLE && lat_raw != LAT_NOT_AVAILABLE},
        {"sog_knots", sog},
        {"sog_available", sog != 63},
        {"cog_deg", cog},
        {"cog_available", cog != 511},
        {"position_latency", latency},
        {"position_latency_text", POSITION_LATENCY[latency]},
        {"spare", bits.uint(95, 1)},
    };
}

}  // namespace aistut
