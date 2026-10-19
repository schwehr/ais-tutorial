//! `msg_22.rs` — message 22: channel management (chapter 13).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.20, Table 75 (168 bits). A base station
//! tells mobile stations which two VHF channels, transmit/receive mode and
//! power level to use, either inside a rectangular region (broadcast form)
//! or for up to two individually addressed stations. Regions and the
//! transitional-zone behaviour are defined in Annex 2 §4.1; channel numbers
//! follow Recommendation ITU-R M.1084 Annex 4 (four-digit form).
//!
//! Bit layout (zero-based offsets; lengths from Table 75):
//!
//! ```text
//!      0- 5   6  message id (22)
//!      6- 7   2  repeat indicator
//!      8-37  30  station id (MMSI of the base station)
//!     38-39   2  spare
//!     40-51  12  channel A number (M.1084)
//!     52-63  12  channel B number (M.1084)
//!     64-67   4  Tx/Rx mode (0 = TxA/TxB RxA/RxB, 1 = TxA RxA/RxB, 2 = TxB RxA/RxB)
//!     68      1  power (0 = high, 1 = low)
//!     69- 86  18  longitude 1 (north-east corner), 1/10 min, 181° = n/a   | or 30-bit MMSI 1 at 69-98
//!     87-103  17  latitude 1 (north-east corner), 1/10 min, 91° = n/a     |    followed by 5 zero bits
//!    104-121  18  longitude 2 (south-west corner)                          | or 30-bit MMSI 2 at 104-133
//!    122-138  17  latitude 2 (south-west corner)                           |    followed by 5 zero bits
//!    139      1  addressed (1) or broadcast/geographical (0) — decides how 69-138 are read
//!    140      1  channel A bandwidth (0 = default; 1 = spare, formerly 12.5 kHz)
//!    141      1  channel B bandwidth
//!    142-144  3  transitional zone size, nautical miles = value + 1 (default 4 → 5 NM)
//!    145-167 23  spare
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `channel_frequency_khz` returns `Option<u64>`: `None` means "not a
//!   channel number we know". `?` is for `Result`, so inside the function
//!   we use early `return None` instead.
//! * `matches!(base, 6 | 8..=17 | 67..=77)` is a macro that evaluates a
//!   pattern to a `bool` — the Rust spelling of Python's `x in some_set`
//!   for a fixed set of ranges.
//! * `Option<Json>` and `Option<u64>` both convert to JSON `null` when
//!   `None`, via the generic `From<Option<T>>` impl.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

pub const LON_NOT_AVAILABLE: i64 = 181 * 600; // 0x1A838, 1/10-minute units
pub const LAT_NOT_AVAILABLE: i64 = 91 * 600; // 0x0D548

/// Table 75. The field is 4 bits wide but only 0–2 are defined.
pub fn txrx_mode_text(code: u64) -> &'static str {
    match code {
        0 => "TxA/TxB, RxA/RxB",
        1 => "TxA, RxA/RxB",
        2 => "TxB, RxA/RxB",
        _ => "not used",
    }
}

/// RR Appendix 18 channels that are simplex (one frequency, "ship & coast"):
/// 6, 8–17 and 67–77. Everything else in 1–28 / 60–88 is a duplex pair.
pub fn is_simplex_channel(base: u64) -> bool {
    matches!(base, 6 | 8..=17 | 67..=77)
}

/// Carrier frequency in kHz for an ITU-R M.1084 Annex 4 channel number.
///
/// Two-digit numbers are the RR Appendix 18 channels 1–28 and 60–88; ship
/// frequencies are 156.050 + 0.050 (n-1) MHz for 1–28 and
/// 156.025 + 0.050 (n-60) MHz for 60–88; the coast frequency of a duplex
/// channel is 4.6 MHz higher. A leading 1 means "simplex use of the ship
/// frequency", a leading 2 "simplex use of the coast frequency" (M.1084
/// Annex 4 §3): 2087 = 161.975 MHz (AIS 1), 2088 = 162.025 MHz (AIS 2).
/// Returns `None` for numbers outside those ranges.
pub fn channel_frequency_khz(number: u64) -> Option<u64> {
    let prefix = number / 1000;
    let base = number % 1000;
    if prefix > 2 {
        return None;
    }
    let ship = match base {
        1..=28 => 156_050 + 50 * (base - 1),
        60..=88 => 156_025 + 50 * (base - 60),
        _ => return None,
    };
    let coast = if is_simplex_channel(base) { ship } else { ship + 4600 };
    Some(if prefix == 2 { coast } else { ship })
}

