//! `msg_5.rs` — message 5: ship static and voyage related data (chapter 7).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.3, Table 52 (field layout, 424 bits, two
//! slots); §3.3.1 (DTE); §3.3.2 Table 53 (type of ship and cargo type);
//! §3.3.3 Figure 41 (reference point and dimensions A/B/C/D); Table 47
//! (six-bit ASCII). The EPFD code list is Table 51 (shared via common.rs).
//!
//! Bit layout (zero-based offsets; lengths from Table 52):
//!
//! ```text
//!       0-  5   6  message id (5)
//!       6-  7   2  repeat indicator
//!       8- 37  30  user id (MMSI)
//!      38- 39   2  AIS version indicator (0 = M.1371-1, 1 = -3, 2 = -5, 3 = future)
//!      40- 69  30  IMO number; 0 = n/a; 1..999999 not used;
//!                  1000000..9999999 IMO number; 10000000.. flag-state number
//!      70-111  42  call sign, 7 six-bit characters, "@@@@@@@" = n/a
//!     112-231 120  name, 20 six-bit characters, all '@' = n/a
//!     232-239   8  type of ship and cargo type (Table 53); 0 = n/a
//!     240-248   9  dimension A, metres from reference point to bow;   511 = ≥511 m
//!     249-257   9  dimension B, metres to stern;                      511 = ≥511 m
//!     258-263   6  dimension C, metres to port side;                   63 = ≥63 m
//!     264-269   6  dimension D, metres to starboard side;              63 = ≥63 m
//!     270-273   4  type of electronic position fixing device (EPFD)
//!     274-277   4  ETA month  1..12;  0 = n/a
//!     278-282   5  ETA day    1..31;  0 = n/a
//!     283-287   5  ETA hour   0..23; 24 = n/a
//!     288-293   6  ETA minute 0..59; 60 = n/a
//!     294-301   8  maximum present static draught, 1/10 m; 0 = n/a; 255 = ≥25.5 m
//!     302-421 120  destination, 20 six-bit characters, all '@' = n/a
//!     422       1  DTE: 0 = data terminal available, 1 = not available (default)
//!     423       1  spare
//! ```
//!
//! Length rule: we require the full 424 bits and ignore anything after bit
//! 423 (426-bit messages from senders that under-count the fill bits decode
//! normally). Shorter messages are reported as `message 5 too short`.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `matches!(month, 1..=12)` is a macro that returns `true` if the value
//!   fits the pattern — a compact way to write a range test.
//! * `imo.to_string().bytes()` gives the decimal digits as ASCII bytes;
//!   `b - b'0'` turns a digit byte into its value. `zip` pairs each digit
//!   with its weight, exactly like Python's `zip(digits, weights)`.
//! * `format!("{month:02}")` is the zero-padded formatting that Python
//!   writes as `f"{month:02d}"`.
//! * Closures: `let two = |v: u64| …` defines a small local function; the
//!   C++ version uses a lambda for the same job.

use crate::bits::Bits;
use crate::common::{epfd_text, require_bits, ship_type_text, DecodeError};
use crate::jsonout::{put, Json, Obj};

/// Table 52, "AIS version indicator".
pub fn ais_version_text(code: u64) -> &'static str {
    match code {
        0 => "ITU-R M.1371-1",
        1 => "ITU-R M.1371-3 or later",
        2 => "ITU-R M.1371-5 or later",
        _ => "future edition", // 3
    }
}

/// Table 52, "DTE" (data terminal equipment ready), §3.3.1.
pub fn dte_text(code: u64) -> &'static str {
    if code == 0 {
        "available"
    } else {
        "not available (default)"
    }
}

/// Which of the four ranges of Table 52 the 30-bit IMO field falls in.
pub fn imo_range_text(imo: u64) -> &'static str {
    match imo {
        0 => "not available (default)",
        1..=999_999 => "not used",
        1_000_000..=9_999_999 => "IMO number",
        _ => "official flag state number",
    }
}

/// True if `imo` is a seven-digit IMO number whose check digit is right.
///
/// Multiply the first six digits by 7, 6, 5, 4, 3, 2, add the products; the
/// last digit of the sum is the check digit:
/// `9074729:  9*7 + 0*6 + 7*5 + 4*4 + 7*3 + 2*2 = 139  ->  9  ok`
pub fn imo_check_digit_ok(imo: u64) -> bool {
    if !(1_000_000..=9_999_999).contains(&imo) {
        return false;
    }
    let digits: Vec<u64> = imo.to_string().bytes().map(|b| (b - b'0') as u64).collect();
    let total: u64 = digits[..6].iter().zip([7, 6, 5, 4, 3, 2]).map(|(d, w)| d * w).sum();
    total % 10 == digits[6]
}

