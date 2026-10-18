//! `msg_26.rs` — message 26: multiple slot binary message with
//! communications state (chapter 16).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.24, Table 82 (layout) and Table 83
//! (maximum data bits per flag combination and slot count); Annex 2
//! §3.3.7.2.2 Tables 18/19 (SOTDMA) and §3.3.7.3.2 Table 20 (ITDMA) for the
//! communication state, decoded by the same functions as messages 1–3.
//!
//! Bit layout (zero-based offsets):
//!
//! ```text
//!      0- 5   6  message id (26)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI)
//!     38      1  destination indicator: 0 = broadcast, 1 = addressed
//!     39      1  binary data flag: 0 = unstructured, 1 = application identifier follows
//!     40-69  30  destination MMSI        (only if destination indicator = 1)
//!     70-71   2  spare                   (only if destination indicator = 1)
//!     next   16  application identifier  (only if binary data flag = 1): DAC 10 + FI 6
//!     next    n  application data (Table 83: 104 … 1 000 bits)
//!     next    4  spare "needed for byte alignment" (Table 82)
//!     last 20     communication state: 1 selector bit (0 = SOTDMA, 1 = ITDMA)
//!                 followed by the 19-bit SOTDMA or ITDMA state
//! ```
//!
//! The communication state is found by counting back 20 bits from the end
//! of the payload; the data length is "whatever is left" and includes the
//! alignment spare, which a receiver cannot tell apart from data.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `pub const COMM_STATE_BITS: usize = 20;` — a typed compile-time
//!   constant. Constants are inlined wherever they are used.
//! * Functions from sibling modules are reached through the crate root:
//!   `crate::msg_25::decode_binary_header`, `crate::msg_1_2_3::decode_sotdma`.
//!   Nothing is copied; the module system makes reuse explicit.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json};
use crate::msg_1_2_3::{decode_itdma, decode_sotdma};
use crate::msg_25::decode_binary_header;

/// 1 selector bit + 19 bits of SOTDMA/ITDMA state.
pub const COMM_STATE_BITS: usize = 20;

/// Decode a message 26. Fails with `DecodeError::TooShort` if the header or
/// the communication state is missing.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let (mut out, offset) = decode_binary_header(bits, 26)?;
    // The communication state is the *last* 20 bits, whatever the data length.
    require_bits(26, bits.len(), offset + COMM_STATE_BITS)?;
    let comm_start = bits.len() - COMM_STATE_BITS;
    put(&mut out, "data", bits.raw(offset, comm_start - offset));
    put(&mut out, "data_bits", comm_start - offset);
    let selector = bits.uint(comm_start, 1);
    put(&mut out, "comm_state_selector", selector);
    let comm_state = if selector == 1 {
        decode_itdma(bits, comm_start + 1)
    } else {
        decode_sotdma(bits, comm_start + 1)
    };
    put(&mut out, "comm_state", comm_state);
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcast_sotdma() {
        // synthetic sentence from chapter 16
        let bits = Bits::from_payload("J52MJh400P4SAFN9btogaJFU0<0I", 0).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["data_bits"], Json::Int(92));
        assert_eq!(out["comm_state_selector"], Json::Int(0));
        let Json::Obj(cs) = &out["comm_state"] else { panic!("not an object") };
        assert_eq!(cs["received_stations"], Json::Int(25));
    }

    #[test]
    fn too_short_for_comm_state() {
        // sample-data/RightwhaleAisProject/20081013.log line 411757 (noise)
        let bits = Bits::from_payload("JNu2<;>wlUg=NuT", 2).unwrap();
        assert_eq!(
            decode(&bits).unwrap_err().to_string(),
            "message 26 too short: 88 bits < 108"
        );
    }
}
