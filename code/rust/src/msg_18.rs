//! `msg_18.rs` — message 18: standard Class B equipment position report (chapter 14).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.16, Table 70 (168 bits; Table 67 in
//! M.1371-3 and -4). Communication states: Annex 2 §3.3.7.2.2 Table 18/19 (SOTDMA)
//! and §3.3.7.3.2 Table 20 (ITDMA). Class B "CS" behaviour: Annex 7.
//!
//! Bit layout (zero-based offsets; lengths from Table 70):
//!
//! ```text
//!      0- 5   6  message id (18)
//!      6- 7   2  repeat indicator (should be 0 for "CS" units)
//!      8-37  30  user id (MMSI)
//!     38-45   8  spare ("regional reserved" in older editions); should be 0
//!     46-55  10  speed over ground, 1/10 knot; 1023 = n/a, 1022 = ≥102.2 kn
//!     56      1  position accuracy (1 = high, ≤10 m)
//!     57-84  28  longitude, signed, 1/10 000 minute; 181° = n/a
//!     85-111 27  latitude,  signed, 1/10 000 minute;  91° = n/a
//!    112-123 12  course over ground, 1/10 degree; 3600 = n/a
//!    124-132  9  true heading, degrees; 511 = n/a
//!    133-138  6  time stamp (UTC second); 60 = n/a; 61-63 not used by "CS"
//!    139-140  2  spare (regional reserved in older editions); should be 0
//!    141      1  Class B unit flag: 0 = SOTDMA ("SO") unit, 1 = "CS" unit
//!    142      1  Class B display flag: 1 = can display messages 12 and 14
//!    143      1  Class B DSC flag: 1 = has DSC function
//!    144      1  Class B band flag: 1 = whole marine band, 0 = upper 525 kHz only
//!    145      1  Class B message 22 flag: 1 = accepts channel management
//!    146      1  assigned mode flag: 1 = assigned mode
//!    147      1  RAIM flag
//!    148      1  communication state selector: 0 = SOTDMA follows, 1 = ITDMA follows
//!    149-167 19  communication state; a "CS" unit fills it with 1100000000000000110
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * One-bit enumerations are written as `match` on `0` and `_`; the
//!   compiler insists every `match` is exhaustive, and `_` is the catch-all.
//! * `decode_common_position` returns an `Obj` (a `BTreeMap`) that the
//!   caller merges with `out.extend(...)`; message 19 calls it too via
//!   `use crate::msg_18::decode_common_position`.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};
use crate::msg_1_2_3::{decode_itdma, decode_sotdma};

/// Table 70: the 19-bit value every Class B "CS" unit puts in its
/// communication state field.
pub const CS_FIXED_COMM_STATE: &str = "1100000000000000110";

pub fn class_b_unit_text(code: u64) -> &'static str {
    match code {
        0 => "Class B SOTDMA unit",
        _ => "Class B CS unit",
    }
}

pub fn class_b_display_text(code: u64) -> &'static str {
    match code {
        0 => "no display for messages 12 and 14",
        _ => "integrated display for messages 12 and 14",
    }
}

pub fn class_b_dsc_text(code: u64) -> &'static str {
    match code {
        0 => "no DSC function",
        _ => "DSC function (dedicated or time-shared)",
    }
}

pub fn class_b_band_text(code: u64) -> &'static str {
    match code {
        0 => "upper 525 kHz of the marine band only",
        _ => "whole marine band",
    }
}

pub fn class_b_msg22_text(code: u64) -> &'static str {
    match code {
        0 => "AIS 1 and AIS 2 only, no message 22 frequency management",
        _ => "frequency management via message 22",
    }
}

pub fn assigned_mode_text(code: u64) -> &'static str {
    match code {
        0 => "autonomous and continuous",
        _ => "assigned",
    }
}

