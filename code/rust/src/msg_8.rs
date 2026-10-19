//! `msg_8.rs` — message 8: binary broadcast message (chapter 8).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.6, Table 57 (layout), Table 58 (slot
//! budget); Annex 5 §2.1 Table 23 (application identifier).
//!
//! Bit layout (zero-based offsets; lengths from Table 57):
//!
//! ```text
//!      0- 5   6  message id (8)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI)
//!     38-39   2  spare
//!     40-49  10  designated area code (DAC)          } 16-bit application
//!     50-55   6  function identifier (FI)             } identifier, Annex 5 Table 23
//!     56-    ≤952 application data
//! ```
//!
//! Length rule: at least the 56-bit header is required; otherwise
//! `DecodeError::TooShort`. The rest is `data`/`data_bits`, plus
//! `application` when `asm.rs` knows the (dac, fi).

use crate::asm;
use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

pub const HEADER_BITS: usize = 56; // message id .. function identifier

/// Decode a message 8. Fails with `DecodeError::TooShort` below 56 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(8, bits.len(), HEADER_BITS)?;
    let dac = bits.uint(40, 10);
    let fi = bits.uint(50, 6);
    let mut out = Obj::new();
    put(&mut out, "msg_type", 8);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
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
    fn whale_and_short() {
        // data/ch08-binary-framing.nmea line 8.
        let bits = Bits::from_payload("803OvqQK`PP7fb0Fg7GI63do4T;0", 0).unwrap();
        let json = decode(&bits).unwrap().to_string();
        assert!(json.contains("\"dac\":366,"));
        assert!(json.contains("\"fi\":34,"));
        assert!(json.contains("\"data_bits\":112"));
        // data/ch08-binary-framing.nmea line 49: 32 bits.
        let short = Bits::from_payload("8>@q3h", 4).unwrap();
        assert_eq!(decode(&short).unwrap_err().to_string(), "message 8 too short: 32 bits < 56");
    }
}
