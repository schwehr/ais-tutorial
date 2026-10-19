//! `asm.rs` — application-specific message (ASM) payloads, keyed by (DAC, FI).
//!
//! Messages 6 and 8 carry a 16-bit application identifier — a 10-bit
//! designated area code (DAC) and a 6-bit function identifier (FI) — followed
//! by data whose layout is defined by whoever owns that DAC
//! [ITU-R M.1371-5 Annex 5 §2.1 Table 23]. This module is the registry that
//! turns a known (dac, fi) pair into decoded fields; `msg_6.rs` and
//! `msg_8.rs` call [`decode`] and attach a `Some` result as `"application"`.
//! The Python original (`aistut/asm.py`) documents every bit offset.
//!
//! Implemented (chapter 9):
//!
//! | (dac, fi) | what | source |
//! |---|---|---|
//! | (1, 0) | IFM 0 text telegram | M.1371-5 Annex 5 §5.1 Tables 25/26 |
//! | (1, 11) | met/hydro, IMO SN/Circ.236 (2004) | Circ.236 trial message 1 (AIVDM "Type 8(1/11)") |
//! | (1, 31) | met/hydro, IMO SN.1/Circ.289 (2010) | Circ.289 Annex Table 1.1 |
//! | (366, 34) | USCG/NOAA right-whale zone notice (2008) | reverse-engineered, see chapter 9 |
//!
//! Output conventions: a not-available sensor value prints as `null` (no
//! `_available` companion); enumerations keep their `_text` companion; a
//! payload that is too short yields `{"application_name", "error"}`.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `fn(&Bits, usize, usize) -> Obj` is a *function pointer type*; plain
//!   functions coerce to it, so a `match` can hand back the right decoder
//!   for a `(dac, fi)` tuple — Rust's way of writing Python's dictionary of
//!   functions. Matching on a tuple, `match (dac, fi) { (1, 0) => …, _ => … }`,
//!   needs no hashing at all.
//! * `Option<u64>` and `Option<Json>` map to `null` through the blanket
//!   `From<Option<T>>` conversion in `jsonout.rs`, so a helper can simply
//!   return `None` for a not-available sentinel.

use crate::bits::Bits;
use crate::jsonout::{put, Json, Obj};

// --- small helpers ---------------------------------------------------------

/// `raw` unless it equals the not-available sentinel, in which case `None`.
fn na(raw: u64, sentinel: u64) -> Option<u64> {
    if raw == sentinel {
        None
    } else {
        Some(raw)
    }
}

/// raw / divisor + offset with `decimals` places, or `None` if raw == sentinel.
fn scaled(raw: i64, sentinel: i64, divisor: f64, decimals: usize, offset: f64) -> Option<Json> {
    if raw == sentinel {
        None
    } else {
        Some(Json::fixed(raw as f64 / divisor + offset, decimals))
    }
}

pub fn tendency_text(code: u64) -> &'static str {
    match code {
        0 => "steady",
        1 => "decreasing",
        2 => "increasing",
        _ => "not available",
    }
}

/// WMO 306 code table 4.201 as reproduced in Circ.289 Table 1.1.
pub fn precipitation_text(code: u64) -> &'static str {
    match code {
        1 => "rain",
        2 => "thunderstorm",
        3 => "freezing rain",
        4 => "mixed/ice",
        5 => "snow",
        7 => "not available",
        _ => "reserved", // 0, 6
    }
}

/// Beaufort scale, Circ.289 Table 1.2.
pub fn sea_state_text(code: u64) -> &'static str {
    match code {
        0 => "calm (flat)",
        1 => "light air (ripples)",
        2 => "light breeze (small wavelets)",
        3 => "gentle breeze (large wavelets)",
        4 => "moderate breeze (small waves)",
        5 => "fresh breeze (moderate waves)",
        6 => "strong breeze (large waves)",
        7 => "near gale (sea heaps up)",
        8 => "gale (moderately high waves)",
        9 => "strong gale (high waves)",
        10 => "storm (very high waves)",
        11 => "violent storm (exceptionally high waves)",
        12 => "hurricane (huge waves)",
        13 => "not available",
        _ => "reserved", // 14, 15
    }
}

