//! `nmea.rs` — the NMEA 0183 sentence layer (chapter 1).
//!
//! An NMEA 0183 sentence (IEC 61162-1 §7) looks like
//!
//! ```text
//! $GPGGA,000429.000,3601.5522,N,10947.9359,W,1,7,4.21,12491.0,M,-20.5,M,,*6D
//! ^^     ^^^                                                              ^^
//! talker formatter                 comma-separated data fields        checksum
//! ```
//!
//! * `$` (or `!` for encapsulated data such as AIS) starts the sentence.
//! * The next two characters are the *talker* (GP = GPS, AI = AIS mobile
//!   station, BS = base station, AN = AtoN, …), then a 3-letter *formatter*
//!   (GGA, RMC, VDM, …). Proprietary sentences start with `P` + a 3-letter
//!   manufacturer code (`PMTK`, `PSRT`, `PFEC`) and have no fixed formatter.
//! * Fields are separated by commas; an empty field is allowed and means
//!   "no data".
//! * `*` followed by two hex digits is the checksum: the XOR of every byte
//!   between `$`/`!` and `*` (exclusive). IEC 61162-1 §7.2.3.
//! * The sentence ends with CR LF and may be at most 82 characters including
//!   `$` and CR LF (§7.1.1). Real logs often break this; we do not enforce it.
//!
//! The USCG / serial_logger convention of appending extra comma fields *after*
//! the checksum (`…*6D,rplane_to_sd,1197072048.14`) is also handled here:
//! those fields are returned separately as `after_checksum`.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `struct Sentence` with `#[derive(Debug, Clone)]` is the counterpart of
//!   Python's `@dataclass`: the compiler writes `Debug` (for `{:?}`
//!   printing) and `Clone` for us.
//! * `Option<i64>` means "an integer, or `None`"; `Result<T, E>` means
//!   "a `T`, or an error `E`". There is no `null` and no exception.
//! * `&str` is a borrowed view of string data; `String` is an owned,
//!   growable buffer. Parsers take `&str` (they only read) and store
//!   `String`s in the structs they return (the struct must own its data
//!   because the input line will not outlive it).
//! * Strings are UTF-8 and indexing them by byte offset is only allowed on
//!   character boundaries. We therefore slice with `split_once`, `find`
//!   and `chars()` rather than numeric indices.
//! * The "Python compatibility" helpers at the bottom exist because
//!   Python's `str.strip()`, `int()`, `float()` and `repr()` accept and
//!   print slightly different things from Rust's `trim()`, `parse()` and
//!   `{:?}`. Byte-identical output requires copying Python's behaviour.

use std::fmt;

use crate::jsonout::{put, Json, Obj};

/// XOR of all characters in `body` (the text between '$' and '*').
///
/// ```
/// use ais_tutorial::nmea::checksum;
/// assert_eq!(checksum("GPGSV,1,1,00"), 0x79);
/// ```
pub fn checksum(body: &str) -> u32 {
    let mut value = 0u32;
    for ch in body.chars() {
        value ^= ch as u32; // `ch as u32` is the code point, like ord()
    }
    value
}

/// One parsed NMEA sentence, before any formatter-specific decoding.
#[derive(Debug, Clone, PartialEq)]
pub struct Sentence {
    pub start: char,                 // '$' or '!'
    pub talker: String,              // "GP", "AI", "P" for proprietary ...
    pub formatter: String,           // "GGA", "VDM", "MTK010" ...
    pub fields: Vec<String>,         // data fields *after* the address field
    pub checksum_given: Option<i64>, // value written in the sentence, or None
    pub checksum_ok: bool,           // true only if present and matching
    pub after_checksum: Vec<String>, // USCG extras
    pub raw: String,
}

impl Sentence {
    /// The address field, e.g. "GPGGA" or "AIVDM".
    pub fn address(&self) -> String {
        format!("{}{}", self.talker, self.formatter)
    }
}

/// Errors for text that is not an NMEA sentence at all, plus number fields
/// that Python's `int()`/`float()` would have rejected with a `ValueError`
/// (the Python reference lets those propagate; we report them as a line
/// error with Python's own wording).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NmeaError {
    NoStart,
    AddressTooShort(String),
    BadNumber(String),
}

