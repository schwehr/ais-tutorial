//! `bits.rs` — turning an AIS payload into bits, and bits into numbers and text.
//!
//! Everything in an AIS message is a *bit field*: "bits 8–37 are the MMSI",
//! "bits 61–88 are the longitude in 1/10 000 minute". The job of this module
//! is therefore very small:
//!
//!   1. Undo the six-bit ASCII "armouring" that NMEA uses to carry binary data
//!      in printable characters (see chapter 2).
//!   2. Give the rest of the code a tidy way to pull out unsigned integers,
//!      signed integers, booleans and six-bit text starting at any bit offset.
//!
//! Spec references:
//!   * Armouring table: IEC 61162-1 (NMEA 0183) §7.3.3 "Six-bit binary field
//!     conversion"; ITU-R M.1371-5 Annex 8 §3 (as reproduced in AIVDM.adoc
//!     "AIVDM/AIVDO Payload Armoring").
//!   * Six-bit ASCII for text fields: ITU-R M.1371-5 Annex 8 Table 47.
//!   * Two's complement for signed fields: ITU-R M.1371-5 Annex 8 §3.3.7.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * Python kept the bits as a string of `'0'`/`'1'`. Rust keeps a
//!   `Vec<u8>` holding the *numbers* 0 and 1 — one byte per bit. That is
//!   still wasteful (a real decoder would pack 8 bits per byte) but it keeps
//!   `bits[61..89]` as readable as the Python slice.
//! * `&self` methods borrow the struct immutably; nothing here mutates a
//!   `Bits` after construction, so every method takes `&self`.
//! * `Result<Bits, BitsError>` replaces Python's `raise ValueError`. The
//!   `?` operator inside `from_payload` forwards the first error it meets.
//! * `uint`/`sint` *panic* (via `assert!`) when asked for bits that do not
//!   exist. That mirrors Python's `IndexError`: both are programmer errors,
//!   because every decoder must call `require_bits` first. Input-dependent
//!   problems (bad characters, bad fill count) are `Result`s instead.
//! * `u64`/`i64` are explicit fixed-width integers; Python's `int` is
//!   unbounded. No AIS field is wider than 64 bits so nothing is lost.

use std::fmt;

// --- 1. Armouring ---------------------------------------------------------
//
// Each payload character carries 6 bits. The sender adds 48 to the 6-bit
// value; if the result is above 87 it adds another 8 to skip the characters
// 'X'..'_' (which were avoided because some old equipment mangled them).
// To decode we do the reverse: subtract 48, and if the result is above 40
// subtract 8 more.

/// Problems turning a payload string into bits. The `Display` texts match
/// the Python `ValueError` messages byte for byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitsError {
    /// A payload character outside `'0'..='w'` or inside `'X'..='_'`.
    IllegalChar(char),
    /// The fill-bit count (field 7 of `!AIVDM`) was not in 0..=5.
    BadFillBits(i64),
}

impl fmt::Display for BitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitsError::IllegalChar(c) => {
                write!(f, "illegal armour character {}", crate::nmea::py_repr_char(*c))
            }
            BitsError::BadFillBits(n) => write!(f, "fill bits must be 0..5, got {n}"),
        }
    }
}

impl std::error::Error for BitsError {}

/// Return the 6-bit value (0–63) carried by one payload character.
///
/// ```
/// use ais_tutorial::bits::armor_char_to_value;
/// assert_eq!(armor_char_to_value('0'), Ok(0));
/// assert_eq!(armor_char_to_value('W'), Ok(39));
/// assert_eq!(armor_char_to_value('`'), Ok(40));
/// assert_eq!(armor_char_to_value('w'), Ok(63));
/// assert!(armor_char_to_value('X').is_err());
/// ```
pub fn armor_char_to_value(ch: char) -> Result<u8, BitsError> {
    let code = ch as u32; // the Unicode code point, like Python's ord()
    if !(48..=119).contains(&code) || (88..=95).contains(&code) {
        // outside '0'..'w', or inside 'X'..'_'
        return Err(BitsError::IllegalChar(ch));
    }
    let mut value = code - 48;
    if value > 40 {
        value -= 8;
    }
    // `as u8` is safe here: value is at most 63. We checked, so the cast
    // cannot truncate.
    Ok(value as u8)
}

