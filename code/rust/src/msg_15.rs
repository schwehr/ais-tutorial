//! `msg_15.rs` — message 15: interrogation.
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.13, Table 66 (88-160 bits) and Table 65
//! (which station class may ask which other class for which messages).
//! Annex 2 §3.3.2.3 ("polled" mode) and Annex 7 §4.3.3.3.3 describe the response:
//! automatic, on the channel where the interrogation was received.
//!
//! Bit layout (zero-based offsets; lengths from Table 66):
//!
//! ```text
//!      0- 5   6  message id (15)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI of the interrogating station)
//!     38-39   2  spare
//!     40-69  30  destination id 1 (first interrogated station)
//!     70-75   6  message id 1.1 (first message type requested from station 1)
//!     76-87  12  slot offset 1.1 (0 = let the responder choose a slot)
//!     ------- 88 bits: one station, one message -------
//!     88-89   2  spare
//!     90-95   6  message id 1.2 (second message type requested from station 1)
//!     96-107 12  slot offset 1.2
//!     108-109 2  spare
//!     ------- 110 bits as written in Table 66; 112 when padded to a byte -------
//!     110-139 30 destination id 2 (second interrogated station)
//!     140-145 6  message id 2.1
//!     146-157 12 slot offset 2.1
//!     158-159 2  spare
//!     ------- 160 bits: two stations -------
//! ```
//!
//! Output and length rules are those of the Python module (msg_15.py):
//! `interrogations` is an array of {mmsi, msg_type, msg_type_text,
//! slot_offset}; the 1.2 group is included only when present and its
//! message id is non-zero; spare fields appear as spare/spare2/spare3/spare4
//! when present; `trailing_bits` counts what is left (2 for the 112-bit
//! form).
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `message_name` is one big `match` with a `_ =>` arm; the compiler
//!   checks it is exhaustive. Python used a dict with `.get(default)`.
//! * `fn request(..) -> Json` builds one element of the array; `Vec<Json>`
//!   then converts into `Json::Arr` through `put`'s `Into<Json>` bound.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

/// Table 46 message names, used for the `_text` companion of a requested
/// message type. The names are abbreviated from the "Name" column.
pub fn message_name(msg_id: u64) -> &'static str {
    match msg_id {
        0 => "none",
        1 => "position report",
        2 => "position report (assigned)",
        3 => "position report (interrogated)",
        4 => "base station report",
        5 => "static and voyage related data",
        6 => "binary addressed message",
        7 => "binary acknowledgement",
        8 => "binary broadcast message",
        9 => "standard SAR aircraft position report",
        10 => "UTC/date inquiry",
        11 => "UTC/date response",
        12 => "addressed safety related message",
        13 => "safety related acknowledgement",
        14 => "safety related broadcast message",
        15 => "interrogation",
        16 => "assignment mode command",
        17 => "DGNSS broadcast binary message",
        18 => "standard Class B equipment position report",
        19 => "extended Class B equipment position report",
        20 => "data link management message",
        21 => "aids-to-navigation report",
        22 => "channel management",
        23 => "group assignment command",
        24 => "static data report",
        25 => "single slot binary message",
        26 => "multiple slot binary message with communications state",
        27 => "position report for long-range applications",
        _ => "undefined",
    }
}

/// One (station, message, slot) triple of the `interrogations` array.
fn request(mmsi: u64, msg_id: u64, slot_offset: u64) -> Json {
    let mut r = Obj::new();
    put(&mut r, "mmsi", mmsi);
    put(&mut r, "msg_type", msg_id);
    put(&mut r, "msg_type_text", message_name(msg_id));
    put(&mut r, "slot_offset", slot_offset);
    Json::Obj(r)
}

/// Decode a message 15. Fails with `DecodeError::TooShort` below 88 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 88)?;

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    let mmsi1 = bits.uint(40, 30);
    let mut interrogations: Vec<Json> = vec![request(mmsi1, bits.uint(70, 6), bits.uint(76, 12))];
    let mut pos = 88;

    // Second request to the first station (bits 88-107, plus its spare).
    if bits.has(88, 20) {
        put(&mut out, "spare2", bits.uint(88, 2));
        let msg_id_12 = bits.uint(90, 6);
        if msg_id_12 != 0 {
            interrogations.push(request(mmsi1, msg_id_12, bits.uint(96, 12)));
        }
        pos = 108;
    }

    // A second station (bits 108-157, plus the trailing spare).
    if bits.has(108, 50) {
        put(&mut out, "spare3", bits.uint(108, 2));
        interrogations.push(request(bits.uint(110, 30), bits.uint(140, 6), bits.uint(146, 12)));
        pos = 158;
        if bits.has(158, 2) {
            put(&mut out, "spare4", bits.uint(158, 2));
            pos = 160;
        }
    } else if bits.has(108, 2) {
        // The 110/112-bit form: the spare after group 1.2, then 0 or 2
        // padding bits.
        put(&mut out, "spare3", bits.uint(108, 2));
        pos = 110;
    }

    put(&mut out, "interrogations", interrogations);
    put(&mut out, "trailing_bits", bits.len() - pos);
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes() {
        // 88 bits, recorded: 338040883 asks 366831230 for message 5.
        let one = Bits::from_payload("?52HH<iGMIWpD00", 2).unwrap();
        assert_eq!(one.len(), 88);
        let text = decode(&one).unwrap().to_string();
        assert!(text.contains("\"msg_type\":5,\"msg_type_text\":\"static and voyage related data\""));
        assert!(text.contains("\"trailing_bits\":0"));
        // 112 bits (byte-aligned 110): two requests, two padding bits.
        let two = Bits::from_payload("?03OviQGMIWp<005000", 2).unwrap();
        assert_eq!(two.len(), 112);
        let text = decode(&two).unwrap().to_string();
        assert!(text.contains("\"trailing_bits\":2"));
        assert_eq!(text.matches("\"slot_offset\"").count(), 2);
        // 87 bits: too short.
        let short = Bits::from_payload("?52HH<iGMIWpD00", 3).unwrap();
        assert_eq!(
            decode(&short).unwrap_err().to_string(),
            "message 15 too short: 87 bits < 88"
        );
    }
}
