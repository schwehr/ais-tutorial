// msg_18.hpp — message 18: standard Class B equipment position report (chapter 14).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.16, Table 70 (168 bits; Table 67 in
// M.1371-3 and -4). Communication states: Annex 2 §3.3.7.2.2 Table 18/19 (SOTDMA)
// and §3.3.7.3.2 Table 20 (ITDMA). Class B "CS" behaviour: Annex 7.
//
// Bit layout (zero-based offsets; lengths from Table 70):
//
//      0- 5   6  message id (18)
//      6- 7   2  repeat indicator (should be 0 for "CS" units)
//      8-37  30  user id (MMSI)
//     38-45   8  spare ("regional reserved" in older editions); should be 0
//     46-55  10  speed over ground, 1/10 knot; 1023 = n/a, 1022 = ≥102.2 kn
//     56      1  position accuracy (1 = high, ≤10 m)
//     57-84  28  longitude, signed, 1/10 000 minute; 181° = n/a
//     85-111 27  latitude,  signed, 1/10 000 minute;  91° = n/a
//    112-123 12  course over ground, 1/10 degree; 3600 = n/a
//    124-132  9  true heading, degrees; 511 = n/a
//    133-138  6  time stamp (UTC second); 60 = n/a; 61-63 not used by "CS"
//    139-140  2  spare (regional reserved in older editions); should be 0
//    141      1  Class B unit flag: 0 = SOTDMA ("SO") unit, 1 = "CS" unit
//    142      1  Class B display flag: 1 = can display messages 12 and 14
//    143      1  Class B DSC flag: 1 = has DSC function
//    144      1  Class B band flag: 1 = whole marine band, 0 = upper 525 kHz only
//    145      1  Class B message 22 flag: 1 = accepts channel management
//    146      1  assigned mode flag: 1 = assigned mode
//    147      1  RAIM flag
//    148      1  communication state selector: 0 = SOTDMA follows, 1 = ITDMA follows
//    149-167 19  communication state; a "CS" unit fills it with 1100000000000000110
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `decode_common_position` returns a `Json` object that the caller merges
//   with `out.update(...)`, exactly like the Python dict it mirrors. The
//   message 19 header includes this file to reuse it — header-only modules
//   can depend on each other with plain `#include`.
// * `bits.raw(149, 19) == CS_FIXED_COMM_STATE` compares a `std::string`
//   with a string literal; `operator==` is overloaded for that, so no
//   `strcmp` is needed.

#pragma once

#include <array>
#include <cstdint>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"
#include "msg_1_2_3.hpp"

namespace aistut {

// Table 70: the 19-bit value every Class B "CS" unit puts in its
// communication state field.
inline const char* CS_FIXED_COMM_STATE = "1100000000000000110";

inline const std::array<const char*, 2> CLASS_B_UNIT = {
    "Class B SOTDMA unit", "Class B CS unit"};
inline const std::array<const char*, 2> CLASS_B_DISPLAY = {
    "no display for messages 12 and 14", "integrated display for messages 12 and 14"};
inline const std::array<const char*, 2> CLASS_B_DSC = {
    "no DSC function", "DSC function (dedicated or time-shared)"};
inline const std::array<const char*, 2> CLASS_B_BAND = {
    "upper 525 kHz of the marine band only", "whole marine band"};
inline const std::array<const char*, 2> CLASS_B_MSG22 = {
    "AIS 1 and AIS 2 only, no message 22 frequency management",
    "frequency management via message 22"};
inline const std::array<const char*, 2> ASSIGNED_MODE = {
    "autonomous and continuous", "assigned"};

// SOG…time stamp as laid out in messages 18 and 19.
//
// Both messages share the block that starts with the 10-bit SOG at `start`
// (46 in both cases) and ends with the 6-bit time stamp, so the same code
// reads both. Offsets below are relative to `start`.
inline Json decode_common_position(const Bits& bits, std::size_t start) {
    std::uint64_t sog_raw = bits.uint(start, 10);
    std::int64_t lon_raw = bits.sint(start + 11, 28);
    std::int64_t lat_raw = bits.sint(start + 39, 27);
    std::uint64_t cog_raw = bits.uint(start + 66, 12);
    std::uint64_t heading = bits.uint(start + 78, 9);
    std::uint64_t timestamp = bits.uint(start + 87, 6);
    return Json::Object{
        {"sog_knots", Json::fixed(static_cast<double>(sog_raw) / 10.0, 1)},
        {"sog_available", sog_raw != 1023},
        {"position_accuracy", bits.uint(start + 10, 1)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600000.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600000.0, 6)},
        {"position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140},
        {"cog_deg", Json::fixed(static_cast<double>(cog_raw) / 10.0, 1)},
        {"cog_available", cog_raw != 3600},
        {"true_heading", heading},
        {"heading_available", heading != 511},
        {"timestamp", timestamp},
    };
}

// Decode a message 18. Throws DecodeError if shorter than 168 bits.
inline Json decode_msg_18(const Bits& bits) {
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 168);
    std::uint64_t unit = bits.uint(141, 1);
    std::uint64_t display = bits.uint(142, 1);
    std::uint64_t dsc = bits.uint(143, 1);
    std::uint64_t band = bits.uint(144, 1);
    std::uint64_t msg22 = bits.uint(145, 1);
    std::uint64_t assigned = bits.uint(146, 1);
    std::uint64_t selector = bits.uint(148, 1);

    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 8)},
    };
    out.update(decode_common_position(bits, 46));
    out.update(Json::Object{
        {"spare2", bits.uint(139, 2)},
        {"class_b_unit", unit},
        {"class_b_unit_text", CLASS_B_UNIT[unit]},
        {"class_b_display", display},
        {"class_b_display_text", CLASS_B_DISPLAY[display]},
        {"class_b_dsc", dsc},
        {"class_b_dsc_text", CLASS_B_DSC[dsc]},
        {"class_b_band", band},
        {"class_b_band_text", CLASS_B_BAND[band]},
        {"class_b_msg22", msg22},
        {"class_b_msg22_text", CLASS_B_MSG22[msg22]},
        {"assigned_mode", assigned},
        {"assigned_mode_text", ASSIGNED_MODE[assigned]},
        {"raim", bits.flag(147)},
        {"comm_state_selector", selector},
        {"cs_fixed_comm_state", bits.raw(149, 19) == CS_FIXED_COMM_STATE},
    });
    if (selector == 1) {
        out["comm_state"] = decode_itdma(bits, 149);
    } else {
        out["comm_state"] = decode_sotdma(bits, 149);
    }
    return out;
}

}  // namespace aistut
