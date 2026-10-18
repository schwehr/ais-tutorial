//! `uscg.rs` — the US Coast Guard "N-AIS" receive metadata (chapter 5).
//!
//! The USCG Research & Development Center's AIS network (and Kurt Schwehr's
//! `serial_logger.py`, which copied the idea) appends extra comma-separated
//! fields *after* the NMEA checksum:
//!
//! ```text
//! !AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,s34486,d-074,T25.49701619,x78124,r003669946,1223901985
//!                                                └ s: signal strength, raw receiver units (0–65535)
//!                                                       └ d: signal strength in dBm
//!                                                              └ T: time of arrival, seconds into the UTC minute
//!                                                                           └ x: receiver's sentence counter
//!                                                                                   └ r: receiving station id
//!                                                                                                └ UNIX time (s)
//! ```
//!
//! The format was never published as a standard; it is described in the GPSD
//! AIVDM.adoc "U.S. Coast Guard Extended AIVDM" section and in the AISUser
//! manual (output format 2, "proprietary USCG R&D Center format"). Field
//! order varies and most fields are optional; only the station id and the
//! final time are always present. Known prefixes:
//!
//! ```text
//! s   received signal strength indicator (RSSI), receiver-specific units
//! d   signal strength, dBm (negative number)
//! T   time of arrival within the minute (fractional seconds)
//! S   slot number (0–2249) in which the message was received
//! x   sentence counter from the receiver
//! r   receiving station id (`b` is also seen for base stations)
//! (none) a bare number: UNIX time in seconds, possibly with a fraction
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `item.chars().next()` gives the first *character* (an `Option<char>`,
//!   because the string might be empty); `&item[1..]` would be a byte
//!   offset and could split a multi-byte character. We use
//!   `split_at(first.len_utf8())` to be safe.
//! * `if let Some(v) = … { } else if … { }` is a chain of pattern tests;
//!   each arm that parses successfully records a value, the final `else`
//!   keeps the item verbatim in `unparsed` so no information is lost.
//! * Python's `int()`/`float()` acceptance rules are reproduced by the
//!   helpers in `nmea.rs` (`py_int_radix`, `py_float`) so the two programs
//!   classify odd items the same way.

use crate::jsonout::{put, Json, Obj};
use crate::nmea::{is_ascii_digits, py_float, py_int_radix};

/// Python's `rest.lstrip("-").isdigit()`.
fn is_signed_digits(rest: &str) -> bool {
    is_ascii_digits(rest.trim_start_matches('-'))
}

/// Interpret the fields that followed the checksum.
///
/// Unknown or malformed items are kept verbatim in `unparsed` so that no
/// information is silently lost.
pub fn parse_uscg_fields(extras: &[String]) -> Obj {
    let mut out = Obj::new();
    let mut unparsed: Vec<Json> = Vec::new();
    for item in extras {
        let Some(first) = item.chars().next() else {
            continue; // empty item
        };
        let (code, rest) = item.split_at(first.len_utf8());
        // Each branch mirrors one `elif` of the Python version. A branch is
        // only taken when its parse succeeds; otherwise control falls to
        // the generic bare-number test and finally to `unparsed`.
        let parsed = match code {
            "s" if is_signed_digits(rest) => py_int_radix(rest, 10).map(|v| put(&mut out, "rssi", v)),
            "d" if is_signed_digits(rest) => {
                py_int_radix(rest, 10).map(|v| put(&mut out, "signal_dbm", v))
            }
            "T" => py_float(rest)
                .ok()
                .map(|v| put(&mut out, "time_of_arrival_s", Json::fixed(v, 8))),
            "S" if is_ascii_digits(rest) => py_int_radix(rest, 10).map(|v| put(&mut out, "slot", v)),
            "x" if is_ascii_digits(rest) => {
                py_int_radix(rest, 10).map(|v| put(&mut out, "sentence_counter", v))
            }
            "r" | "b" if !rest.is_empty() => {
                put(&mut out, "station", rest);
                put(&mut out, "station_kind", if code == "b" { "base" } else { "receiver" });
                Some(())
            }
            _ => parse_bare_number(item).map(|v| put(&mut out, "time", v)),
        };
        if parsed.is_none() {
            unparsed.push(Json::from(item.as_str()));
        }
    }
    if !unparsed.is_empty() {
        put(&mut out, "unparsed", unparsed);
    }
    out
}

/// Python: `item.replace(".", "", 1).isdigit()` then `float(item)` if it has
/// a dot (printed with 2 decimals) else `int(item)`.
fn parse_bare_number(item: &str) -> Option<Json> {
    if !is_ascii_digits(&item.replacen('.', "", 1)) {
        return None;
    }
    if item.contains('.') {
        py_float(item).ok().map(|v| Json::fixed(v, 2))
    } else {
        py_int_radix(item, 10).map(Json::Int)
    }
}

/// Combine the coarse UNIX time with the fine `T` field.
///
/// `T` is seconds past the start of the UTC minute measured by the
/// receiver's clock, so the precise arrival time is "start of some UTC
/// minute + T". The logging computer's `unix_time` tells us *which*
/// minute: we try the minute containing `unix_time` and its neighbours
/// and keep the candidate closest to `unix_time` (the log time is a few
/// seconds late, and may have rolled into the next minute).
pub fn time_of_arrival_to_utc(unix_time: i64, toa_s: Option<f64>) -> Option<Json> {
    let toa_s = toa_s?;
    let minute_start = (unix_time - unix_time.rem_euclid(60)) as f64;
    let unix = unix_time as f64;
    let mut precise = minute_start - 60.0 + toa_s;
    for k in [0.0, 60.0] {
        let candidate = minute_start + k + toa_s;
        // Strictly closer only, so ties keep the earlier candidate like
        // Python's `min`.
        if (candidate - unix).abs() < (precise - unix).abs() {
            precise = candidate;
        }
    }
    if (precise - unix).abs() > 30.0 {
        return None;
    }
    Some(Json::fixed(precise, 3))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn typical_line() {
        let out = parse_uscg_fields(&fields(&[
            "s34486", "d-074", "T25.49701619", "x78124", "r003669946", "1223901985",
        ]));
        assert_eq!(
            Json::Obj(out).to_string(),
            r#"{"rssi":34486,"sentence_counter":78124,"signal_dbm":-74,"station":"003669946","station_kind":"receiver","time":1223901985,"time_of_arrival_s":25.49701619}"#
        );
    }

    #[test]
    fn odd_items() {
        let out = parse_uscg_fields(&fields(&["", "b42", "1179170462.74", "T", "s--5", "zzz"]));
        assert_eq!(
            Json::Obj(out).to_string(),
            r#"{"station":"42","station_kind":"base","time":1179170462.74,"unparsed":["T","s--5","zzz"]}"#
        );
    }

    #[test]
    fn toa() {
        assert_eq!(time_of_arrival_to_utc(1223901985, Some(25.5)).unwrap().to_string(), "1223901985.500");
        assert_eq!(time_of_arrival_to_utc(1223901985, None), None);
    }
}
