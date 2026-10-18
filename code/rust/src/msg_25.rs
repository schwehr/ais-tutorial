//! `msg_25.rs` — message 25: single slot binary message (chapter 16).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.23, Table 80 (layout) and Table 81
//! (maximum data bits per flag combination); Annex 5 §1.1 for the 16-bit
//! application identifier (10-bit DAC + 6-bit FI).
//!
//! Bit layout (zero-based offsets). Two flags decide which fields exist, so
//! the offsets after bit 39 are *variable*:
//!
//! ```text
//!      0- 5   6  message id (25)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI)
//!     38      1  destination indicator: 0 = broadcast, 1 = addressed
//!     39      1  binary data flag: 0 = unstructured, 1 = application identifier follows
//!     40-69  30  destination MMSI        (only if destination indicator = 1)
//!     70-71   2  spare                   (only if destination indicator = 1)
//!     next   16  application identifier  (only if binary data flag = 1): DAC 10 + FI 6
//!     rest    n  application data, to the end of the payload (Table 81: ≤ 128/112/96/80)
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `decode_binary_header` returns `Result<(Obj, usize), DecodeError>`: a
//!   tuple inside a `Result`. The caller writes
//!   `let (mut out, offset) = decode_binary_header(bits, 25)?;` — the `?`
//!   unwraps the `Result` (returning early on error) and the pattern on
//!   the left destructures the tuple.
//! * `Json::Null` is the explicit null value; Python used `None`.
//! * `offset` is a `usize` and `bits.len() - offset` would *panic* in debug
//!   builds (and wrap in release builds) if it went negative. It cannot,
//!   because `require_bits` has checked `offset <= bits.len()` first.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

/// The part of messages 25 and 26 that precedes the application data.
///
/// Returns the decoded fields and the offset of the first data bit. Fails
/// if the optional fields announced by the flags are missing.
pub fn decode_binary_header(bits: &Bits, msg_type: u32) -> Result<(Obj, usize), DecodeError> {
    require_bits(msg_type, bits.len(), 40)?;
    let addressed = bits.flag(38);
    let structured = bits.flag(39);
    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "addressed", addressed); // destination indicator, bit 38
    put(&mut out, "structured", structured); // binary data flag, bit 39
    put(&mut out, "dest_mmsi", Json::Null);
    put(&mut out, "dac", Json::Null);
    put(&mut out, "fi", Json::Null);
    let mut offset = 40;
    if addressed {
        require_bits(msg_type, bits.len(), offset + 32)?;
        put(&mut out, "dest_mmsi", bits.uint(offset, 30));
        put(&mut out, "spare", bits.uint(offset + 30, 2));
        offset += 32;
    }
    if structured {
        require_bits(msg_type, bits.len(), offset + 16)?;
        put(&mut out, "dac", bits.uint(offset, 10));
        put(&mut out, "fi", bits.uint(offset + 10, 6));
        offset += 16;
    }
    put(&mut out, "data_offset", offset);
    Ok((out, offset))
}

/// Decode a message 25. Fails with `DecodeError::TooShort` if a required
/// part is missing.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let (mut out, offset) = decode_binary_header(bits, 25)?;
    // Everything after the header is application data; we cannot know its
    // internal structure, so we hand it on as a bit string (CONVENTIONS §3).
    put(&mut out, "data", bits.raw(offset, bits.len() - offset));
    put(&mut out, "data_bits", bits.len() - offset);
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcast_unstructured() {
        // sample-data/tide3/ais/ais-2011-05-26 line 1648
        let bits = Bits::from_payload("I?wwwwhHS>0fgGQ0UImCTdkfB=>R", 0).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["mmsi"], Json::Int(1073741823));
        assert_eq!(out["addressed"], Json::Bool(false));
        assert_eq!(out["data_bits"], Json::Int(128));
    }

    #[test]
    fn addressed_structured() {
        // synthetic sentence from chapter 16
        let bits = Bits::from_payload("I52MJh=GBVL0004B=5H", 2).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["dest_mmsi"], Json::Int(366123456));
        assert_eq!(out["dac"], Json::Int(0));
        assert_eq!(out["fi"], Json::Int(1));
        assert_eq!(out["data_offset"], Json::Int(88));
        assert_eq!(out["data_bits"], Json::Int(24));
    }

    #[test]
    fn too_short() {
        let bits = Bits::from_payload("ILPo0h", 4).unwrap();
        assert_eq!(
            decode(&bits).unwrap_err().to_string(),
            "message 25 too short: 32 bits < 40"
        );
    }
}
