//! `msg_14.rs` — message 14: safety related broadcast message.
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.12, Table 63 (layout), Table 64 (how
//! many characters fit in 1–5 slots), Table 47 (six-bit ASCII). Annex 1
//! §2.1.6–2.1.8 prescribe the fixed texts sent by AIS-SART, MOB-AIS and
//! EPIRB-AIS devices ("SART ACTIVE", "SART TEST", "MOB ACTIVE", "MOB TEST",
//! "EPIRB ACTIVE", "EPIRB TEST"); §3.12 repeats the list.
//!
//! Bit layout (zero-based offsets; lengths from Table 63):
//!
//! ```text
//!      0- 5   6  message id (14)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI)
//!     38-39   2  spare
//!     40-     ≤968  safety related text, six-bit ASCII, 0-161 characters
//! ```
//!
//! Length rule: the 40-bit header is required (`DecodeError::TooShort`
//! otherwise); the text handling is shared with message 12
//! (`msg_12::decode_text_field`).
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `beacon_text` is a `match` on a `&str` that returns
//!   `Option<&'static str>`; `Json::from(Option<T>)` prints `None` as null.
//!   Matching on string literals is allowed because `&str` implements
//!   `PartialEq` structurally — no table or map is needed for six entries.
//! * Reading the text back out of the `Obj` we just built would need a
//!   `match` on the `Json` enum; decoding the (short) field twice is simpler
//!   and mirrors the C++ version.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};
use crate::msg_12::decode_text_field;

const HEADER_BITS: usize = 40;

/// The six texts that distress beacons are required to send (Annex 1
/// §2.1.6-2.1.8). A watch-keeping application matches on these.
pub fn beacon_text(text: &str) -> Option<&'static str> {
    match text {
        "SART ACTIVE" => Some("AIS-SART active"),
        "SART TEST" => Some("AIS-SART test"),
        "MOB ACTIVE" => Some("MOB-AIS active"),
        "MOB TEST" => Some("MOB-AIS test"),
        "EPIRB ACTIVE" => Some("EPIRB-AIS active"),
        "EPIRB TEST" => Some("EPIRB-AIS test"),
        _ => None,
    }
}

/// Decode a message 14. Fails with `DecodeError::TooShort` below 40 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(14, bits.len(), HEADER_BITS)?;
    let mut out = Obj::new();
    put(&mut out, "msg_type", bits.uint(0, 6));
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    out.extend(decode_text_field(bits, HEADER_BITS));
    let text = bits.text(HEADER_BITS, bits.len() - HEADER_BITS);
    put(&mut out, "beacon_text", beacon_text(&text));
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sart_active() {
        // Synthetic sentence from data/ch12-safety-text.nmea.
        let bits = Bits::from_payload(">>M4eTA<59B04=@UHD0", 2).unwrap();
        assert_eq!(bits.len(), 112);
        let s = decode(&bits).unwrap().to_string();
        assert!(s.contains("\"beacon_text\":\"AIS-SART active\""));
        assert!(s.contains("\"mmsi\":970010001"));
    }
}
