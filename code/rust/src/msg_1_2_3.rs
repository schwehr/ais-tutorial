//! `msg_1_2_3.rs` — Class A position reports, messages 1, 2 and 3 (chapter 3).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.1, Table 48 (field layout, 168 bits);
//! Table 49 (which communication state each message carries); Annex 2
//! §3.3.7.2.2 Table 18/19 (SOTDMA communication state) and §3.3.7.3.2
//! Table 20 (ITDMA communication state).
//!
//! Bit layout (zero-based offsets; lengths from Table 48):
//!
//! ```text
//!      0- 5   6  message id (1, 2 or 3)
//!      6- 7   2  repeat indicator
//!      8-37  30  user id (MMSI)
//!     38-41   4  navigational status
//!     42-49   8  rate of turn, signed, special encoding
//!     50-59  10  speed over ground, 1/10 knot; 1023 = n/a, 1022 = ≥102.2 kn
//!     60      1  position accuracy (1 = high, ≤10 m)
//!     61-88  28  longitude, signed, 1/10 000 minute; 181° = n/a
//!     89-115 27  latitude,  signed, 1/10 000 minute;  91° = n/a
//!    116-127 12  course over ground, 1/10 degree; 3600 = n/a
//!    128-136  9  true heading, degrees; 511 = n/a
//!    137-142  6  time stamp (UTC second); 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
//!    143-144  2  special manoeuvre indicator
//!    145-147  3  spare
//!    148      1  RAIM flag
//!    149-167 19  communication state (SOTDMA for 1 and 2, ITDMA for 3)
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * The lookup tables are functions with a `match` returning
//!   `&'static str`. Because the field widths are known (4 bits → 0..=15),
//!   every value has an arm and no "unknown" fallback is needed; the `_`
//!   arms exist only because the compiler cannot see the width.
//! * `decode` returns `Result<Json, DecodeError>`; the one place that can
//!   fail is `require_bits(...)?`. After it, every `bits.uint` call is
//!   guaranteed in range.
//! * Integer-to-float conversion is explicit: `sog_raw as f64 / 10.0`.
//!   Rust never converts numeric types silently.
//! * `out.extend(decode_rot(...))` merges one `BTreeMap` into another —
//!   the counterpart of Python's `dict.update`.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

/// Navigational status, Table 48.
pub fn nav_status_text(code: u64) -> &'static str {
    match code {
        0 => "under way using engine",
        1 => "at anchor",
        2 => "not under command",
        3 => "restricted manoeuvrability",
        4 => "constrained by her draught",
        5 => "moored",
        6 => "aground",
        7 => "engaged in fishing",
        8 => "under way sailing",
        9 => "reserved for HSC",
        10 => "reserved for WIG",
        11 => "power-driven vessel towing astern",
        12 => "power-driven vessel pushing ahead or towing alongside",
        13 => "reserved for future use",
        14 => "AIS-SART / MOB-AIS / EPIRB-AIS active",
        _ => "undefined (default)", // 15
    }
}

/// Synchronisation state of the communication state, Annex 2 Table 18.
pub fn sync_state_text(code: u64) -> &'static str {
    match code {
        0 => "UTC direct",
        1 => "UTC indirect",
        2 => "synchronized to base station",
        _ => "synchronized to another station", // 3
    }
}

/// Special manoeuvre indicator, Table 48.
pub fn special_manoeuvre_text(code: u64) -> &'static str {
    match code {
        0 => "not available",
        1 => "not engaged",
        2 => "engaged",
        _ => "reserved", // 3
    }
}

/// Interpret the 8-bit rate-of-turn field (Table 48).
///
/// The sensor value is compressed with a square root:
/// `ROT_AIS = 4.733 * sqrt(ROT_sensor)` (sign preserved)
/// so to recover degrees per minute we square: `(raw / 4.733)**2`.
pub fn decode_rot(raw: i64) -> Obj {
    let mut out = Obj::new();
    put(&mut out, "rot_raw", raw);
    match raw {
        -128 => {
            put(&mut out, "rot_deg_per_min", Json::Null);
            put(&mut out, "rot_text", "not available");
        }
        127 => {
            put(&mut out, "rot_deg_per_min", Json::Null);
            put(&mut out, "rot_text", "turning right > 5 deg per 30 s (no turn indicator)");
        }
        -127 => {
            put(&mut out, "rot_deg_per_min", Json::Null);
            put(&mut out, "rot_text", "turning left > 5 deg per 30 s (no turn indicator)");
        }
        _ => {
            let scaled = raw as f64 / 4.733;
            // `scaled * scaled` is the correctly rounded square, identical
            // to Python's `** 2` for this exponent.
            let mut value = scaled * scaled;
            if raw < 0 {
                value = -value;
            }
            put(&mut out, "rot_deg_per_min", Json::fixed(value, 1));
            let text = if raw > 0 {
                "turning right"
            } else if raw < 0 {
                "turning left"
            } else {
                "not turning"
            };
            put(&mut out, "rot_text", text);
        }
    }
    out
}

