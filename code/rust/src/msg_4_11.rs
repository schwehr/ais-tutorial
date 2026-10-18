//! `msg_4_11.rs` — messages 4 and 11: base station report / UTC-date response
//! (chapter 6).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.2, Table 51 (field layout, 168 bits). The
//! SOTDMA communication state is Annex 2 §3.3.7.2.2 Table 18/19 and is decoded
//! by `decode_sotdma` from `msg_1_2_3` — a message 4 always carries SOTDMA,
//! never ITDMA [Table 51, last row].
//!
//! Message 4 is sent by a base station every 10 s (3 1/3 s when it acts as
//! the synchronisation "semaphore") [Annex 1 §4.2.1 Table 2 note 1; Annex 2
//! §3.1.3.3.1]. Message 11 is the *same layout* sent by a mobile station in
//! answer to a UTC-date inquiry (message 10) [Annex 8 §3.2, §3.9].
//!
//! Bit layout (zero-based offsets; lengths from Table 51):
//!
//! ```text
//!      0- 5   6  message id (4 or 11)
//!      6- 7   2  repeat indicator
//!      8-37  30  user id (MMSI)
//!     38-51  14  UTC year,   1-9999; 0 = n/a
//!     52-55   4  UTC month,  1-12;   0 = n/a; 13-15 not used
//!     56-60   5  UTC day,    1-31;   0 = n/a
//!     61-65   5  UTC hour,   0-23;  24 = n/a; 25-31 not used
//!     66-71   6  UTC minute, 0-59;  60 = n/a; 61-63 not used
//!     72-77   6  UTC second, 0-59;  60 = n/a; 61-63 not used
//!     78      1  position accuracy (1 = high, <= 10 m)
//!     79-106 28  longitude, signed, 1/10 000 minute; 181° (0x6791AC0) = n/a
//!    107-133 27  latitude,  signed, 1/10 000 minute;  91° (0x3412140) = n/a
//!    134-137  4  type of electronic position fixing device (EPFD)
//!    138      1  transmission control for long-range broadcast message
//!    139-147  9  spare
//!    148      1  RAIM flag
//!    149-167 19  SOTDMA communication state
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `use crate::msg_1_2_3::decode_sotdma;` borrows a `pub fn` from a sibling
//!   module. Anything not marked `pub` would be invisible here — Rust's
//!   privacy is per module, enforced by the compiler.
//! * `utc_iso` returns `Option<String>`. `jsonout` implements
//!   `From<Option<T>>` for `Json`, so `put(&mut out, "utc_iso", iso)` prints
//!   either the string or `null` with no `match` at the call site.
//! * `format!("{year:04}")` zero-pads to four digits; the `{name:width}`
//!   syntax inside the braces is the same mini-language Python uses.
//! * `!(1..=9999).contains(&year)` is the idiomatic range test; the `&` is
//!   because `contains` takes a reference.

use crate::bits::Bits;
use crate::common::{epfd_text, require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};
use crate::msg_1_2_3::decode_sotdma;

/// Not-available sentinels for the six UTC fields, Table 51.
pub const UTC_YEAR_NA: u64 = 0;
pub const UTC_MONTH_NA: u64 = 0;
pub const UTC_DAY_NA: u64 = 0;
pub const UTC_HOUR_NA: u64 = 24;
pub const UTC_MINUTE_NA: u64 = 60;
pub const UTC_SECOND_NA: u64 = 60;

/// Transmission control for long-range broadcast message, Table 51.
pub fn transmission_control_text(code: u64) -> &'static str {
    match code {
        0 => "Class A stops message 27 within base station coverage (default)",
        _ => "Class A requested to transmit message 27 within base station coverage", // 1
    }
}