/// 'MM-DD HH:MM' in UTC with '??' for each part that is not available.
pub fn eta_text(month: u64, day: u64, hour: u64, minute: u64) -> String {
    let two = |v: u64| format!("{v:02}");
    let mm = if matches!(month, 1..=12) { two(month) } else { "??".to_string() };
    let dd = if matches!(day, 1..=31) { two(day) } else { "??".to_string() };
    let hh = if hour <= 23 { two(hour) } else { "??".to_string() };
    let mi = if minute <= 59 { two(minute) } else { "??".to_string() };
    format!("{mm}-{dd} {hh}:{mi}")
}

/// Decode a message 5. Fails with `DecodeError::TooShort` if the payload
/// has fewer than 424 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), 424)?;
    let ais_version = bits.uint(38, 2);
    let imo = bits.uint(40, 30);
    let ship_type = bits.uint(232, 8);
    let dim_a = bits.uint(240, 9); // to bow
    let dim_b = bits.uint(249, 9); // to stern
    let dim_c = bits.uint(258, 6); // to port
    let dim_d = bits.uint(264, 6); // to starboard
    let epfd = bits.uint(270, 4);
    let month = bits.uint(274, 4);
    let day = bits.uint(278, 5);
    let hour = bits.uint(283, 5);
    let minute = bits.uint(288, 6);
    let draught_raw = bits.uint(294, 8);
    let dte = bits.uint(422, 1);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "ais_version", ais_version);
    put(&mut out, "ais_version_text", ais_version_text(ais_version));
    put(&mut out, "imo", imo);
    put(&mut out, "imo_available", imo != 0);
    put(&mut out, "imo_range_text", imo_range_text(imo));
    put(&mut out, "imo_valid", imo_check_digit_ok(imo));
    put(&mut out, "call_sign", bits.text(70, 42));
    put(&mut out, "name", bits.text(112, 120));
    put(&mut out, "ship_type", ship_type);
    put(&mut out, "ship_type_text", ship_type_text(ship_type));
    put(&mut out, "dim_bow_m", dim_a);
    put(&mut out, "dim_stern_m", dim_b);
    put(&mut out, "dim_port_m", dim_c);
    put(&mut out, "dim_starboard_m", dim_d);
    // Figure 41: all four zero = neither reference point nor dimensions;
    // A = C = 0 with B, D non-zero = dimensions but no reference point.
    put(
        &mut out,
        "dimensions_available",
        !(dim_a == 0 && dim_b == 0 && dim_c == 0 && dim_d == 0),
    );
    put(&mut out, "reference_point_available", !(dim_a == 0 && dim_c == 0));
    put(&mut out, "length_m", dim_a + dim_b);
    put(&mut out, "beam_m", dim_c + dim_d);
    put(&mut out, "epfd", epfd);
    put(&mut out, "epfd_text", epfd_text(epfd));
    put(&mut out, "eta_month", month);
    put(&mut out, "eta_day", day);
    put(&mut out, "eta_hour", hour);
    put(&mut out, "eta_minute", minute);
    put(&mut out, "eta_text", eta_text(month, day, hour, minute));
    put(
        &mut out,
        "eta_available",
        matches!(month, 1..=12) && matches!(day, 1..=31) && hour <= 23 && minute <= 59,
    );
    put(&mut out, "draught_m", Json::fixed(draught_raw as f64 / 10.0, 1));
    put(&mut out, "draught_available", draught_raw != 0);
    put(&mut out, "destination", bits.text(302, 120));
    put(&mut out, "dte", dte);
    put(&mut out, "dte_text", dte_text(dte));
    put(&mut out, "spare", bits.uint(423, 1));
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imo_check_digit() {
        assert!(imo_check_digit_ok(9074729));
        assert!(imo_check_digit_ok(9271420)); // USNS SACAGAWEA
        assert!(!imo_check_digit_ok(9271421));
        assert!(!imo_check_digit_ok(0));
        assert!(!imo_check_digit_ok(926974000)); // IMO 9269740 typed with two extra zeros
        assert_eq!(imo_range_text(926974000), "official flag state number");
    }

    #[test]
    fn eta_defaults() {
        assert_eq!(eta_text(0, 0, 24, 60), "??-?? ??:??");
        assert_eq!(eta_text(1, 1, 0, 26), "01-01 00:26");
    }

    #[test]
    fn capecod_test_station() {
        let bits = Bits::from_payload(
            "503OvqP00000ptpD000pt461@E=@00000000000W000004@PJ03Sm80H5QDliC000000000",
            2,
        )
        .unwrap();
        assert_eq!(bits.len(), 424);
        let out = match decode(&bits).unwrap() {
            Json::Obj(obj) => obj,
            other => panic!("expected an object, got {other:?}"),
        };
        assert_eq!(out["name"], Json::from("NOAA TEST"));
        assert_eq!(out["destination"], Json::from("NOT A VESSEL"));
        assert_eq!(out["mmsi"], Json::from(3669734u64));
    }
}