/// Convert an armoured payload to a vector of 0/1 bytes.
///
/// `fill_bits` is field 7 of the !AIVDM sentence: how many of the low-order
/// bits of the *last* character are padding and must be thrown away.
///
/// ```
/// use ais_tutorial::bits::payload_to_bits;
/// assert_eq!(payload_to_bits("1", 0).unwrap(), vec![0, 0, 0, 0, 0, 1]);
/// // last char 'w' = 111111, drop 4 bits
/// assert_eq!(payload_to_bits("1w", 4).unwrap(), vec![0, 0, 0, 0, 0, 1, 1, 1]);
/// ```
pub fn payload_to_bits(payload: &str, fill_bits: i64) -> Result<Vec<u8>, BitsError> {
    if !(0..=5).contains(&fill_bits) {
        return Err(BitsError::BadFillBits(fill_bits));
    }
    let mut bits = Vec::with_capacity(payload.len() * 6);
    for ch in payload.chars() {
        let value = armor_char_to_value(ch)?; // `?`: stop at the first bad char
        // Most significant bit first: shift right by 5, 4, ... 0 and mask.
        for shift in (0..6).rev() {
            bits.push((value >> shift) & 1);
        }
    }
    // Python: bits[:-fill_bits]. `saturating_sub` never goes below zero,
    // which matches how Python slicing clamps a too-large negative index.
    let keep = bits.len().saturating_sub(fill_bits as usize);
    bits.truncate(keep);
    Ok(bits)
}

// --- 2. Six-bit ASCII used *inside* text fields -----------------------------
//
// Do not confuse this with the armouring above! Text fields (ship name, call
// sign, destination, safety messages) pack one character into 6 bits using a
// different table: 0–31 map to '@'..'_' and 32–63 map to ' '..'?'.

/// The 64-character six-bit alphabet, indexed by field value.
pub const SIXBIT_ASCII: &[u8; 64] =
    b"@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_ !\"#$%&'()*+,-./0123456789:;<=>?";

// --- 3. The Bits reader -----------------------------------------------------

/// A bit string with typed accessors.
///
/// All offsets are *zero-based* bit positions, exactly as in the field
/// tables of this tutorial (ITU-R M.1371 itself counts bits per field,
/// GPSD's AIVDM.adoc and this tutorial count absolute offsets from 0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bits {
    bits: Vec<u8>, // each element is 0 or 1
}

impl Bits {
    /// Wrap an existing bit vector. Panics if any element is not 0 or 1 —
    /// that would be a bug in the caller, not bad input.
    pub fn new(bits: Vec<u8>) -> Bits {
        assert!(
            bits.iter().all(|&b| b == 0 || b == 1),
            "Bits expects a vector of 0 and 1"
        );
        Bits { bits }
    }

    /// Build directly from an !AIVDM payload and its fill-bit count.
    pub fn from_payload(payload: &str, fill_bits: i64) -> Result<Bits, BitsError> {
        Ok(Bits::new(payload_to_bits(payload, fill_bits)?))
    }

    /// Number of bits in the message (Python's `len(bits)`).
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// `true` for an empty message. Rust convention: a type with `len()`
    /// also offers `is_empty()` (the linter insists).
    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    /// True if the field [start, start+length) lies inside the message.
    pub fn has(&self, start: usize, length: usize) -> bool {
        start + length <= self.bits.len()
    }

    /// Unsigned big-endian integer from `length` bits at `start`.
    pub fn uint(&self, start: usize, length: usize) -> u64 {
        assert!(
            self.has(start, length),
            "field {start}+{length} beyond {} bits",
            self.bits.len()
        );
        assert!(length <= 64, "uint() supports at most 64 bits");
        let mut value: u64 = 0;
        for &bit in &self.bits[start..start + length] {
            value = (value << 1) | u64::from(bit);
        }
        value
    }

