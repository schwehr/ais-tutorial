//! `decode.rs` — command-line driver: NMEA/AIS log lines in, JSON lines out.
//!
//! ```text
//! decode FILE [FILE...]        # or '-' for standard input
//! ```
//!
//! For every input line we print at most one JSON object (see jsonout.rs for
//! why the JSON is hand-rolled). The same program exists in Python, C++ and
//! TypeScript, and the test-suite checks that all four print identical text.
//!
//! Output shapes
//! -------------
//! ```text
//! GPS sentence          {"line":7,"sentence":"GGA", ...fields...}
//! AIS message           {"line":9,"ais":{...decoded...},"channel":"A","talker":"AI",
//!                        "own_ship":false,"payload_bits":168,"fragments":1,
//!                        "uscg":{...},"tagblock":{...}}
//!                       ("uscg"/"tagblock" only when present)
//! unknown formatter     {"line":3,"address":"PMTK010","skipped":"no parser"}
//! problem               {"line":5,"error":"checksum mismatch","raw":"..."}
//! ```
//! Logger comments (# ...) and blank lines print nothing.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `decoder_for` returns an `Option<fn(&Bits) -> Result<Json, DecodeError>>`
//!   — a plain *function pointer*. Python kept the same mapping in a dict;
//!   a `match` that returns a function is the Rust way to write a dispatch
//!   table without any allocation.
//! * `process_line` returns `Option<Obj>`: `None` means "print nothing",
//!   exactly like the Python version returning `None`.
//! * Error handling is by `match` on each `Result`; there is no try/except
//!   that can accidentally swallow an unrelated failure.
//! * Input is read as raw bytes and converted with
//!   `String::from_utf8_lossy`, because real logs contain stray bytes.
//!   Invalid sequences become U+FFFD, the same thing Python's
//!   `errors="replace"` does.
//! * Python opened files in *universal newlines* mode, where `\r`, `\n`
//!   and `\r\n` all end a line. `str::lines()` only understands the last
//!   two, so `for_each_line` implements the same rule by hand — otherwise
//!   the two programs would disagree on line numbers.
//! * Writing to stdout can fail (`head` closed the pipe). We treat that as
//!   a normal end of output, as Python does for `BrokenPipeError`.

use std::io::{self, BufWriter, Read, Write};

use crate::bits::Bits;
use crate::common::DecodeError;
use crate::jsonout::{put, Json, Obj};
use crate::nmea::{gps_parser, parse_sentence};
use crate::tagblock::split_tag_block;
use crate::uscg::parse_uscg_fields;
use crate::vdm::{parse_vdm, Assembler, Message};
use crate::{
    msg_1_2_3, msg_10, msg_12, msg_14, msg_15, msg_16, msg_17, msg_18, msg_19, msg_20, msg_21,
    msg_22, msg_23, msg_24, msg_25, msg_26, msg_27, msg_4_11, msg_5, msg_6, msg_7_13, msg_8, msg_9,
};

/// Every message decoder has this shape.
pub type Decoder = fn(&Bits) -> Result<Json, DecodeError>;

/// One module per message type (or family). Python's `DECODERS` dict.
pub fn decoder_for(msg_type: u64) -> Option<Decoder> {
    match msg_type {
        1..=3 => Some(msg_1_2_3::decode),
        4 | 11 => Some(msg_4_11::decode),
        5 => Some(msg_5::decode),
        6 => Some(msg_6::decode),
        7 | 13 => Some(msg_7_13::decode),
        8 => Some(msg_8::decode),
        9 => Some(msg_9::decode),
        10 => Some(msg_10::decode),
        12 => Some(msg_12::decode),
        14 => Some(msg_14::decode),
        15 => Some(msg_15::decode),
        16 => Some(msg_16::decode),
        17 => Some(msg_17::decode),
        18 => Some(msg_18::decode),
        19 => Some(msg_19::decode),
        20 => Some(msg_20::decode),
        21 => Some(msg_21::decode),
        22 => Some(msg_22::decode),
        23 => Some(msg_23::decode),
        24 => Some(msg_24::decode),
        25 => Some(msg_25::decode),
        26 => Some(msg_26::decode),
        27 => Some(msg_27::decode),
        _ => None,
    }
}