/// `channel_frequency_khz` as a JSON number with three decimals (MHz), or null.
pub fn channel_mhz(number: u64) -> Json {
    match channel_frequency_khz(number) {
        Some(khz) => Json::fixed(khz as f64 / 1000.0, 3),
        None => Json::Null,
    }
}

/// Decode a message 22. Fails with `DecodeError::TooShort` below 168 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(22, bits.len(), 168)?;
    let channel_a = bits.uint(40, 12);
    let channel_b = bits.uint(52, 12);
    let txrx = bits.uint(64, 4);
    let power = bits.uint(68, 1);
    let addressed = bits.flag(139);
    let zone = bits.uint(142, 3);
    let mut out = Obj::new();
    put(&mut out, "msg_type", bits.uint(0, 6));
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    put(&mut out, "channel_a", channel_a);
    put(&mut out, "channel_a_mhz", channel_mhz(channel_a));
    put(&mut out, "channel_b", channel_b);
    put(&mut out, "channel_b_mhz", channel_mhz(channel_b));
    put(&mut out, "txrx_mode", txrx);
    put(&mut out, "txrx_mode_text", txrx_mode_text(txrx));
    put(&mut out, "power", power);
    put(&mut out, "power_text", if power == 0 { "high" } else { "low" });
    put(&mut out, "addressed", addressed);
    put(&mut out, "band_a", bits.uint(140, 1));
    put(&mut out, "band_b", bits.uint(141, 1));
    put(&mut out, "zone_size", zone);
    put(&mut out, "zone_size_nm", zone + 1);
    put(&mut out, "spare2", bits.uint(145, 23));
    if addressed {
        // Table 75: the 30-bit MMSI occupies the 18-bit longitude field and
        // the first 12 bits of the 17-bit latitude field; 5 zero bits follow.
        put(&mut out, "dest_mmsi_1", bits.uint(69, 30));
        put(&mut out, "dest_spare_1", bits.uint(99, 5));
        put(&mut out, "dest_mmsi_2", bits.uint(104, 30));
        put(&mut out, "dest_spare_2", bits.uint(134, 5));
    } else {
        let ne_lon = bits.sint(69, 18);
        let ne_lat = bits.sint(87, 17);
        let sw_lon = bits.sint(104, 18);
        let sw_lat = bits.sint(122, 17);
        put(&mut out, "ne_lon", Json::fixed(ne_lon as f64 / 600.0, 6));
        put(&mut out, "ne_lat", Json::fixed(ne_lat as f64 / 600.0, 6));
        put(&mut out, "sw_lon", Json::fixed(sw_lon as f64 / 600.0, 6));
        put(&mut out, "sw_lat", Json::fixed(sw_lat as f64 / 600.0, 6));
        put(
            &mut out,
            "region_available",
            ne_lon != LON_NOT_AVAILABLE
                && ne_lat != LAT_NOT_AVAILABLE
                && sw_lon != LON_NOT_AVAILABLE
                && sw_lat != LAT_NOT_AVAILABLE,
        );
    }
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequencies() {
        assert_eq!(channel_frequency_khz(2087), Some(161_975));
        assert_eq!(channel_frequency_khz(2088), Some(162_025));
        assert_eq!(channel_frequency_khz(1075), Some(156_775));
        assert_eq!(channel_frequency_khz(87), Some(157_375));
        assert_eq!(channel_frequency_khz(16), Some(156_800));
        assert_eq!(channel_frequency_khz(2016), Some(156_800));
        assert_eq!(channel_frequency_khz(0), None);
        assert_eq!(channel_frequency_khz(4087), None);
    }

    #[test]
    fn broadcast_defaults() {
        // sample-data/2007hydro/norfolk-log.ais line 115 (own-ship VDO)
        let bits = Bits::from_payload("F52HH<j2N2P3D73EB6`>6bT20000", 0).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["channel_a"], Json::Int(2087));
        assert_eq!(out["channel_b"], Json::Int(2088));
        assert_eq!(out["addressed"], Json::Bool(false));
        assert_eq!(out["region_available"], Json::Bool(false));
        assert_eq!(out["zone_size_nm"], Json::Int(5));
    }

    #[test]
    fn addressed() {
        // synthetic sentence from chapter 13
        let bits = Bits::from_payload("F03OviQvevh2fwrs85MwmnPB0000", 0).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["addressed"], Json::Bool(true));
        assert_eq!(out["dest_mmsi_1"], Json::Int(366999001));
        assert_eq!(out["dest_mmsi_2"], Json::Int(366999002));
        assert!(!out.contains_key("ne_lon"));
    }
}
