// msg_17.hpp — message 17: GNSS broadcast binary message (chapter 13).
//
// Spec: ITU-R M.1371-5 Annex 8 §3.15, Table 68 (80-bit header plus 0–736
// bits of differential-correction data) and Table 69 (layout of the data:
// a 40-bit RTCM SC-104 style header followed by N 24-bit data words, parity
// removed). The content follows Recommendation ITU-R M.823.
//
// Bit layout (zero-based offsets; lengths from Table 68):
//
//      0- 5   6  message id (17)
//      6- 7   2  repeat indicator
//      8-37  30  source id (MMSI of the base station)
//     38-39   2  spare
//     40-57  18  longitude of the DGNSS reference station, signed, 1/10 min; 181° = n/a
//     58-74  17  latitude of the DGNSS reference station, signed, 1/10 min; 91° = n/a
//     75-79   5  spare
//     80-     0–736 differential correction data (Table 69):
//       80- 85   6  RTCM message type
//       86- 95  10  RTCM station id
//       96-108  13  Z count, units of 0.6 s (0–3599.4 s)
//      109-111   3  sequence number (cyclic 0–7)
//      112-116   5  N, number of 24-bit data words following this header (≤ 29)
//      117-119   3  reference station health
//      120-     N × 24 bits of DGNSS data words
//
// Length rule: at least 80 bits (else DecodeError). With 40 or more data
// bits we decode the RTCM header and hand back the remaining bits as a
// string; `words_complete` says whether exactly N × 24 bits were present.
// Fewer than 40 data bits gives `rtcm: null`.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `Json out = ...; out["rtcm"] = cond ? Json(x) : Json(nullptr);` — both
//   branches of `?:` must have the same type, so we wrap each in `Json(...)`.
// * `bits.size() - 80` is unsigned arithmetic on `std::size_t`; it is safe
//   only because `require_bits` has already guaranteed size() >= 80.

#pragma once

#include <array>
#include <cstdint>
#include <string>

#include "bits.hpp"
#include "common.hpp"
#include "jsonout.hpp"

namespace aistut {

inline constexpr std::int64_t M17_LON_NOT_AVAILABLE = 181 * 600;  // 0x1A838, 1/10-minute units
inline constexpr std::int64_t M17_LAT_NOT_AVAILABLE = 91 * 600;   // 0x0D548

// Reference-station health, Table 69 → ITU-R M.823 (the RTCM SC-104 v2 code).
inline const std::array<const char*, 8> RTCM_HEALTH = {
    "UDRE scale factor 1.0",
    "UDRE scale factor 0.75",
    "UDRE scale factor 0.5",
    "UDRE scale factor 0.3",
    "UDRE scale factor 0.2",
    "UDRE scale factor 0.1",
    "reference station transmission not monitored",
    "reference station not working",
};

// The 40-bit RTCM header of Table 69 plus whatever data words follow.
inline Json decode_rtcm_header(const Bits& bits, std::size_t start) {
    std::uint64_t n_words = bits.uint(start + 32, 5);
    std::uint64_t health = bits.uint(start + 37, 3);
    std::uint64_t z_count = bits.uint(start + 16, 13);
    std::size_t words_start = start + 40;
    std::size_t words_bits = bits.size() - words_start;
    return Json::Object{
        {"message_type", bits.uint(start, 6)},
        {"station_id", bits.uint(start + 6, 10)},
        {"z_count", z_count},
        {"z_count_s", Json::fixed(static_cast<double>(z_count) * 0.6, 1)},
        {"sequence", bits.uint(start + 22, 3)},
        {"n_words", n_words},
        {"health", health},
        {"health_text", RTCM_HEALTH[health]},
        {"words", bits.raw(words_start, words_bits)},
        {"words_bits", words_bits},
        {"words_complete", words_bits == n_words * 24},
    };
}

// Decode a message 17. Throws DecodeError below 80 bits.
inline Json decode_msg_17(const Bits& bits) {
    require_bits(17, bits.size(), 80);
    std::int64_t lon_raw = bits.sint(40, 18);
    std::int64_t lat_raw = bits.sint(58, 17);
    std::size_t data_bits = bits.size() - 80;
    Json out = Json::Object{
        {"msg_type", bits.uint(0, 6)},
        {"repeat_indicator", bits.uint(6, 2)},
        {"mmsi", bits.uint(8, 30)},
        {"spare", bits.uint(38, 2)},
        {"lon", Json::fixed(static_cast<double>(lon_raw) / 600.0, 6)},
        {"lat", Json::fixed(static_cast<double>(lat_raw) / 600.0, 6)},
        {"position_available", lon_raw != M17_LON_NOT_AVAILABLE && lat_raw != M17_LAT_NOT_AVAILABLE},
        {"spare2", bits.uint(75, 5)},
        {"data", bits.raw(80, data_bits)},
        {"data_bits", data_bits},
    };
    // Table 69: the two-word (40-bit) header is only meaningful when complete.
    out["rtcm"] = data_bits >= 40 ? decode_rtcm_header(bits, 80) : Json(nullptr);
    return out;
}

}  // namespace aistut