/// Turn an assembled VDM payload into the `ais`/`error` part of the output.
pub fn decode_ais(msg: &Message) -> Obj {
    let mut out = Obj::new();
    let bits = match Bits::from_payload(&msg.payload, msg.fill_bits) {
        Ok(b) => b,
        Err(e) => {
            put(&mut out, "error", e.to_string());
            return out;
        }
    };
    if bits.len() < 6 {
        put(&mut out, "error", format!("payload too short: {} bits", bits.len()));
        put(&mut out, "payload_bits", bits.len());
        return out;
    }
    let msg_type = bits.uint(0, 6);
    put(&mut out, "payload_bits", bits.len());
    let Some(decoder) = decoder_for(msg_type) else {
        put(&mut out, "error", format!("unknown message type {msg_type}"));
        put(&mut out, "msg_type", msg_type);
        return out;
    };
    match decoder(&bits) {
        Ok(ais) => put(&mut out, "ais", ais),
        Err(DecodeError::NotImplemented(n)) => {
            let mut stub = Obj::new();
            put(&mut stub, "msg_type", n);
            put(&mut stub, "not_implemented", true);
            put(&mut out, "ais", stub);
        }
        Err(e) => {
            put(&mut out, "error", e.to_string());
            put(&mut out, "msg_type", msg_type);
        }
    }
    out
}

/// Handle one text line. Returns an object to print, or `None` to stay quiet.
pub fn process_line(line: &str, lineno: usize, asm: &mut Assembler) -> Option<Obj> {
    let stripped = crate::nmea::py_strip(line);
    if stripped.is_empty() || stripped.starts_with('#') {
        return None;
    }
    let mut out = Obj::new();
    put(&mut out, "line", lineno);

    let (tag, rest) = match split_tag_block(stripped) {
        Ok(parts) => parts,
        Err(e) => {
            put(&mut out, "error", e.to_string());
            put(&mut out, "raw", stripped);
            return Some(out);
        }
    };
    if let Some(tag) = tag {
        put(&mut out, "tagblock", tag.to_json());
    }

    let s = match parse_sentence(rest) {
        Ok(s) => s,
        Err(e) => {
            put(&mut out, "error", e.to_string());
            put(&mut out, "raw", stripped);
            return Some(out);
        }
    };
    if !s.after_checksum.is_empty() {
        put(&mut out, "uscg", parse_uscg_fields(&s.after_checksum));
    }
    if !s.checksum_ok {
        let text = if s.checksum_given.is_some() {
            "checksum mismatch"
        } else {
            "no checksum"
        };
        put(&mut out, "error", text);
        put(&mut out, "raw", s.raw.as_str());
        return Some(out);
    }

    if s.formatter == "VDM" || s.formatter == "VDO" {
        let frag = match parse_vdm(&s) {
            Ok(f) => f,
            Err(e) => {
                put(&mut out, "error", e.to_string());
                put(&mut out, "raw", s.raw.as_str());
                return Some(out);
            }
        };
        // `asm.add` takes ownership of `frag`; `msg` owns all fragments.
        let msg = asm.add(frag)?; // None: waiting for more fragments
        out.extend(decode_ais(&msg));
        put(&mut out, "channel", msg.channel.as_str());
        put(&mut out, "talker", msg.talker.as_str());
        put(&mut out, "own_ship", msg.own_ship);
        put(&mut out, "fragments", msg.fragments.len());
        return Some(out);
    }

    let parser = if s.talker == "P" { None } else { gps_parser(&s.formatter) };
    let Some(parser) = parser else {
        put(&mut out, "address", s.address());
        put(&mut out, "skipped", "no parser");
        return Some(out);
    };
    match parser(&s) {
        Ok(fields) => out.extend(fields),
        Err(e) => {
            // The Python reference would raise here (uncaught ValueError);
            // we report the same text as a line error instead of crashing.
            put(&mut out, "error", e.to_string());
            put(&mut out, "raw", s.raw.as_str());
        }
    }
    Some(out)
}

