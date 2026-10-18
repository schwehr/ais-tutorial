//! `jsonout.rs` — a tiny, deterministic JSON writer.
//!
//! Why not a JSON crate? Because the Python, C++ and TypeScript versions of
//! this tutorial must print *byte-identical* output so that the test-suite
//! can diff them, and because the tutorial has a no-dependencies rule. So we
//! follow two rules everywhere:
//!
//!   1. Object keys are written in sorted order.
//!   2. Every non-integer number carries an explicit number of decimals
//!      (`Json::Fixed(value, decimals)`) and is printed with exactly that
//!      many (`format!("{:.*}")`, which rounds the exact binary value the
//!      same way Python's `format(x, '.6f')`, C's `printf` and JavaScript's
//!      `toFixed` do).
//!
//! Supported values: null, bool, integer, fixed-point number, string, array,
//! object.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `enum Json` is a *sum type*: a value is exactly one of the listed
//!   variants, and variants can carry data (`Int(i64)`, `Arr(Vec<Json>)`).
//!   Python used the dynamic types `None`/`bool`/`int`/`Fixed`/`str`/`list`/
//!   `dict` for the same job; Rust makes the set of possibilities explicit
//!   and `match` forces us to handle every one.
//! * `BTreeMap<String, Json>` is an ordered map — iterating it yields keys
//!   in sorted order, so rule 1 comes for free. (`HashMap` would not.)
//! * The `From<T> for Json` implementations let callers write
//!   `put(&mut obj, "mmsi", 123)` instead of `Json::Int(123)`; the
//!   conversion is picked by the compiler from the argument type.
//! * `impl fmt::Display for Json` is how a type opts into `to_string()`,
//!   `format!("{}")` and `println!("{}")`.

use std::collections::BTreeMap;
use std::fmt;

/// A JSON object. The alias keeps signatures short.
pub type Obj = BTreeMap<String, Json>;

/// One JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    /// A float printed with a fixed number of decimals: `Fixed(42.0740401, 6)`
    /// prints as `42.074040`.
    Fixed(f64, usize),
    Str(String),
    Arr(Vec<Json>),
    Obj(Obj),
}

impl Json {
    /// Shorthand for `Json::Fixed`; reads better at call sites.
    pub fn fixed(value: f64, decimals: usize) -> Json {
        Json::Fixed(value, decimals)
    }
}

/// Insert `key: value` into an object, converting `value` with `Into<Json>`.
///
/// `impl Into<Json>` is an *argument-position impl Trait*: the function is
/// generic over any type that can be converted into a `Json`.
pub fn put(obj: &mut Obj, key: &str, value: impl Into<Json>) {
    obj.insert(key.to_string(), value.into());
}

// --- Conversions -----------------------------------------------------------
//
// Each `impl From<X> for Json` teaches the compiler how to turn an `X` into a
// `Json`. `Option<T>` maps `None` to `Json::Null`, mirroring Python's `None`.

impl From<bool> for Json {
    fn from(v: bool) -> Json {
        Json::Bool(v)
    }
}
impl From<i64> for Json {
    fn from(v: i64) -> Json {
        Json::Int(v)
    }
}
impl From<i32> for Json {
    fn from(v: i32) -> Json {
        Json::Int(i64::from(v))
    }
}
impl From<u32> for Json {
    fn from(v: u32) -> Json {
        Json::Int(i64::from(v))
    }
}
impl From<u64> for Json {
    fn from(v: u64) -> Json {
        // AIS fields are at most 30 bits wide, so this never truncates; the
        // `try_from` + fallback keeps the conversion honest anyway.
        Json::Int(i64::try_from(v).unwrap_or(i64::MAX))
    }
}
impl From<usize> for Json {
    fn from(v: usize) -> Json {
        Json::Int(i64::try_from(v).unwrap_or(i64::MAX))
    }
}
impl From<&str> for Json {
    fn from(v: &str) -> Json {
        Json::Str(v.to_string())
    }
}
impl From<String> for Json {
    fn from(v: String) -> Json {
        Json::Str(v)
    }
}
impl From<Vec<Json>> for Json {
    fn from(v: Vec<Json>) -> Json {
        Json::Arr(v)
    }
}
impl From<Obj> for Json {
    fn from(v: Obj) -> Json {
        Json::Obj(v)
    }
}
impl<T: Into<Json>> From<Option<T>> for Json {
    fn from(v: Option<T>) -> Json {
        match v {
            Some(x) => x.into(),
            None => Json::Null,
        }
    }
}

// --- Serialisation ---------------------------------------------------------

/// Format a float with `decimals` places, printing negative zero as positive.
fn format_fixed(value: f64, decimals: usize) -> String {
    if value.is_nan() {
        return "nan".to_string(); // Python spells it in lower case
    }
    // `{:.*}` takes the precision from the first argument.
    let s = format!("{:.*}", decimals, value);
    // "-0.000" is mathematically zero; Python's version drops the sign and
    // so do we. The check "every character after '-' is '0' or '.'" is the
    // same as "float(s) == 0.0" for a fixed-point string.
    match s.strip_prefix('-') {
        Some(rest) if rest.chars().all(|c| c == '0' || c == '.') => rest.to_string(),
        _ => s,
    }
}

/// Write `s` as a JSON string literal with minimal escaping.
fn escape(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                // Other control characters: \u00XX with lower-case hex.
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c), // non-ASCII passes through as UTF-8
        }
    }
    out.push('"');
}

/// Recursive worker: append the compact JSON text of `value` to `out`.
fn write(value: &Json, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Int(i) => out.push_str(&i.to_string()),
        Json::Fixed(v, d) => out.push_str(&format_fixed(*v, *d)),
        Json::Str(s) => escape(s, out),
        Json::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write(item, out);
            }
            out.push(']');
        }
        Json::Obj(map) => {
            out.push('{');
            // BTreeMap iterates in key order — sorted output for free.
            for (i, (key, item)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                escape(key, out);
                out.push(':');
                write(item, out);
            }
            out.push('}');
        }
    }
}

impl fmt::Display for Json {
    /// `Display` is what `to_string()` and `{}` use; we build the whole
    /// string first and hand it to the formatter in one go.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        write(self, &mut out);
        f.write_str(&out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_numbers() {
        assert_eq!(Json::fixed(42.0740401, 6).to_string(), "42.074040");
        assert_eq!(Json::fixed(-0.0, 1).to_string(), "0.0");
        assert_eq!(Json::fixed(-0.00001, 2).to_string(), "0.00");
        assert_eq!(Json::fixed(-0.5, 1).to_string(), "-0.5");
        assert_eq!(Json::fixed(1179170462.74, 2).to_string(), "1179170462.74");
    }

    #[test]
    fn escaping_and_sorting() {
        let mut obj = Obj::new();
        put(&mut obj, "b", "q\"\\\n\u{1}");
        put(&mut obj, "a", Option::<i64>::None);
        put(&mut obj, "c", vec![Json::from(true), Json::from(2i64)]);
        assert_eq!(
            Json::Obj(obj).to_string(),
            r#"{"a":null,"b":"q\"\\\n\u0001","c":[true,2]}"#
        );
    }
}
