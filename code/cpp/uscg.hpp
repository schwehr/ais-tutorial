// uscg.hpp — the US Coast Guard "N-AIS" receive metadata (chapter 5).
//
// The USCG Research & Development Center's AIS network (and Kurt Schwehr's
// `serial_logger.py`, which copied the idea) appends extra comma-separated
// fields *after* the NMEA checksum:
//
//     !AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,s34486,d-074,T25.49701619,x78124,r003669946,1223901985
//                                                    └ s: signal strength, raw receiver units (0–65535)
//                                                           └ d: signal strength in dBm
//                                                                  └ T: time of arrival, seconds into the UTC minute
//                                                                               └ x: receiver's sentence counter
//                                                                                       └ r: receiving station id
//                                                                                                    └ UNIX time (s)
//
// The format was never published as a standard; it is described in the GPSD
// AIVDM.adoc "U.S. Coast Guard Extended AIVDM" section and in the AISUser
// manual (output format 2, "proprietary USCG R&D Center format"). Field
// order varies and most fields are optional; only the station id and the
// final time are always present. Known prefixes:
//
//     s   received signal strength indicator (RSSI), receiver-specific units
//     d   signal strength, dBm (negative number)
//     T   time of arrival within the minute (fractional seconds)
//     S   slot number (0–2249) in which the message was received
//     x   sentence counter from the receiver
//     r   receiving station id (`b` is also seen for base stations)
//     (none) a bare number: UNIX time in seconds, possibly with a fraction
//
// C++ notes for readers new to the language
// -----------------------------------------
// * Python wraps the whole if/elif chain in `try: … except ValueError` and
//   treats any conversion failure as "unparsed". C++ exceptions would work
//   the same way, but here each conversion returns `std::optional`, so the
//   failure path is an ordinary `if (!value)` — no exception needed. Both
//   styles are idiomatic; optionals are preferred when failure is routine.
// * `item[0]` is a `char`; `item.substr(1)` is the rest. Characters compare
//   with single quotes: `code == 's'`.
// * `std::abs` from <cmath> works on doubles; the <cstdlib> one is for ints.

#pragma once

#include <cmath>
#include <optional>
#include <string>
#include <vector>

#include "jsonout.hpp"
#include "pycompat.hpp"

namespace aistut {

// Interpret the fields that followed the checksum.
//
// Unknown or malformed items are kept verbatim in `unparsed` so that no
// information is silently lost.
inline Json parse_uscg_fields(const std::vector<std::string>& extras) {
    Json out = Json::object();
    Json::Array unparsed;
    for (const std::string& item : extras) {
        if (item.empty()) continue;
        char code = item[0];
        std::string rest = item.substr(1);
        bool handled = false;   // set to true once a branch consumed the item
        if (code == 's' && py::isdigit(py::lstrip(rest, '-'))) {
            if (auto v = py::to_int(rest)) { out["rssi"] = *v; handled = true; }
        } else if (code == 'd' && py::isdigit(py::lstrip(rest, '-'))) {
            if (auto v = py::to_int(rest)) { out["signal_dbm"] = *v; handled = true; }
        } else if (code == 'T') {
            if (auto v = py::to_float(rest)) { out["time_of_arrival_s"] = Json::fixed(*v, 8); handled = true; }
        } else if (code == 'S' && py::isdigit(rest)) {
            if (auto v = py::to_int(rest)) { out["slot"] = *v; handled = true; }
        } else if (code == 'x' && py::isdigit(rest)) {
            if (auto v = py::to_int(rest)) { out["sentence_counter"] = *v; handled = true; }
        } else if ((code == 'r' || code == 'b') && !rest.empty()) {
            out["station"] = rest;
            out["station_kind"] = (code == 'b') ? "base" : "receiver";
            handled = true;
        } else {
            // A bare number is the UNIX receive time. Python's test is
            // item.replace(".", "", 1).isdigit(): digits with at most one dot.
            std::string without_dot = item;
            std::size_t dot = without_dot.find('.');
            if (dot != std::string::npos) without_dot.erase(dot, 1);
            if (py::isdigit(without_dot)) {
                if (dot != std::string::npos) {
                    if (auto v = py::to_float(item)) { out["time"] = Json::fixed(*v, 2); handled = true; }
                } else {
                    if (auto v = py::to_int(item)) { out["time"] = *v; handled = true; }
                }
            }
        }
        if (!handled) unparsed.emplace_back(item);
    }
    if (!unparsed.empty()) out["unparsed"] = unparsed;
    return out;
}

// Combine the coarse UNIX time with the fine `T` field.
//
// `T` is seconds past the start of the UTC minute measured by the receiver's
// clock, so the precise arrival time is "start of some UTC minute + T". The
// logging computer's `unix_time` tells us *which* minute: we try the minute
// containing `unix_time` and its neighbours and keep the candidate closest
// to `unix_time` (the log time is a few seconds late, and may have rolled
// into the next minute). Returns null when `toa_s` is absent or implausible.
inline Json time_of_arrival_to_utc(long long unix_time, std::optional<double> toa_s) {
    if (!toa_s) return Json();
    // Python's % is never negative for a positive divisor; C++'s can be, so
    // normalise (only matters for times before 1970).
    long long minute_start = unix_time - (((unix_time % 60) + 60) % 60);
    double best = 0.0;
    double best_distance = -1.0;
    for (int k : {-60, 0, 60}) {
        double candidate = static_cast<double>(minute_start + k) + *toa_s;
        double distance = std::abs(candidate - static_cast<double>(unix_time));
        if (best_distance < 0 || distance < best_distance) {   // Python's min() keeps the first minimum
            best = candidate;
            best_distance = distance;
        }
    }
    if (best_distance > 30) return Json();
    return Json::fixed(best, 3);
}

}  // namespace aistut
