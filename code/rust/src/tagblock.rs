//! `tagblock.rs` — NMEA 0183 v4.10 TAG blocks (chapter 4).
//!
//! A TAG block is metadata that *precedes* a sentence on the same line:
//!
//! ```text
//! \s:r003669945,c:1224006993*2B\!AIVDM,1,1,,A,15N4cJ`005Jrek0H@9n`DW5608EP,0*13
//! ^                            ^ ^
//! backslash  parameter list   checksum of the text between the backslashes
//! ```
//!
//! Defined in IEC 61162-1 ed. 4 / NMEA 0183 v4.10 §7.1.2 "TAG block".
//! Parameters are `code:value` pairs separated by commas:
//!
//! ```text
//! c:  UNIX time (seconds, or milliseconds if > 10^10 or configured so)
//! d:  destination identification (≤ 15 chars)
//! g:  sentence grouping "n-total-group_id" for multi-sentence groups
//! n:  line count (an incrementing serial number)
//! r:  relative time
//! s:  source identification (≤ 15 chars) — the receiving station
//! t:  free text (≤ 15 chars)
//! ```
//!
//! The checksum is computed exactly like a sentence checksum: XOR of all
//! characters between the two backslashes, excluding the `*hh`.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `split_tag_block(line: &str) -> Result<(Option<TagBlock>, &str), _>`
//!   returns a *slice of its input* (the remainder after the TAG block).
//!   The compiler ties the lifetime of that `&str` to the argument
//!   automatically ("lifetime elision"): the remainder is valid exactly as
//!   long as `line` is. No copy is made.
//! * `TagBlock::time()` and `group()` are methods that compute a typed view
//!   of the raw parameters on demand, like Python `@property`.
//! * `BTreeMap<String, String>` keeps the parameters sorted, which the JSON
//!   writer needs anyway.
//! * `if let Some(h) = hexpart && h.chars().count() >= 2 { … }` is a *let
//!   chain*: a pattern match and an ordinary condition in one `if`, with the
//!   bound name usable in the condition. Stable since edition 2024.

use std::collections::BTreeMap;
use std::fmt;

use crate::jsonout::{put, Json, Obj};
use crate::nmea::{checksum, is_ascii_digits, py_int_radix, py_strip};

/// A parsed TAG block.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagBlock {
    pub params: BTreeMap<String, String>,
    pub checksum_ok: bool,
    pub raw: String,
}

impl TagBlock {
    // Typed views of the common parameters. Missing -> None.

    /// The `c:` parameter as an integer, if it is all digits.
    pub fn time(&self) -> Option<i64> {
        let c = self.params.get("c")?;
        if is_ascii_digits(c) {
            c.parse().ok()
        } else {
            None
        }
    }

    /// The `s:` parameter (receiving station).
    pub fn source(&self) -> Option<&str> {
        self.params.get("s").map(String::as_str)
    }

    /// (sentence number, total sentences, group id) from the `g:` parameter.
    pub fn group(&self) -> Option<(i64, i64, i64)> {
        let g = self.params.get("g")?;
        if g.is_empty() {
            return None;
        }
        let parts: Vec<&str> = g.split('-').collect();
        if parts.len() != 3 || !parts.iter().all(|p| is_ascii_digits(p)) {
            return None;
        }
        Some((parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?))
    }

    /// The JSON shape printed under `"tagblock"`.
    pub fn to_json(&self) -> Json {
        let mut params = Obj::new();
        for (k, v) in &self.params {
            put(&mut params, k, v.as_str());
        }
        let mut d = Obj::new();
        put(&mut d, "params", params);
        put(&mut d, "checksum_ok", self.checksum_ok);
        if let Some(t) = self.time() {
            put(&mut d, "time", t);
        }
        if let Some((n, total, id)) = self.group() {
            put(&mut d, "group", vec![Json::from(n), Json::from(total), Json::from(id)]);
        }
        Json::Obj(d)
    }
}

