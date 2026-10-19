// msg_21.hpp — message 21: aids-to-navigation (AtoN) report (chapter 15).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.19, Table 73 (field layout, 272–360 bits);
// Table 74 (the 32 AtoN types); Figure 41/41bis and §3.3.3 (dimension /
// reference-point encoding, shared with message 5); Table 51 (EPFD types,
// shared with message 4). The eight "AtoN status" bits are only *reserved*
// by ITU; their page-7 layout (RACON / light / health) comes from IALA
// Recommendation R0126 (A-126) ed. 2 §4.8.4 Figure 3.
//
// Bit layout (zero-based offsets; lengths from Table 73):
//
//      0- 5    6  message id (21)
//      6- 7    2  repeat indicator
//      8- 37  30  user id (MMSI; AtoN format 99MIDxxxx per ITU-R M.585)
//     38- 42   5  type of aid to navigation (Table 74); 0 = not available
//     43-162 120  name, 20 six-bit characters; all '@' = not available
//    163       1  position accuracy (1 = high, ≤10 m)
//    164-191  28  longitude, signed, 1/10 000 minute; 181° = n/a
//    192-218  27  latitude,  signed, 1/10 000 minute;  91° = n/a
//    219-227   9  dimension A (reference point to "bow" / true north), m
//    228-236   9  dimension B (to "stern"), m
//    237-242   6  dimension C (to port), m
//    243-248   6  dimension D (to starboard), m
//                 (decoded by decode_dimensions from msg_19.hpp, same keys as message 5)
//    249-252   4  type of EPFD (Table 51); 7 = surveyed is normal for fixed/virtual aids
//    253-258   6  UTC second; 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
//    259       1  off-position indicator (floating AtoN only; valid only if second <= 59)
//    260-267   8  AtoN status — "reserved for the indication of the AtoN status"
//    268       1  RAIM flag
//    269       1  virtual AtoN flag (1 = does not physically exist)
//    270       1  assigned mode flag
//    271       1  spare
//    272-      0..84  name extension: 0–14 more six-bit characters, then
//                 0, 2, 4 or 6 spare bits to reach a byte boundary
//
// Variable length rule: everything after bit 272 is the name extension. We
// decode floor((len - 272) / 6) whole characters (at most 14) and report the
// raw number of extension bits as `name_extension_bits`.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `std::array<const char*, 32>` for the AtoN types: a 5-bit field is
//   always 0–31, so direct indexing is safe and no lookup/fallback is needed.
// * `std::min(a, b)` needs both arguments to have the *same* type.
//   `bits.size()` is a `std::size_t`, so the constant is declared as
//   `std::size_t` too; mixing `int` and `size_t` would not compile.
// * `name + name_extension` on two `std::string`s allocates a new string —
//   the same as Python's `+`, unlike C where you would need `strcat`.
// * `(status >> 3) & 0x3` extracts two bits; on an unsigned value `>>` is a
//   plain logical shift. We keep `status` as `std::uint64_t` (what
//   `bits.uint` returns) to avoid signed/unsigned warnings under -Wextra.

#pragma once

#include <algorithm>
#include <array>
#include <cstdint>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"
#include "msg_19.hpp"  // decode_dimensions

