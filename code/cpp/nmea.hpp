// nmea.hpp — the NMEA 0183 sentence layer (chapter 1).
//
// An NMEA 0183 sentence (IEC 61162-1 §7) looks like
//
//     $GPGGA,000429.000,3601.5522,N,10947.9359,W,1,7,4.21,12491.0,M,-20.5,M,,*6D
//     ^^     ^^^                                                              ^^
//     talker formatter                 comma-separated data fields        checksum
//
// * `$` (or `!` for encapsulated data such as AIS) starts the sentence.
// * The next two characters are the *talker* (GP = GPS, AI = AIS mobile
//   station, BS = base station, AN = AtoN, …), then a 3-letter *formatter*
//   (GGA, RMC, VDM, …). Proprietary sentences start with `P` + a 3-letter
//   manufacturer code (`PMTK`, `PSRT`, `PFEC`) and have no fixed formatter.
// * Fields are separated by commas; an empty field is allowed and means
//   "no data".
// * `*` followed by two hex digits is the checksum: the XOR of every byte
//   between `$`/`!` and `*` (exclusive). IEC 61162-1 §7.2.3.
// * The sentence ends with CR LF and may be at most 82 characters including
//   `$` and CR LF (§7.1.1). Real logs often break this; we do not enforce it.
//
// The USCG / serial_logger convention of appending extra comma fields *after*
// the checksum (`…*6D,rplane_to_sd,1197072048.14`) is also handled here:
// those fields are returned separately as `after_checksum`.
//
// C++ notes for readers new to the language
// -----------------------------------------
// * `struct Sentence` is a plain bundle of named fields — the C++ twin of a
//   Python `@dataclass`. Members get default values with `= …`; the compiler
//   writes the constructor and copy operations for us ("aggregate").
// * `class NmeaError : public std::invalid_argument` declares a new exception
//   type. We use `std::invalid_argument` as the C++ spelling of Python's
//   `ValueError` throughout the tutorial, so a caller can catch either the
//   specific or the general type.
// * `std::optional<double>` is returned where Python returns `float | None`.
//   `if (value)` tests for presence and `*value` reads it.
// * Lookup tables are `const std::map<...>` objects defined `inline` at
//   namespace scope (C++17 "inline variables" let a header define a global
//   without violating the one-definition rule). `.find(key)` returns an
//   iterator; comparing it with `.end()` is the C++ idiom for "not found".
// * A "function pointer" `Json (*)(const Sentence&)` lets us put parsers in a
//   table (`GPS_PARSERS`), exactly as Python stores functions in a dict.
// * `std::optional::transform` (C++23) is one of the *monadic* operations
//   (`transform`, `and_then`, `or_else`): it applies a function to the value
//   if there is one and passes the emptiness through otherwise, saving an
//   `if (!value) return …` at each call site.

#pragma once

#include <algorithm>
#include <map>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

#include "jsonout.hpp"
#include "pycompat.hpp"

namespace aistut {

// XOR of all characters in `body` (the text between '$' and '*').
//
//     checksum("GPGSV,1,1,00") == 0x79
//
// Python XORs code points (`ord(ch)`), so a non-ASCII character counts once
// and with its full value — we do the same rather than XOR raw UTF-8 bytes.
[[nodiscard]] inline int checksum(std::string_view body) {
    int value = 0;
    for (char32_t cp : py::codepoints(body)) {
        value ^= static_cast<int>(cp);
    }
    return value;
}

// One parsed NMEA sentence, before any formatter-specific decoding.
struct Sentence {
    char start = '$';                          // '$' or '!'
    std::string talker;                        // "GP", "AI", "P" for proprietary ...
    std::string formatter;                     // "GGA", "VDM", "MTK010" ...
    std::vector<std::string> fields;           // data fields *after* the address field
    std::optional<int> checksum_given;         // value written in the sentence, or none
    bool checksum_ok = false;                  // true only if present and matching
    std::vector<std::string> after_checksum;   // USCG extras
    std::string raw;