impl fmt::Display for NmeaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NmeaError::NoStart => write!(f, "no '$' or '!' in line"),
            NmeaError::AddressTooShort(a) => {
                write!(f, "address field too short: {}", py_repr(a))
            }
            NmeaError::BadNumber(text) => f.write_str(text),
        }
    }
}

impl std::error::Error for NmeaError {}

/// Split one line into its NMEA parts and verify the checksum.
///
/// Leading/trailing whitespace and CR/LF are ignored. Characters before the
/// `$`/`!` (such as a `file-name:` prefix from grep) are discarded; TAG
/// blocks (chapter 4) must be stripped by the caller first.
pub fn parse_sentence(line: &str) -> Result<Sentence, NmeaError> {
    let line = py_strip(line);
    // Python: min(i for i in (find('$'), find('!')) if i >= 0). Both finds
    // return byte offsets of ASCII characters, so slicing at them is safe.
    let start_at = match (line.find('$'), line.find('!')) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => return Err(NmeaError::NoStart),
    };
    let line = &line[start_at..];
    let start = if line.starts_with('$') { '$' } else { '!' };

    // Python: body, star, tail = line[1:].partition("*")
    let after_start = &line[1..];
    let mut given: Option<i64> = None;
    let mut ok = false;
    let mut extras: Vec<String> = Vec::new();
    let body = match after_start.split_once('*') {
        Some((body, tail)) => {
            // Python: hexpart, _, rest = tail.partition(","). `unwrap_or`
            // supplies the "no comma" answer instead of a two-arm `match`.
            let (hexpart, rest) = tail.split_once(',').unwrap_or((tail, ""));
            let first_two: String = hexpart.chars().take(2).collect();
            given = py_int_radix(&first_two, 16);
            ok = given == Some(i64::from(checksum(body)));
            if !rest.is_empty() {
                extras = rest.split(',').map(str::to_string).collect();
            }
            body
        }
        // If there was no '*' we still parse the fields but checksum_ok is false.
        None => after_start,
    };

    // Python: address, *fields = body.split(",")
    let mut parts = body.split(',');
    // `split` always yields at least one item, so this `unwrap_or` is only
    // defensive; it never triggers.
    let address = parts.next().unwrap_or("");
    let fields: Vec<String> = parts.map(str::to_string).collect();
    if address.chars().count() < 3 {
        return Err(NmeaError::AddressTooShort(address.to_string()));
    }
    let (talker, formatter) = match address.strip_prefix('P') {
        Some(rest) => ("P".to_string(), rest.to_string()), // proprietary
        None => {
            // The first two *characters* (not bytes) are the talker.
            let split_at = address
                .char_indices()
                .nth(2)
                .map(|(i, _)| i)
                .unwrap_or(address.len());
            (address[..split_at].to_string(), address[split_at..].to_string())
        }
    };
    Ok(Sentence {
        start,
        talker,
        formatter,
        fields,
        checksum_given: given,
        checksum_ok: ok,
        after_checksum: extras,
        raw: line.to_string(),
    })
}

// --- Helpers for the GPS formatters ----------------------------------------

/// `"4312.2805","N"` → `43.204675`; empty → `None`.
///
/// NMEA writes latitude as DDMM.MMMM: two digits of degrees followed by
/// minutes with a decimal fraction. Longitude uses three degree digits
/// (DDDMM.MMMM); see `parse_longitude`.
pub fn parse_latitude(ddmm: &str, hemisphere: &str) -> Result<Option<f64>, NmeaError> {
    parse_angle(ddmm, 2, hemisphere == "S")
}

/// `"07052.8474","W"` → `-70.88079`
pub fn parse_longitude(dddmm: &str, hemisphere: &str) -> Result<Option<f64>, NmeaError> {
    parse_angle(dddmm, 3, hemisphere == "W")
}

/// Shared body of the two functions above. `degree_digits` is 2 or 3.
fn parse_angle(text: &str, degree_digits: usize, negative: bool) -> Result<Option<f64>, NmeaError> {
    if text.is_empty() {
        return Ok(None);
    }
    let (deg_part, min_part) = split_chars(text, degree_digits);
    let degrees = py_int(deg_part)?;
    let minutes = py_float(min_part)?;
    let value = degrees as f64 + minutes / 60.0;
    Ok(Some(if negative { -value } else { value }))
}

