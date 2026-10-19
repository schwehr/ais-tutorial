//! `msg_20.rs` — message 20: data link management message (chapter 13).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.18, Table 72. A base station announces the
//! slots it has reserved for itself (FATDMA — fixed access TDMA) so that
//! mobile stations within 120 NM keep out of them (Annex 2 §3.3.4.3).
//!
//! Bit layout (zero-based offsets; lengths from Table 72):
//!
//! ```text
//!      0- 5   6  message id (20)
//!      6- 7   2  repeat indicator
//!      8-37  30  source station id (MMSI of the base station)
//!     38-39   2  spare
//!     40-69  30  reservation 1:  offset number (12), number of slots (4),
//!                                time-out in minutes (3), increment (11)
//!     70-99  30  reservation 2 (optional)
//!    100-129 30  reservation 3 (optional)
//!    130-159 30  reservation 4 (optional)
//!    then 0, 2, 4 or 6 spare bits to reach a byte boundary; 72–160 bits total.
//! ```
//!
//! Length rule: at least 72 bits (else `DecodeError::TooShort`). Reservation
//! groups are read while 30 bits remain (at most four); whatever is left
//! over is reported as `spare2` with its width in `spare2_bits` (null if
//! wider than 32 bits — that is a broken message, not a spare field).
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `Vec<Json>` + `while … { v.push(…) }` is the repeated-group loop first
//!   seen in msg_7_13.rs.
//! * `spare2` is built as an `Option<u64>` in a `match` over three ranges
//!   of `spare_bits`; `From<Option<T>>` turns `None` into `null`.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

pub const RESERVATION_BITS: usize = 30;
pub const MAX_RESERVATIONS: usize = 4;

/// One offset/slots/time-out/increment group (Table 72).
pub fn decode_reservation(bits: &Bits, start: usize) -> Obj {
    let offset = bits.uint(start, 12);
    let slots = bits.uint(start + 12, 4);
    let timeout = bits.uint(start + 16, 3);
    let increment = bits.uint(start + 19, 11);
    let mut r = Obj::new();
    put(&mut r, "offset", offset);
    put(&mut r, "slots", slots);
    put(&mut r, "timeout_min", timeout);
    put(&mut r, "increment", increment);
    // Table 72 footnote: an interrogated base station with nothing to
    // announce sends one group with offset, slots and time-out all zero.
    put(&mut r, "available", !(offset == 0 && slots == 0 && timeout == 0));
    r
}

/// Decode a message 20. Fails with `DecodeError::TooShort` below 72 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(20, bits.len(), 72)?;
    let mut reservations: Vec<Json> = Vec::new();
    let mut pos = 40;
    while reservations.len() < MAX_RESERVATIONS && bits.has(pos, RESERVATION_BITS) {
        reservations.push(Json::Obj(decode_reservation(bits, pos)));
        pos += RESERVATION_BITS;
    }
    let spare_bits = bits.len() - pos;
    // The spec allows 0, 2, 4 or 6 trailing bits. We print whatever is there
    // as an integer so that odd lengths show up; anything wider than 32 bits
    // is not a spare field but a broken message and is reported as null.
    let spare2: Option<u64> = match spare_bits {
        0 => Some(0),
        1..=32 => Some(bits.uint(pos, spare_bits)),
        _ => None,
    };
    let mut out = Obj::new();
    put(&mut out, "msg_type", bits.uint(0, 6));
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    put(&mut out, "reservations", reservations);
    put(&mut out, "spare2", spare2);
    put(&mut out, "spare2_bits", spare_bits);
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_block() {
        // sample-data/2007hydro/norfolk-log.ais line 22
        let bits = Bits::from_payload("D03OviQsIN9H", 0).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["mmsi"], Json::Int(3669702));
        let Json::Arr(res) = &out["reservations"] else { panic!("not an array") };
        assert_eq!(res.len(), 1);
        let Json::Obj(r) = &res[0] else { panic!("not an object") };
        assert_eq!(r["offset"], Json::Int(1974));
        assert_eq!(r["slots"], Json::Int(5));
        assert_eq!(r["timeout_min"], Json::Int(7));
        assert_eq!(r["increment"], Json::Int(150));
        assert_eq!(out["spare2_bits"], Json::Int(2));
    }

    #[test]
    fn two_blocks_and_too_short() {
        // synthetic sentence from chapter 13: two reservation blocks, 104 bits
        let bits = Bits::from_payload("D03OviP1QN9I7lW6D0", 4).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        let Json::Arr(res) = &out["reservations"] else { panic!("not an array") };
        assert_eq!(res.len(), 2);
        assert_eq!(out["spare2_bits"], Json::Int(4));
        let short = Bits::from_payload("D03OviQsIN9", 0).unwrap();
        assert_eq!(
            decode(&short).unwrap_err().to_string(),
            "message 20 too short: 66 bits < 72"
        );
    }
}
