//! `common.rs` — small things shared by several message decoders.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * Python raised `DecodeError` as an exception; Rust has no exceptions.
//!   Instead a decoder returns `Result<Json, DecodeError>`: either
//!   `Ok(value)` or `Err(error)`. The caller must look inside (usually with
//!   `match` or the `?` operator), so an error can never be forgotten.
//! * `DecodeError` is an `enum` whose variants carry data. Pattern matching
//!   on it (`match e { DecodeError::TooShort { .. } => … }`) is how the
//!   driver tells "not implemented yet" apart from "too few bits".
//! * `impl fmt::Display` gives the error its human-readable text, which is
//!   what `e.to_string()` and `format!("{e}")` print. The text must match
//!   the Python version exactly because the test-suite compares it.
//! * `impl std::error::Error` is a marker that lets this type be used with
//!   `Box<dyn Error>` and `?` in other code. It needs no methods.
//! * The lookup tables are `match` expressions rather than `HashMap`s: a
//!   `match` over integer ranges (`1..=19`) is the idiomatic replacement for
//!   Python's `dict.get(code, default)` plus `if lo <= code <= hi` chains.

use std::fmt;

/// A message could not be decoded.
///
/// All four language versions print the same text for the too-short
/// situation so that their outputs can be compared:
/// `message 5 too short: 160 bits < 424`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The payload has fewer bits than the message's fixed part needs.
    TooShort {
        msg_type: u32,
        need: usize,
        have: usize,
    },
    /// The decoder for this message type is still a stub. Python raised
    /// `NotImplementedError`; the driver turns either into
    /// `{"msg_type":N,"not_implemented":true}`.
    NotImplemented(u32),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::TooShort {
                msg_type,
                need,
                have,
            } => write!(f, "message {msg_type} too short: {have} bits < {need}"),
            DecodeError::NotImplemented(n) => write!(f, "message {n} decoder not written yet"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Fail with `DecodeError::TooShort` unless the payload has `need` bits.
///
/// Returning `Result<(), DecodeError>` lets a decoder write
/// `require_bits(1, bits.len(), 168)?;` — the `?` returns the error to the
/// caller immediately, or does nothing on `Ok(())`.
pub fn require_bits(msg_type: u32, bits_len: usize, need: usize) -> Result<(), DecodeError> {
    if bits_len < need {
        return Err(DecodeError::TooShort {
            msg_type,
            need,
            have: bits_len,
        });
    }
    Ok(())
}

/// Type of electronic position fixing device (EPFD). ITU-R M.1371-5 Annex 8
/// Table 51 (message 4); the same code list is used in messages 5, 9, 11,
/// 19, 21 and 24.
pub fn epfd_text(code: u64) -> &'static str {
    // `&'static str` is a string literal baked into the program; it lives
    // for the whole run so we can hand out references to it freely.
    match code {
        0 => "undefined",
        1 => "GPS",
        2 => "GLONASS",
        3 => "combined GPS/GLONASS",
        4 => "Loran-C",
        5 => "Chayka",
        6 => "integrated navigation system",
        7 => "surveyed",
        8 => "Galileo",
        15 => "internal GNSS",
        _ => "not used",
    }
}

/// Type of ship and cargo type. ITU-R M.1371-5 Annex 8 Table 53. The first
/// digit is the vessel category, the second digit (for 2x–9x) the hazard
/// category (1 = DG/HS/MP category X … 4 = Y, 5–8 reserved, 9 = no additional
/// information). We spell out the common ones and derive the rest.
pub fn ship_type_text(code: u64) -> String {
    // Python's SHIP_TYPE_SPECIAL dictionary, as a match.
    let special = match code {
        0 => Some("not available (default)"),
        30 => Some("fishing"),
        31 => Some("towing"),
        32 => Some("towing, length > 200 m or breadth > 25 m"),
        33 => Some("dredging or underwater operations"),
        34 => Some("diving operations"),
        35 => Some("military operations"),
        36 => Some("sailing"),
        37 => Some("pleasure craft"),
        38 | 39 => Some("reserved"),
        50 => Some("pilot vessel"),
        51 => Some("search and rescue vessel"),
        52 => Some("tug"),
        53 => Some("port tender"),
        54 => Some("anti-pollution equipment"),
        55 => Some("law enforcement"),
        56 | 57 => Some("spare - local vessel"),
        58 => Some("medical transport"),
        59 => Some("noncombatant ship (RR Res. 18)"),
        _ => None,
    };
    if let Some(text) = special {
        return text.to_string();
    }
    match code {
        1..=19 => return "reserved for future use".to_string(),
        100..=199 => return "reserved for regional use".to_string(),
        200.. => return "reserved for future use".to_string(),
        _ => {}
    }
    let (tens, ones) = (code / 10, code % 10);
    // Python's SHIP_TYPE_FIRST_DIGIT without 3 and 5 (those are all listed
    // individually above or fall through to "reserved").
    let category = match tens {
        2 => "WIG",
        4 => "HSC",
        6 => "passenger",
        7 => "cargo",
        8 => "tanker",
        9 => "other",
        _ => return "reserved".to_string(),
    };
    let hazard = match ones {
        0 => "all ships of this type",
        1 => "hazardous category A",
        2 => "hazardous category B",
        3 => "hazardous category C",
        4 => "hazardous category D",
        9 => "no additional information",
        _ => "reserved", // 5..=8
    };
    format!("{category}, {hazard}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_text() {
        let e = DecodeError::TooShort {
            msg_type: 5,
            need: 424,
            have: 160,
        };
        assert_eq!(e.to_string(), "message 5 too short: 160 bits < 424");
        assert!(require_bits(1, 168, 168).is_ok());
        assert!(require_bits(1, 100, 168).is_err());
    }

    #[test]
    fn ship_types() {
        assert_eq!(ship_type_text(70), "cargo, all ships of this type");
        assert_eq!(ship_type_text(81), "tanker, hazardous category A");
        assert_eq!(ship_type_text(36), "sailing");
        assert_eq!(ship_type_text(45), "HSC, reserved");
        assert_eq!(ship_type_text(20), "WIG, all ships of this type");
        assert_eq!(ship_type_text(40), "HSC, all ships of this type");
        assert_eq!(ship_type_text(60), "passenger, all ships of this type");
        assert_eq!(ship_type_text(99), "other, no additional information");
        assert_eq!(ship_type_text(5), "reserved for future use");
        assert_eq!(ship_type_text(150), "reserved for regional use");
        assert_eq!(ship_type_text(255), "reserved for future use");
        assert_eq!(epfd_text(1), "GPS");
        assert_eq!(epfd_text(9), "not used");
    }
}
