// msg_23.hpp — message 23: group assignment command (chapter 13).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.21, Table 76 (160 bits) and Table 77
// (reporting-interval codes). A base station acting as controlling entity
// selects mobile stations by region, station type and ship type, and
// commands their Tx/Rx mode, reporting interval and a quiet time.
//
// Bit layout (zero-based offsets; lengths from Table 76):
//
//      0- 5   6  message id (23)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI of the base station)
//     38-39   2  spare
//     40-57  18  longitude 1 (north-east corner), 1/10 min
//     58-74  17  latitude 1 (north-east corner), 1/10 min
//     75-92  18  longitude 2 (south-west corner), 1/10 min
//     93-109 17  latitude 2 (south-west corner), 1/10 min
//    110-113  4  station type
//    114-121  8  type of ship and cargo type (Table 53; 0 = all)
//    122-143 22  spare
//    144-145  2  Tx/Rx mode (0 = TxA/TxB RxA/RxB, 1 = TxA RxA/RxB, 2 = TxB RxA/RxB, 3 = reserved)
//    146-149  4  reporting interval (Table 77)
//    150-153  4  quiet time in minutes (0 = none)
//    154-159  6  spare
//
// Table 76 gives no not-available value for the region; we apply the 181°/91°
// sentinels of message 22 (the same 1/10-minute fields) for `region_available`.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * Three `std::array<const char*, N>` tables sized exactly to the field
//   width (16, 4, 16 entries) so that indexing with the raw field value can
//   never go out of range — the same trick as the AtoN types in msg_21.hpp.
// * `ship_type == 0 ? "all types (default)" : ship_type_text(ship_type)` —
//   the two branches of `?:` are a `const char*` and a `std::string`; the
//   compiler converts the literal to `std::string` so the types agree.

#pragma once

#include <array>
#include <cstdint>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

inline constexpr std::int64_t M23_LON_NOT_AVAILABLE = 181 * 600;  // 0x1A838, 1/10-minute units
inline constexpr std::int64_t M23_LAT_NOT_AVAILABLE = 91 * 600;   // 0x0D548

// Table 76.
inline const std::array<const char*, 16> STATION_TYPE = {
    "all types of mobiles (default)",
    "Class A mobile stations only",
    "all types of Class B mobile stations",
    "SAR airborne mobile station",
    "Class B SO mobile stations only",
    "Class B CS shipborne mobile station only",
    "inland waterways",
    "regional use",
    "regional use",
    "regional use",
    "base station coverage area",
    "for future use",
    "for future use",
    "for future use",
    "for future use",
    "for future use",
};

// Table 76.
inline const std::array<const char*, 4> TXRX_MODE_23 = {
    "TxA/TxB, RxA/RxB",
    "TxA, RxA/RxB",
    "TxB, RxA/RxB",
    "reserved for future use",
};

// Table 77.
inline const std::array<const char*, 16> REPORTING_INTERVAL = {
    "as given by the autonomous mode",
    "10 min",
    "6 min",
    "3 min",
    "1 min",
    "30 s",
    "15 s",
    "10 s",
    "5 s",
    "next shorter reporting interval",
    "next longer reporting interval",
    "2 s (not applicable to Class B CS and SO)",
    "reserved for future use",
    "reserved for future use",
    "reserved for future use",
    "reserved for future use",
};

// Decode a message 23. Throws DecodeError below 160 bits.
inline Json decode_msg_23(const Bits& bits) {
    require_bits(23, bits.size(), 160);
    std::int64_t ne_lon = bits.sint(40, 18);
    std::int64_t ne_lat = bits.sint(58, 17);
    std::int64_t sw_lon = bits.sint(75, 18);
    std::int64_t sw_lat = bits.sint(93, 17);
    std::uint64_t station_type = bits.uint(110, 4);
    std::int32_t ship_type = static_cast<std::int32_t>(bits.uint(114, 8));
    std::uint64_t txrx = bits.uint(144, 2);
    std::uint64_t interval = bits.uint(146, 4);
    return Json::Object{
        {"msg_type", bits.uint(0, 6)},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
        {"ne_lon", Json::fixed(static_cast<double>(ne_lon) / 600.0, 6)},
        {"ne_lat", Json::fixed(static_cast<double>(ne_lat) / 600.0, 6)},
        {"sw_lon", Json::fixed(static_cast<double>(sw_lon) / 600.0, 6)},
        {"sw_lat", Json::fixed(static_cast<double>(sw_lat) / 600.0, 6)},
        {"region_available", ne_lon != M23_LON_NOT_AVAILABLE && ne_lat != M23_LAT_NOT_AVAILABLE &&
                                 sw_lon != M23_LON_NOT_AVAILABLE && sw_lat != M23_LAT_NOT_AVAILABLE},
        {"station_type", station_type},
        {"station_type_text", STATION_TYPE[station_type]},
        {"ship_type", ship_type},
        // Table 76 redefines 0 as "all types"; 1–255 follow Table 53.
        {"ship_type_text", ship_type == 0 ? std::string("all types (default)") : ship_type_text(ship_type)},
        {"spare2", bits.uint(122, 22)},
        {"txrx_mode", txrx},
        {"txrx_mode_text", TXRX_MODE_23[txrx]},
        {"reporting_interval", interval},
        {"reporting_interval_text", REPORTING_INTERVAL[interval]},
        {"quiet_time_min", bits.uint(150, 4)},
        {"spare3", bits.uint(154, 6)},
    };
}

}  // namespace aistut
