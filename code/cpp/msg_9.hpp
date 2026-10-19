// msg_9.hpp — message 9: standard search and rescue (SAR) aircraft position report.
//
// Spec: ITU-R M.1371-5 Annex 8 §3.7, Table 59 (168 bits); the 19-bit
// communication state is SOTDMA (Annex 2 §3.3.7.2.2 Tables 18/19) or ITDMA
// (Annex 2 §3.3.7.3.2 Table 20) according to the selector bit, exactly as in
// message 18 (chapter 14), so we reuse `decode_sotdma` / `decode_itdma` from
// msg_1_2_3.hpp.
//
// Bit layout (zero-based offsets; lengths from Table 59):
//
//      0- 5   6  message id (9)
//      6- 7   2  repeat indicator
//      8-37  30  user id (MMSI; SAR aircraft use 111MIDxxx, ITU-R M.585-9 Annex 1 §3)
//     38-49  12  altitude in metres; 4095 = n/a, 4094 = 4094 m or higher
//     50-59  10  speed over ground in whole knots; 1023 = n/a, 1022 = 1022 kn or higher
//     60      1  position accuracy (1 = high, <=10 m)
//     61-88  28  longitude, signed, 1/10 000 minute; 181° (0x6791AC0) = n/a
//     89-115 27  latitude,  signed, 1/10 000 minute;  91° (0x3412140) = n/a
//    116-127 12  course over ground, 1/10 degree; 3600 = n/a
//    128-133  6  time stamp (UTC second); 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
//    134      1  altitude sensor (0 = GNSS, 1 = barometric)
//    135-141  7  spare
//    142      1  DTE (0 = data terminal available, 1 = not available = default)
//    143-145  3  spare
//    146      1  assigned mode flag (0 = autonomous, 1 = assigned)
//    147      1  RAIM flag
//    148      1  communication state selector (0 = SOTDMA, 1 = ITDMA)
//    149-167 19  communication state
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `sog` is stored as `std::uint64_t` and handed to `Json` *without* a
//   `Json::fixed(...)` wrapper, so it prints as an integer (`150`), as in
//   the Python version. Compare message 1-3, where SOG is in tenths and
//   printed with one decimal.
// * The two tiny enumerations are `std::array<const char*, 2>` indexed by
//   the one-bit field — the same trick as NAV_STATUS in msg_1_2_3.hpp.

#pragma once

#include <array>
#include <cstdint>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"
#include "msg_1_2_3.hpp"

namespace aistut {

namespace msg_9 {
constexpr std::uint64_t ALTITUDE_NOT_AVAILABLE = 4095;
constexpr std::uint64_t ALTITUDE_MAX = 4094;   // "4 094 m or higher"
constexpr std::uint64_t SOG_NOT_AVAILABLE = 1023;
constexpr std::uint64_t SOG_MAX = 1022;        // "1 022 knots or higher"

inline const std::array<const char*, 2> ALTITUDE_SENSOR = {"GNSS", "barometric"};
inline const std::array<const char*, 2> ASSIGNED_MODE = {"autonomous and continuous", "assigned"};
}  // namespace msg_9

// Decode a message 9. Throws DecodeError if shorter than 168 bits.
inline Json decode_msg_9(const Bits& bits) {
    using namespace msg_9;
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 168);
    std::uint64_t altitude = bits.uint(38, 12);
    std::uint64_t sog = bits.uint(50, 10);
    std::int64_t lon_raw = bits.sint(61, 28);
    std::int64_t lat_raw = bits.sint(89, 27);
    std::uint64_t cog_raw = bits.uint(116, 12);
    std::uint64_t sensor = bits.uint(134, 1);
    std::uint64_t assigned = bits.uint(146, 1);
    std::uint64_t selector = bits.uint(148, 1);

    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"altitude_m", altitude},
        {"altitude_available", altitude != ALTITUDE_NOT_AVAILABLE},
        {"altitude_at_max", altitude == ALTITUDE_MAX},
        {"sog_knots", sog},   // whole knots: an integer, not Json::fixed
        {"sog_available", sog != SOG_NOT_AVAILABLE},
        {"sog_at_max", sog == SOG_MAX},
        {"position_accuracy", bits.uint(60, 1)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600000.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600000.0, 6)},
        {"position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140},
        {"cog_deg", Json::fixed(static_cast<double>(cog_raw) / 10.0, 1)},
        {"cog_available", cog_raw != 3600},
        {"timestamp", bits.uint(128, 6)},
        {"altitude_sensor", sensor},
        {"altitude_sensor_text", ALTITUDE_SENSOR[sensor]},
        {"spare", bits.uint(135, 7)},
        {"dte", bits.uint(142, 1)},
        {"spare2", bits.uint(143, 3)},
        {"assigned_mode", assigned},
        {"assigned_mode_text", ASSIGNED_MODE[assigned]},
        {"raim", bits.flag(147)},
        {"comm_state_selector", selector},
    };
    if (selector == 1) {
        out["comm_state"] = decode_itdma(bits, 149);
    } else {
        out["comm_state"] = decode_sotdma(bits, 149);
    }
    return out;
}

}  // namespace aistut
