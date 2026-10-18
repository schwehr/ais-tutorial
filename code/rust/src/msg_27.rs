//! `msg_27.rs` — message 27: long-range AIS broadcast message (chapter 16).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.25, Table 84 (96 bits); Annex 4 §3
//! (long-range applications by broadcast: channels 75/76, 3-minute
//! interval, Table 22 packet structure); Table 51 (message 4 "transmission
//! control for long-range broadcast message" bit).
//!
//! Bit layout (zero-based offsets; lengths from Table 84):
//!
//! ```text
//!      0- 5   6  message id (27)
//!      6- 7   2  repeat indicator ("always 3": do not repeat)
//!      8-37  30  user id (MMSI)
//!     38      1  position accuracy (as message 1)
//!     39      1  RAIM flag (as message 1)
//!     40-43   4  navigational status (as message 1)
//!     44-61  18  longitude, signed, 1/10 minute; 181° (0x1A838) = n/a
//!     62-78  17  latitude,  signed, 1/10 minute;  91° (0xD548)  = n/a
//!     79-84   6  speed over ground, whole knots 0–62; 63 = n/a
//!     85-93   9  course over ground, whole degrees 0–359; 511 = n/a
//!     94      1  position latency: 0 = < 5 s, 1 = > 5 s (default)
//!                (M.1371-4 called this bit "status of current GNSS position")
//!     95      1  spare
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * The sentinels are `const … : i64` because they are compared with the
//!   signed result of `bits.sint`; Rust refuses to compare `i64` with `u64`
//!   without an explicit cast, which is exactly the kind of mistake the
//!   type system is there to catch.
//! * `nav_status_text` is reused from `msg_1_2_3` — one table, one truth.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};
use crate::msg_1_2_3::nav_status_text;

/// 181 degrees * 600 — longitude not available (Table 84).
pub const LON_NOT_AVAILABLE: i64 = 0x1A838;
/// 91 degrees * 600 — latitude not available (Table 84).
pub const LAT_NOT_AVAILABLE: i64 = 0xD548;

/// Position latency flag, Table 84.
pub fn position_latency_text(code: u64) -> &'static str {
    match code {
        0 => "less than 5 s",
        _ => "greater than 5 s (default)", // 1
    }
}

/// Decode a message 27. Fails with `DecodeError::TooShort` if the payload
/// has fewer than 96 bits. Longer payloads are decoded from their first 96
/// bits; the driver's `payload_bits` tells the caller that something was odd.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(27, bits.len(), 96)?;
    let nav = bits.uint(40, 4);
    let lon_raw = bits.sint(44, 18);
    let lat_raw = bits.sint(62, 17);
    let sog = bits.uint(79, 6);
    let cog = bits.uint(85, 9);
    let latency = bits.uint(94, 1);

    let mut out = Obj::new();
    put(&mut out, "msg_type", 27);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "position_accuracy", bits.uint(38, 1));
    put(&mut out, "raim", bits.flag(39));
    put(&mut out, "nav_status", nav);
    put(&mut out, "nav_status_text", nav_status_text(nav));
    put(&mut out, "lon", Json::fixed(lon_raw as f64 / 600.0, 6));
    put(&mut out, "lat", Json::fixed(lat_raw as f64 / 600.0, 6));
    put(
        &mut out,
        "position_available",
        lon_raw != LON_NOT_AVAILABLE && lat_raw != LAT_NOT_AVAILABLE,
    );
    put(&mut out, "sog_knots", sog);
    put(&mut out, "sog_available", sog != 63);
    put(&mut out, "cog_deg", cog);
    put(&mut out, "cog_available", cog != 511);
    put(&mut out, "position_latency", latency);
    put(&mut out, "position_latency_text", position_latency_text(latency));
    put(&mut out, "spare", bits.uint(95, 1));
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_report() {
        // synthetic sentence from chapter 16: 12 kn / 095 deg at 42.35 N 70.6 W
        let bits = Bits::from_payload("Km2MJh8=JR36Q65t", 0).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["lon"].to_string(), "-70.600000");
        assert_eq!(out["lat"].to_string(), "42.350000");
        assert_eq!(out["sog_knots"], Json::Int(12));
        assert_eq!(out["cog_deg"], Json::Int(95));
    }

    #[test]
    fn defaults() {
        let bits = Bits::from_payload("Km2MJh3n`>6bTOwv", 0).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["position_available"], Json::Bool(false));
        assert_eq!(out["sog_available"], Json::Bool(false));
        assert_eq!(out["cog_available"], Json::Bool(false));
    }

    #[test]
    fn too_short() {
        let bits = Bits::from_payload("Kq`jfHko", 0).unwrap();
        assert_eq!(
            decode(&bits).unwrap_err().to_string(),
            "message 27 too short: 48 bits < 96"
        );
    }
}
