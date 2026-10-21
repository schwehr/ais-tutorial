//! `msg_16.rs` — message 16: assignment mode command.
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.14, Table 67 (96 or 144 bits) and the
//! note (1) and paragraph below it that give the meaning of offset and
//! increment. Annex 2 §3.3.6 describes what the assigned station does.
//!
//! Bit layout (zero-based offsets; lengths from Table 67):
//!
//! ```text
//!      0- 5   6  message id (16)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI of the assigning base station)
//!     38-39   2  spare
//!     40-69  30  destination id A
//!     70-81  12  offset A
//!     82-91  10  increment A
//!     ------- then EITHER
//!     92-95   4  spare (padding to a byte boundary)   -> 96 bits
//!     ------- OR
//!     92-121 30  destination id B
//!    122-133 12  offset B
//!    134-143 10  increment B                          -> 144 bits
//! ```
//!
//! Meaning of (offset, increment) [Table 67 note 1]: increment 0 = reporting
//! rate assignment, offset = reports per 10 min; increment 1-6 = slot
//! assignment with 1125/375/225/125/75/45 slots between transmissions;
//! 7 = undefined (disregard); 8-1023 not defined in M.1371-5.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `Option<u64>` is Rust's "value or nothing". `put` accepts it because
//!   jsonout implements `From<Option<T>>`, mapping `None` to `null`.
//! * `slot_increment` returns `Option<u64>` from a `match` whose arms are
//!   `Some(1125)` … `_ => None`; the caller then matches on *that* to pick
//!   the text — two small matches instead of one if/else chain.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

/// Table 67 text under note (1): increment code -> slots between assigned slots.
pub fn slot_increment(code: u64) -> Option<u64> {
    match code {
        1 => Some(1125),
        2 => Some(375),
        3 => Some(225),
        4 => Some(125),
        5 => Some(75),
        6 => Some(45),
        _ => None,
    }
}

/// Interpret one (destination, offset, increment) triple.
fn assignment(mmsi: u64, offset: u64, increment: u64) -> Json {
    let mut reports: Option<u64> = None;
    let slots = slot_increment(increment);
    let text = if increment == 0 {
        reports = Some(offset);
        "reporting rate"
    } else if slots.is_some() {
        "slot assignment"
    } else if increment == 7 {
        "undefined increment (disregard)"
    } else {
        "increment not defined in M.1371-5"
    };
    let mut a = Obj::new();
    put(&mut a, "mmsi", mmsi);
    put(&mut a, "offset", offset);
    put(&mut a, "increment", increment);
    put(&mut a, "assignment_text", text);
    put(&mut a, "reports_per_10_min", reports);
    put(&mut a, "slot_increment", slots);
    Json::Obj(a)
}

/// Decode a message 16. Fails with `DecodeError::TooShort` below 96 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 96)?;

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    let mut assignments: Vec<Json> =
        vec![assignment(bits.uint(40, 30), bits.uint(70, 12), bits.uint(82, 10))];
    // `if` is an expression: each branch does its work and then yields the
    // offset where the fixed part ends, so `pos` is initialised in one step.
    let pos = if bits.has(92, 52) {
        assignments.push(assignment(bits.uint(92, 30), bits.uint(122, 12), bits.uint(134, 10)));
        144
    } else {
        put(&mut out, "spare2", bits.uint(92, 4));
        96
    };
    put(&mut out, "assignments", assignments);
    put(&mut out, "trailing_bits", bits.len() - pos);
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_and_slot() {
        // Chapter 11 §11.3 synthetic: 60 reports per 10 min.
        let rate = Bits::from_payload("@03OviQGMIWp3h00", 0).unwrap();
        let text = decode(&rate).unwrap().to_string();
        assert!(text.contains("\"reports_per_10_min\":60"));
        assert!(text.contains("\"slot_increment\":null"));
        // Slot assignment, increment code 2 = 375 slots.
        let slot = Bits::from_payload("@03OviQGMIWp1@0P", 0).unwrap();
        let text = decode(&slot).unwrap().to_string();
        assert!(text.contains("\"slot_increment\":375"));
        // 144 bits: two assignments.
        let two = Bits::from_payload("@03OviQGMIWp1@0UN6kFPN00", 0).unwrap();
        assert_eq!(two.len(), 144);
        let text = decode(&two).unwrap().to_string();
        assert_eq!(text.matches("\"increment\"").count(), 2);
    }
}