/// Split a string after `n` characters (Python `s[:n], s[n:]`), clamping
/// like Python when the string is shorter.
fn split_chars(s: &str, n: usize) -> (&str, &str) {
    let at = s.char_indices().nth(n).map(|(i, _)| i).unwrap_or(s.len());
    (&s[..at], &s[at..])
}

/// `"000429.000"` → `"00:04:29.000"` (kept as text; no date is available).
pub fn parse_utc_time(hhmmss: &str) -> Option<String> {
    if hhmmss.chars().count() < 6 {
        return None;
    }
    let (hh, rest) = split_chars(hhmmss, 2);
    let (mm, ss) = split_chars(rest, 2);
    Some(format!("{hh}:{mm}:{ss}"))
}

/// `"081207"` → `"2007-12-08"`. Two-digit years: NMEA predates Y2K, we
/// assume 2000–2099 which is right for every file in this tutorial.
pub fn parse_date(ddmmyy: &str) -> Option<String> {
    if ddmmyy.chars().count() != 6 {
        return None;
    }
    let (dd, rest) = split_chars(ddmmyy, 2);
    let (mm, yy) = split_chars(rest, 2);
    Some(format!("20{yy}-{mm}-{dd}"))
}

/// Python's `float(s) if s else None`.
pub fn opt_float(s: &str) -> Result<Option<f64>, NmeaError> {
    if s.is_empty() {
        Ok(None)
    } else {
        py_float(s).map(Some)
    }
}

/// Python's `int(s) if s else None`.
pub fn opt_int(s: &str) -> Result<Option<i64>, NmeaError> {
    if s.is_empty() {
        Ok(None)
    } else {
        py_int(s).map(Some)
    }
}

/// Wrap an optional float for printing with `decimals` places (see jsonout.rs).
pub fn fixed(value: Option<f64>, decimals: usize) -> Json {
    match value {
        Some(v) => Json::fixed(v, decimals),
        None => Json::Null,
    }
}

/// Copy the fields and pad with empty strings up to `n` entries. Python:
/// `s.fields + [""] * (n - len(s.fields))`.
fn padded_fields(s: &Sentence, n: usize) -> Vec<String> {
    let mut f = s.fields.clone();
    if f.len() < n {
        f.resize(n, String::new());
    }
    f
}

// --- The four GPS sentences of chapter 1 -----------------------------------
//
// Field numbers below are 1-based positions *after* the address field, which
// is how the NMEA standard numbers them.

fn gga_fix_quality_text(quality: i64) -> &'static str {
    match quality {
        0 => "invalid",
        1 => "GPS fix (SPS)",
        2 => "DGPS fix",
        3 => "PPS fix",
        4 => "RTK fixed",
        5 => "RTK float",
        6 => "estimated (dead reckoning)",
        7 => "manual input",
        8 => "simulation",
        _ => "unknown",
    }
}

/// GGA — Global positioning system fix data (IEC 61162-1 §8.3.x / NMEA 0183 v3 §GGA).
///
/// ```text
/// 1 UTC time hhmmss.ss        6 fix quality (table above)
/// 2 latitude ddmm.mmmm        7 number of satellites in use
/// 3 N/S                        8 HDOP
/// 4 longitude dddmm.mmmm      9 antenna altitude, 10 'M'
/// 5 E/W                       11 geoidal separation, 12 'M'
///                             13 age of DGPS data (s), 14 DGPS station id
/// ```
pub fn parse_gga(s: &Sentence) -> Result<Obj, NmeaError> {
    let f = padded_fields(s, 14); // pad short sentences
    let quality = opt_int(&f[5])?;
    let mut out = Obj::new();
    put(&mut out, "sentence", "GGA");
    put(&mut out, "time_utc", parse_utc_time(&f[0]));
    put(&mut out, "lat", fixed(parse_latitude(&f[1], &f[2])?, 6));
    put(&mut out, "lon", fixed(parse_longitude(&f[3], &f[4])?, 6));
    put(&mut out, "fix_quality", quality);
    put(&mut out, "fix_quality_text", quality.map(gga_fix_quality_text));
    put(&mut out, "satellites_used", opt_int(&f[6])?);
    put(&mut out, "hdop", fixed(opt_float(&f[7])?, 2));
    put(&mut out, "altitude_m", fixed(opt_float(&f[8])?, 1));
    put(&mut out, "geoid_separation_m", fixed(opt_float(&f[10])?, 1));
    put(&mut out, "dgps_age_s", fixed(opt_float(&f[12])?, 1));
    put(&mut out, "dgps_station", non_empty(&f[13]));
    Ok(out)
}