/// SOTDMA communication state, 19 bits (Annex 2 Table 18 and 19).
pub fn decode_sotdma(bits: &Bits, start: usize) -> Obj {
    let sync = bits.uint(start, 2);
    let timeout = bits.uint(start + 2, 3);
    let sub = bits.uint(start + 5, 14);
    let mut out = Obj::new();
    put(&mut out, "scheme", "SOTDMA");
    put(&mut out, "sync_state", sync);
    put(&mut out, "sync_state_text", sync_state_text(sync));
    put(&mut out, "slot_timeout", timeout);
    // The meaning of the 14-bit sub-message depends on the slot time-out.
    match timeout {
        3 | 5 | 7 => put(&mut out, "received_stations", sub),
        2 | 4 | 6 => put(&mut out, "slot_number", sub),
        1 => {
            put(&mut out, "utc_hour", sub >> 9); // bits 13..9
            put(&mut out, "utc_minute", (sub >> 2) & 0x7F); // bits 8..2 (7 bits)
        }
        _ => put(&mut out, "slot_offset", sub), // timeout == 0
    }
    out
}

/// ITDMA communication state, 19 bits (Annex 2 Table 20).
pub fn decode_itdma(bits: &Bits, start: usize) -> Obj {
    let sync = bits.uint(start, 2);
    let increment = bits.uint(start + 2, 13);
    let nslots = bits.uint(start + 15, 3);
    let keep = bits.flag(start + 18);
    let mut out = Obj::new();
    put(&mut out, "scheme", "ITDMA");
    put(&mut out, "sync_state", sync);
    put(&mut out, "sync_state_text", sync_state_text(sync));
    put(&mut out, "slot_increment", increment);
    put(&mut out, "slots_raw", nslots);
    put(&mut out, "keep_flag", keep);
    // Table 20: 0..4 mean 1..5 slots; 5..7 mean 1..3 slots with +8192 offset.
    if nslots <= 4 {
        put(&mut out, "slots", nslots + 1);
        put(&mut out, "slot_offset", increment);
    } else {
        put(&mut out, "slots", nslots - 4);
        put(&mut out, "slot_offset", increment + 8192);
    }
    out
}

/// Decode a message 1, 2 or 3. Fails with `DecodeError::TooShort` if the
/// payload has fewer than 168 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 168)?;
    let nav = bits.uint(38, 4);
    let sog_raw = bits.uint(50, 10);
    let lon_raw = bits.sint(61, 28);
    let lat_raw = bits.sint(89, 27);
    let cog_raw = bits.uint(116, 12);
    let heading = bits.uint(128, 9);
    let timestamp = bits.uint(137, 6);
    let manoeuvre = bits.uint(143, 2);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "nav_status", nav);
    put(&mut out, "nav_status_text", nav_status_text(nav));
    put(&mut out, "sog_knots", Json::fixed(sog_raw as f64 / 10.0, 1));
    put(&mut out, "sog_available", sog_raw != 1023);
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
    put(&mut out, "true_heading", heading);
    put(&mut out, "heading_available", heading != 511);
    put(&mut out, "timestamp", timestamp);
    put(&mut out, "special_manoeuvre", manoeuvre);
    put(&mut out, "special_manoeuvre_text", special_manoeuvre_text(manoeuvre));
    put(&mut out, "spare", bits.uint(145, 3));
    put(&mut out, "raim", bits.flag(148));
    out.extend(decode_rot(bits.sint(42, 8)));
    let comm_state = if msg_type == 3 {
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
    fn rot_special_values() {
        assert_eq!(decode_rot(-128)["rot_text"], Json::from("not available"));
        assert_eq!(decode_rot(0)["rot_deg_per_min"].to_string(), "0.0");
        assert_eq!(decode_rot(-10)["rot_deg_per_min"].to_string(), "-4.5");
        assert_eq!(decode_rot(10)["rot_text"], Json::from("turning right"));
    }

    #[test]
    fn too_short() {
        let bits = Bits::from_payload("15N8?>PP01JRVP6E7aLLkOvt0HAO", 0).unwrap();
        assert_eq!(bits.len(), 168);
        assert!(decode(&bits).is_ok());
        let short = Bits::from_payload("15N8?>PP01JRVP6E7aLLkOvt", 0).unwrap();
        assert_eq!(
            decode(&short).unwrap_err().to_string(),
            "message 1 too short: 144 bits < 168"
        );
    }
}
