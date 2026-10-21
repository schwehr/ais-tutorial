//! `vdm.rs` — the !AIVDM / !AIVDO encapsulation sentence (chapter 2).
//!
//! ```text
//! !AIVDM,1,1,,B,15N8?>PP01JRVP6E7aLLkOvt0HAO,0*3C
//!  │  │  │ │ │ │ │                            │ └ checksum
//!  │  │  │ │ │ │ │                            └── fill bits (0–5)
//!  │  │  │ │ │ │ └─────────────────────────────── armoured payload
//!  │  │  │ │ │ └───────────────────────────────── radio channel A/B (or 1/2, or empty)
//!  │  │  │ │ └─────────────────────────────────── sequential message id (0–9) for multi-sentence groups
//!  │  │  │ └───────────────────────────────────── this sentence's number within the group
//!  │  │  └─────────────────────────────────────── total sentences in the group
//!  │  └────────────────────────────────────────── VDM = received from another station,
//!  │                                                VDO = own station's transmission
//!  └───────────────────────────────────────────── talker: AI mobile, AB/BS base, AN AtoN, AR receiver, …
//! ```
//!
//! IEC 61162-1 §8.3.90 (VDM) and §8.3.91 (VDO). An AIS message longer than
//! one sentence can hold (the 82-character limit leaves room for about 60
//! payload characters = 360 bits) is split across several VDM sentences with
//! the same sequential id; only the *last* fragment carries non-zero fill bits.
//!
//! Rust notes for readers new to the language
//! ------------------------------------------
//! * `parse_vdm` borrows the `Sentence` (`&Sentence`) and *clones* it into
//!   the `Vdm`. The clone is deliberate: the `Assembler` keeps fragments
//!   alive across many input lines, so it must own them outright rather
//!   than borrow from a line that is about to be dropped.
//! * `Assembler::add` takes `&mut self` because it changes the pending
//!   table. The borrow checker guarantees nobody else reads the table while
//!   it is being modified.
//! * `HashMap<(String, String, Option<i64>), Vec<Vdm>>` uses a *tuple* as
//!   the key, just like the Python `dict[tuple, list[Vdm]]`. Tuples of
//!   hashable types are hashable.
//! * `Option<Message>` is the return type of `add`: `None` means "keep
//!   feeding me fragments", `Some(msg)` means "here is a complete message".
//! * `else if let Some(group) = … && group.last().is_some_and(…)` is a
//!   *let chain* (edition 2024): one `if` both looks the pending group up
//!   and checks that the new fragment continues it. `is_some_and` asks a
//!   question of an `Option` without a `match`.

use std::collections::HashMap;
use std::fmt;

use crate::nmea::{py_int, py_repr, Sentence};

/// One `!AIVDM`/`!AIVDO` sentence, interpreted.
#[derive(Debug, Clone, PartialEq)]
pub struct Vdm {
    pub talker: String,
    pub own_ship: bool, // true for VDO
    pub total: i64,
    pub number: i64,
    pub seq_id: Option<i64>,
    pub channel: String,
    pub payload: String,
    pub fill_bits: i64,
    pub sentence: Sentence, // the enclosing NMEA sentence, for metadata
}

/// Why a sentence could not be read as a VDM. Texts match Python's
/// `ValueError` messages (including the one `int()` raises).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VdmError {
    NotVdm(String),
    FieldCount(usize),
    BadInt(String),
}

impl fmt::Display for VdmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VdmError::NotVdm(address) => write!(f, "not a VDM/VDO sentence: {address}"),
            VdmError::FieldCount(n) => write!(f, "VDM needs 6 fields, got {n}"),
            VdmError::BadInt(text) => {
                write!(f, "invalid literal for int() with base 10: {}", py_repr(text))
            }
        }
    }
}

impl std::error::Error for VdmError {}

/// Python's `int(field)` for the numeric VDM fields.
fn vdm_int(field: &str) -> Result<i64, VdmError> {
    py_int(field).map_err(|_| VdmError::BadInt(field.to_string()))
}

/// Interpret the six data fields of a VDM/VDO sentence.
pub fn parse_vdm(s: &Sentence) -> Result<Vdm, VdmError> {
    if s.formatter != "VDM" && s.formatter != "VDO" {
        return Err(VdmError::NotVdm(s.address()));
    }
    if s.fields.len() < 6 {
        return Err(VdmError::FieldCount(s.fields.len()));
    }
    // Destructure the first six fields by reference. The pattern binds
    // `total`, `number`, ... to `&String`s inside the slice.
    let [total, number, seq, channel, payload, fill] = &s.fields[..6] else {
        // The length check above makes this unreachable, but the compiler
        // cannot know that, so a slice pattern must have a fallback arm.
        return Err(VdmError::FieldCount(s.fields.len()));
    };
    Ok(Vdm {
        talker: s.talker.clone(),
        own_ship: s.formatter == "VDO",
        total: vdm_int(total)?,
        number: vdm_int(number)?,
        seq_id: if seq.is_empty() { None } else { Some(vdm_int(seq)?) },
        channel: channel.clone(),
        payload: payload.clone(),
        fill_bits: if fill.is_empty() { 0 } else { vdm_int(fill)? },
        sentence: s.clone(),
    })
}

