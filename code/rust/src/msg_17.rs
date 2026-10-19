//! `msg_17.rs` — message 17: GNSS broadcast binary message (chapter 13).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.15, Table 68 (80-bit header plus 0–736
//! bits of differential-correction data) and Table 69 (layout of the data:
//! a 40-bit RTCM SC-104 style header followed by N 24-bit data words, parity
//! removed). The content follows Recommendation ITU-R M.823.
//!
//! Bit layout (zero-based offsets; lengths from Table 68):
//!
//! ```text
//!      0- 5   6  message id (17)
//!      6- 7   2  repeat indicator
//!      8-37  30  source id (MMSI of the base station)
//!     38-39   2  spare
//!     40-57  18  longitude of the DGNSS reference station, signed, 1/10 min; 181° = n/a
//!     58-74  17  latitude of the DGNSS reference station, signed, 1/10 min; 91° = n/a
//!     75-79   5  spare
//!     80-     0–736 differential correction data (Table 69):
//!       80- 85   6  RTCM message type
//!       86- 95  10  RTCM station id
//!       96-108  13  Z count, units of 0.6 s (0–3599.4 s)
//!      109-111   3  sequence number (cyclic 0–7)
//!      112-116   5  N, number of 24-bit data words following this header (≤ 29)
//!      117-119   3  reference station health
//!      120-     N × 24 bits of DGNSS data words
//! ```
//!
//! Length rule: at least 80 bits (else `DecodeError::TooShort`). With 40 or
//! more data bits we decode the RTCM header and hand back the remaining bits
//! as a string; `words_complete` says whether exactly N × 24 bits were
//! present. Fewer than 40 data bits gives `rtcm: null`.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `Option<Obj>` is converted to JSON by the `From<Option<T>>` impl in
//!   jsonout.rs: `Some(obj)` becomes the object, `None` becomes `null`. So
//!   `put(&mut out, "rtcm", if … { Some(…) } else { None })` reads like the
//!   Python conditional expression.
//! * A `match` on a 3-bit field needs no `_` arm when every value 0–7 is
//!   listed *and* the compiler can see the range — it cannot for a `u64`,
//!   so `health_text` keeps a final `_` arm for 7.

use crate::bits::Bits;
use crate::common::{require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};

pub const LON_NOT_AVAILABLE: i64 = 181 * 600; // 0x1A838, 1/10-minute units
pub const LAT_NOT_AVAILABLE: i64 = 91 * 600; // 0x0D548

/// Reference-station health, Table 69 → ITU-R M.823 (the RTCM SC-104 v2 code).
pub fn health_text(code: u64) -> &'static str {
    match code {
        0 => "UDRE scale factor 1.0",
        1 => "UDRE scale factor 0.75",
        2 => "UDRE scale factor 0.5",
        3 => "UDRE scale factor 0.3",
        4 => "UDRE scale factor 0.2",
        5 => "UDRE scale factor 0.1",
        6 => "reference station transmission not monitored",
        _ => "reference station not working", // 7
    }
}

/// The 40-bit RTCM header of Table 69 plus whatever data words follow.
pub fn decode_rtcm_header(bits: &Bits, start: usize) -> Obj {
    let n_words = bits.uint(start + 32, 5);
    let health = bits.uint(start + 37, 3);
    let z_count = bits.uint(start + 16, 13);
    let words_start = start + 40;
    let words_bits = bits.len() - words_start;
    let mut rtcm = Obj::new();
    put(&mut rtcm, "message_type", bits.uint(start, 6));
    put(&mut rtcm, "station_id", bits.uint(start + 6, 10));
    put(&mut rtcm, "z_count", z_count);
    put(&mut rtcm, "z_count_s", Json::fixed(z_count as f64 * 0.6, 1));
    put(&mut rtcm, "sequence", bits.uint(start + 22, 3));
    put(&mut rtcm, "n_words", n_words);
    put(&mut rtcm, "health", health);
    put(&mut rtcm, "health_text", health_text(health));
    put(&mut rtcm, "words", bits.raw(words_start, words_bits));
    put(&mut rtcm, "words_bits", words_bits);
    put(&mut rtcm, "words_complete", words_bits as u64 == n_words * 24);
    rtcm
}

/// Decode a message 17. Fails with `DecodeError::TooShort` below 80 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    require_bits(17, bits.len(), 80)?;
    let lon_raw = bits.sint(40, 18);
    let lat_raw = bits.sint(58, 17);
    let data_bits = bits.len() - 80;
    let mut out = Obj::new();
    put(&mut out, "msg_type", bits.uint(0, 6));
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "spare", bits.uint(38, 2));
    put(&mut out, "lon", Json::fixed(lon_raw as f64 / 600.0, 6));
    put(&mut out, "lat", Json::fixed(lat_raw as f64 / 600.0, 6));
    put(
        &mut out,
        "position_available",
        lon_raw != LON_NOT_AVAILABLE && lat_raw != LAT_NOT_AVAILABLE,
    );
    put(&mut out, "spare2", bits.uint(75, 5));
    put(&mut out, "data", bits.raw(80, data_bits));
    put(&mut out, "data_bits", data_bits);
    // Table 69: the two-word (40-bit) header is only meaningful when complete.
    let rtcm = if data_bits >= 40 { Some(decode_rtcm_header(bits, 80)) } else { None };
    put(&mut out, "rtcm", rtcm);
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_two_words() {
        // synthetic sentence from chapter 13: reference station at 36 58.5 N 076 20.0 W
        let bits = Bits::from_payload("A03OviSCAPcE80AT9aD@bbbbEEEE", 0).unwrap();
        let Json::Obj(out) = decode(&bits).unwrap() else { panic!("not an object") };
        assert_eq!(out["lon"].to_string(), "-76.333333");
        assert_eq!(out["lat"].to_string(), "36.975000");
        assert_eq!(out["data_bits"], Json::Int(88));
        let Json::Obj(rtcm) = &out["rtcm"] else { panic!("rtcm not an object") };
        assert_eq!(rtcm["message_type"], Json::Int(1));
        assert_eq!(rtcm["station_id"], Json::Int(100));
        assert_eq!(rtcm["z_count"], Json::Int(1234));
        assert_eq!(rtcm["n_words"], Json::Int(2));
        assert_eq!(rtcm["words_complete"], Json::Bool(true));
    }

    #[test]
    fn too_short() {
        let bits = Bits::from_payload("A34PBT6JT4i8", 0).unwrap();
        assert_eq!(
            decode(&bits).unwrap_err().to_string(),
            "message 17 too short: 72 bits < 80"
        );
    }
}
