//! `commstate.rs` — TDMA communication state schedule and timing helpers (chapter 17).
//!
//! Spec: ITU-R M.1371-5 Annex 2 §3.1 (synchronisation and frame structure),
//! §3.3.4 (SOTDMA and ITDMA slot allocation), §3.3.7.2.2 Tables 18/19 (SOTDMA
//! communication state), and §3.3.7.3.2 Table 20 (ITDMA communication state).
//!
//! Eight AIS message types end with a communication state:
//!
//! ```text
//!     1, 2      168 bits   bits 149-167 (19)   always SOTDMA
//!     3         168 bits   bits 149-167 (19)   always ITDMA
//!     4, 11     168 bits   bits 149-167 (19)   always SOTDMA
//!     9         168 bits   bits 148-167 (20)   selector at 148 + 19-bit state
//!     18        168 bits   bits 148-167 (20)   selector at 148 + 19-bit state
//!     26     <=1064 bits   last 20 bits        selector + 19-bit state
//! ```
//!
//! `msg_1_2_3.rs` decodes the 19-bit SOTDMA and ITDMA bit fields into a `Json`
//! object; this module re-exports those decoders and adds the frame-arithmetic
//! and clock-recovery functions that turn raw slot numbers, offsets, and UTC
//! sub-messages into physical schedules and UTC arrival times.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `pub use crate::msg_1_2_3::{decode_itdma, decode_sotdma, sync_state_text};`
//!   re-exports items from another module so callers can import everything
//!   related to the communication state from `aistut::commstate`.
//! * `i64::rem_euclid` is Euclidean (non-negative) remainder — the exact
//!   counterpart of Python's `%` for positive divisors, whereas Rust's `%`
//!   keeps the sign of the dividend.

use crate::jsonout::Json;
pub use crate::msg_1_2_3::{decode_itdma, decode_sotdma, sync_state_text};

/// Annex 2 §3.2.1 Table 10: one 60 s frame has 2 250 slots (26.667 ms each).
pub const SLOTS_PER_FRAME: i64 = 2250;
pub const FRAME_DURATION_S: f64 = 60.0;
pub const SLOT_DURATION_S: f64 = FRAME_DURATION_S / (SLOTS_PER_FRAME as f64);

/// Annex 2 §3.3.4.4 Table 16: a newly allocated SOTDMA slot is assigned a
/// random integer time-out between `TMO_MIN` (3) and `TMO_MAX` (7) frames.
pub const TMO_MIN: u64 = 3;
pub const TMO_MAX: u64 = 7;

/// Annex 2 §3.3.7.2.2 Table 19: in the `slot_timeout == 1` UTC hour/minute
/// sub-message, hour `0..=23` and minute `0..=59` are valid; 24 and 60 mean n/a.
pub const UTC_HOUR_NA: u64 = 24;
pub const UTC_MINUTE_NA: u64 = 60;

/// Convert a slot number (`0..2249`) to seconds into the UTC minute.
///
/// Each TDMA frame is 60 s divided into 2 250 slots, so slot `k` begins at
/// `k * 60 / 2250` seconds past the minute [Annex 2 §3.2.1]. Returns
/// `Json::Null` if `slot_number` is outside `0..2249`.
pub fn slot_to_seconds(slot_number: i64) -> Json {
    if !(0..SLOTS_PER_FRAME).contains(&slot_number) {
        return Json::Null;
    }
    Json::fixed((slot_number as f64) * FRAME_DURATION_S / (SLOTS_PER_FRAME as f64), 3)
}

/// Return the slot in the *next* frame announced by a SOTDMA `slot_offset`.
///
/// When a SOTDMA stream reaches `slot_timeout == 0`, its 14-bit sub-message
/// is a `slot_offset` relative to the current slot on the same channel
/// [Annex 2 §3.3.7.2.2 Table 19]:
///   * `slot_offset == 0` means the slot is de-allocated after this
///     transmission (`None`);
///   * any non-zero offset selects `(slot_number + slot_offset) % 2250` in
///     the next frame.
pub fn next_sotdma_slot(slot_number: i64, slot_offset: i64) -> Option<i64> {
    if !(0..SLOTS_PER_FRAME).contains(&slot_number) || slot_offset <= 0 {
        return None;
    }
    Some((slot_number + slot_offset).rem_euclid(SLOTS_PER_FRAME))
}

/// Return the slot of the next transmission announced by an ITDMA `slot_offset`.
///
/// In ITDMA (`decode_itdma`), `slot_offset` is already expanded to
/// `slot_increment` (`0..8191`) or `slot_increment + 8192` when `slots_raw`
/// is 5, 6, or 7 [Annex 2 §3.3.7.3.2 Table 20]. An offset of `0` means this
/// is the last transmission for the slot (`None`); otherwise the next
/// transmission on the same channel lands in `(slot_number + slot_offset) % 2250`.
pub fn next_itdma_slot(slot_number: i64, slot_offset: i64) -> Option<i64> {
    if !(0..SLOTS_PER_FRAME).contains(&slot_number) || slot_offset <= 0 {
        return None;
    }
    Some((slot_number + slot_offset).rem_euclid(SLOTS_PER_FRAME))
}

