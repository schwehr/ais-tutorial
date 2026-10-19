//! `msg_23.rs` — message 23: group assignment command (chapter 13).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.21, Table 76 (160 bits) and Table 77
//! (reporting-interval codes). A base station acting as controlling entity
//! selects mobile stations by region, station type and ship type, and
//! commands their Tx/Rx mode, reporting interval and a quiet time.
//!
//! Bit layout (zero-based offsets; lengths from Table 76):
//!
//! ```text
//!      0- 5   6  message id (23)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI of the base station)
//!     38-39   2  spare
//!     40-57  18  longitude 1 (north-east corner), 1/10 min
//!     58-74  17  latitude 1 (north-east corner), 1/10 min
//!     75-92  18  longitude 2 (south-west corner), 1/10 min
//!     93-109 17  latitude 2 (south-west corner), 1/10 min
//!    110-113  4  station type
//!    114-121  8  type of ship and cargo type (Table 53; 0 = all)
//!    122-143 22  spare
//!    144-145  2  Tx/Rx mode (0 = TxA/TxB RxA/RxB, 1 = TxA RxA/RxB, 2 = TxB RxA/RxB, 3 = reserved)
//!    146-149  4  reporting interval (Table 77)
//!    150-153  4  quiet time in minutes (0 = none)
//!    154-159  6  spare
//! ```
//!
//! Table 76 gives no not-available value for the region; we apply the
//! 181°/91° sentinels of message 22 (the same 1/10-minute fields) for
//! `region_available`.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `7..=9 => "regional use"` — a range pattern covers several values in
//!   one arm; `11..=15` likewise. The final `_` arm is still required
//!   because the compiler does not know the field is only 4 bits wide.
//! * `ship_type_text` returns an owned `String`, and the literal
//!   `"all types (default)"` is a `&str`; `.to_string()` makes both arms
//!   of the `if` the same type.

use crate::bits::Bits;
use crate::common::{require_bits, ship_type_text, DecodeError};
use crate::jsonout::{put, Json, Obj};

pub const LON_NOT_AVAILABLE: i64 = 181 * 600; // 0x1A838, 1/10-minute units
pub const LAT_NOT_AVAILABLE: i64 = 91 * 600; // 0x0D548

/// Table 76.
pub fn station_type_text(code: u64) -> &'static str {
    match code {
        0 => "all types of mobiles (default)",
        1 => "Class A mobile stations only",
        2 => "all types of Class B mobile stations",
        3 => "SAR airborne mobile station",
        4 => "Class B SO mobile stations only",
        5 => "Class B CS shipborne mobile station only",
        6 => "inland waterways",
        7..=9 => "regional use",
        10 => "base station coverage area",
        _ => "for future use", // 11–15
    }
}

/// Table 76.
pub fn txrx_mode_text(code: u64) -> &'static str {
    match code {
        0 => "TxA/TxB, RxA/RxB",
        1 => "TxA, RxA/RxB",
        2 => "TxB, RxA/RxB",
        _ => "reserved for future use", // 3
    }
}

/// Table 77.
pub fn reporting_interval_text(code: u64) -> &'static str {
    match code {
        0 => "as given by the autonomous mode",
        1 => "10 min",
        2 => "6 min",
        3 => "3 min",
        4 => "1 min",
        5 => "30 s",
        6 => "15 s",
        7 => "10 s",
        8 => "5 s",
        9 => "next shorter reporting interval",
        10 => "next longer reporting interval",
        11 => "2 s (not applicable to Class B CS and SO)",
        _ => "reserved for future use", // 12–15
    }
}

/// Decode a message 23. Fails with `DecodeError::TooShort` below 160 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(23, bits.len(), 160)?;
    let ne_lon = bits.sint(40, 18);
    let ne_lat = bits.sint(58, 17);
    let sw_lon = bits.sint(75, 18);
    let sw_lat = bits.sint(93, 17);
    let station_type = bits.uint(110, 4);
    let ship_type = bits.uint(114, 8);
    let txrx = bits.uint(144, 2);
    let interval = bits.uint(146, 4);
    let mut out = Obj::new();
    put(&mut out, "msg_type", bits.uint(0, 6));
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
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
    put(&mut out, "station_type", station_type);
    put(&mut out, "station_type_text", station_type_text(station_type));
    put(&mut out, "ship_type", ship_type);
    // Table 76 redefines 0 as "all types"; 1–255 follow Table 53.
    let ship_text = if ship_type == 0 {
        "all types (default)".to_string()
    } else {
        ship_type_text(ship_type)
    };
    put(&mut out, "ship_type_text", ship_text);
    put(&mut out, "spare2", bits.uint(122, 22));
    put(&mut out, "txrx_mode", txrx);
    put(&mut out, "txrx_mode_text", txrx_mode_text(txrx));
    put(&mut out, "reporting_interval", interval);
    put(&mut out, "reporting_interval_text", reporting_interval_text(interval));
    put(&mut out, "quiet_time_min", bits.uint(150, 4));
    put(&mut out, "spare3", bits.uint(154, 6));
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_quiet_time() {
        // synthetic sentence from chapter 13: pleasure craft, TxA only, quiet 3 min
        let bits = Bits::from_payload("G03OviSCN0cG6VFQEt09@000@<0", 2).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["ne_lat"].to_string(), "37.000000");
        assert_eq!(out["sw_lon"].to_string(), "-76.500000");
        assert_eq!(out["ship_type"], Json::Int(37));
        assert_eq!(out["ship_type_text"], Json::Str("pleasure craft".to_string()));
        assert_eq!(out["txrx_mode"], Json::Int(1));
        assert_eq!(out["quiet_time_min"], Json::Int(3));
    }

    #[test]
    fn too_short() {
        // sample-data/RightwhaleAisProject/20081013.log line 393089
        let bits = Bits::from_payload("Gob<Mwf<", 0).unwrap();
        assert_eq!(
            decode(&bits).unwrap_err().to_string(),
            "message 23 too short: 48 bits < 160"
        );
    }
}
