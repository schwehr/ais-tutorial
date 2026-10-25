// msg_1_2_3.hpp — Class A position reports, messages 1, 2 and 3 (chapter 3).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.1, Table 48 (field layout, 168 bits);
// Table 49 (which communication state each message carries); Annex 2
// §3.3.7.2.2 Table 18/19 (SOTDMA communication state) and §3.3.7.3.2
// Table 20 (ITDMA communication state).
//
// Bit layout (zero-based offsets; lengths from Table 48):
//
//      0- 5   6  message id (1, 2 or 3)
//      6- 7   2  repeat indicator
//      8-37  30  user id (MMSI)
//     38-41   4  navigational status
//     42-49   8  rate of turn, signed, special encoding
//     50-59  10  speed over ground, 1/10 knot; 1023 = n/a, 1022 = ≥102.2 kn
//     60      1  position accuracy (1 = high, ≤10 m)
//     61-88  28  longitude, signed, 1/10 000 minute; 181° = n/a
//     89-115 27  latitude,  signed, 1/10 000 minute;  91° = n/a
//    116-127 12  course over ground, 1/10 degree; 3600 = n/a
//    128-136  9  true heading, degrees; 511 = n/a
//    137-142  6  time stamp (UTC second); 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
//    143-144  2  special manoeuvre indicator
//    145-147  3  spare
//    148      1  RAIM flag
//    149-167 19  communication state (SOTDMA for 1 and 2, ITDMA for 3)
//
// C++ notes for readers new to the language
// -----------------------------------------
// * The enumeration texts are `std::array<const char*, N>` indexed directly
//   by the field value. That is safe here because a 4-bit field can only be
//   0–15 and the array has 16 entries; for sparse tables we use std::map.
// * Dividing two integers in C++ gives an integer (`7 / 2 == 3`). Writing
//   `sog_raw / 10.0` with a floating-point literal forces a floating-point
//   division, as Python's `/` always does.
// * `std::pow(x, 2.0)` rather than `x * x` so that the result is bit-for-bit
//   what Python's `x ** 2` produces (both call the C library's pow()).
// * `0x6791AC0` is a hexadecimal literal; the compiler does not care which
//   base you use, but the spec gives these sentinels in hex.

#pragma once

#include <array>
#include <cmath>
#include <cstdint>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

inline const std::array<const char*, 16> NAV_STATUS = {
    "under way using engine",
    "at anchor",
    "not under command",
    "restricted manoeuvrability",
    "constrained by her draught",
    "moored",
    "aground",
    "engaged in fishing",
    "under way sailing",
    "reserved for HSC",
    "reserved for WIG",
    "power-driven vessel towing astern",
    "power-driven vessel pushing ahead or towing alongside",
    "reserved for future use",
    "AIS-SART / MOB-AIS / EPIRB-AIS active",
    "undefined (default)",
};

inline const std::array<const char*, 4> SYNC_STATE = {
    "UTC direct",
    "UTC indirect",
    "synchronized to base station",
    "synchronized to another station",
};

inline const std::array<const char*, 4> SPECIAL_MANOEUVRE = {
    "not available", "not engaged", "engaged", "reserved",
};

// Interpret the 8-bit rate-of-turn field (Table 48).
//
// The sensor value is compressed with a square root:
//     ROT_AIS = 4.733 * sqrt(ROT_sensor)   (sign preserved)
// so to recover degrees per minute we square: (raw / 4.733)**2.
inline Json decode_rot(std::int64_t raw) {
    Json out = Json::Object{{"rot_raw", raw}};
    if (raw == -128) {
        out["rot_deg_per_min"] = nullptr;
        out["rot_text"] = "not available";
    } else if (raw == 127) {
        out["rot_deg_per_min"] = nullptr;
        out["rot_text"] = "turning right > 5 deg per 30 s (no turn indicator)";
    } else if (raw == -127) {
        out["rot_deg_per_min"] = nullptr;
        out["rot_text"] = "turning left > 5 deg per 30 s (no turn indicator)";
    } else {
        double value = std::pow(static_cast<double>(raw) / 4.733, 2.0);
        if (raw < 0) value = -value;
        out["rot_deg_per_min"] = Json::fixed(value, 1);
        out["rot_text"] = raw > 0 ? "turning right" : raw < 0 ? "turning left" : "not turning";
    }
    return out;
}