/// Python's `s or None`: an empty string becomes null.
fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn gsa_mode_text(mode: i64) -> Option<&'static str> {
    match mode {
        1 => Some("no fix"),
        2 => Some("2D"),
        3 => Some("3D"),
        _ => None,
    }
}

/// GSA — GNSS DOP and active satellites.
///
/// ```text
/// 1 mode: M = manual, A = automatic 2D/3D switching
/// 2 mode: 1 = fix not available, 2 = 2D, 3 = 3D
/// 3–14 PRN numbers of the (up to 12) satellites used in the solution
/// 15 PDOP  16 HDOP  17 VDOP
/// ```
pub fn parse_gsa(s: &Sentence) -> Result<Obj, NmeaError> {
    let f = padded_fields(s, 17);
    let fix_mode = opt_int(&f[1])?;
    let mut sats: Vec<Json> = Vec::new();
    for p in &f[2..14] {
        if !p.is_empty() {
            sats.push(Json::from(py_int(p)?));
        }
    }
    let mut out = Obj::new();
    put(&mut out, "sentence", "GSA");
    put(&mut out, "selection_mode", non_empty(&f[0]));
    put(&mut out, "fix_mode", fix_mode);
    // `and_then` chains two Options: None stays None, Some(m) is looked up.
    put(&mut out, "fix_mode_text", fix_mode.and_then(gsa_mode_text));
    put(&mut out, "satellites_used", sats);
    put(&mut out, "pdop", fixed(opt_float(&f[14])?, 2));
    put(&mut out, "hdop", fixed(opt_float(&f[15])?, 2));
    put(&mut out, "vdop", fixed(opt_float(&f[16])?, 2));
    Ok(out)
}

/// GSV — GNSS satellites in view. One sentence carries up to 4 satellites.
///
/// ```text
/// 1 total number of GSV sentences in this group
/// 2 this sentence's number (1-based)
/// 3 total satellites in view
/// then repeating groups of 4: PRN, elevation (deg), azimuth (deg), SNR (dB-Hz)
/// ```
/// Elevation/azimuth/SNR are empty when not tracked.
pub fn parse_gsv(s: &Sentence) -> Result<Obj, NmeaError> {
    let f = &s.fields;
    let mut sats: Vec<Json> = Vec::new();
    // Python: for i in range(3, len(f) - 3, 4). `chunks_exact(4)` over the
    // tail yields only complete groups of four, which is the same set.
    if f.len() > 3 {
        for group in f[3..].chunks_exact(4) {
            let (prn, elev, az, snr) = (&group[0], &group[1], &group[2], &group[3]);
            if !prn.is_empty() {
                let mut sat = Obj::new();
                put(&mut sat, "prn", py_int(prn)?);
                put(&mut sat, "elevation_deg", opt_int(elev)?);
                put(&mut sat, "azimuth_deg", opt_int(az)?);
                put(&mut sat, "snr_db", opt_int(snr)?);
                sats.push(Json::Obj(sat));
            }
        }
    }
    // `f.get(i)` returns None past the end instead of panicking; the
    // `map(String::as_str).unwrap_or("")` turns a missing field into "".
    let field = |i: usize| f.get(i).map(String::as_str).unwrap_or("");
    let mut out = Obj::new();
    put(&mut out, "sentence", "GSV");
    put(&mut out, "total_sentences", opt_int(field(0))?);
    put(&mut out, "sentence_number", opt_int(field(1))?);
    put(&mut out, "satellites_in_view", opt_int(field(2))?);
    put(&mut out, "satellites", sats);
    Ok(out)
}

fn rmc_mode_text(mode: &str) -> Option<&'static str> {
    match mode {
        "A" => Some("autonomous"),
        "D" => Some("differential"),
        "E" => Some("estimated"),
        "M" => Some("manual"),
        "S" => Some("simulator"),
        "N" => Some("not valid"),
        _ => None,
    }
}