/// SOG…time stamp as laid out in messages 18 and 19.
///
/// Both messages share the block that starts with the 10-bit SOG at `start`
/// (46 in both cases) and ends with the 6-bit time stamp, so the same code
/// reads both. Offsets below are relative to `start`.
pub fn decode_common_position(bits: &Bits, start: usize) -> Obj {
    let sog_raw = bits.uint(start, 10);
    let lon_raw = bits.sint(start + 11, 28);
    let lat_raw = bits.sint(start + 39, 27);
    let cog_raw = bits.uint(start + 66, 12);
    let heading = bits.uint(start + 78, 9);
    let timestamp = bits.uint(start + 87, 6);
    let mut out = Obj::new();
    put(&mut out, "sog_knots", Json::fixed(sog_raw as f64 / 10.0, 1));
    put(&mut out, "sog_available", sog_raw != 1023);
    put(&mut out, "position_accuracy", bits.uint(start + 10, 1));
    put(&mut out, "lon", Json::fixed(lon_raw as f64 / 600000.0, 6));
    put(&mut out, "lat", Json::fixed(lat_raw as f64 / 600000.0, 6));
    put(
        &mut out,
        "position_available",
        lon_raw != 0x6791AC0 && lat_raw != 0x3412140,
    );
    put(&mut out, "cog_deg", Json::fixed(cog_raw as f64 / 10.0, 1));
    put(&mut out, "cog_available", cog_raw != 3600);
    put(&mut out, "true_heading", heading);
    put(&mut out, "heading_available", heading != 511);
    put(&mut out, "timestamp", timestamp);
    out
}

/// Decode a message 18. Fails with `DecodeError::TooShort` below 168 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 168)?;
    let unit = bits.uint(141, 1);
    let display = bits.uint(142, 1);
    let dsc = bits.uint(143, 1);
    let band = bits.uint(144, 1);
    let msg22 = bits.uint(145, 1);
    let assigned = bits.uint(146, 1);
    let selector = bits.uint(148, 1);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 8));
    out.extend(decode_common_position(bits, 46));
    put(&mut out, "spare2", bits.uint(139, 2));
    put(&mut out, "class_b_unit", unit);
    put(&mut out, "class_b_unit_text", class_b_unit_text(unit));
    put(&mut out, "class_b_display", display);
    put(&mut out, "class_b_display_text", class_b_display_text(display));
    put(&mut out, "class_b_dsc", dsc);
    put(&mut out, "class_b_dsc_text", class_b_dsc_text(dsc));
    put(&mut out, "class_b_band", band);
    put(&mut out, "class_b_band_text", class_b_band_text(band));
    put(&mut out, "class_b_msg22", msg22);
    put(&mut out, "class_b_msg22_text", class_b_msg22_text(msg22));
    put(&mut out, "assigned_mode", assigned);
    put(&mut out, "assigned_mode_text", assigned_mode_text(assigned));
    put(&mut out, "raim", bits.flag(147));
    put(&mut out, "comm_state_selector", selector);
    put(
        &mut out,
        "cs_fixed_comm_state",
        bits.raw(149, 19) == CS_FIXED_COMM_STATE,
    );
    let comm_state = if selector == 1 {
        decode_itdma(bits, 149)
    } else {
        decode_sotdma(bits, 149)
    };
    put(&mut out, "comm_state", comm_state);
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unwrap the object inside a `Json::Obj` (tests only).
    fn as_obj(json: Json) -> Obj {
        match json {
            Json::Obj(o) => o,
            _ => panic!("not an object"),
        }
    }

    #[test]
    fn cs_unit_at_sea() {
        // data/ch14-class-b.nmea line 20: AMY PHILBRICK, 11.2 kn.
        let bits = Bits::from_payload("B5N6=MP0L6g3idV:FdDjSws5kP06", 0).unwrap();
        let out = as_obj(decode(&bits).unwrap());
        assert_eq!(out["mmsi"], Json::from(367103350u64));
        assert_eq!(out["sog_knots"].to_string(), "11.2");
        assert_eq!(out["class_b_unit"], Json::from(1u64));
        assert_eq!(out["cs_fixed_comm_state"], Json::from(true));
    }

    #[test]
    fn too_short() {
        let short = Bits::from_payload("B@80P@`420100P30R27095hq@h", 4).unwrap();
        assert_eq!(
            decode(&short).unwrap_err().to_string(),
            "message 18 too short: 152 bits < 168"
        );
    }
}
