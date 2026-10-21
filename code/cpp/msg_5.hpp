// msg_5.hpp — message 5: ship static and voyage related data (chapter 7).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.3, Table 52 (field layout, 424 bits, two
// slots); §3.3.1 (DTE); §3.3.2 Table 53 (type of ship and cargo type);
// §3.3.3 Figure 41 (reference point and dimensions A/B/C/D); Table 47
// (six-bit ASCII). The EPFD code list is Table 51 (shared via common.hpp).
//
// Bit layout (zero-based offsets; lengths from Table 52):
//
//       0-  5   6  message id (5)
//       6-  7   2  repeat indicator
//       8- 37  30  user id (MMSI)
//      38- 39   2  AIS version indicator (0 = M.1371-1, 1 = -3, 2 = -5, 3 = future)
//      40- 69  30  IMO number; 0 = n/a; 1..999999 not used;
//                  1000000..9999999 IMO number; 10000000.. flag-state number
//      70-111  42  call sign, 7 six-bit characters, "@@@@@@@" = n/a
//     112-231 120  name, 20 six-bit characters, all '@' = n/a
//     232-239   8  type of ship and cargo type (Table 53); 0 = n/a
//     240-248   9  dimension A, metres from reference point to bow;   511 = ≥511 m
//     249-257   9  dimension B, metres to stern;                      511 = ≥511 m
//     258-263   6  dimension C, metres to port side;                   63 = ≥63 m
//     264-269   6  dimension D, metres to starboard side;              63 = ≥63 m
//     270-273   4  type of electronic position fixing device (EPFD)
//     274-277   4  ETA month  1..12;  0 = n/a
//     278-282   5  ETA day    1..31;  0 = n/a
//     283-287   5  ETA hour   0..23; 24 = n/a
//     288-293   6  ETA minute 0..59; 60 = n/a
//     294-301   8  maximum present static draught, 1/10 m; 0 = n/a; 255 = ≥25.5 m
//     302-421 120  destination, 20 six-bit characters, all '@' = n/a
//     422       1  DTE: 0 = data terminal available, 1 = not available (default)
//     423       1  spare
//
// Length rule: we require the full 424 bits and ignore anything after bit
// 423 (426-bit messages from senders that under-count the fill bits decode
// normally). Shorter messages are reported as `message 5 too short`.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `std::to_string(imo)` turns the number into decimal text so that the
//   check-digit loop can look at single digits — the same trick as Python's
//   `str(imo)`. `c - '0'` converts a digit character back to its value.
// * A `std::string` is built with `+` and `+=`; `std::format("{:02}", v)`
//   (C++20) produces the zero-padded two-digit pieces of the ETA text — the
//   same format spec as Python's f"{month:02d}".
// * `static_cast<int>` appears wherever a 64-bit `uint64` from `Bits`
//   meets an `int` parameter: C++ would convert silently, but the explicit
//   cast documents that we know the value fits.

#pragma once

#include <array>
#include <cstdint>
#include <format>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

// Table 52, "AIS version indicator".
inline const std::array<const char*, 4> AIS_VERSION = {
    "ITU-R M.1371-1",
    "ITU-R M.1371-3 or later",
    "ITU-R M.1371-5 or later",
    "future edition",
};

// Table 52, "DTE" (data terminal equipment ready), §3.3.1.
inline const std::array<const char*, 2> DTE_TEXT = {"available", "not available (default)"};

// Which of the four ranges of Table 52 the 30-bit IMO field falls in.
inline std::string imo_range_text(std::uint64_t imo) {
    if (imo == 0) return "not available (default)";
    if (imo <= 999999) return "not used";
    if (imo <= 9999999) return "IMO number";
    return "official flag state number";
}

