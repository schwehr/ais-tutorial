//! `msg_12.rs` — message 12: addressed safety related message.
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.10, Table 61 (layout), Table 62 (how
//! many characters fit in 1–5 slots), Table 47 (six-bit ASCII); Annex 2
//! §5.3.1 (sequence numbers, retransmission, acknowledgement by message 13).
//!
//! Bit layout (zero-based offsets; lengths from Table 61):
//!
//! ```text
//!      0- 5   6  message id (12)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI)
//!     38-39   2  sequence number, 0-3
//!     40-69  30  destination id (MMSI)
//!     70      1  retransmit flag (0 = first transmission, 1 = retransmitted)
//!     71      1  spare
//!     72-     ≤936  safety related text, six-bit ASCII, 0-156 characters
//! ```
//!
//! Length rule: the 72-bit header is required (`DecodeError::TooShort`
//! otherwise). Every complete six-bit group after it is one character. The
//! 0-5 bits left over at the end are *not* text: the radio layer pads
//! messages to a byte boundary (Annex 2 §5.2.1), and the receiver's
//! armouring pads again to six bits, so a real message often carries 2 or 4
//! stray bits after the last character. We count them in `trailing_bits`
//! and ignore them.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `pub fn decode_text_field` is public so that `msg_14.rs` can call it
//!   with `use crate::msg_12::decode_text_field;` — Rust modules hide
//!   everything that is not marked `pub`.
//! * Integer `/` and `%` on `usize` behave like Python's `//` and `%` for
//!   non-negative values; there are no floats anywhere in this module.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

const HEADER_BITS: usize = 72;

/// Decode a variable-length six-bit text field that runs to the end.
///
/// Shared by messages 12 and 14. `Bits::text` already stops before a
/// partial six-bit group, cuts the text at the first '@' (the Table 47
/// padding / not-available character) and strips trailing spaces; here we
/// add the bookkeeping numbers.
pub fn decode_text_field(bits: &Bits, start: usize) -> Obj {
    let text_bits = bits.len() - start;
    let mut out = Obj::new();
    put(&mut out, "text", bits.text(start, text_bits));
    put(&mut out, "text_bits", text_bits);
    put(&mut out, "text_chars", text_bits / 6); // characters on the air, '@' padding included
    put(&mut out, "trailing_bits", text_bits % 6); // byte-alignment leftovers, not text
    out
}

/// Decode a message 12. Fails with `DecodeError::TooShort` below 72 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(12, bits.len(), HEADER_BITS)?;
    let mut out = Obj::new();
    put(&mut out, "msg_type", bits.uint(0, 6));
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "sequence_number", bits.uint(38, 2));
    put(&mut out, "dest_mmsi", bits.uint(40, 30));
    put(&mut out, "retransmit", bits.flag(70));
    put(&mut out, "spare", bits.uint(71, 1));
    out.extend(decode_text_field(bits, HEADER_BITS));
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_and_padding() {
        // data/ch12-safety-text.nmea: "MSG FROM 368905000", 184 bits =
        // 72 header + 18 characters + 4 byte-alignment bits.
        let bits = Bits::from_payload("<5Ol;:5GTGd0=C7P6B?=Pknpqhmhhh0", 2).unwrap();
        assert_eq!(bits.len(), 184);
        let json = decode(&bits).unwrap();
        let s = json.to_string();
        assert!(s.contains("\"text\":\"MSG FROM 368905000\""));
        assert!(s.contains("\"text_chars\":18"));
        assert!(s.contains("\"trailing_bits\":4"));
    }

    #[test]
    fn too_short() {
        let bits = Bits::from_payload("<000001H=C0", 0).unwrap(); // 66 bits
        assert_eq!(
            decode(&bits).unwrap_err().to_string(),
            "message 12 too short: 66 bits < 72"
        );
    }
}
