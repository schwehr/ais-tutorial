//! `msg_10.rs` — message 10: UTC and date inquiry.
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.8, Table 60 (72 bits). The answer is a
//! message 11 (same layout as message 4, chapter 6), which "is only
//! transmitted as a result of a UTC request message (Message 10)" and goes
//! out "on the channel where the UTC request message was received"
//! [Annex 8 §3.2].
//!
//! Bit layout (zero-based offsets; lengths from Table 60):
//!
//! ```text
//!      0- 5   6  message id (10)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI of the station that wants to know the time)
//!     38-39   2  spare
//!     40-69  30  destination id (MMSI of the station being asked)
//!     70-71   2  spare
//! ```
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * Nothing new here — this is the smallest decoder in the crate and a good
//!   template: read the type, check the length with `?`, fill a `BTreeMap`,
//!   wrap it in `Json::Obj` and return `Ok`.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

/// Decode a message 10. Fails with `DecodeError::TooShort` below 72 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 72)?;
    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    put(&mut out, "dest_mmsi", bits.uint(40, 30));
    put(&mut out, "spare2", bits.uint(70, 2));
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_inquiry() {
        // tide3/ais/ais-2010-03-09 line 131: 354741000 asks 367315470.
        let bits = Bits::from_payload(":5BCc21GTjPp", 0).unwrap();
        assert_eq!(bits.len(), 72);
        let text = decode(&bits).unwrap().to_string();
        assert!(text.contains("\"dest_mmsi\":367315470"));
        assert!(text.contains("\"mmsi\":354741000"));
    }
}