/// Join the six UTC fields into `YYYY-MM-DDTHH:MM:SSZ`, or `None`.
///
/// `None` is returned when *any* component is its not-available value or
/// outside its valid range (Table 51: month 13-15, hour 25-31, minute and
/// second 61-63 are "not used"). We do not check the calendar (31 April
/// passes) — that is left to the reader as an exercise.
pub fn utc_iso(year: u64, month: u64, day: u64, hour: u64, minute: u64, second: u64) -> Option<String> {
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

/// Decode a message 4 or 11. Fails with `DecodeError::TooShort` if the
/// payload has fewer than 168 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 168)?;
    let year = bits.uint(38, 14);
    let month = bits.uint(52, 4);
    let day = bits.uint(56, 5);
    let hour = bits.uint(61, 5);
    let minute = bits.uint(66, 6);
    let second = bits.uint(72, 6);
    let lon_raw = bits.sint(79, 28);
    let lat_raw = bits.sint(107, 27);
    let epfd = bits.uint(134, 4);
    let tx_control = bits.uint(138, 1);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "utc_year", year);
    put(&mut out, "utc_year_available", year != UTC_YEAR_NA);
    put(&mut out, "utc_month", month);
    put(&mut out, "utc_month_available", month != UTC_MONTH_NA);
    put(&mut out, "utc_day", day);
    put(&mut out, "utc_day_available", day != UTC_DAY_NA);
    put(&mut out, "utc_hour", hour);
    put(&mut out, "utc_hour_available", hour != UTC_HOUR_NA);
    put(&mut out, "utc_minute", minute);
    put(&mut out, "utc_minute_available", minute != UTC_MINUTE_NA);
    put(&mut out, "utc_second", second);
    put(&mut out, "utc_second_available", second != UTC_SECOND_NA);
    put(&mut out, "utc_iso", utc_iso(year, month, day, hour, minute, second));
    put(&mut out, "position_accuracy", bits.uint(78, 1));
    put(&mut out, "lon", Json::fixed(lon_raw as f64 / 600000.0, 6));
    put(&mut out, "lat", Json::fixed(lat_raw as f64 / 600000.0, 6));
    put(
        &mut out,
        "position_available",
        lon_raw != 0x6791AC0 && lat_raw != 0x3412140,
    );
    put(&mut out, "epfd", epfd);
    put(&mut out, "epfd_text", epfd_text(epfd));
    put(&mut out, "transmission_control", tx_control);
    put(&mut out, "transmission_control_text", transmission_control_text(tx_control));
    put(&mut out, "spare", bits.uint(139, 9));
    put(&mut out, "raim", bits.flag(148));
    put(&mut out, "comm_state", decode_sotdma(bits, 149));
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_iso_sentinels() {
        assert_eq!(utc_iso(2008, 3, 27, 22, 9, 54).as_deref(), Some("2008-03-27T22:09:54Z"));
        assert_eq!(utc_iso(0, 3, 27, 22, 9, 54), None); // year n/a
        assert_eq!(utc_iso(2008, 3, 27, 24, 9, 54), None); // hour n/a
        assert_eq!(utc_iso(2008, 3, 27, 22, 60, 54), None); // minute n/a
        assert_eq!(utc_iso(2008, 13, 27, 22, 9, 54), None); // month "not used"
    }

    /// Unwrap the top-level object so that tests can index it by key.
    fn obj(bits_payload: &str) -> Obj {
        let bits = Bits::from_payload(bits_payload, 0).unwrap();
        match decode(&bits).unwrap() {
            Json::Obj(o) => o,
            other => panic!("expected an object, got {other:?}"),
        }
    }

    #[test]
    fn surveyed_base_station() {
        // data/ch06-base-station-reports.nmea line 3 (Norfolk, VA, 2007-05-17).
        let json = obj("403OviQuMHQG9rRO9>E6fE700@5r");
        assert_eq!(json["mmsi"], Json::from(3669702u64));
        assert_eq!(json["utc_iso"], Json::from("2007-05-17T01:23:09Z"));
        assert_eq!(json["epfd_text"], Json::from("surveyed"));
        assert_eq!(json["lon"].to_string(), "-76.352362");
        assert_eq!(json["lat"].to_string(), "36.883767");
    }

    #[test]
    fn no_fix_defaults() {
        // data/ch06-base-station-reports.nmea line 16: L-3 base station on the
        // bench without a GNSS antenna.
        let json = obj("4h0C5GQuPun9n<tSF0l4Q@001`;f");
        assert_eq!(json["position_available"], Json::from(false));
        assert_eq!(json["epfd"], Json::from(0u64));
        assert_eq!(json["repeat_indicator"], Json::from(3u64));
    }

    #[test]
    fn too_short() {
        let short = Bits::from_payload(";P@SGS1`i30t", 0).unwrap();
        assert_eq!(
            decode(&short).unwrap_err().to_string(),
            "message 11 too short: 72 bits < 168"
        );
    }
}