/// Recover the UTC transmission timestamp from a logger clock and SOTDMA state.
///
/// When `utc_hour` (`0..=23`) and `utc_minute` (`0..=59`) are known — from a
/// nearby `slot_timeout == 1` sub-message — the full time-of-day within the
/// 86 400 s UTC day is fixed, resolving logger clock offsets of many minutes
/// or hours. Otherwise we fall back to the nearest minute within 30 s of
/// `unix_time` (the same rule as `uscg::time_of_arrival_to_utc`).
pub fn slot_to_utc(
    unix_time: f64,
    slot_number: i64,
    utc_hour: Option<u64>,
    utc_minute: Option<u64>,
) -> Json {
    if !(0..SLOTS_PER_FRAME).contains(&slot_number) {
        return Json::Null;
    }
    let slot_sec = (slot_number as f64) * FRAME_DURATION_S / (SLOTS_PER_FRAME as f64);
    let whole = unix_time as i64;

    if let (Some(h), Some(m)) = (utc_hour, utc_minute)
        && h <= 23
        && m <= 59
    {
        let day_start = whole - whole.rem_euclid(86400);
        let tod = (h as f64) * 3600.0 + (m as f64) * 60.0 + slot_sec;
        let candidates = [-1i64, 0, 1].map(|k| ((day_start + k * 86400) as f64) + tod);
        let mut best = candidates[0];
        for t in candidates {
            if (t - unix_time).abs() < (best - unix_time).abs() {
                best = t;
            }
        }
        if (best - unix_time).abs() > 43200.0 {
            return Json::Null;
        }
        return Json::fixed(best, 3);
    }

    let minute_start = whole - whole.rem_euclid(60);
    let candidates = [-60i64, 0, 60].map(|k| ((minute_start + k) as f64) + slot_sec);
    let mut best = candidates[0];
    for t in candidates {
        if (t - unix_time).abs() < (best - unix_time).abs() {
            best = t;
        }
    }
    if (best - unix_time).abs() > 30.0 {
        return Json::Null;
    }
    Json::fixed(best, 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bits::Bits;

    #[test]
    fn slot_seconds_and_transitions() {
        assert_eq!(slot_to_seconds(0).to_string(), "0.000");
        assert_eq!(slot_to_seconds(453).to_string(), "12.080");
        assert_eq!(slot_to_seconds(1177).to_string(), "31.387");
        assert_eq!(slot_to_seconds(1203).to_string(), "32.080");
        assert_eq!(slot_to_seconds(-1), Json::Null);
        assert_eq!(slot_to_seconds(2250), Json::Null);

        assert_eq!(next_sotdma_slot(1177, 2276), Some(1203));
        assert_eq!(next_sotdma_slot(453, 2231), Some(434));
        assert_eq!(next_sotdma_slot(782, 0), None);

        assert_eq!(next_itdma_slot(779, 754), Some(1533));
        assert_eq!(next_itdma_slot(1533, 738), Some(21));
        assert_eq!(next_itdma_slot(21, 0), None);
    }

    #[test]
    fn slot_to_utc_resolves_minute_ambiguity() {
        // ais-2008-06-12 line 12: logger is 52.58 s fast.
        assert_eq!(slot_to_utc(1213228834.26, 1563, None, None).to_string(), "1213228841.680");
        assert_eq!(
            slot_to_utc(1213228834.26, 1563, Some(23), Some(59)).to_string(),
            "1213228781.680"
        );
        assert_eq!(
            slot_to_utc(1213228924.66, 453, Some(0), Some(1)).to_string(),
            "1213228872.080"
        );
        assert_eq!(
            slot_to_utc(1213228924.66, 453, Some(24), Some(60)).to_string(),
            "1213228932.080"
        );
        assert_eq!(slot_to_utc(1213228924.66, 2250, Some(0), Some(1)), Json::Null);
    }

    #[test]
    fn reexported_decoders() {
        let sotdma_bits = Bits::from_payload("1000000P01Jt;qDHaP?o2?vt00ST", 0).unwrap();
        let cs1 = decode_sotdma(&sotdma_bits, 149);
        assert_eq!(cs1["scheme"], Json::from("SOTDMA"));
        assert_eq!(cs1["slot_timeout"], Json::from(0u64));
        assert_eq!(cs1["slot_offset"], Json::from(2276u64));

        let itdma_bits = Bits::from_payload("38153r0000rsw2dHb?ndK;:T82tQ", 0).unwrap();
        let cs3 = decode_itdma(&itdma_bits, 149);
        assert_eq!(cs3["scheme"], Json::from("ITDMA"));
        assert_eq!(cs3["slot_increment"], Json::from(754u64));
        assert_eq!(cs3["keep_flag"], Json::from(true));
    }
}