/// RMC — Recommended minimum specific GNSS data.
///
/// ```text
/// 1 UTC time   2 status A = valid, V = warning (no fix)
/// 3,4 latitude   5,6 longitude   7 SOG knots   8 COG degrees true
/// 9 date ddmmyy   10,11 magnetic variation deg, E/W
/// 12 mode indicator (NMEA 2.3+)   13 navigational status (NMEA 4.1+)
/// ```
pub fn parse_rmc(s: &Sentence) -> Result<Obj, NmeaError> {
    let f = padded_fields(s, 13);
    let mut var = opt_float(&f[9])?;
    if f[10] == "W" {
        var = var.map(|v| -v);
    }
    let mut out = Obj::new();
    put(&mut out, "sentence", "RMC");
    put(&mut out, "time_utc", parse_utc_time(&f[0]));
    put(&mut out, "status", non_empty(&f[1]));
    put(&mut out, "valid", f[1] == "A");
    put(&mut out, "lat", fixed(parse_latitude(&f[2], &f[3])?, 6));
    put(&mut out, "lon", fixed(parse_longitude(&f[4], &f[5])?, 6));
    put(&mut out, "sog_knots", fixed(opt_float(&f[6])?, 2));
    put(&mut out, "cog_deg", fixed(opt_float(&f[7])?, 2));
    put(&mut out, "date", parse_date(&f[8]));
    put(&mut out, "magnetic_variation_deg", fixed(var, 1));
    put(&mut out, "mode", non_empty(&f[11]));
    put(&mut out, "mode_text", rmc_mode_text(&f[11]));
    Ok(out)
}

/// The type of every GPS parser: a plain function pointer.
pub type GpsParser = fn(&Sentence) -> Result<Obj, NmeaError>;

/// Python's `GPS_PARSERS.get(formatter)`.
pub fn gps_parser(formatter: &str) -> Option<GpsParser> {
    match formatter {
        "GGA" => Some(parse_gga),
        "GSA" => Some(parse_gsa),
        "GSV" => Some(parse_gsv),
        "RMC" => Some(parse_rmc),
        _ => None,
    }
}

// --- Python compatibility helpers -------------------------------------------
//
// These reproduce the exact acceptance rules and error texts of the Python
// built-ins the reference implementation relies on, so that the two programs
// agree even on malformed input.

/// Python's `str.isspace()`: Unicode White_Space plus the four ASCII
/// separators 0x1C–0x1F, which Rust's `char::is_whitespace` leaves out.
pub fn is_py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python's `str.strip()` with no arguments.
pub fn py_strip(s: &str) -> &str {
    s.trim_matches(is_py_space)
}

/// Python's `str.isdigit()` restricted to ASCII: non-empty and all `0-9`.
pub fn is_ascii_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Python's `repr()` of a string: single quotes unless the text contains a
/// single quote and no double quote; backslash escapes for the quote,
/// backslash, `\n`, `\r`, `\t` and other control characters.
pub fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `repr()` of a one-character string.
pub fn py_repr_char(c: char) -> String {
    py_repr(&c.to_string())
}

