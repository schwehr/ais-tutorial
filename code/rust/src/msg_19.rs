//! `msg_19.rs` — message 19: extended Class B equipment position report (chapter 14).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.17, Table 71 (312 bits, two slots; Table
//! 68 in M.1371-4). Dimension/reference-point field: §3.3.3 Fig. 41. Ship
//! type: §3.3.2 Table 53. EPFD: Table 51. DTE: §3.3.1.
//!
//! M.1371-5 says this message "is not needed and should not be used" by new
//! equipment — a Class B unit should send message 18 plus message 24 A/B
//! instead — but legacy units still send it once every 6 minutes.
//!
//! Bit layout (zero-based offsets; lengths from Table 71). Bits 0-138 are
//! laid out exactly as in message 18, so we reuse that module's helper.
//!
//! ```text
//!      0- 5   6  message id (19)
//!      6- 7   2  repeat indicator
//!      8-37  30  user id (MMSI)
//!     38-45   8  spare
//!     46-55  10  speed over ground, 1/10 knot; 1023 = n/a
//!     56      1  position accuracy
//!     57-84  28  longitude, signed, 1/10 000 minute; 181° = n/a
//!     85-111 27  latitude,  signed, 1/10 000 minute;  91° = n/a
//!    112-123 12  course over ground, 1/10 degree; 3600 = n/a
//!    124-132  9  true heading; 511 = n/a
//!    133-138  6  time stamp (UTC second); 60 = n/a
//!    139-142  4  spare
//!    143-262 120 name, 20 six-bit characters; all '@' = n/a
//!    263-270  8  type of ship and cargo type; 0 = n/a
//!    271-300 30  dimensions / reference point: A 9, B 9, C 6, D 6 (Fig. 41)
//!    301-304  4  type of EPFD; 0 = undefined
//!    305      1  RAIM flag
//!    306      1  DTE: 0 = data terminal available, 1 = not available (default)
//!    307      1  assigned mode flag
//!    308-311  4  spare
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `decode_dimensions` is `pub` so that `msg_24.rs` can `use` it. Rust
//!   modules are private by default; every item another module needs must
//!   be marked `pub` explicitly.

use crate::bits::Bits;
use crate::common::{epfd_text, require_bits, ship_type_text, DecodeError};
use crate::jsonout::{put, Json, Obj};
use crate::msg_18::{assigned_mode_text, decode_common_position};

pub fn dte_text(code: u64) -> &'static str {
    match code {
        0 => "data terminal available",
        _ => "data terminal not available",
    }
}

/// The 30-bit dimension / reference-point field (M.1371-5 §3.3.3 Fig. 41).
///
/// A = distance from the reference point (the GNSS antenna) to the bow,
/// B = to the stern, C = to the port side, D = to the starboard side, all in
/// metres. A and B saturate at 511, C and D at 63. All four zero means
/// "nothing available"; A = C = 0 with B and D non-zero means "dimensions
/// known but reference point not". Keys and availability rules are exactly
/// those of message 5 (`msg_5.rs`); message 21 reuses this helper as well.
pub fn decode_dimensions(bits: &Bits, start: usize) -> Obj {
    let a = bits.uint(start, 9);
    let b = bits.uint(start + 9, 9);
    let c = bits.uint(start + 18, 6);
    let d = bits.uint(start + 24, 6);
    let mut out = Obj::new();
    put(&mut out, "dim_bow_m", a);
    put(&mut out, "dim_stern_m", b);
    put(&mut out, "dim_port_m", c);
    put(&mut out, "dim_starboard_m", d);
    // Figure 41: all four zero = neither reference point nor dimensions;
    // A = C = 0 with B, D non-zero = dimensions but no reference point.
    put(
        &mut out,
        "dimensions_available",
        !(a == 0 && b == 0 && c == 0 && d == 0),
    );
    put(&mut out, "reference_point_available", !(a == 0 && c == 0));
    put(&mut out, "length_m", a + b);
    put(&mut out, "beam_m", c + d);
    out
}

/// Decode a message 19. Fails with `DecodeError::TooShort` below 312 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 312)?;
    let ship_type = bits.uint(263, 8);
    let epfd = bits.uint(301, 4);
    let dte = bits.uint(306, 1);
    let assigned = bits.uint(307, 1);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 8));
    out.extend(decode_common_position(bits, 46));
    put(&mut out, "spare2", bits.uint(139, 4));
    put(&mut out, "name", bits.text(143, 120));
    put(&mut out, "ship_type", ship_type);
    put(&mut out, "ship_type_text", ship_type_text(ship_type));
    out.extend(decode_dimensions(bits, 271));
    put(&mut out, "epfd", epfd);
    put(&mut out, "epfd_text", epfd_text(epfd));
    put(&mut out, "raim", bits.flag(305));
    put(&mut out, "dte", dte);
    put(&mut out, "dte_text", dte_text(dte));
    put(&mut out, "assigned_mode", assigned);
    put(&mut out, "assigned_mode_text", assigned_mode_text(assigned));
    put(&mut out, "spare3", bits.uint(308, 4));
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
    fn synthetic_cocheco() {
        // data/ch14-class-b.nmea lines 37-38 (synthetic, see chapter 14).
        let payload = "C5Pm;j0006g4rwV:ELUFGwr0UNe06N6@:6N000000000L0L21PRP";
        let bits = Bits::from_payload(payload, 0).unwrap();
        assert_eq!(bits.len(), 312);
        let out = as_obj(decode(&bits).unwrap());
        assert_eq!(out["name"], Json::from("R/V COCHECO"));
        assert_eq!(out["dim_bow_m"], Json::from(7u64));
        assert_eq!(out["length_m"], Json::from(11u64));
        assert_eq!(out["reference_point_available"], Json::from(true));
        assert_eq!(out["epfd_text"], Json::from("GPS"));
    }

    #[test]
    fn all_zero_dimensions_have_no_reference_point() {
        // Figure 41: A = B = C = D = 0 means neither dimensions nor
        // reference point are available.
        let bits = Bits::from_payload("000000", 0).unwrap();
        let out = decode_dimensions(&bits, 0);
        assert_eq!(out["dimensions_available"], Json::from(false));
        assert_eq!(out["reference_point_available"], Json::from(false));
    }
}
