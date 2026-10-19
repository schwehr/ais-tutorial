//! `msg_24.rs` — message 24: static data report, parts A and B (chapter 14).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.22, Table 78 (part A, 160 bits) and
//! Table 79 (part B, 168 bits), Table 79A (vendor ID sub-fields). Message 24
//! was introduced in M.1371-2 (Annex 7 Tables 60–61), together with the rule
//! that an unregistered daughter vessel puts the mother ship's MMSI in the
//! dimension field; M.1371-4 (Tables 75, 76, 76A) added the vendor-ID split;
//! M.1371-5 dropped the mother-ship sentence and redefined the last six bits
//! of part B as a 4-bit type of EPFD plus 2 spare bits. Dimensions: §3.3.3
//! Fig. 41. Ship type: Table 53. EPFD: Table 51. Auxiliary-craft MMSI format
//! 98MIDxxxx: ITU-R M.585 Annex 1.
//!
//! Bit layout (zero-based offsets):
//!
//! ```text
//!   common   0- 5   6  message id (24)
//!            6- 7   2  repeat indicator
//!            8-37  30  user id (MMSI)
//!           38-39   2  part number: 0 = part A, 1 = part B, 2-3 not allowed
//!   part A  40-159 120 name, 20 six-bit characters; all '@' = n/a
//!          160-167  8  (spare) — present only on units that send 168 bits
//!   part B  40- 47  8  type of ship and cargo type; 0 = n/a
//!           48- 89 42  vendor ID; since M.1371-4 split into
//!                         48-65 18  manufacturer's ID, 3 six-bit characters
//!                         66-69  4  unit model code
//!                         70-89 20  unit serial number
//!           90-131 42  call sign, 7 six-bit characters; all '@' = n/a
//!          132-161 30  dimensions / reference point (A 9, B 9, C 6, D 6), or
//!                      for an auxiliary craft (MMSI 98MIDxxxx) the MMSI of the
//!                      mother ship (M.1371-2 to -4 rule)
//!          162-165  4  type of EPFD (Table 79; spare in M.1371-2 to -4)
//!          166-167  2  spare
//! ```
//!
//! EPFD on old units: a unit built to M.1371-2/-3/-4 sends six spare bits,
//! which are zero, so it decodes as `epfd: 0` = "undefined" — the same value
//! a -5 unit sends when it does not know its fix type. `epfd: 0` therefore
//! says nothing about the edition the unit was built to; a non-zero `epfd`
//! is only possible on a -5 unit (or a bit error).
//!
//! Length rule: part A needs 160 bits; anything beyond 160 is reported as
//! `spare` (at most 8 bits are decoded) with its width in `spare_bits`.
//! Part B needs 168 bits. Part numbers 2 and 3 return the remainder of the
//! payload undecoded as `data`/`data_bits`.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `match part { 0 => …, 1 => …, _ => … }` replaces Python's
//!   `if/elif/else`; each arm is a block that mutates `out`.
//! * `(980_000_000..990_000_000).contains(&mmsi)` is the idiomatic range
//!   test; the underscores in the literals are ignored by the compiler.
//! * `spare_bits.min(8)` is a method on integers, no `std::cmp::min` import
//!   needed.

use crate::bits::Bits;
use crate::common::{epfd_text, require_bits, ship_type_text, DecodeError};
use crate::jsonout::{put, Json, Obj};
use crate::msg_19::decode_dimensions;

pub fn part_text(code: u64) -> &'static str {
    match code {
        0 => "part A",
        1 => "part B",
        _ => "reserved",
    }
}

/// True for MMSIs of the form 98MIDxxxx (craft associated with a parent ship).
pub fn is_auxiliary_craft(mmsi: u64) -> bool {
    (980_000_000..990_000_000).contains(&mmsi)
}

/// Decode a message 24 part A or part B. Fails with `DecodeError::TooShort`
/// if the part's fixed length is not present.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 40)?;
    let mmsi = bits.uint(8, 30);
    let part = bits.uint(38, 2);
    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", mmsi);
    put(&mut out, "part_number", part);
    put(&mut out, "part_text", part_text(part));

    match part {
        0 => {
            require_bits(msg_type as u32, bits.len(), 160)?;
            put(&mut out, "name", bits.text(40, 120));
            let spare_bits = bits.len() - 160;
            put(&mut out, "spare_bits", spare_bits);
            let spare = if spare_bits > 0 {
                bits.uint(160, spare_bits.min(8))
            } else {
                0
            };
            put(&mut out, "spare", spare);
        }
        1 => {
            require_bits(msg_type as u32, bits.len(), 168)?;
            let ship_type = bits.uint(40, 8);
            put(&mut out, "ship_type", ship_type);
            put(&mut out, "ship_type_text", ship_type_text(ship_type));
            put(&mut out, "vendor_id", bits.text(48, 42));
            put(&mut out, "manufacturer_id", bits.text(48, 18));
            put(&mut out, "unit_model_code", bits.uint(66, 4));
            put(&mut out, "unit_serial_number", bits.uint(70, 20));
            put(&mut out, "call_sign", bits.text(90, 42));
            put(&mut out, "auxiliary_craft", is_auxiliary_craft(mmsi));
            let epfd = bits.uint(162, 4);
            put(&mut out, "epfd", epfd);
            put(&mut out, "epfd_text", epfd_text(epfd));
            put(&mut out, "spare", bits.uint(166, 2));
            if is_auxiliary_craft(mmsi) {
                put(&mut out, "mothership_mmsi", bits.uint(132, 30));
            } else {
                out.extend(decode_dimensions(bits, 132));
            }
        }
        _ => {
            put(&mut out, "data", bits.raw(40, bits.len() - 40));
            put(&mut out, "data_bits", bits.len() - 40);
        }
    }
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unwrap the object inside a `Json::Obj` (tests only).
    fn as_obj(json: Json) -> Obj {
        match json {
            Json::Obj(o) => o,
            _ => panic!("not an object"),
        }
    }

    #[test]
    fn part_a_and_b() {
        // data/ch14-class-b.nmea lines 21-22: AMY PHILBRICK.
        let a = Bits::from_payload("H5N6=MP4mV10PTh98T<d0000000", 2).unwrap();
        let out = as_obj(decode(&a).unwrap());
        assert_eq!(out["name"], Json::from("AMY PHILBRICK"));
        assert_eq!(out["spare_bits"], Json::from(0u64));
        let b = Bits::from_payload("H5N6=MTNC9=B140G3Glipq1@D520", 0).unwrap();
        let out = as_obj(decode(&b).unwrap());
        assert_eq!(out["vendor_id"], Json::from("SIMRAD"));
        assert_eq!(out["manufacturer_id"], Json::from("SIM"));
        assert_eq!(out["call_sign"], Json::from("WCW4189"));
        assert_eq!(out["dim_stern_m"], Json::from(20u64));
        assert_eq!(out["length_m"], Json::from(30u64));
        assert_eq!(out["epfd"], Json::from(0u64));
        assert_eq!(out["spare"], Json::from(0u64));
    }

    #[test]
    fn auxiliary_craft_range() {
        assert!(is_auxiliary_craft(983380001));
        assert!(!is_auxiliary_craft(979000001));
        assert!(!is_auxiliary_craft(990000000));
    }
}