    /// Signed (two's complement) integer. The first bit is the sign.
    pub fn sint(&self, start: usize, length: usize) -> i64 {
        let raw = self.uint(start, length);
        // Values wider than 63 bits would not fit an i64 — not an AIS case.
        assert!(length < 64, "sint() supports at most 63 bits");
        let mut value = raw as i64; // fits: raw < 2^63
        if length > 0 && self.bits[start] == 1 {
            value -= 1i64 << length; // negative: subtract 2**length
        }
        value
    }

    /// A single bit as a boolean.
    pub fn flag(&self, start: usize) -> bool {
        self.uint(start, 1) == 1
    }

    /// Six-bit ASCII text occupying `length` bits (a multiple of 6).
    ///
    /// Per ITU-R M.1371-5 Annex 8 §3.3.8, unused characters are '@'; in
    /// practice names are often space-padded instead. We cut at the first
    /// '@' and strip trailing spaces.
    pub fn text(&self, start: usize, length: usize) -> String {
        let mut s = String::new();
        // Python: range(start, start + length - 5, 6). `saturating_sub`
        // keeps the loop empty for lengths below 6, as Python's range does.
        let stop = (start + length).saturating_sub(5);
        let mut pos = start;
        while pos < stop {
            if !self.has(pos, 6) {
                break; // tolerate short (truncated) fields
            }
            let index = self.uint(pos, 6) as usize; // 0..=63, always in range
            s.push(SIXBIT_ASCII[index] as char);
            pos += 6;
        }
        if let Some(at) = s.find('@') {
            s.truncate(at);
        }
        s.trim_end_matches(' ').to_string()
    }

    /// The field as a '0'/'1' string — for binary payloads we cannot decode.
    pub fn raw(&self, start: usize, length: usize) -> String {
        // Clamp like a Python slice instead of panicking.
        let end = (start + length).min(self.bits.len());
        let start = start.min(end);
        self.bits[start..end]
            .iter()
            .map(|&b| if b == 1 { '1' } else { '0' })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armouring() {
        assert_eq!(armor_char_to_value('0'), Ok(0));
        assert_eq!(armor_char_to_value('W'), Ok(39));
        assert_eq!(armor_char_to_value('`'), Ok(40));
        assert_eq!(armor_char_to_value('w'), Ok(63));
        assert_eq!(armor_char_to_value('X'), Err(BitsError::IllegalChar('X')));
        assert_eq!(
            armor_char_to_value('x').unwrap_err().to_string(),
            "illegal armour character 'x'"
        );
        assert_eq!(
            payload_to_bits("1", 7).unwrap_err().to_string(),
            "fill bits must be 0..5, got 7"
        );
    }

    #[test]
    fn signed_and_unsigned() {
        // 1000 0000 = -128 signed, 128 unsigned; 0111 1111 = 127 both.
        let b = Bits::new(vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(b.uint(0, 8), 128);
        assert_eq!(b.sint(0, 8), -128);
        assert_eq!(b.sint(8, 8), 127);
        assert_eq!(b.sint(7, 2), 0);
        assert_eq!(b.sint(8, 2), 1);
        assert_eq!(b.sint(9, 2), -1);
        assert!(b.flag(0));
        assert!(!b.flag(1));
        assert_eq!(b.raw(6, 4), "0001");
        assert_eq!(b.raw(14, 10), "11"); // clamps like a Python slice
        assert_eq!(b.len(), 16);
        assert!(b.has(0, 16));
        assert!(!b.has(1, 16));
    }

    #[test]
    fn six_bit_text() {
        // "AB@ " -> 000001 000010 000000 100000 -> "AB"
        let b = Bits::new(vec![
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
        ]);
        assert_eq!(b.text(0, 24), "AB");
        assert_eq!(b.text(0, 12), "AB");
        assert_eq!(b.text(0, 13), "AB"); // partial last character ignored
        assert_eq!(b.text(0, 3), "");
    }
}
