// asm.hpp — application-specific message (ASM) payloads, keyed by (DAC, FI).
//
// Messages 6 and 8 carry a 16-bit application identifier — a 10-bit
// designated area code (DAC) and a 6-bit function identifier (FI) — followed
// by data whose layout is defined by whoever owns that DAC
// [ITU-R M.1371-5 Annex 5 §2.1 Table 23]. This header is the registry that
// turns a known (dac, fi) pair into decoded fields; msg_6.hpp and msg_8.hpp
// call `decode_asm(dac, fi, bits, offset)` and attach a non-null result as
// "application". The Python original (aistut/asm.py) documents every bit
// offset; the comments here only repeat the layout summaries.
//
// Implemented (chapter 9):
//   (1, 0)    IFM 0 text telegram                 M.1371-5 Annex 5 §5.1 Tables 25/26
//   (1, 11)   Met/hydro, IMO SN/Circ.236 (2004)   Circ.236 trial message 1 (AIVDM "Type 8(1/11)")
//   (1, 31)   Met/hydro, IMO SN.1/Circ.289 (2010) Circ.289 Annex Table 1.1
//   (366, 34) USCG/NOAA right-whale zone notice   reverse-engineered, see chapter 9
//
// Output conventions: a not-available sensor value prints as null (no
// `_available` companion); enumerations keep their `_text` companion; a
// payload that is too short yields {"application_name", "error"}.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `std::optional<T>` is how we say "a value or nothing". `std::nullopt` is
//   the empty state and the Json constructor turns it into `null`.
// * A *function pointer* type, `using AsmDecoder = Json (*)(const Bits&,
//   std::size_t, std::size_t);`, lets us store decoders in a table exactly
//   like Python's dictionary of functions. `std::map<std::pair<int,int>, …>`
//   works because `std::pair` already defines `operator<`.
// * `static const` tables inside a function are initialised once, on first
//   use — a common way to keep a lookup table next to the code that uses it.

#pragma once

#include <cstddef>
#include <cstdint>
#include <map>
#include <optional>
#include <string>
#include <utility>

#include "bits.hpp"
#include "jsonout.hpp"

