//! `msg_9.rs` — message 9: standard search and rescue (SAR) aircraft position report.
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.7, Table 59 (168 bits); the 19-bit
//! communication state is SOTDMA (Annex 2 §3.3.7.2.2 Tables 18/19) or ITDMA
//! (Annex 2 §3.3.7.3.2 Table 20) according to the selector bit, exactly as
//! in message 18 (chapter 14), so we reuse `decode_sotdma` / `decode_itdma`
//! from `msg_1_2_3`.
//!
//! Bit layout (zero-based offsets; lengths from Table 59):
//!
//! ```text
//!      0- 5   6  message id (9)
//!      6- 7   2  repeat indicator
//!      8-37  30  user id (MMSI; SAR aircraft use 111MIDxxx, ITU-R M.585-9 Annex 1 §3)
//!     38-49  12  altitude in metres; 4095 = n/a, 4094 = 4094 m or higher
//!     50-59  10  speed over ground in whole knots; 1023 = n/a, 1022 = 1022 kn or higher
//!     60      1  position accuracy (1 = high, <=10 m)
//!     61-88  28  longitude, signed, 1/10 000 minute; 181° (0x6791AC0) = n/a
//!     89-115 27  latitude,  signed, 1/10 000 minute;  91° (0x3412140) = n/a
//!    116-127 12  course over ground, 1/10 degree; 3600 = n/a
//!    128-133  6  time stamp (UTC second); 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
//!    134      1  altitude sensor (0 = GNSS, 1 = barometric)
//!    135-141  7  spare
//!    142      1  DTE (0 = data terminal available, 1 = not available = default)
//!    143-145  3  spare
//!    146      1  assigned mode flag (0 = autonomous, 1 = assigned)
//!    147      1  RAIM flag
//!    148      1  communication state selector (0 = SOTDMA, 1 = ITDMA)
//!    149-167 19  communication state
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `pub const ALTITUDE_NOT_AVAILABLE: u64 = 4095;` — a typed compile-time
//!   constant. `bits.uint` returns `u64`, so comparing against a `u64`
//!   constant needs no cast.
//! * `sog` is put into the object as a plain `u64`, which `Json` prints as
//!   an integer; the Class A report wrapped SOG in `Json::fixed(.., 1)`
//!   because it was in tenths of a knot.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};
use crate::msg_1_2_3::{decode_itdma, decode_sotdma};

pub const ALTITUDE_NOT_AVAILABLE: u64 = 4095;
pub const ALTITUDE_MAX: u64 = 4094; // "4 094 m or higher"
pub const SOG_NOT_AVAILABLE: u64 = 1023;
pub const SOG_MAX: u64 = 1022; // "1 022 knots or higher"

pub fn altitude_sensor_text(code: u64) -> &'static str {
    match code {
        0 => "GNSS",
        _ => "barometric",
    }
}

pub fn assigned_mode_text(code: u64) -> &'static str {
    match code {
        0 => "autonomous and continuous",
        _ => "assigned",
    }
}

/// Decode a message 9. Fails with `DecodeError::TooShort` below 168 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 168)?;
    let altitude = bits.uint(38, 12);
    let sog = bits.uint(50, 10);
    let lon_raw = bits.sint(61, 28);
    let lat_raw = bits.sint(89, 27);
    let cog_raw = bits.uint(116, 12);
    let sensor = bits.uint(134, 1);
    let assigned = bits.uint(146, 1);
    let selector = bits.uint(148, 1);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "altitude_m", altitude);
    put(&mut out, "altitude_available", altitude != ALTITUDE_NOT_AVAILABLE);
    put(&mut out, "altitude_at_max", altitude == ALTITUDE_MAX);
    put(&mut out, "sog_knots", sog); // whole knots: an integer, not Json::fixed
    put(&mut out, "sog_available", sog != SOG_NOT_AVAILABLE);
    put(&mut out, "sog_at_max", sog == SOG_MAX);
    put(&mut out, "position_accuracy", bits.uint(60, 1));
    put(&mut out, "lon", Json::fixed(lon_raw as f64 / 600000.0, 6));
    put(&mut out, "lat", Json::fixed(lat_raw as f64 / 600000.0, 6));
    put(
        &mut out,
        "position_available",
        lon_raw != 0x6791AC0 && lat_raw != 0x3412140,
    );
    put(&mut out, "cog_deg", Json::fixed(cog_raw as f64 / 10.0, 1));
    put(&mut out, "cog_available", cog_raw != 3600);
    put(&mut out, "timestamp", bits.uint(128, 6));
    put(&mut out, "altitude_sensor", sensor);
    put(&mut out, "altitude_sensor_text", altitude_sensor_text(sensor));
    put(&mut out, "spare", bits.uint(135, 7));
    put(&mut out, "dte", bits.uint(142, 1));
    put(&mut out, "spare2", bits.uint(143, 3));
    put(&mut out, "assigned_mode", assigned);
    put(&mut out, "assigned_mode_text", assigned_mode_text(assigned));
    put(&mut out, "raim", bits.flag(147));
    put(&mut out, "comm_state_selector", selector);
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

    #[test]
    fn synthetic_sar_aircraft() {
        // Chapter 10 §10.3: 111366101 at 457 m, 150 kn, SOTDMA.
        let bits = Bits::from_payload("91b=?mAjBFrv4v@HAF98kl@00<2F", 0).unwrap();
        let json = decode(&bits).unwrap();
        let text = json.to_string();
        assert!(text.contains("\"mmsi\":111366101"));
        assert!(text.contains("\"altitude_m\":457"));
        assert!(text.contains("\"sog_knots\":150,"));
        assert!(text.contains("\"scheme\":\"SOTDMA\""));
    }

    #[test]
    fn too_short() {
        let bits = Bits::from_payload("9P=t4>pBN4:04Ak8010p", 0).unwrap();
        assert_eq!(
            decode(&bits).unwrap_err().to_string(),
            "message 9 too short: 120 bits < 168"
        );
    }
}