/// A complete AIS message: the concatenated payload of all its fragments.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub payload: String,
    pub fill_bits: i64,
    pub channel: String,
    pub talker: String,
    pub own_ship: bool,
    pub fragments: Vec<Vdm>,
}

impl Message {
    /// Build from a group of fragments whose payloads are concatenated.
    fn from_fragments(fragments: Vec<Vdm>) -> Message {
        // `last()` returns Option; the groups we build are never empty, but
        // we fall back to harmless defaults instead of `unwrap()`.
        let last = fragments.last();
        Message {
            payload: fragments.iter().map(|g| g.payload.as_str()).collect(),
            fill_bits: last.map_or(0, |g| g.fill_bits),
            channel: last.map_or(String::new(), |g| g.channel.clone()),
            talker: last.map_or(String::new(), |g| g.talker.clone()),
            own_ship: last.is_some_and(|g| g.own_ship),
            fragments,
        }
    }

    pub fn first(&self) -> Option<&Vdm> {
        self.fragments.first()
    }

    pub fn last(&self) -> Option<&Vdm> {
        self.fragments.last()
    }
}

/// The key that fragments of one group share: (talker, channel, seq id).
type GroupKey = (String, String, Option<i64>);

/// Collects VDM fragments and hands back complete messages.
///
/// Fragments of a group share (talker, channel, sequential id). We keep
/// one pending group per key; a fragment that does not continue the
/// pending group replaces it (so a lost fragment only loses that message).
///
/// Usage:
/// ```text
/// let mut asm = Assembler::new();
/// for line in lines {
///     if let Some(msg) = asm.add(parse_vdm(&parse_sentence(line)?)?) {
///         ...decode msg.payload...
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct Assembler {
    pending: HashMap<GroupKey, Vec<Vdm>>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler::default()
    }

    /// Feed one fragment; get a `Message` back when a group is complete.
    pub fn add(&mut self, frag: Vdm) -> Option<Message> {
        if frag.total == 1 {
            return Some(Message::from_fragments(vec![frag]));
        }
        let key: GroupKey = (frag.talker.clone(), frag.channel.clone(), frag.seq_id);
        // Remember what we need *before* `frag` is moved into the table;
        // after a move the old variable can no longer be used.
        let completes_group = frag.number == frag.total;
        if frag.number == 1 {
            self.pending.insert(key.clone(), vec![frag]);
        } else if let Some(group) = self.pending.get_mut(&key)
            && group.last().is_some_and(|last| last.number == frag.number - 1)
        {
            // A let chain: the fragment continues the pending group, and
            // `group` is already borrowed mutably so we can push onto it.
            group.push(frag);
        } else {
            self.pending.remove(&key); // out of order: drop
            return None;
        }
        if completes_group {
            // `remove` hands the Vec back to us (ownership moves out of the map).
            let group = self.pending.remove(&key)?;
            return Some(Message::from_fragments(group));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nmea::parse_sentence;

    fn vdm(line: &str) -> Vdm {
        parse_vdm(&parse_sentence(line).unwrap()).unwrap()
    }

    #[test]
    fn two_fragment_message() {
        let mut asm = Assembler::new();
        let a = vdm("!AIVDM,2,1,3,B,55P5TL01VIaAL@7WKO@mBplU@<PDhh000000001S;AJ::4A80?4i@E53,0*3E");
        let b = vdm("!AIVDM,2,2,3,B,1@0000000000000,2*55");
        assert!(asm.add(a).is_none());
        let msg = asm.add(b).expect("complete message");
        assert_eq!(msg.fragments.len(), 2);
        assert_eq!(msg.fill_bits, 2);
        assert!(msg.payload.starts_with("55P5TL01"));
        assert!(msg.payload.ends_with("1@0000000000000"));
    }

    #[test]
    fn out_of_order_fragment_is_dropped() {
        let mut asm = Assembler::new();
        let b = vdm("!AIVDM,2,2,3,B,1@0000000000000,2*55");
        assert!(asm.add(b).is_none());
    }

    #[test]
    fn errors() {
        let s = parse_sentence("!AIVDM,1,1,,B*2A").unwrap();
        assert_eq!(parse_vdm(&s).unwrap_err().to_string(), "VDM needs 6 fields, got 4");
        let s = parse_sentence("!AIVDM,x,1,,B,1,0*00").unwrap();
        assert_eq!(
            parse_vdm(&s).unwrap_err().to_string(),
            "invalid literal for int() with base 10: 'x'"
        );
    }
}
