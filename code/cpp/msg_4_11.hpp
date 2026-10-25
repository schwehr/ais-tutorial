// msg_4_11.hpp — messages 4 and 11: base station report / UTC-date response
// (chapter 6).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.2, Table 51 (field layout, 168 bits). The
// SOTDMA communication state is Annex 2 §3.3.7.2.2 Table 18/19 and is decoded
// by `decode_sotdma` from msg_1_2_3.hpp — a message 4 always carries SOTDMA,
// never ITDMA [Table 51, last row].
//
// Message 4 is sent by a base station every 10 s (3 1/3 s when it acts as
// the synchronisation "semaphore") [Annex 1 §4.2.1 Table 2 note 1; Annex 2
// §3.1.3.3.1]. Message 11 is the *same layout* sent by a mobile station in
// answer to a UTC-date inquiry (message 10) [Annex 8 §3.2, §3.9].
//
// Bit layout (zero-based offsets; lengths from Table 51):
//
//      0- 5   6  message id (4 or 11)
//      6- 7   2  repeat indicator
//      8-37  30  user id (MMSI)
//     38-51  14  UTC year,   1-9999; 0 = n/a
//     52-55   4  UTC month,  1-12;   0 = n/a; 13-15 not used
//     56-60   5  UTC day,    1-31;   0 = n/a
//     61-65   5  UTC hour,   0-23;  24 = n/a; 25-31 not used
//     66-71   6  UTC minute, 0-59;  60 = n/a; 61-63 not used
//     72-77   6  UTC second, 0-59;  60 = n/a; 61-63 not used
//     78      1  position accuracy (1 = high, <= 10 m)
//     79-106 28  longitude, signed, 1/10 000 minute; 181° (0x6791AC0) = n/a
//    107-133 27  latitude,  signed, 1/10 000 minute;  91° (0x3412140) = n/a
//    134-137  4  type of electronic position fixing device (EPFD)
//    138      1  transmission control for long-range broadcast message
//    139-147  9  spare
//    148      1  RAIM flag
//    149-167 19  SOTDMA communication state
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `#include "msg_1_2_3.hpp"` pulls in `decode_sotdma`. Because every module
//   is header-only and marked `inline`, including the same header from two
//   places (here and decode.cpp) does not violate the one-definition rule.
// * `std::optional<std::string>` is "a string or nothing". `std::nullopt` is
//   the nothing; `Json` has a constructor for it that prints `null`. Python
//   used `None` for the same job.
// * `std::format` with "{:04}" zero-pads to four digits — the same format
//   spec as Python's `{year:04d}`; "{:02}" pads to two.

#pragma once

#include <array>
#include <cstdint>
#include <format>
#include <optional>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"
#include "msg_1_2_3.hpp"

namespace aistut {

// Not-available sentinels for the six UTC fields, Table 51.
inline constexpr std::uint64_t UTC_YEAR_NA = 0;
inline constexpr std::uint64_t UTC_MONTH_NA = 0;
inline constexpr std::uint64_t UTC_DAY_NA = 0;
inline constexpr std::uint64_t UTC_HOUR_NA = 24;
inline constexpr std::uint64_t UTC_MINUTE_NA = 60;
inline constexpr std::uint64_t UTC_SECOND_NA = 60;

inline const std::array<const char*, 2> TRANSMISSION_CONTROL = {
    "Class A stops message 27 within base station coverage (default)",
    "Class A requested to transmit message 27 within base station coverage",
};

// Join the six UTC fields into "YYYY-MM-DDTHH:MM:SSZ", or nullopt.
//
// nullopt is returned when *any* component is its not-available value or
// outside its valid range (Table 51: month 13-15, hour 25-31, minute and
// second 61-63 are "not used"). We do not check the calendar (31 April
// passes) — that is left to the reader as an exercise.
inline std::optional<std::string> utc_iso(std::uint64_t year, std::uint64_t month, std::uint64_t day,
                                          std::uint64_t hour, std::uint64_t minute, std::uint64_t second) {
    if (year < 1 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31) return std::nullopt;
    if (hour > 23 || minute > 59 || second > 59) return std::nullopt;
    return std::format("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hour, minute, second);
}

// Decode a message 4 or 11. Throws DecodeError if shorter than 168 bits.
inline Json decode_msg_4_11(const Bits& bits) {
    std::int32_t msg_type = static_cast<std::int32_t>(bits.uint(0, 6));
    require_bits(msg_type, bits.size(), 168);
    std::uint64_t year = bits.uint(38, 14);
    std::uint64_t month = bits.uint(52, 4);
    std::uint64_t day = bits.uint(56, 5);
    std::uint64_t hour = bits.uint(61, 5);
    std::uint64_t minute = bits.uint(66, 6);
    std::uint64_t second = bits.uint(72, 6);
    std::int64_t lon_raw = bits.sint(79, 28);
    std::int64_t lat_raw = bits.sint(107, 27);
    std::uint64_t epfd = bits.uint(134, 4);
    std::uint64_t tx_control = bits.uint(138, 1);

    std::optional<std::string> iso = utc_iso(year, month, day, hour, minute, second);
    Json out = Json::Object{
        {"msg_type", msg_type},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"utc_year", year},
        {"utc_year_available", year != UTC_YEAR_NA},
        {"utc_month", month},
        {"utc_month_available", month != UTC_MONTH_NA},
        {"utc_day", day},
        {"utc_day_available", day != UTC_DAY_NA},
        {"utc_hour", hour},
        {"utc_hour_available", hour != UTC_HOUR_NA},
        {"utc_minute", minute},
        {"utc_minute_available", minute != UTC_MINUTE_NA},
        {"utc_second", second},
        {"utc_second_available", second != UTC_SECOND_NA},
        {"utc_iso", iso ? Json(*iso) : Json(nullptr)},
        {"position_accuracy", bits.uint(78, 1)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600000.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600000.0, 6)},
        {"position_available", lon_raw != 0x6791AC0 && lat_raw != 0x3412140},
        {"epfd", epfd},
        {"epfd_text", epfd_text(static_cast<std::int32_t>(epfd))},
        {"transmission_control", tx_control},
        {"transmission_control_text", TRANSMISSION_CONTROL[tx_control]},
        {"spare", bits.uint(139, 9)},
        {"raim", bits.flag(148)},
        {"comm_state", decode_sotdma(bits, 149)},
    };
    return out;
}

}  // namespace aistut