// True if `imo` is a seven-digit IMO number whose check digit is right.
//
// Multiply the first six digits by 7, 6, 5, 4, 3, 2, add the products; the
// last digit of the sum is the check digit:
//     9074729:  9*7 + 0*6 + 7*5 + 4*4 + 7*3 + 2*2 = 139  ->  9  ok
inline bool imo_check_digit_ok(std::uint64_t imo) {
    if (imo < 1000000 || imo > 9999999) return false;
    std::string digits = std::to_string(imo);  // exactly 7 characters here
    int total = 0;
    for (int i = 0; i < 6; ++i) {
        total += (digits[i] - '0') * (7 - i);
    }
    return total % 10 == digits[6] - '0';
}

// 'MM-DD HH:MM' in UTC with '??' for each part that is not available.
inline std::string eta_text(int month, int day, int hour, int minute) {
    auto two = [](int v) { return std::format("{:02}", v); };
    std::string mm = (1 <= month && month <= 12) ? two(month) : "??";
    std::string dd = (1 <= day && day <= 31) ? two(day) : "??";
    std::string hh = hour <= 23 ? two(hour) : "??";
    std::string mi = minute <= 59 ? two(minute) : "??";
    return mm + "-" + dd + " " + hh + ":" + mi;
}

// Decode a message 5. Throws DecodeError if shorter than 424 bits.
inline Json decode_msg_5(const Bits& bits) {
    int msg_type = static_cast<int>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 424);
    std::uint64_t ais_version = bits.uint(38, 2);
    std::uint64_t imo = bits.uint(40, 30);
    std::uint64_t ship_type = bits.uint(232, 8);
    std::uint64_t dim_a = bits.uint(240, 9);   // to bow
    std::uint64_t dim_b = bits.uint(249, 9);   // to stern
    std::uint64_t dim_c = bits.uint(258, 6);   // to port
    std::uint64_t dim_d = bits.uint(264, 6);   // to starboard
    std::uint64_t epfd = bits.uint(270, 4);
    int month = static_cast<int>(bits.uint(274, 4));
    int day = static_cast<int>(bits.uint(278, 5));
    int hour = static_cast<int>(bits.uint(283, 5));
    int minute = static_cast<int>(bits.uint(288, 6));
    std::uint64_t draught_raw = bits.uint(294, 8);
    std::uint64_t dte = bits.uint(422, 1);

    return Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"ais_version", ais_version},
        {"ais_version_text", AIS_VERSION[ais_version]},
        {"imo", imo},
        {"imo_available", imo != 0},
        {"imo_range_text", imo_range_text(imo)},
        {"imo_valid", imo_check_digit_ok(imo)},
        {"call_sign", bits.text(70, 42)},
        {"name", bits.text(112, 120)},
        {"ship_type", ship_type},
        {"ship_type_text", ship_type_text(static_cast<int>(ship_type))},
        {"dim_bow_m", dim_a},
        {"dim_stern_m", dim_b},
        {"dim_port_m", dim_c},
        {"dim_starboard_m", dim_d},
        // Figure 41: all four zero = neither reference point nor dimensions;
        // A = C = 0 with B, D non-zero = dimensions but no reference point.
        {"dimensions_available", !(dim_a == 0 && dim_b == 0 && dim_c == 0 && dim_d == 0)},
        {"reference_point_available", !(dim_a == 0 && dim_c == 0)},
        {"length_m", dim_a + dim_b},
        {"beam_m", dim_c + dim_d},
        {"epfd", epfd},
        {"epfd_text", epfd_text(static_cast<int>(epfd))},
        {"eta_month", month},
        {"eta_day", day},
        {"eta_hour", hour},
        {"eta_minute", minute},
        {"eta_text", eta_text(month, day, hour, minute)},
        {"eta_available", 1 <= month && month <= 12 && 1 <= day && day <= 31 && hour <= 23 && minute <= 59},
        {"draught_m", Json::fixed(static_cast<double>(draught_raw) / 10.0, 1)},
        {"draught_available", draught_raw != 0},
        {"destination", bits.text(302, 120)},
        {"dte", dte},
        {"dte_text", DTE_TEXT[dte]},
        {"spare", bits.uint(423, 1)},
    };
}

}  // namespace aistut
