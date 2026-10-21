//! `msg_7_13.rs` — messages 7 and 13: binary / safety related acknowledgement.
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.5 (message 7, and by reference §3.11
//! message 13), Table 56. Annex 2 §5.3.1 describes when the acknowledgement
//! is sent (after an addressed message 6 or 12) and what the sequence number
//! means.
//!
//! Bit layout (zero-based offsets; lengths from Table 56):
//!
//! ```text
//!      0- 5   6  message id (7 = binary ack, 13 = safety related ack)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI of the station that acknowledges)
//!     38-39   2  spare
//!     40-69  30  destination id 1 (MMSI whose message is acknowledged)
//!     70-71   2  sequence number of the acknowledged message, 0-3
//!     72-101 30  destination id 2       } "should be omitted if no
//!    102-103  2  sequence number 2      }  destination ID2"
//!    104-135 32  destination id 3 + sequence number 3
//!    136-167 32  destination id 4 + sequence number 4
//! ```
//!
//! Total 72, 104, 136 or 168 bits. The repeated 32-bit groups are output as
//! a JSON array `acks`, one object per acknowledged station.
//!
//! Length rule: at least 72 bits (one destination) are required; otherwise
//! `DecodeError::TooShort`. After the first group we take as many complete
//! 32-bit groups as are present, up to four. Bits left over after the last
//! complete group are counted in `trailing_bits` and never interpreted.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `Vec<Json>` is the growable array; `push` appends. The vector is
//!   declared `let mut` because we modify it after creation — Rust bindings
//!   are immutable unless you say otherwise.
//! * `const NAME: usize = …;` is a compile-time constant; `usize` is the
//!   unsigned type used for sizes and indices, which is also what
//!   `bits.len()` and `bits.has()` work in.
//! * `while cond { … }` is the plain loop; Rust has no `do`/`while` and the
//!   condition needs no parentheses.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

const HEADER_BITS: usize = 40; // message id .. spare
const GROUP_BITS: usize = 32; // destination id (30) + sequence number (2)
const MAX_GROUPS: usize = 4;

/// Decode a message 7 or 13. Fails with `DecodeError::TooShort` below 72 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), HEADER_BITS + GROUP_BITS)?;

    let mut acks: Vec<Json> = Vec::new();
    let mut pos = HEADER_BITS;
    // Loop over the repeated groups. `bits.has` is the guard that stops us
    // at the end of a 72-, 104-, 136- or 168-bit message.
    while acks.len() < MAX_GROUPS && bits.has(pos, GROUP_BITS) {
        let mut ack = Obj::new();
        put(&mut ack, "mmsi", bits.uint(pos, 30));
        put(&mut ack, "sequence_number", bits.uint(pos + 30, 2));
        acks.push(Json::Obj(ack));
        pos += GROUP_BITS;
    }

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    put(&mut out, "acks", acks);
    put(&mut out, "trailing_bits", bits.len() - pos);
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_and_two_groups() {
        // 72 bits: one acknowledged station.
        let one = Bits::from_payload("=000001H=C0<", 0).unwrap();
        assert_eq!(one.len(), 72);
        let json = decode(&one).unwrap();
        assert!(json.to_string().contains("\"trailing_bits\":0"));
        // 71 bits: too short.
        let short = Bits::from_payload("=000001H=C0<", 1).unwrap();
        assert_eq!(
            decode(&short).unwrap_err().to_string(),
            "message 13 too short: 71 bits < 72"
        );
    }
}
