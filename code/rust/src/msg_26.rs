//! `msg_26.rs` — message 26: multiple slot binary message with communications state.
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.24, Table 79.
//!
//! STUB: this decoder has not been written yet. The chapter that covers this
//! message replaces this file. Keep the signature
//! `pub fn decode(bits: &Bits) -> Result<Json, DecodeError>`.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * Python's stub raised `NotImplementedError`; Rust has no exceptions, so
//!   the stub returns `Err(DecodeError::NotImplemented(n))` and the driver
//!   prints `{"msg_type":n,"not_implemented":true}`.
//! * The message type is read back from the first six bits rather than
//!   hard-coded, because this module may serve more than one type.

use crate::bits::Bits;
use crate::common::DecodeError;
use crate::jsonout::Json;

/// Not written yet — always returns `DecodeError::NotImplemented`.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    Err(DecodeError::NotImplemented(bits.uint(0, 6) as u32))
}