/// Remove Python's digit-grouping underscores (`1_000`). Returns `None` when
/// an underscore is not between two digits, which Python also rejects.
fn strip_underscores(s: &str, radix: u32) -> Option<String> {
    if !s.contains('_') {
        return Some(s.to_string());
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c == '_' {
            let prev_ok = i > 0 && chars[i - 1].is_digit(radix);
            let next_ok = i + 1 < chars.len() && chars[i + 1].is_digit(radix);
            if !(prev_ok && next_ok) {
                return None;
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

/// Python's `int(s, radix)` for radix 10 or 16: surrounding whitespace and a
/// sign are allowed, underscores between digits too; anything else is an
/// error (`None`).
pub fn py_int_radix(s: &str, radix: u32) -> Option<i64> {
    let s = py_strip(s);
    let (negative, digits) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    // Base 16 accepts an optional 0x prefix (Python: int("0x1f", 16) == 31).
    let digits = if radix == 16 {
        digits
            .strip_prefix("0x")
            .or_else(|| digits.strip_prefix("0X"))
            .map(|d| d.strip_prefix('_').unwrap_or(d))
            .unwrap_or(digits)
    } else {
        digits
    };
    let digits = strip_underscores(digits, radix)?;
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    let magnitude = i64::from_str_radix(&digits, radix).ok()?;
    Some(if negative { -magnitude } else { magnitude })
}

/// Python's `int(s)`, with Python's error text on failure.
pub fn py_int(s: &str) -> Result<i64, NmeaError> {
    py_int_radix(s, 10).ok_or_else(|| {
        NmeaError::BadNumber(format!(
            "invalid literal for int() with base 10: {}",
            py_repr(s)
        ))
    })
}

/// Python's `float(s)`, with Python's error text on failure.
pub fn py_float(s: &str) -> Result<f64, NmeaError> {
    let fail = || NmeaError::BadNumber(format!("could not convert string to float: {}", py_repr(s)));
    let t = py_strip(s);
    let t = strip_underscores(t, 10).ok_or_else(fail)?;
    // Rust's parser agrees with Python's on decimal/exponent syntax and on
    // the spellings "inf", "infinity" and "nan" (any case, optional sign).
    t.parse::<f64>().map_err(|_| fail())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_matches_spec_example() {
        assert_eq!(checksum("GPGSV,1,1,00"), 0x79);
        let s = parse_sentence("$GPGSV,1,1,00*79\r\n").unwrap();
        assert!(s.checksum_ok);
        assert_eq!(s.checksum_given, Some(0x79));
        assert_eq!(s.address(), "GPGSV");
        assert_eq!(s.fields, vec!["1", "1", "00"]);
    }

    #[test]
    fn uscg_extras_and_prefix() {
        let s = parse_sentence("foo.log:!AIVDM,1,1,,A,1,0*5C,r003,1223901985").unwrap();
        assert_eq!(s.start, '!');
        assert_eq!(s.talker, "AI");
        assert_eq!(s.formatter, "VDM");
        assert_eq!(s.after_checksum, vec!["r003", "1223901985"]);
        assert_eq!(s.raw, "!AIVDM,1,1,,A,1,0*5C,r003,1223901985");
        let p = parse_sentence("$PMTK010,001*2E").unwrap();
        assert_eq!(p.talker, "P");
        assert_eq!(p.formatter, "MTK010");
    }

    #[test]
    fn errors() {
        assert_eq!(parse_sentence("hello").unwrap_err().to_string(), "no '$' or '!' in line");
        assert_eq!(
            parse_sentence("$GP,1*00").unwrap_err().to_string(),
            "address field too short: 'GP'"
        );
        let s = parse_sentence("$GPGGA,1").unwrap();
        assert!(!s.checksum_ok);
        assert_eq!(s.checksum_given, None);
    }

    #[test]
    fn angles() {
        assert_eq!(parse_latitude("4312.2805", "N").unwrap(), Some(43.204675));
        assert_eq!(parse_longitude("07052.8474", "W").unwrap(), Some(-70.88079));
        assert_eq!(parse_latitude("", "N").unwrap(), None);
        assert_eq!(parse_utc_time("000429.000"), Some("00:04:29.000".to_string()));
        assert_eq!(parse_date("081207"), Some("2007-12-08".to_string()));
    }

    #[test]
    fn python_builtins() {
        assert_eq!(py_int_radix("6D", 16), Some(0x6d));
        assert_eq!(py_int_radix(" 7", 16), Some(7));
        assert_eq!(py_int_radix("0x", 16), None);
        assert_eq!(py_int_radix("", 16), None);
        assert_eq!(py_int("+12"), Ok(12));
        assert_eq!(py_int("1_0"), Ok(10));
        assert!(py_int("1_").is_err());
        assert_eq!(
            py_int("12x").unwrap_err().to_string(),
            "invalid literal for int() with base 10: '12x'"
        );
        assert_eq!(py_float(" 1.5 "), Ok(1.5));
        assert_eq!(py_float(".5"), Ok(0.5));
        assert!(py_float("").is_err());
        assert_eq!(py_repr("a'b"), "\"a'b\"");
        assert_eq!(py_repr("a\\b\n"), "'a\\\\b\\n'");
        assert_eq!(py_repr_char('\u{1}'), "'\\x01'");
        assert_eq!(py_strip("\u{1c} x \r\n"), "x");
    }
}
