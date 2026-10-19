//! `msg_6.rs` — message 6: addressed binary message (chapter 8).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.4, Table 54 (layout), Table 55 (slot
//! budget); Annex 2 §5.3.1 (sequence number, retries, retransmit flag and the
//! message 7 acknowledgement); Annex 5 §2.1 Table 23 (application identifier).
//!
//! Bit layout (zero-based offsets; lengths from Table 54):
//!
//! ```text
//!      0- 5   6  message id (6)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI)
//!     38-39   2  sequence number 0-3 (Annex 2 §5.3.1; echoed by message 7)
//!     40-69  30  destination id (MMSI)
//!     70      1  retransmit flag: 0 = no retransmission (default), 1 = retransmitted
//!     71      1  spare
//!     72-81  10  designated area code (DAC)          } 16-bit application
//!     82-87   6  function identifier (FI)             } identifier, Annex 5 Table 23
//!     88-    ≤920 application data
//! ```
//!
//! Length rule: at least the 88-bit header is required; otherwise
//! `DecodeError::TooShort`. The rest is `data`/`data_bits`, plus
//! `application` when `asm.rs` knows the (dac, fi).
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `if let Some(app) = asm::decode(…) { … }` runs the block only when the
//!   `Option` holds a value and binds that value to `app` — the idiomatic
//!   way to use an optional result without `unwrap()`.

use crate::asm;
use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

pub const HEADER_BITS: usize = 88; // message id .. function identifier

/// Decode a message 6. Fails with `DecodeError::TooShort` below 88 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(6, bits.len(), HEADER_BITS)?;
    let dac = bits.uint(72, 10);
    let fi = bits.uint(82, 6);
    let mut out = Obj::new();
    put(&mut out, "msg_type", 6);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "sequence_number", bits.uint(38, 2));
    put(&mut out, "dest_mmsi", bits.uint(40, 30));
    put(&mut out, "retransmit", bits.flag(70));
    put(&mut out, "spare", bits.uint(71, 1));
    put(&mut out, "dac", dac);
    put(&mut out, "fi", fi);
    put(&mut out, "data", bits.raw(HEADER_BITS, bits.len() - HEADER_BITS));
    put(&mut out, "data_bits", bits.len() - HEADER_BITS);
    if let Some(app) = asm::decode(dac, fi, bits, HEADER_BITS) {
        put(&mut out, "application", app);
    }
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_telegram_and_short() {
        // data/ch08-binary-framing.nmea line 21: 120 bits, DAC 1 FI 0.
        let bits = Bits::from_payload("603Ovl@0>5Q@04000000", 0).unwrap();
        let json = decode(&bits).unwrap().to_string();
        assert!(json.contains("\"dest_mmsi\":923156"));
        assert!(json.contains("\"application_name\":\"IFM 0 text telegram\""));
        let short = Bits::from_payload("603Ovl@0>5Q@04", 0).unwrap();
        assert_eq!(decode(&short).unwrap_err().to_string(), "message 6 too short: 84 bits < 88");
    }
}
