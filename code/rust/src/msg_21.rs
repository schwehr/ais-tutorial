//! `msg_21.rs` — message 21: aids-to-navigation (AtoN) report (chapter 15).
//!
//! Spec: ITU-R M.1371-5 Annex 8 §3.19, Table 73 (field layout, 272–360 bits);
//! Table 74 (the 32 AtoN types); Figure 41/41bis and §3.3.3 (dimension /
//! reference-point encoding, shared with message 5); Table 51 (EPFD types,
//! shared with message 4). The eight "AtoN status" bits are only *reserved*
//! by ITU; their page-7 layout (RACON / light / health) comes from IALA
//! Recommendation R0126 (A-126) ed. 2 §4.8.4 Figure 3.
//!
//! Bit layout (zero-based offsets; lengths from Table 73):
//!
//! ```text
//!      0- 5    6  message id (21)
//!      6- 7    2  repeat indicator
//!      8- 37  30  user id (MMSI; AtoN format 99MIDxxxx per ITU-R M.585)
//!     38- 42   5  type of aid to navigation (Table 74); 0 = not available
//!     43-162 120  name, 20 six-bit characters; all '@' = not available
//!    163       1  position accuracy (1 = high, ≤10 m)
//!    164-191  28  longitude, signed, 1/10 000 minute; 181° = n/a
//!    192-218  27  latitude,  signed, 1/10 000 minute;  91° = n/a
//!    219-227   9  dimension A (reference point to "bow" / true north), m
//!    228-236   9  dimension B (to "stern"), m
//!    237-242   6  dimension C (to port), m
//!    243-248   6  dimension D (to starboard), m
//!                 (decoded by msg_19::decode_dimensions, same keys as message 5)
//!    249-252   4  type of EPFD (Table 51); 7 = surveyed is normal for fixed/virtual aids
//!    253-258   6  UTC second; 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
//!    259       1  off-position indicator (floating AtoN only; valid only if second <= 59)
//!    260-267   8  AtoN status — "reserved for the indication of the AtoN status"
//!    268       1  RAIM flag
//!    269       1  virtual AtoN flag (1 = does not physically exist)
//!    270       1  assigned mode flag
//!    271       1  spare
//!    272-      0..84  name extension: 0–14 more six-bit characters, then
//!                 0, 2, 4 or 6 spare bits to reach a byte boundary
//! ```
//!
//! Variable length rule: everything after bit 272 is the name extension. We
//! decode floor((len - 272) / 6) whole characters (at most 14) and report the
//! raw number of extension bits as `name_extension_bits`.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `matches!(aton_type, 4 | 20..=31)` is a macro that expands to a `match`
//!   returning `bool`; `20..=31` is an *inclusive* range pattern.
//! * `usize::min(a, b)` (or `a.min(b)`) is the counterpart of Python's
//!   `min(a, b)`; both operands must be `usize` here because `bits.len()` is.
//! * `format!("{}{}", name, name_extension)` builds the full name without
//!   moving `name` out, so `name` can still be stored in the object
//!   afterwards. Writing `name + &name_extension` would *consume* `name`
//!   (ownership moves into the `+`), and a later use would not compile.
//! * `out.extend(decode_status(status))` merges a second `BTreeMap` into
//!   the first — Python's `dict.update`.

use crate::bits::Bits;
use crate::common::{epfd_text, require_bits, DecodeError};
use crate::jsonout::{put, Json, Obj};
use crate::msg_19::decode_dimensions;

/// Table 73: the fixed part (two slots).
pub const MIN_BITS: usize = 272;
/// Table 73: "up to 14 additional 6-bit-ASCII characters".
pub const MAX_EXTENSION_CHARS: usize = 14;