/// Call `f` for every line of `text`, splitting on `\n`, `\r\n` or a lone
/// `\r` (Python's universal newlines). A final line without a terminator
/// is still delivered; an empty tail after the last terminator is not.
pub fn for_each_line(text: &str, mut f: impl FnMut(&str)) {
    let mut start = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                f(&text[start..i]);
                i += 1;
                start = i;
            }
            b'\r' => {
                f(&text[start..i]);
                i += 1;
                if i < bytes.len() && bytes[i] == b'\n' {
                    i += 1; // "\r\n" counts as one terminator
                }
                start = i;
            }
            _ => i += 1,
        }
    }
    if start < bytes.len() {
        f(&text[start..]);
    }
}

/// Read a whole file (or stdin for "-") as bytes and decode it lossily.
fn read_input(name: &str) -> io::Result<String> {
    let mut data = Vec::new();
    if name == "-" {
        io::stdin().lock().read_to_end(&mut data)?;
    } else {
        std::fs::File::open(name)?.read_to_end(&mut data)?;
    }
    // `into_owned` turns the `Cow<str>` (borrowed if the bytes were valid
    // UTF-8, owned if replacements were made) into a plain `String`.
    Ok(String::from_utf8_lossy(&data).into_owned())
}

const USAGE: &str = "decode.rs — command-line driver: NMEA/AIS log lines in, JSON lines out.

    decode FILE [FILE...]        # or '-' for standard input
";

/// Program entry point; returns the process exit status.
pub fn main(argv: &[String]) -> i32 {
    if argv.is_empty() {
        println!("{USAGE}");
        return 2;
    }
    let stdout = io::stdout();
    // Buffering matters: unbuffered `println!` would flush on every line.
    let mut out = BufWriter::new(stdout.lock());
    let mut asm = Assembler::new();
    let mut lineno = 0usize;
    for name in argv {
        let text = match read_input(name) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("decode: {name}: {e}");
                return 1;
            }
        };
        let mut write_error = false;
        for_each_line(&text, |line| {
            lineno += 1;
            if write_error {
                return; // stdout is gone; just count lines and finish
            }
            if let Some(result) = process_line(line, lineno, &mut asm) {
                let json = Json::Obj(result);
                if writeln!(out, "{json}").is_err() {
                    write_error = true; // e.g. piped into `head`
                }
            }
        });
        if write_error {
            return 0;
        }
    }
    if out.flush().is_err() {
        return 0;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(line: &str) -> String {
        let mut asm = Assembler::new();
        match process_line(line, 1, &mut asm) {
            Some(obj) => Json::Obj(obj).to_string(),
            None => String::new(),
        }
    }

    #[test]
    fn shapes() {
        assert_eq!(run("# comment"), "");
        assert_eq!(run("   "), "");
        assert_eq!(
            run("$PMTK010,001*2E"),
            r#"{"address":"PMTK010","line":1,"skipped":"no parser"}"#
        );
        assert_eq!(
            run("hello"),
            r#"{"error":"no '$' or '!' in line","line":1,"raw":"hello"}"#
        );
        assert_eq!(
            run("$GPGSV,1,1,00*78"),
            r#"{"error":"checksum mismatch","line":1,"raw":"$GPGSV,1,1,00*78"}"#
        );
        assert_eq!(
            run("$GPGSV,1,1,00"),
            r#"{"error":"no checksum","line":1,"raw":"$GPGSV,1,1,00"}"#
        );
        // A one-character payload reaches the message 5 decoder and fails its
        // length check; the error object keeps the AIS envelope fields.
        let short = run("!AIVDM,1,1,,A,5,0*13");
        assert!(
            short.contains(r#""error":"message 5 too short: 6 bits < 424""#),
            "{short}"
        );
        assert!(short.contains(r#""msg_type":5"#), "{short}");
        assert!(short.contains(r#""payload_bits":6"#), "{short}");
    }

    #[test]
    fn universal_newlines() {
        let mut seen = Vec::new();
        for_each_line("a\nb\r\nc\rd", |l| seen.push(l.to_string()));
        assert_eq!(seen, vec!["a", "b", "c", "d"]);
        seen.clear();
        for_each_line("a\n", |l| seen.push(l.to_string()));
        assert_eq!(seen, vec!["a"]);
    }
}