pub fn ice_text(code: u64) -> &'static str {
    match code {
        0 => "no",
        1 => "yes",
        2 => "reserved",
        _ => "not available",
    }
}

// --- (1, 0) IFM 0: text telegram (Annex 5 Tables 25/26) --------------------

pub fn decode_text_telegram(bits: &Bits, o: usize, data_bits: usize) -> Obj {
    let text_bits = (data_bits - 12) / 6 * 6;
    let mut out = Obj::new();
    put(&mut out, "ack_required", bits.flag(o));
    put(&mut out, "text_sequence_number", bits.uint(o + 1, 11));
    put(&mut out, "text", bits.text(o + 12, text_bits));
    put(&mut out, "text_chars", text_bits / 6);
    put(&mut out, "spare_bits", data_bits - 12 - text_bits);
    out
}

// --- (1, 11) IMO SN/Circ.236 met/hydro, 296 data bits ----------------------
// Latitude before longitude; temperatures and water level are offset-coded
// (value/10 - 60, - 20, - 10); every not-available value is the field maximum.

pub const MET_HYDRO_236_BITS: usize = 296;

pub fn decode_met_hydro_236(bits: &Bits, o: usize, _data_bits: usize) -> Obj {
    let lat_raw = bits.sint(o, 24);
    let lon_raw = bits.sint(o + 24, 25);
    let tendency = bits.uint(o + 134, 2);
    let level_trend = bits.uint(o + 153, 2);
    let sea_state = bits.uint(o + 262, 4);
    let precip = bits.uint(o + 276, 3);
    let ice = bits.uint(o + 288, 2);
    let pressure_raw = bits.uint(o + 125, 9);
    let u = |start: usize, len: usize| bits.uint(o + start, len) as i64;

    let mut out = Obj::new();
    put(&mut out, "lat", if lat_raw == 0x7F_FFFF { None } else { Some(Json::fixed(lat_raw as f64 / 60000.0, 6)) });
    put(&mut out, "lon", if lon_raw == 0xFF_FFFF { None } else { Some(Json::fixed(lon_raw as f64 / 60000.0, 6)) });
    put(&mut out, "day", na(bits.uint(o + 49, 5), 31));
    put(&mut out, "hour", na(bits.uint(o + 54, 5), 31));
    put(&mut out, "minute", na(bits.uint(o + 59, 6), 63));
    put(&mut out, "wind_speed_knots", na(bits.uint(o + 65, 7), 127));
    put(&mut out, "wind_gust_knots", na(bits.uint(o + 72, 7), 127));
    put(&mut out, "wind_dir_deg", na(bits.uint(o + 79, 9), 511));
    put(&mut out, "wind_gust_dir_deg", na(bits.uint(o + 88, 9), 511));
    put(&mut out, "air_temp_c", scaled(u(97, 11), 2047, 10.0, 1, -60.0));
    put(&mut out, "humidity_pct", na(bits.uint(o + 108, 7), 127));
    put(&mut out, "dew_point_c", scaled(u(115, 10), 1023, 10.0, 1, -20.0));
    put(&mut out, "pressure_hpa", if pressure_raw == 511 { None } else { Some(pressure_raw + 800) });
    put(&mut out, "pressure_tendency", tendency);
    put(&mut out, "pressure_tendency_text", tendency_text(tendency));
    put(&mut out, "visibility_nm", scaled(u(136, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "water_level_m", scaled(u(144, 9), 511, 10.0, 1, -10.0));
    put(&mut out, "water_level_trend", level_trend);
    put(&mut out, "water_level_trend_text", tendency_text(level_trend));
    put(&mut out, "current_speed_knots", scaled(u(155, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "current_dir_deg", na(bits.uint(o + 163, 9), 511));
    put(&mut out, "current_2_speed_knots", scaled(u(172, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "current_2_dir_deg", na(bits.uint(o + 180, 9), 511));
    put(&mut out, "current_2_depth_m", na(bits.uint(o + 189, 5), 31));
    put(&mut out, "current_3_speed_knots", scaled(u(194, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "current_3_dir_deg", na(bits.uint(o + 202, 9), 511));
    put(&mut out, "current_3_depth_m", na(bits.uint(o + 211, 5), 31));
    put(&mut out, "wave_height_m", scaled(u(216, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "wave_period_s", na(bits.uint(o + 224, 6), 63));
    put(&mut out, "wave_dir_deg", na(bits.uint(o + 230, 9), 511));
    put(&mut out, "swell_height_m", scaled(u(239, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "swell_period_s", na(bits.uint(o + 247, 6), 63));
    put(&mut out, "swell_dir_deg", na(bits.uint(o + 253, 9), 511));
    put(&mut out, "sea_state", sea_state);
    put(&mut out, "sea_state_text", sea_state_text(sea_state));
    put(&mut out, "water_temp_c", scaled(u(266, 10), 1023, 10.0, 1, -10.0));
    put(&mut out, "precipitation", precip);
    put(&mut out, "precipitation_text", precipitation_text(precip));
    put(&mut out, "salinity_ppt", scaled(u(279, 9), 511, 10.0, 1, 0.0));
    put(&mut out, "ice", ice);
    put(&mut out, "ice_text", ice_text(ice));
    put(&mut out, "spare", bits.uint(o + 290, 6));
    out
}

// --- (1, 31) IMO SN.1/Circ.289 met/hydro, 304 data bits --------------------

pub const MET_HYDRO_289_BITS: usize = 304;
pub const LON_1000_NOT_AVAILABLE: i64 = 181 * 60000; // 10 860 000
pub const LAT_1000_NOT_AVAILABLE: i64 = 91 * 60000; //  5 460 000

/// Circ.289 air pressure: 1-401 map to 800-1200 hPa; the ends are clamped.
fn pressure_289(raw: u64) -> Option<u64> {
    match raw {
        511 => None,
        0 => Some(799),          // "pressure 799 hPa or less"
        402..=510 => Some(1201), // "pressure 1201 hPa or greater"
        _ => Some(raw + 799),
    }
}

pub fn decode_met_hydro_289(bits: &Bits, o: usize, _data_bits: usize) -> Obj {
    let lon_raw = bits.sint(o, 25);
    let lat_raw = bits.sint(o + 25, 24);
    let tendency = bits.uint(o + 135, 2);
    let vis_raw = bits.uint(o + 137, 8);
    let level_trend = bits.uint(o + 157, 2);
    let sea_state = bits.uint(o + 266, 4);
    let precip = bits.uint(o + 280, 3);
    let salinity_raw = bits.uint(o + 283, 9);
    let ice = bits.uint(o + 292, 2);
    let u = |start: usize, len: usize| bits.uint(o + start, len) as i64;

    let mut out = Obj::new();
    put(&mut out, "lon", if lon_raw == LON_1000_NOT_AVAILABLE { None } else { Some(Json::fixed(lon_raw as f64 / 60000.0, 6)) });
    put(&mut out, "lat", if lat_raw == LAT_1000_NOT_AVAILABLE { None } else { Some(Json::fixed(lat_raw as f64 / 60000.0, 6)) });
    put(&mut out, "position_accuracy", bits.uint(o + 49, 1));
    put(&mut out, "day", na(bits.uint(o + 50, 5), 0));
    put(&mut out, "hour", na(bits.uint(o + 55, 5), 24));
    put(&mut out, "minute", na(bits.uint(o + 60, 6), 60));
    put(&mut out, "wind_speed_knots", na(bits.uint(o + 66, 7), 127));
    put(&mut out, "wind_gust_knots", na(bits.uint(o + 73, 7), 127));
    put(&mut out, "wind_dir_deg", na(bits.uint(o + 80, 9), 360));
    put(&mut out, "wind_gust_dir_deg", na(bits.uint(o + 89, 9), 360));
    put(&mut out, "air_temp_c", scaled(bits.sint(o + 98, 11), -1024, 10.0, 1, 0.0));
    put(&mut out, "humidity_pct", na(bits.uint(o + 109, 7), 101));
    put(&mut out, "dew_point_c", scaled(bits.sint(o + 116, 10), 501, 10.0, 1, 0.0));
    put(&mut out, "pressure_hpa", pressure_289(bits.uint(o + 126, 9)));
    put(&mut out, "pressure_tendency", tendency);
    put(&mut out, "pressure_tendency_text", tendency_text(tendency));
    put(&mut out, "visibility_nm", scaled((vis_raw & 0x7F) as i64, 127, 10.0, 1, 0.0));
    put(&mut out, "visibility_greater_than", (vis_raw >> 7) == 1);
    put(&mut out, "water_level_m", scaled(u(145, 12), 4001, 100.0, 2, -10.0));
    put(&mut out, "water_level_trend", level_trend);
    put(&mut out, "water_level_trend_text", tendency_text(level_trend));
    put(&mut out, "current_speed_knots", scaled(u(159, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "current_dir_deg", na(bits.uint(o + 167, 9), 360));
    put(&mut out, "current_2_speed_knots", scaled(u(176, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "current_2_dir_deg", na(bits.uint(o + 184, 9), 360));
    put(&mut out, "current_2_depth_m", na(bits.uint(o + 193, 5), 31));
    put(&mut out, "current_3_speed_knots", scaled(u(198, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "current_3_dir_deg", na(bits.uint(o + 206, 9), 360));
    put(&mut out, "current_3_depth_m", na(bits.uint(o + 215, 5), 31));
    put(&mut out, "wave_height_m", scaled(u(220, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "wave_period_s", na(bits.uint(o + 228, 6), 63));
    put(&mut out, "wave_dir_deg", na(bits.uint(o + 234, 9), 360));
    put(&mut out, "swell_height_m", scaled(u(243, 8), 255, 10.0, 1, 0.0));
    put(&mut out, "swell_period_s", na(bits.uint(o + 251, 6), 63));
    put(&mut out, "swell_dir_deg", na(bits.uint(o + 257, 9), 360));
    put(&mut out, "sea_state", sea_state);
    put(&mut out, "sea_state_text", sea_state_text(sea_state));
    put(&mut out, "water_temp_c", scaled(bits.sint(o + 270, 10), 501, 10.0, 1, 0.0));
    put(&mut out, "precipitation", precip);
    put(&mut out, "precipitation_text", precipitation_text(precip));
    put(&mut out, "salinity_ppt", if salinity_raw >= 510 { None } else { Some(Json::fixed(salinity_raw as f64 / 10.0, 1)) });
    put(&mut out, "salinity_sensor_available", salinity_raw != 511);
    put(&mut out, "ice", ice);
    put(&mut out, "ice_text", ice_text(ice));
    put(&mut out, "spare", bits.uint(o + 294, 10));
    out
}

// --- (366, 34) USCG/NOAA right-whale zone notice, 2008 form ----------------
// Reverse-engineered (chapter 9): a sequence of 112-bit records, each a
// 36-bit header (zone id 5, zone type 7, day 5, hour 5, minute 6, duration
// in 10-minute units 8) and one 76-bit circle (shape 1, lon 28, lat 27,
// radius 16, spare 4).

pub const WHALE_RECORD_BITS: usize = 112;

pub fn whale_zone_type_text(code: u64) -> &'static str {
    match code {
        0 => "right whales not observed",
        1 => "right whales present",
        _ => "unknown",
    }
}

pub fn decode_whale_notice(bits: &Bits, o: usize, data_bits: usize) -> Obj {
    let mut notices: Vec<Json> = Vec::new();
    let mut r = o;
    while bits.has(r, WHALE_RECORD_BITS) {
        let zone_type = bits.uint(r + 5, 7);
        let duration_raw = bits.uint(r + 28, 8);
        let z = r + 36;
        let mut n = Obj::new();
        put(&mut n, "zone_id", bits.uint(r, 5));
        put(&mut n, "zone_type", zone_type);
        put(&mut n, "zone_type_text", whale_zone_type_text(zone_type));
        put(&mut n, "start_day", bits.uint(r + 12, 5));
        put(&mut n, "start_hour", bits.uint(r + 17, 5));
        put(&mut n, "start_minute", bits.uint(r + 22, 6));
        put(&mut n, "duration_raw", duration_raw);
        put(&mut n, "duration_minutes", duration_raw * 10);
        put(&mut n, "shape", bits.uint(z, 1));
        put(&mut n, "lon", Json::fixed(bits.sint(z + 1, 28) as f64 / 600000.0, 6));
        put(&mut n, "lat", Json::fixed(bits.sint(z + 29, 27) as f64 / 600000.0, 6));
        put(&mut n, "radius_m", bits.uint(z + 56, 16));
        put(&mut n, "spare", bits.uint(z + 72, 4));
        notices.push(Json::Obj(n));
        r += WHALE_RECORD_BITS;
    }
    let mut out = Obj::new();
    put(&mut out, "notices", notices);
    put(&mut out, "trailing_bits", o + data_bits - r);
    out
}

// --- the registry ----------------------------------------------------------

/// A decoder takes the bits, the offset of the first data bit and the number
/// of data bits, and returns the decoded fields.
pub type AsmDecoder = fn(&Bits, usize, usize) -> Obj;

/// (name, minimum data bits, decoder) for a known (dac, fi); `None` otherwise.
pub fn lookup(dac: u64, fi: u64) -> Option<(&'static str, usize, AsmDecoder)> {
    match (dac, fi) {
        (1, 0) => Some(("IFM 0 text telegram", 12, decode_text_telegram)),
        (1, 11) => Some((
            "IMO Circ.236 meteorological and hydrological data",
            MET_HYDRO_236_BITS,
            decode_met_hydro_236,
        )),
        (1, 31) => Some((
            "IMO Circ.289 meteorological and hydrographic data",
            MET_HYDRO_289_BITS,
            decode_met_hydro_289,
        )),
        (366, 34) => Some((
            "USCG/NOAA right whale zone notice (2008)",
            WHALE_RECORD_BITS,
            decode_whale_notice,
        )),
        _ => None,
    }
}

/// Decode the application data that starts at `offset`.
///
/// Returns `None` when (dac, fi) is unknown. For a known pair the object
/// always carries `application_name`; if the payload is shorter than the
/// layout needs it carries an `error` text instead of the fields.
pub fn decode(dac: u64, fi: u64, bits: &Bits, offset: usize) -> Option<Json> {
    let (name, need, func) = lookup(dac, fi)?;
    let data_bits = bits.len() - offset;
    let mut out = Obj::new();
    put(&mut out, "application_name", name);
    if data_bits < need {
        put(&mut out, "error", format!("needs {need} data bits, have {data_bits}"));
        return Some(Json::Obj(out));
    }
    out.extend(func(bits, offset, data_bits));
    Some(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whale_notice_record() {
        // data/ch08-binary-framing.nmea line 8: one 112-bit record.
        let bits = Bits::from_payload("803OvqQK`PP7fb0Fg7GI63do4T;0", 0).unwrap();
        let json = decode(366, 34, &bits, 56).unwrap().to_string();
        assert!(json.contains("\"zone_id\":1"));
        assert!(json.contains("\"lon\":-70.678317"));
        assert!(json.contains("\"radius_m\":9260"));
        assert!(json.contains("\"trailing_bits\":0"));
    }

    #[test]
    fn unknown_and_short() {
        let bits = Bits::from_payload("803OvqQK`PP7fb0Fg7GI63do4T;0", 0).unwrap();
        assert!(decode(366, 56, &bits, 56).is_none());
        let json = decode(1, 31, &bits, 56).unwrap().to_string();
        assert!(json.contains("needs 304 data bits, have 112"));
    }
}