/// Type of aid to navigation, Table 74. Codes 5–19 are listed under
/// "Fixed AtoN", 20–31 under "Floating AtoN"; 1–4 are general. Code 4 was
/// "spare" in M.1371-4 and became "Emergency Wreck Marking Buoy" in M.1371-5.
pub fn aton_type_text(code: u64) -> &'static str {
    match code {
        0 => "not specified (default)",
        1 => "reference point",
        2 => "RACON",
        3 => "fixed structure off-shore",
        4 => "emergency wreck marking buoy",
        5 => "light, without sectors",
        6 => "light, with sectors",
        7 => "leading light front",
        8 => "leading light rear",
        9 => "beacon, cardinal N",
        10 => "beacon, cardinal E",
        11 => "beacon, cardinal S",
        12 => "beacon, cardinal W",
        13 => "beacon, port hand",
        14 => "beacon, starboard hand",
        15 => "beacon, preferred channel port hand",
        16 => "beacon, preferred channel starboard hand",
        17 => "beacon, isolated danger",
        18 => "beacon, safe water",
        19 => "beacon, special mark",
        20 => "cardinal mark N",
        21 => "cardinal mark E",
        22 => "cardinal mark S",
        23 => "cardinal mark W",
        24 => "port hand mark",
        25 => "starboard hand mark",
        26 => "preferred channel port hand",
        27 => "preferred channel starboard hand",
        28 => "isolated danger",
        29 => "safe water",
        30 => "special mark",
        _ => "light vessel / LANBY / rig", // 31
    }
}

/// IALA R0126 Figure 3, status page 7: RACON bits.
pub fn racon_status_text(code: u64) -> &'static str {
    match code {
        0 => "no RACON installed",
        1 => "RACON installed but not monitored",
        2 => "RACON operational",
        _ => "RACON error", // 3
    }
}

/// IALA R0126 Figure 3, status page 7: light bits.
pub fn light_status_text(code: u64) -> &'static str {
    match code {
        0 => "no light or no monitoring",
        1 => "light on",
        2 => "light off",
        _ => "light fail or at reduced range", // 3
    }
}

/// True for the Table 74 codes that describe a floating aid (a buoy).
/// Table 74 heads codes 20–31 "Floating AtoN"; code 4, the emergency wreck
/// marking buoy, is listed above that heading but is a buoy as well.
pub fn is_floating(aton_type: u64) -> bool {
    matches!(aton_type, 4 | 20..=31)
}

/// Interpret the 8 AtoN status bits.
///
/// ITU only reserves them (Table 73). IALA R0126 §4.8.4 splits them into a
/// 3-bit page id (the most significant bits) and 5 page-dependent bits;
/// page 0 is "no data" (the all-zero default) and page 7 carries the
/// RACON / light / health summary that monitoring systems key on.
pub fn decode_status(status: u64) -> Obj {
    let page = status >> 5;
    let mut out = Obj::new();
    put(&mut out, "aton_status_page", page);
    if page == 7 {
        let racon = (status >> 3) & 0x3;
        let light = (status >> 1) & 0x3;
        put(&mut out, "racon_status", racon);
        put(&mut out, "racon_status_text", racon_status_text(racon));
        put(&mut out, "light_status", light);
        put(&mut out, "light_status_text", light_status_text(light));
        put(&mut out, "health_alarm", (status & 0x1) == 1);
    }
    out
}