    // The address field, e.g. "GPGGA" or "AIVDM".
    std::string address() const { return talker + formatter; }
};

// Thrown for text that is not an NMEA sentence at all.
class NmeaError : public std::invalid_argument {
public:
    explicit NmeaError(const std::string& what) : std::invalid_argument(what) {}
};

// Split one line into its NMEA parts and verify the checksum.
//
// Leading/trailing whitespace and CR/LF are ignored. Characters before the
// `$`/`!` (such as a `file-name:` prefix from grep) are discarded; TAG
// blocks (chapter 4) must be stripped by the caller first.
inline Sentence parse_sentence(std::string_view input) {
    std::string line = py::strip(input);

    // Start at the first '$' or '!', whichever comes first.
    std::size_t dollar = line.find('$');
    std::size_t bang = line.find('!');
    std::size_t start_at = std::min(dollar, bang);   // npos is the largest value
    if (start_at == std::string::npos) {
        throw NmeaError("no '$' or '!' in line");
    }
    line = line.substr(start_at);
    char start = line[0];

    py::Partition at_star = py::partition(std::string_view(line).substr(1), '*');
    const std::string& body = at_star.head;
    std::optional<int> given;
    bool ok = false;
    std::vector<std::string> extras;
    if (at_star.found) {
        py::Partition at_comma = py::partition(at_star.tail, ',');
        std::optional<long long> parsed = py::to_int(py::slice(at_comma.head, 0, 2), 16);
        if (parsed) given = static_cast<int>(*parsed);
        ok = given.has_value() && *given == checksum(body);
        if (at_comma.found && !at_comma.tail.empty()) {
            extras = py::split(at_comma.tail, ',');
        }
    }
    // If there was no '*' we still parse the fields but checksum_ok is false.

    std::vector<std::string> parts = py::split(body, ',');
    std::string address = parts[0];
    std::vector<std::string> fields(parts.begin() + 1, parts.end());
    if (address.size() < 3) {
        throw NmeaError("address field too short: " + py::repr(address));
    }
    Sentence s;
    s.start = start;
    if (address[0] == 'P') {                  // proprietary: P + 3-char maker code
        s.talker = "P";
        s.formatter = address.substr(1);
    } else {
        s.talker = address.substr(0, 2);
        s.formatter = address.substr(2);
    }
    s.fields = std::move(fields);
    s.checksum_given = given;
    s.checksum_ok = ok;
    s.after_checksum = std::move(extras);
    s.raw = line;
    return s;
}

// --- Helpers for the GPS formatters ----------------------------------------

// "4312.2805","N" -> 43.204675 ; empty -> none.
//
// NMEA writes latitude as DDMM.MMMM: two digits of degrees followed by
// minutes with a decimal fraction. Longitude uses three degree digits
// (DDDMM.MMMM); see parse_longitude.
inline std::optional<double> parse_latitude(const std::string& ddmm, const std::string& hemisphere) {
    if (ddmm.empty()) return std::nullopt;
    double degrees = static_cast<double>(py::int_or_throw(py::slice(ddmm, 0, 2)));
    double minutes = py::float_or_throw(py::slice(ddmm, 2));
    double value = degrees + minutes / 60.0;
    return hemisphere == "S" ? -value : value;
}

// "07052.8474","W" -> -70.88079
inline std::optional<double> parse_longitude(const std::string& dddmm, const std::string& hemisphere) {
    if (dddmm.empty()) return std::nullopt;
    double degrees = static_cast<double>(py::int_or_throw(py::slice(dddmm, 0, 3)));
    double minutes = py::float_or_throw(py::slice(dddmm, 3));
    double value = degrees + minutes / 60.0;
    return hemisphere == "W" ? -value : value;
}

// "000429.000" -> "00:04:29.000" (kept as text; no date is available).
inline std::optional<std::string> parse_utc_time(const std::string& hhmmss) {
    if (hhmmss.size() < 6) return std::nullopt;
    return py::slice(hhmmss, 0, 2) + ":" + py::slice(hhmmss, 2, 4) + ":" + py::slice(hhmmss, 4);
}

// "081207" -> "2007-12-08". Two-digit years: NMEA predates Y2K, we assume
// 2000–2099 which is right for every file in this tutorial.
inline std::optional<std::string> parse_date(const std::string& ddmmyy) {
    if (ddmmyy.size() != 6) return std::nullopt;
    return "20" + py::slice(ddmmyy, 4, 6) + "-" + py::slice(ddmmyy, 2, 4) + "-" + py::slice(ddmmyy, 0, 2);
}

inline std::optional<double> opt_float(const std::string& s) {
    if (s.empty()) return std::nullopt;
    return py::float_or_throw(s);
}

// Wrap an optional float for printing with `decimals` places (see jsonout.hpp).
// `optional::transform` (C++23) applies the lambda only when a value is
// present and yields an empty optional otherwise; `Json` turns that empty
// optional into `null`. Python: `None if value is None else Fixed(value, d)`.
inline Json fixed(std::optional<double> value, int decimals) {
    return value.transform([decimals](double v) { return Fixed{.value = v, .decimals = decimals}; });
}

inline std::optional<long long> opt_int(const std::string& s) {
    if (s.empty()) return std::nullopt;
    return py::int_or_throw(s);
}

// `f[1] or None` in Python: an empty string becomes null.
inline Json string_or_null(const std::string& s) {
    if (s.empty()) return Json();
    return Json(s);
}

// Python pads a short field list with "" so that every index is valid.
inline std::vector<std::string> padded(const std::vector<std::string>& fields, std::size_t count) {
    std::vector<std::string> f = fields;
    while (f.size() < count) f.emplace_back("");
    return f;
}

// --- The four GPS sentences of chapter 1 -----------------------------------
//
// Field numbers below are 1-based positions *after* the address field, which
// is how the NMEA standard numbers them.

inline const std::map<long long, std::string> GGA_FIX_QUALITY = {
    {0, "invalid"}, {1, "GPS fix (SPS)"}, {2, "DGPS fix"}, {3, "PPS fix"},
    {4, "RTK fixed"}, {5, "RTK float"}, {6, "estimated (dead reckoning)"},
    {7, "manual input"}, {8, "simulation"},
};

// GGA — Global positioning system fix data (IEC 61162-1 §8.3.x / NMEA 0183 v3 §GGA).
//
//     1 UTC time hhmmss.ss        6 fix quality (table above)
//     2 latitude ddmm.mmmm        7 number of satellites in use
//     3 N/S                        8 HDOP
//     4 longitude dddmm.mmmm      9 antenna altitude, 10 'M'
//     5 E/W                       11 geoidal separation, 12 'M'
//                                 13 age of DGPS data (s), 14 DGPS station id
inline Json parse_gga(const Sentence& s) {
    std::vector<std::string> f = padded(s.fields, 14);     // pad short sentences
    std::optional<long long> quality = opt_int(f[5]);
    Json quality_text;                                      // null unless known
    if (quality) {
        auto it = GGA_FIX_QUALITY.find(*quality);
        quality_text = (it == GGA_FIX_QUALITY.end()) ? Json("unknown") : Json(it->second);
    }
    return Json::Object{
        {"sentence", "GGA"},
        {"time_utc", parse_utc_time(f[0])},
        {"lat", fixed(parse_latitude(f[1], f[2]), 6)},
        {"lon", fixed(parse_longitude(f[3], f[4]), 6)},
        {"fix_quality", quality},
        {"fix_quality_text", quality_text},
        {"satellites_used", opt_int(f[6])},
        {"hdop", fixed(opt_float(f[7]), 2)},
        {"altitude_m", fixed(opt_float(f[8]), 1)},
        {"geoid_separation_m", fixed(opt_float(f[10]), 1)},
        {"dgps_age_s", fixed(opt_float(f[12]), 1)},
        {"dgps_station", string_or_null(f[13])},
    };
}

inline const std::map<long long, std::string> GSA_MODE2 = {{1, "no fix"}, {2, "2D"}, {3, "3D"}};

// GSA — GNSS DOP and active satellites.
//
//     1 mode: M = manual, A = automatic 2D/3D switching
//     2 mode: 1 = fix not available, 2 = 2D, 3 = 3D
//     3–14 PRN numbers of the (up to 12) satellites used in the solution
//     15 PDOP  16 HDOP  17 VDOP
inline Json parse_gsa(const Sentence& s) {
    std::vector<std::string> f = padded(s.fields, 17);
    std::optional<long long> fix_mode = opt_int(f[1]);
    Json fix_mode_text;
    if (fix_mode) {
        auto it = GSA_MODE2.find(*fix_mode);
        if (it != GSA_MODE2.end()) fix_mode_text = it->second;   // else stays null
    }
    Json::Array used;
    for (std::size_t i = 2; i < 14; ++i) {
        if (!f[i].empty()) used.emplace_back(py::int_or_throw(f[i]));
    }
    return Json::Object{
        {"sentence", "GSA"},
        {"selection_mode", string_or_null(f[0])},
        {"fix_mode", fix_mode},
        {"fix_mode_text", fix_mode_text},
        {"satellites_used", used},
        {"pdop", fixed(opt_float(f[14]), 2)},
        {"hdop", fixed(opt_float(f[15]), 2)},
        {"vdop", fixed(opt_float(f[16]), 2)},
    };
}

// GSV — GNSS satellites in view. One sentence carries up to 4 satellites.
//
//     1 total number of GSV sentences in this group
//     2 this sentence's number (1-based)
//     3 total satellites in view
//     then repeating groups of 4: PRN, elevation (deg), azimuth (deg), SNR (dB-Hz)
//     Elevation/azimuth/SNR are empty when not tracked.
inline Json parse_gsv(const Sentence& s) {
    const std::vector<std::string>& f = s.fields;
    Json::Array sats;
    // Python: for i in range(3, len(f) - 3, 4). Careful with unsigned
    // arithmetic: compute the limit only when len(f) >= 4.
    for (std::size_t i = 3; i + 4 <= f.size(); i += 4) {
        const std::string& prn = f[i];
        if (!prn.empty()) {
            sats.emplace_back(Json::Object{
                {"prn", py::int_or_throw(prn)},
                {"elevation_deg", opt_int(f[i + 1])},
                {"azimuth_deg", opt_int(f[i + 2])},
                {"snr_db", opt_int(f[i + 3])},
            });
        }
    }
    return Json::Object{
        {"sentence", "GSV"},
        {"total_sentences", f.size() > 0 ? Json(opt_int(f[0])) : Json()},
        {"sentence_number", f.size() > 1 ? Json(opt_int(f[1])) : Json()},
        {"satellites_in_view", f.size() > 2 ? Json(opt_int(f[2])) : Json()},
        {"satellites", sats},
    };
}

inline const std::map<std::string, std::string> RMC_MODE = {
    {"A", "autonomous"}, {"D", "differential"}, {"E", "estimated"},
    {"M", "manual"}, {"S", "simulator"}, {"N", "not valid"},
};

// RMC — Recommended minimum specific GNSS data.
//
//     1 UTC time   2 status A = valid, V = warning (no fix)
//     3,4 latitude   5,6 longitude   7 SOG knots   8 COG degrees true
//     9 date ddmmyy   10,11 magnetic variation deg, E/W
//     12 mode indicator (NMEA 2.3+)   13 navigational status (NMEA 4.1+)
inline Json parse_rmc(const Sentence& s) {
    std::vector<std::string> f = padded(s.fields, 13);
    std::optional<double> var = opt_float(f[9]);
    if (var && f[10] == "W") var = -*var;
    Json mode_text;
    if (!f[11].empty()) {
        auto it = RMC_MODE.find(f[11]);
        if (it != RMC_MODE.end()) mode_text = it->second;
    }
    return Json::Object{
        {"sentence", "RMC"},
        {"time_utc", parse_utc_time(f[0])},
        {"status", string_or_null(f[1])},
        {"valid", f[1] == "A"},
        {"lat", fixed(parse_latitude(f[2], f[3]), 6)},
        {"lon", fixed(parse_longitude(f[4], f[5]), 6)},
        {"sog_knots", fixed(opt_float(f[6]), 2)},
        {"cog_deg", fixed(opt_float(f[7]), 2)},
        {"date", parse_date(f[8])},
        {"magnetic_variation_deg", fixed(var, 1)},
        {"mode", string_or_null(f[11])},
        {"mode_text", mode_text},
    };
}

// The parser table: formatter -> function. `GpsParser` is a function-pointer
// type; Python keeps plain function objects in a dict for the same purpose.
using GpsParser = Json (*)(const Sentence&);

inline const std::map<std::string, GpsParser> GPS_PARSERS = {
    {"GGA", parse_gga}, {"GSA", parse_gsa}, {"GSV", parse_gsv}, {"RMC", parse_rmc},
};

}  // namespace aistut