namespace aistut {

// Table 74. Codes 5–19 are listed under "Fixed AtoN", 20–31 under "Floating
// AtoN"; 1–4 are general. Code 4 was "spare" in M.1371-4 and became
// "Emergency Wreck Marking Buoy" in M.1371-5.
inline const std::array<const char*, 32> ATON_TYPES = {
    "not specified (default)",
    "reference point",
    "RACON",
    "fixed structure off-shore",
    "emergency wreck marking buoy",
    "light, without sectors",
    "light, with sectors",
    "leading light front",
    "leading light rear",
    "beacon, cardinal N",
    "beacon, cardinal E",
    "beacon, cardinal S",
    "beacon, cardinal W",
    "beacon, port hand",
    "beacon, starboard hand",
    "beacon, preferred channel port hand",
    "beacon, preferred channel starboard hand",
    "beacon, isolated danger",
    "beacon, safe water",
    "beacon, special mark",
    "cardinal mark N",
    "cardinal mark E",
    "cardinal mark S",
    "cardinal mark W",
    "port hand mark",
    "starboard hand mark",
    "preferred channel port hand",
    "preferred channel starboard hand",
    "isolated danger",
    "safe water",
    "special mark",
    "light vessel / LANBY / rig",
};

// IALA R0126 Figure 3, status page 7 ("111 RR LL H").
inline const std::array<const char*, 4> RACON_STATUS = {
    "no RACON installed",
    "RACON installed but not monitored",
    "RACON operational",
    "RACON error",
};
inline const std::array<const char*, 4> LIGHT_STATUS = {
    "no light or no monitoring",
    "light on",
    "light off",
    "light fail or at reduced range",
};

constexpr std::size_t MSG21_MIN_BITS = 272;            // Table 73: the fixed part (two slots)
constexpr std::size_t MSG21_MAX_EXTENSION_CHARS = 14;  // "up to 14 additional 6-bit-ASCII characters"

inline std::string aton_type_text(std::uint64_t code) { return ATON_TYPES[code]; }

// True for the Table 74 codes that describe a floating aid (a buoy).
// Table 74 heads codes 20–31 "Floating AtoN"; code 4, the emergency wreck
// marking buoy, is listed above that heading but is a buoy as well.
inline bool is_floating_aton(std::uint64_t aton_type) {
    return aton_type == 4 || (20 <= aton_type && aton_type <= 31);
}

// Interpret the 8 AtoN status bits.
//
// ITU only reserves them (Table 73). IALA R0126 §4.8.4 splits them into a
// 3-bit page id (the most significant bits) and 5 page-dependent bits;
// page 0 is "no data" (the all-zero default) and page 7 carries the
// RACON / light / health summary that monitoring systems key on.
inline Json decode_aton_status(std::uint64_t status) {
    std::uint64_t page = status >> 5;
    Json out = Json::Object{{"aton_status_page", page}};
    if (page == 7) {
        std::uint64_t racon = (status >> 3) & 0x3;
        std::uint64_t light = (status >> 1) & 0x3;
        out["racon_status"] = racon;
        out["racon_status_text"] = RACON_STATUS[racon];
        out["light_status"] = light;
        out["light_status_text"] = LIGHT_STATUS[light];
        out["health_alarm"] = (status & 0x1) == 1;
    }
    return out;
}

// Decode a message 21. Throws DecodeError if shorter than 272 bits.
inline Json decode_msg_21(const Bits& bits) {
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), MSG21_MIN_BITS);
    std::uint64_t aton_type = bits.uint(38, 5);
    std::int64_t lon_raw = bits.sint(164, 28);
    std::int64_t lat_raw = bits.sint(192, 27);
    std::uint64_t epfd = bits.uint(249, 4);
    std::uint64_t timestamp = bits.uint(253, 6);
    std::uint64_t status = bits.uint(260, 8);

    // --- the variable-length name extension -------------------------------
    // Table 73: "0, 6, 12, … 84" bits of six-bit characters followed by 0, 2,
    // 4 or 6 spare bits so that the message ends on a byte boundary. Only
    // whole characters are decoded; a partial character is ignored.
    std::size_t extension_bits = bits.size() - MSG21_MIN_BITS;
    std::size_t extension_chars = std::min(extension_bits / 6, MSG21_MAX_EXTENSION_CHARS);
    std::string name = bits.text(43, 120);
    std::string name_extension = bits.text(MSG21_MIN_BITS, extension_chars * 6);

    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"aton_type", aton_type},
        {"aton_type_text", aton_type_text(aton_type)},
        {"name", name},
        {"name_extension", name_extension},
        {"name_extension_bits", extension_bits},
        {"full_name", name + name_extension},
        {"position_accuracy", bits.uint(163, 1)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600000.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600000.0, 6)},
        {"position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140},
        {"epfd", epfd},
        {"epfd_text", epfd_text(static_cast<int>(epfd))},
        {"timestamp", timestamp},
        {"off_position", bits.flag(259)},
        // Table 73 note 1: the flag means something only for a floating aid
        // whose time stamp is a real second (0–59).
        {"off_position_valid", is_floating_aton(aton_type) && timestamp <= 59},
        {"aton_status", status},
        {"aton_status_bits", bits.raw(260, 8)},
        {"raim", bits.flag(268)},
        {"virtual_aton", bits.flag(269)},
        {"assigned_mode", bits.flag(270)},
        {"spare", bits.uint(271, 1)},
    };
    // Bits 219-248: the same 30-bit A/B/C/D block as messages 5, 19 and 24B,
    // with the same keys (dim_bow_m … beam_m) — one definition in msg_19.hpp.
    out.update(decode_dimensions(bits, 219));
    out.update(decode_aton_status(status));
    return out;
}

}  // namespace aistut