// SOTDMA communication state, 19 bits (Annex 2 Table 18 and 19).
inline Json decode_sotdma(const Bits& bits, std::size_t start) {
    std::uint64_t sync = bits.uint(start, 2);
    std::uint64_t timeout = bits.uint(start + 2, 3);
    std::uint64_t sub = bits.uint(start + 5, 14);
    Json out = Json::Object{
        {"scheme", "SOTDMA"},
        {"sync_state", sync},
        {"sync_state_text", SYNC_STATE[sync]},
        {"slot_timeout", timeout},
    };
    // The meaning of the 14-bit sub-message depends on the slot time-out.
    if (timeout == 3 || timeout == 5 || timeout == 7) {
        out["received_stations"] = sub;
    } else if (timeout == 2 || timeout == 4 || timeout == 6) {
        out["slot_number"] = sub;
    } else if (timeout == 1) {
        out["utc_hour"] = sub >> 9;               // bits 13..9
        out["utc_minute"] = (sub >> 2) & 0x7F;    // bits 8..2 (7 bits)
    } else {  // timeout == 0
        out["slot_offset"] = sub;
    }
    return out;
}

// ITDMA communication state, 19 bits (Annex 2 Table 20).
inline Json decode_itdma(const Bits& bits, std::size_t start) {
    std::uint64_t sync = bits.uint(start, 2);
    std::uint64_t increment = bits.uint(start + 2, 13);
    std::uint64_t nslots = bits.uint(start + 15, 3);
    bool keep = bits.flag(start + 18);
    Json out = Json::Object{
        {"scheme", "ITDMA"},
        {"sync_state", sync},
        {"sync_state_text", SYNC_STATE[sync]},
        {"slot_increment", increment},
        {"slots_raw", nslots},
        {"keep_flag", keep},
    };
    // Table 20: 0..4 mean 1..5 slots; 5..7 mean 1..3 slots with +8192 offset.
    if (nslots <= 4) {
        out["slots"] = nslots + 1;
        out["slot_offset"] = increment;
    } else {
        out["slots"] = nslots - 4;
        out["slot_offset"] = increment + 8192;
    }
    return out;
}

// Decode a message 1, 2 or 3. Throws DecodeError if too short.
inline Json decode_msg_1_2_3(const Bits& bits) {
    std::int32_t msg_type = static_cast<std::int32_t>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 168);
    std::uint64_t nav = bits.uint(38, 4);
    std::uint64_t sog_raw = bits.uint(50, 10);
    std::int64_t lon_raw = bits.sint(61, 28);
    std::int64_t lat_raw = bits.sint(89, 27);
    std::uint64_t cog_raw = bits.uint(116, 12);
    std::uint64_t heading = bits.uint(128, 9);
    std::uint64_t timestamp = bits.uint(137, 6);
    std::uint64_t manoeuvre = bits.uint(143, 2);

    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"nav_status", nav},
        {"nav_status_text", NAV_STATUS[nav]},
        {"sog_knots", Json::fixed(static_cast<double>(sog_raw) / 10.0, 1)},
        {"sog_available", sog_raw != 1023},
        {"position_accuracy", bits.uint(60, 1)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600000.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600000.0, 6)},
        {"position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140},
        {"cog_deg", Json::fixed(static_cast<double>(cog_raw) / 10.0, 1)},
        {"cog_available", cog_raw != 3600},
        {"true_heading", heading},
        {"heading_available", heading != 511},
        {"timestamp", timestamp},
        {"special_manoeuvre", manoeuvre},
        {"special_manoeuvre_text", SPECIAL_MANOEUVRE[manoeuvre]},
        {"spare", bits.uint(145, 3)},
        {"raim", bits.flag(148)},
    };
    out.update(decode_rot(bits.sint(42, 8)));
    if (msg_type == 3) {
        out["comm_state"] = decode_itdma(bits, 149);
    } else {
        out["comm_state"] = decode_sotdma(bits, 149);
    }
    return out;
}

}  // namespace aistut