/// The only way a TAG block can be malformed enough to reject the line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnterminatedTagBlock;

impl fmt::Display for UnterminatedTagBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unterminated TAG block")
    }
}

impl std::error::Error for UnterminatedTagBlock {}

/// Separate a leading TAG block from the sentence that follows it.
///
/// Returns `(TagBlock or None, remainder)`. Lines without a leading
/// backslash are returned unchanged.
pub fn split_tag_block(line: &str) -> Result<(Option<TagBlock>, &str), UnterminatedTagBlock> {
    let line = py_strip(line);
    let Some(after_open) = line.strip_prefix('\\') else {
        return Ok((None, line));
    };
    // `find` on the text after the opening backslash; the offset is relative
    // to `after_open`, so `inner` is exactly what sits between the two.
    let end = after_open.find('\\').ok_or(UnterminatedTagBlock)?;
    let inner = &after_open[..end];
    let remainder = &after_open[end + 1..];

    let (body, hexpart) = match inner.split_once('*') {
        Some((b, h)) => (b, Some(h)),
        None => (inner, None),
    };
    // A *let chain* (edition 2024): the pattern test and the length test
    // are joined with `&&` in one `if`, and `h` is in scope for the second
    // test. Before 2024 this needed two nested `if`s.
    let mut ok = false;
    if let Some(h) = hexpart
        && h.chars().count() >= 2
    {
        let first_two: String = h.chars().take(2).collect();
        ok = py_int_radix(&first_two, 16) == Some(i64::from(checksum(body)));
    }
    let mut params = BTreeMap::new();
    for item in body.split(',') {
        if let Some((code, value)) = item.split_once(':') {
            params.insert(code.to_string(), value.to_string());
        }
    }
    Ok((
        Some(TagBlock {
            params,
            checksum_ok: ok,
            raw: inner.to_string(),
        }),
        remainder,
    ))
}

/// Build a TAG block string from parameters (used in chapter 4 to convert
/// USCG-format lines into TAG block lines).
pub fn format_tag_block(params: &BTreeMap<String, String>) -> String {
    let body: Vec<String> = params.iter().map(|(k, v)| format!("{k}:{v}")).collect();
    let body = body.join(",");
    format!("\\{body}*{:02X}\\", checksum(&body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_and_checks() {
        let line = "\\s:r003669945,c:1224006993*7C\\!AIVDM,1,1,,A,15N4cJ`005Jrek0H@9n`DW5608EP,0*13";
        let (tag, rest) = split_tag_block(line).unwrap();
        let tag = tag.unwrap();
        assert!(tag.checksum_ok);
        assert_eq!(tag.time(), Some(1224006993));
        assert_eq!(tag.source(), Some("r003669945"));
        assert!(rest.starts_with("!AIVDM"));
        assert_eq!(
            tag.to_json().to_string(),
            r#"{"checksum_ok":true,"params":{"c":"1224006993","s":"r003669945"},"time":1224006993}"#
        );
    }

    #[test]
    fn group_and_errors() {
        let (tag, _) = split_tag_block("\\g:1-2-7*00\\$GPGGA").unwrap();
        assert_eq!(tag.unwrap().group(), Some((1, 2, 7)));
        assert_eq!(
            split_tag_block("\\g:1-2-7*00").unwrap_err().to_string(),
            "unterminated TAG block"
        );
        let (none, rest) = split_tag_block("  $GPGGA,1*00  ").unwrap();
        assert!(none.is_none());
        assert_eq!(rest, "$GPGGA,1*00");
    }

    #[test]
    fn round_trip() {
        let mut p = BTreeMap::new();
        p.insert("c".to_string(), "1224006993".to_string());
        p.insert("s".to_string(), "r003669945".to_string());
        let text = format_tag_block(&p);
        let (tag, _) = split_tag_block(&text).unwrap();
        assert!(tag.unwrap().checksum_ok);
    }
}