namespace aistut {

namespace asm_detail {

// `raw` unless it equals the not-available sentinel, in which case null.
inline Json na(std::uint64_t raw, std::uint64_t sentinel) {
    if (raw == sentinel) return nullptr;
    return Json(static_cast<std::int64_t>(raw));
}

// raw / divisor + offset with `decimals` places, or null if raw == sentinel.
inline Json scaled(std::int64_t raw, std::int64_t sentinel, double divisor, std::int32_t decimals,
                   double offset = 0.0) {
    if (raw == sentinel) return nullptr;
    return Json::fixed(static_cast<double>(raw) / divisor + offset, decimals);
}

inline const char* tendency_text(std::uint64_t code) {
    static const char* const T[] = {"steady", "decreasing", "increasing", "not available"};
    return T[code & 3];
}

// WMO 306 code table 4.201 as reproduced in Circ.289 Table 1.1.
inline const char* precipitation_text(std::uint64_t code) {
    static const char* const T[] = {"reserved", "rain", "thunderstorm", "freezing rain",
                                    "mixed/ice", "snow", "reserved", "not available"};
    return T[code & 7];
}

// Beaufort scale, Circ.289 Table 1.2.
inline const char* sea_state_text(std::uint64_t code) {
    static const char* const T[] = {
        "calm (flat)", "light air (ripples)", "light breeze (small wavelets)",
        "gentle breeze (large wavelets)", "moderate breeze (small waves)",
        "fresh breeze (moderate waves)", "strong breeze (large waves)",
        "near gale (sea heaps up)", "gale (moderately high waves)",
        "strong gale (high waves)", "storm (very high waves)",
        "violent storm (exceptionally high waves)", "hurricane (huge waves)",
        "not available", "reserved", "reserved",
    };
    return T[code & 15];
}

inline const char* ice_text(std::uint64_t code) {
    static const char* const T[] = {"no", "yes", "reserved", "not available"};
    return T[code & 3];
}

// --- (1, 0) IFM 0: text telegram (Annex 5 Tables 25/26) --------------------
inline Json decode_text_telegram(const Bits& bits, std::size_t o, std::size_t data_bits) {
    std::size_t text_bits = (data_bits - 12) / 6 * 6;
    return Json::Object{
        {"ack_required", bits.flag(o)},
        {"text_sequence_number", bits.uint(o + 1, 11)},
        {"text", bits.text(o + 12, text_bits)},
        {"text_chars", text_bits / 6},
        {"spare_bits", data_bits - 12 - text_bits},
    };
}

// --- (1, 11) IMO SN/Circ.236 met/hydro, 296 data bits ----------------------
// Latitude before longitude; temperatures and water level are offset-coded
// (value/10 - 60, - 20, - 10); every not-available value is the field maximum.
constexpr std::size_t MET_HYDRO_236_BITS = 296;

inline Json decode_met_hydro_236(const Bits& bits, std::size_t o, std::size_t) {
    std::int64_t lat_raw = bits.sint(o, 24);
    std::int64_t lon_raw = bits.sint(o + 24, 25);
    std::uint64_t tendency = bits.uint(o + 134, 2);
    std::uint64_t level_trend = bits.uint(o + 153, 2);
    std::uint64_t sea_state = bits.uint(o + 262, 4);
    std::uint64_t precip = bits.uint(o + 276, 3);
    std::uint64_t ice = bits.uint(o + 288, 2);
    std::uint64_t pressure_raw = bits.uint(o + 125, 9);
    return Json::Object{
        {"lat", lat_raw == 0x7FFFFF ? Json(nullptr) : Json::fixed(static_cast<double>(lat_raw) / 60000.0, 6)},
        {"lon", lon_raw == 0xFFFFFF ? Json(nullptr) : Json::fixed(static_cast<double>(lon_raw) / 60000.0, 6)},
        {"day", na(bits.uint(o + 49, 5), 31)},
        {"hour", na(bits.uint(o + 54, 5), 31)},
        {"minute", na(bits.uint(o + 59, 6), 63)},
        {"wind_speed_knots", na(bits.uint(o + 65, 7), 127)},
        {"wind_gust_knots", na(bits.uint(o + 72, 7), 127)},
        {"wind_dir_deg", na(bits.uint(o + 79, 9), 511)},
        {"wind_gust_dir_deg", na(bits.uint(o + 88, 9), 511)},
        {"air_temp_c", scaled(static_cast<std::int64_t>(bits.uint(o + 97, 11)), 2047, 10.0, 1, -60.0)},
        {"humidity_pct", na(bits.uint(o + 108, 7), 127)},
        {"dew_point_c", scaled(static_cast<std::int64_t>(bits.uint(o + 115, 10)), 1023, 10.0, 1, -20.0)},
        {"pressure_hpa", pressure_raw == 511 ? Json(nullptr) : Json(pressure_raw + 800)},
        {"pressure_tendency", tendency},
        {"pressure_tendency_text", tendency_text(tendency)},
        {"visibility_nm", scaled(static_cast<std::int64_t>(bits.uint(o + 136, 8)), 255, 10.0, 1)},
        {"water_level_m", scaled(static_cast<std::int64_t>(bits.uint(o + 144, 9)), 511, 10.0, 1, -10.0)},
        {"water_level_trend", level_trend},
        {"water_level_trend_text", tendency_text(level_trend)},
        {"current_speed_knots", scaled(static_cast<std::int64_t>(bits.uint(o + 155, 8)), 255, 10.0, 1)},
        {"current_dir_deg", na(bits.uint(o + 163, 9), 511)},
        {"current_2_speed_knots", scaled(static_cast<std::int64_t>(bits.uint(o + 172, 8)), 255, 10.0, 1)},
        {"current_2_dir_deg", na(bits.uint(o + 180, 9), 511)},
        {"current_2_depth_m", na(bits.uint(o + 189, 5), 31)},
        {"current_3_speed_knots", scaled(static_cast<std::int64_t>(bits.uint(o + 194, 8)), 255, 10.0, 1)},
        {"current_3_dir_deg", na(bits.uint(o + 202, 9), 511)},
        {"current_3_depth_m", na(bits.uint(o + 211, 5), 31)},
        {"wave_height_m", scaled(static_cast<std::int64_t>(bits.uint(o + 216, 8)), 255, 10.0, 1)},
        {"wave_period_s", na(bits.uint(o + 224, 6), 63)},
        {"wave_dir_deg", na(bits.uint(o + 230, 9), 511)},
        {"swell_height_m", scaled(static_cast<std::int64_t>(bits.uint(o + 239, 8)), 255, 10.0, 1)},
        {"swell_period_s", na(bits.uint(o + 247, 6), 63)},
        {"swell_dir_deg", na(bits.uint(o + 253, 9), 511)},
        {"sea_state", sea_state},
        {"sea_state_text", sea_state_text(sea_state)},
        {"water_temp_c", scaled(static_cast<std::int64_t>(bits.uint(o + 266, 10)), 1023, 10.0, 1, -10.0)},
        {"precipitation", precip},
        {"precipitation_text", precipitation_text(precip)},
        {"salinity_ppt", scaled(static_cast<std::int64_t>(bits.uint(o + 279, 9)), 511, 10.0, 1)},
        {"ice", ice},
        {"ice_text", ice_text(ice)},
        {"spare", bits.uint(o + 290, 6)},
    };
}

// --- (1, 31) IMO SN.1/Circ.289 met/hydro, 304 data bits --------------------
constexpr std::size_t MET_HYDRO_289_BITS = 304;
constexpr std::int64_t LON_1000_NOT_AVAILABLE = 181LL * 60000;   // 10 860 000
constexpr std::int64_t LAT_1000_NOT_AVAILABLE = 91LL * 60000;    //  5 460 000

// Circ.289 air pressure: 1-401 map to 800-1200 hPa; the ends are clamped.
inline Json pressure_289(std::uint64_t raw) {
    if (raw == 511) return nullptr;
    if (raw == 0) return 799;          // "pressure 799 hPa or less"
    if (raw >= 402) return 1201;       // "pressure 1201 hPa or greater"
    return Json(raw + 799);
}

inline Json decode_met_hydro_289(const Bits& bits, std::size_t o, std::size_t) {
    std::int64_t lon_raw = bits.sint(o, 25);
    std::int64_t lat_raw = bits.sint(o + 25, 24);
    std::uint64_t tendency = bits.uint(o + 135, 2);
    std::uint64_t vis_raw = bits.uint(o + 137, 8);
    std::uint64_t level_trend = bits.uint(o + 157, 2);
    std::uint64_t sea_state = bits.uint(o + 266, 4);
    std::uint64_t precip = bits.uint(o + 280, 3);
    std::uint64_t salinity_raw = bits.uint(o + 283, 9);
    std::uint64_t ice = bits.uint(o + 292, 2);
    return Json::Object{
        {"lon", lon_raw == LON_1000_NOT_AVAILABLE ? Json(nullptr) : Json::fixed(static_cast<double>(lon_raw) / 60000.0, 6)},
        {"lat", lat_raw == LAT_1000_NOT_AVAILABLE ? Json(nullptr) : Json::fixed(static_cast<double>(lat_raw) / 60000.0, 6)},
        {"position_accuracy", bits.uint(o + 49, 1)},
        {"day", na(bits.uint(o + 50, 5), 0)},
        {"hour", na(bits.uint(o + 55, 5), 24)},
        {"minute", na(bits.uint(o + 60, 6), 60)},
        {"wind_speed_knots", na(bits.uint(o + 66, 7), 127)},
        {"wind_gust_knots", na(bits.uint(o + 73, 7), 127)},
        {"wind_dir_deg", na(bits.uint(o + 80, 9), 360)},
        {"wind_gust_dir_deg", na(bits.uint(o + 89, 9), 360)},
        {"air_temp_c", scaled(bits.sint(o + 98, 11), -1024, 10.0, 1)},
        {"humidity_pct", na(bits.uint(o + 109, 7), 101)},
        {"dew_point_c", scaled(bits.sint(o + 116, 10), 501, 10.0, 1)},
        {"pressure_hpa", pressure_289(bits.uint(o + 126, 9))},
        {"pressure_tendency", tendency},
        {"pressure_tendency_text", tendency_text(tendency)},
        {"visibility_nm", scaled(static_cast<std::int64_t>(vis_raw & 0x7F), 127, 10.0, 1)},
        {"visibility_greater_than", (vis_raw >> 7) == 1},
        {"water_level_m", scaled(static_cast<std::int64_t>(bits.uint(o + 145, 12)), 4001, 100.0, 2, -10.0)},
        {"water_level_trend", level_trend},
        {"water_level_trend_text", tendency_text(level_trend)},
        {"current_speed_knots", scaled(static_cast<std::int64_t>(bits.uint(o + 159, 8)), 255, 10.0, 1)},
        {"current_dir_deg", na(bits.uint(o + 167, 9), 360)},
        {"current_2_speed_knots", scaled(static_cast<std::int64_t>(bits.uint(o + 176, 8)), 255, 10.0, 1)},
        {"current_2_dir_deg", na(bits.uint(o + 184, 9), 360)},
        {"current_2_depth_m", na(bits.uint(o + 193, 5), 31)},
        {"current_3_speed_knots", scaled(static_cast<std::int64_t>(bits.uint(o + 198, 8)), 255, 10.0, 1)},
        {"current_3_dir_deg", na(bits.uint(o + 206, 9), 360)},
        {"current_3_depth_m", na(bits.uint(o + 215, 5), 31)},
        {"wave_height_m", scaled(static_cast<std::int64_t>(bits.uint(o + 220, 8)), 255, 10.0, 1)},
        {"wave_period_s", na(bits.uint(o + 228, 6), 63)},
        {"wave_dir_deg", na(bits.uint(o + 234, 9), 360)},
        {"swell_height_m", scaled(static_cast<std::int64_t>(bits.uint(o + 243, 8)), 255, 10.0, 1)},
        {"swell_period_s", na(bits.uint(o + 251, 6), 63)},
        {"swell_dir_deg", na(bits.uint(o + 257, 9), 360)},
        {"sea_state", sea_state},
        {"sea_state_text", sea_state_text(sea_state)},
        {"water_temp_c", scaled(bits.sint(o + 270, 10), 501, 10.0, 1)},
        {"precipitation", precip},
        {"precipitation_text", precipitation_text(precip)},
        {"salinity_ppt", salinity_raw >= 510 ? Json(nullptr) : Json::fixed(static_cast<double>(salinity_raw) / 10.0, 1)},
        {"salinity_sensor_available", salinity_raw != 511},
        {"ice", ice},
        {"ice_text", ice_text(ice)},
        {"spare", bits.uint(o + 294, 10)},
    };
}

// --- (366, 34) USCG/NOAA right-whale zone notice, 2008 form ----------------
// Reverse-engineered (chapter 9): a sequence of 112-bit records, each a
// 36-bit header (zone id 5, zone type 7, day 5, hour 5, minute 6, duration
// in 10-minute units 8) and one 76-bit circle (shape 1, lon 28, lat 27,
// radius 16, spare 4).
constexpr std::size_t WHALE_RECORD_BITS = 112;

inline const char* whale_zone_type_text(std::uint64_t code) {
    if (code == 0) return "right whales not observed";
    if (code == 1) return "right whales present";
    return "unknown";
}

inline Json decode_whale_notice(const Bits& bits, std::size_t o, std::size_t data_bits) {
    Json::Array notices;
    std::size_t r = o;
    while (bits.has(r, WHALE_RECORD_BITS)) {
        std::uint64_t zone_type = bits.uint(r + 5, 7);
        std::uint64_t duration_raw = bits.uint(r + 28, 8);
        std::size_t z = r + 36;
        // Built as a named object first: pushing a temporary `Json::Object`
        // straight into the vector draws a spurious "may be used
        // uninitialized" warning from some GCC versions.
        Json notice = Json::Object{
            {"zone_id", bits.uint(r, 5)},
            {"zone_type", zone_type},
            {"zone_type_text", whale_zone_type_text(zone_type)},
            {"start_day", bits.uint(r + 12, 5)},
            {"start_hour", bits.uint(r + 17, 5)},
            {"start_minute", bits.uint(r + 22, 6)},
            {"duration_raw", duration_raw},
            {"duration_minutes", duration_raw * 10},
            {"shape", bits.uint(z, 1)},
            {"lon", Json::fixed(static_cast<double>(bits.sint(z + 1, 28)) / 600000.0, 6)},
            {"lat", Json::fixed(static_cast<double>(bits.sint(z + 29, 27)) / 600000.0, 6)},
            {"radius_m", bits.uint(z + 56, 16)},
            {"spare", bits.uint(z + 72, 4)},
        };
        notices.push_back(std::move(notice));
        r += WHALE_RECORD_BITS;
    }
    return Json::Object{
        {"notices", notices},
        {"trailing_bits", o + data_bits - r},
    };
}

// --- the registry -----------------------------------------------------------
using AsmDecoder = Json (*)(const Bits&, std::size_t, std::size_t);

struct AsmEntry {
    const char* name;
    std::size_t need_bits;
    AsmDecoder func;
};

inline const std::map<std::pair<std::int32_t, std::int32_t>, AsmEntry>& registry() {
    static const std::map<std::pair<std::int32_t, std::int32_t>, AsmEntry> table = {
        {{1, 0}, {"IFM 0 text telegram", 12, decode_text_telegram}},
        {{1, 11}, {"IMO Circ.236 meteorological and hydrological data", MET_HYDRO_236_BITS, decode_met_hydro_236}},
        {{1, 31}, {"IMO Circ.289 meteorological and hydrographic data", MET_HYDRO_289_BITS, decode_met_hydro_289}},
        {{366, 34}, {"USCG/NOAA right whale zone notice (2008)", WHALE_RECORD_BITS, decode_whale_notice}},
    };
    return table;
}

}  // namespace asm_detail

// Decode the application data that starts at `offset`. Returns an empty
// optional when (dac, fi) is unknown. For a known pair the object always
// carries "application_name"; too few bits yield an "error" text instead of
// the fields.
inline std::optional<Json> decode_asm(std::int32_t dac, std::int32_t fi, const Bits& bits, std::size_t offset) {
    const auto& table = asm_detail::registry();
    auto it = table.find({dac, fi});
    if (it == table.end()) return std::nullopt;
    const asm_detail::AsmEntry& entry = it->second;
    std::size_t data_bits = bits.size() - offset;
    Json out = Json::Object{{"application_name", entry.name}};
    if (data_bits < entry.need_bits) {
        out["error"] = "needs " + std::to_string(entry.need_bits) + " data bits, have " +
                       std::to_string(data_bits);
        return out;
    }
    out.update(entry.func(bits, offset, data_bits));
    return out;
}

}  // namespace aistut