/// Decode a message 21. Fails with `DecodeError::TooShort` if the payload
/// has fewer than 272 bits.
pub fn decode(bits: &Bits) -> Result<Json, DecodeError> {
    let msg_type = bits.uint(0, 6);
    require_bits(msg_type as u32, bits.len(), MIN_BITS)?;
    let aton_type = bits.uint(38, 5);
    let lon_raw = bits.sint(164, 28);
    let lat_raw = bits.sint(192, 27);
    let epfd = bits.uint(249, 4);
    let timestamp = bits.uint(253, 6);
    let status = bits.uint(260, 8);

    // --- the variable-length name extension -------------------------------
    // Table 73: "0, 6, 12, … 84" bits of six-bit characters followed by 0, 2,
    // 4 or 6 spare bits so that the message ends on a byte boundary. Only
    // whole characters are decoded; a partial character is ignored.
    let extension_bits = bits.len() - MIN_BITS;
    let extension_chars = usize::min(extension_bits / 6, MAX_EXTENSION_CHARS);
    let name = bits.text(43, 120);
    let name_extension = bits.text(MIN_BITS, extension_chars * 6);

    let mut out = Obj::new();
    put(&mut out, "msg_type", msg_type);
    put(&mut out, "repeat_indicator", bits.uint(6, 2));
    put(&mut out, "mmsi", bits.uint(8, 30));
    put(&mut out, "aton_type", aton_type);
    put(&mut out, "aton_type_text", aton_type_text(aton_type));
    put(&mut out, "full_name", format!("{}{}", name, name_extension));
    put(&mut out, "name", name);
    put(&mut out, "name_extension", name_extension);
    put(&mut out, "name_extension_bits", extension_bits);
    put(&mut out, "position_accuracy", bits.uint(163, 1));
    put(&mut out, "lon", Json::fixed(lon_raw as f64 / 600000.0, 6));
    put(&mut out, "lat", Json::fixed(lat_raw as f64 / 600000.0, 6));
    put(
        &mut out,
        "position_available",
        lon_raw != 0x6791AC0 && lat_raw != 0x3412140,
    );
    // Bits 219-248: the same 30-bit A/B/C/D block as messages 5, 19 and 24B,
    // with the same keys (dim_bow_m … beam_m) — one definition in msg_19.rs.
    out.extend(decode_dimensions(bits, 219));
    put(&mut out, "epfd", epfd);
    put(&mut out, "epfd_text", epfd_text(epfd));
    put(&mut out, "timestamp", timestamp);
    put(&mut out, "off_position", bits.flag(259));
    // Table 73 note 1: the flag means something only for a floating aid
    // whose time stamp is a real second (0–59).
    put(
        &mut out,
        "off_position_valid",
        is_floating(aton_type) && timestamp <= 59,
    );
    put(&mut out, "aton_status", status);
    put(&mut out, "aton_status_bits", bits.raw(260, 8));
    put(&mut out, "raim", bits.flag(268));
    put(&mut out, "virtual_aton", bits.flag(269));
    put(&mut out, "assigned_mode", bits.flag(270));
    put(&mut out, "spare", bits.uint(271, 1));
    out.extend(decode_status(status));
    Ok(Json::Obj(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whale_notices_station() {
        // sample-data/2010-11-l3-aton/ccom-sr162g-2010-09.aivdm line 1
        let bits = Bits::from_payload("E03Ovrh77PPh;T0V2h77b4QRaP0MM`Lp<Em<000000V@20", 4).unwrap();
        assert_eq!(bits.len(), 272);
        let json = decode(&bits).unwrap();
        let obj = match json {
            Json::Obj(o) => o,
            _ => panic!("expected object"),
        };
        assert_eq!(obj["mmsi"], Json::from(3669739u64));
        assert_eq!(obj["name"], Json::from("NOAA WHALE NOTICES"));
        assert_eq!(obj["off_position"], Json::from(true));
        assert_eq!(obj["name_extension_bits"], Json::from(0usize));
    }

    #[test]
    fn status_page_7() {
        let obj = decode_status(0b1111_0111);
        assert_eq!(obj["aton_status_page"], Json::from(7u64));
        assert_eq!(obj["racon_status_text"], Json::from("RACON operational"));
        assert_eq!(obj["light_status"], Json::from(3u64));
        assert_eq!(obj["health_alarm"], Json::from(true));
        assert!(decode_status(1).get("racon_status").is_none());
    }

    #[test]
    fn too_short() {
        let short = Bits::from_payload("E03Ovrh77PPh;T0V2h77b4QRaP0MM`Lp<Em<000000", 0).unwrap();
        assert_eq!(
            decode(&short).unwrap_err().to_string(),
            "message 21 too short: 252 bits < 272"
        );
    }
}
