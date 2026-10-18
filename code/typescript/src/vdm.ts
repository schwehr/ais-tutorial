/**
 * vdm.ts — the !AIVDM / !AIVDO encapsulation sentence (chapter 2).
 *
 *     !AIVDM,1,1,,B,15N8?>PP01JRVP6E7aLLkOvt0HAO,0*3C
 *      │  │  │ │ │ │ │                            │ └ checksum
 *      │  │  │ │ │ │ │                            └── fill bits (0–5)
 *      │  │  │ │ │ │ └─────────────────────────────── armoured payload
 *      │  │  │ │ │ └───────────────────────────────── radio channel A/B (or 1/2, or empty)
 *      │  │  │ │ └─────────────────────────────────── sequential message id (0–9) for multi-sentence groups
 *      │  │  │ └───────────────────────────────────── this sentence's number within the group
 *      │  │  └─────────────────────────────────────── total sentences in the group
 *      │  └────────────────────────────────────────── VDM = received from another station,
 *      │                                                VDO = own station's transmission
 *      └───────────────────────────────────────────── talker: AI mobile, AB/BS base, AN AtoN, AR receiver, …
 *
 * IEC 61162-1 §8.3.90 (VDM) and §8.3.91 (VDO). An AIS message longer than
 * one sentence can hold (the 82-character limit leaves room for about 60
 * payload characters = 360 bits) is split across several VDM sentences with
 * the same sequential id; only the *last* fragment carries non-zero fill bits.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `Vdm` and `Message` are *interfaces*: pure descriptions of an object's
 *   shape with no methods and no run-time existence. An object literal
 *   `{ talker: ..., ownShip: ... }` satisfies the interface if it has the
 *   right properties — no `new` needed. Compare `Sentence` in nmea.ts,
 *   which is a *class* because it has a getter.
 * * `Assembler` keeps pending fragments in a `Map<string, Vdm[]>`. Python
 *   can use a tuple as a dict key; JavaScript `Map`s compare object keys by
 *   identity, so two equal tuples would be *different* keys. We therefore
 *   build a string key with `JSON.stringify([...])`.
 * * `private readonly pending = new Map()` is a field initialiser; `private`
 *   is checked by the compiler only.
 */

import { Sentence } from "./nmea";
import { ValueError, pyInt } from "./common";

export interface Vdm {
  readonly talker: string;
  readonly ownShip: boolean;       // true for VDO
  readonly total: number;
  readonly number: number;
  readonly seqId: number | null;
  readonly channel: string;
  readonly payload: string;
  readonly fillBits: number;
  readonly sentence: Sentence;     // the enclosing NMEA sentence, for metadata
}

/** Interpret the six data fields of a VDM/VDO sentence. */
export function parseVdm(s: Sentence): Vdm {
  if (s.formatter !== "VDM" && s.formatter !== "VDO") {
    throw new ValueError(`not a VDM/VDO sentence: ${s.address}`);
  }
  if (s.fields.length < 6) {
    throw new ValueError(`VDM needs 6 fields, got ${s.fields.length}`);
  }
  // Array destructuring, like Python's tuple unpacking.
  const [total, number, seq, channel, payload, fill] = s.fields;
  return {
    talker: s.talker,
    ownShip: s.formatter === "VDO",
    total: pyInt(total),
    number: pyInt(number),
    seqId: seq ? pyInt(seq) : null,
    channel,
    payload,
    fillBits: fill ? pyInt(fill) : 0,
    sentence: s,
  };
}

/** A complete AIS message: the concatenated payload of all its fragments. */
export interface Message {
  readonly payload: string;
  readonly fillBits: number;
  readonly channel: string;
  readonly talker: string;
  readonly ownShip: boolean;
  readonly fragments: readonly Vdm[];
}

/**
 * Collects VDM fragments and hands back complete messages.
 *
 * Fragments of a group share (talker, channel, sequential id). We keep
 * one pending group per key; a fragment that does not continue the
 * pending group replaces it (so a lost fragment only loses that message).
 *
 * Usage:
 *     const asm = new Assembler();
 *     for (const line of lines) {
 *       const msg = asm.add(parseVdm(parseSentence(line)));
 *       if (msg !== null) { ...decode msg.payload... }
 *     }
 */
export class Assembler {
  private readonly pending = new Map<string, Vdm[]>();

  add(frag: Vdm): Message | null {
    if (frag.total === 1) {
      return { payload: frag.payload, fillBits: frag.fillBits, channel: frag.channel,
               talker: frag.talker, ownShip: frag.ownShip, fragments: [frag] };
    }
    const key = JSON.stringify([frag.talker, frag.channel, frag.seqId]);
    let group: Vdm[];
    if (frag.number === 1) {
      group = [frag];
      this.pending.set(key, group);
    } else {
      const found = this.pending.get(key);
      if (found === undefined || found[found.length - 1].number !== frag.number - 1) {
        this.pending.delete(key); // out of order: drop
        return null;
      }
      found.push(frag);
      group = found;
    }
    if (frag.number === frag.total) {
      this.pending.delete(key);
      const payload = group.map((g) => g.payload).join("");
      return { payload, fillBits: frag.fillBits, channel: frag.channel,
               talker: frag.talker, ownShip: frag.ownShip, fragments: group };
    }
    return null;
  }
}
