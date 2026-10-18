/**
 * bits.ts — turning an AIS payload into bits, and bits into numbers and text.
 *
 * Everything in an AIS message is a *bit field*: "bits 8–37 are the MMSI",
 * "bits 61–88 are the longitude in 1/10 000 minute". The job of this module
 * is therefore very small:
 *
 *   1. Undo the six-bit ASCII "armouring" that NMEA uses to carry binary data
 *      in printable characters (see chapter 2).
 *   2. Give the rest of the code a tidy way to pull out unsigned integers,
 *      signed integers, booleans and six-bit text starting at any bit offset.
 *
 * Spec references:
 *   * Armouring table: IEC 61162-1 (NMEA 0183) §7.3.3 "Six-bit binary field
 *     conversion"; ITU-R M.1371-5 Annex 8 §3 (as reproduced in AIVDM.adoc
 *     "AIVDM/AIVDO Payload Armoring").
 *   * Six-bit ASCII for text fields: ITU-R M.1371-5 Annex 8 Table 47.
 *   * Two's complement for signed fields: ITU-R M.1371-5 Annex 8 §3.3.7.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * The Python version keeps the bits as a string of '0'/'1'. Here we use a
 *   `Uint8Array` — a *typed array* of bytes — holding one 0 or 1 per
 *   element. It reads just as clearly (`bits[61]` is bit 61) and shows the
 *   one JavaScript type that stores raw numbers compactly.
 * * **The 32-bit pitfall.** JavaScript's bitwise operators (`<<`, `>>`,
 *   `|`, `&`) first convert their operands to *signed 32-bit* integers.
 *   `1 << 31` is therefore -2147483648, and `x << 30` can silently lose
 *   bits. AIS fields are up to 30 bits (MMSI) and we also compute
 *   `2**length` for the sign correction, so `uint()` builds its result
 *   with ordinary multiplication (`value * 2 + bit`), which is exact up to
 *   2**53 (`Number.MAX_SAFE_INTEGER`). Fields longer than 53 bits would
 *   need `BigInt`; AIS has none that we read as a single number.
 * * `static fromPayload(...)` is a factory — the equivalent of Python's
 *   `@classmethod`. `get length()` is a getter, replacing `__len__`.
 */

import { ValueError, pyRepr } from "./common";

// --- 1. Armouring ---------------------------------------------------------
//
// Each payload character carries 6 bits. The sender adds 48 to the 6-bit
// value; if the result is above 87 it adds another 8 to skip the characters
// 'X'..'_' (which were avoided because some old equipment mangled them).
// To decode we do the reverse: subtract 48, and if the result is above 40
// subtract 8 more.

/** Return the 6-bit value (0–63) carried by one payload character. */
export function armorCharToValue(ch: string): number {
  const code = ch.codePointAt(0) ?? -1;
  if (code < 48 || code > 119 || (88 <= code && code <= 95)) { // outside '0'..'w', or 'X'..'_'
    throw new ValueError(`illegal armour character ${pyRepr(ch)}`);
  }
  let value = code - 48;
  if (value > 40) {
    value -= 8;
  }
  return value;
}

/**
 * Convert an armoured payload to an array of 0/1 values.
 *
 * `fillBits` is field 7 of the !AIVDM sentence: how many of the low-order
 * bits of the *last* character are padding and must be thrown away.
 */
export function payloadToBits(payload: string, fillBits = 0): Uint8Array {
  if (!(0 <= fillBits && fillBits <= 5)) {
    throw new ValueError(`fill bits must be 0..5, got ${fillBits}`);
  }
  // Array.from(string) splits into Unicode code points, like Python's `for ch in s`.
  const chars = Array.from(payload);
  const bits = new Uint8Array(chars.length * 6);
  for (let i = 0; i < chars.length; i++) {
    const value = armorCharToValue(chars[i]);
    for (let b = 0; b < 6; b++) {
      // Most significant bit first: (value >> 5) & 1, then (value >> 4) & 1, ...
      // These are 6-bit values, so the 32-bit operators are safe here.
      bits[i * 6 + b] = (value >> (5 - b)) & 1;
    }
  }
  return fillBits ? bits.slice(0, Math.max(0, bits.length - fillBits)) : bits;
}

// --- 2. Six-bit ASCII used *inside* text fields -----------------------------
//
// Do not confuse this with the armouring above! Text fields (ship name, call
// sign, destination, safety messages) pack one character into 6 bits using a
// different table: 0–31 map to '@'..'_' and 32–63 map to ' '..'?'.

export const SIXBIT_ASCII = "@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_ !\"#$%&'()*+,-./0123456789:;<=>?";
if (SIXBIT_ASCII.length !== 64) {
  throw new Error("SIXBIT_ASCII must have 64 entries");
}

// --- 3. The Bits reader -----------------------------------------------------

/**
 * A bit string with typed accessors.
 *
 * All offsets are *zero-based* bit positions, exactly as in the field
 * tables of this tutorial (ITU-R M.1371 itself counts bits per field,
 * GPSD's AIVDM.adoc and this tutorial count absolute offsets from 0).
 */
export class Bits {
  private readonly bits: Uint8Array;

  constructor(bits: Uint8Array) {
    for (const b of bits) {
      if (b !== 0 && b !== 1) {
        throw new ValueError("Bits expects an array of 0 and 1");
      }
    }
    this.bits = bits;
  }

  /** Build directly from an !AIVDM payload and its fill-bit count. */
  static fromPayload(payload: string, fillBits = 0): Bits {
    return new Bits(payloadToBits(payload, fillBits));
  }

  /** Number of bits (Python: `len(bits)`). */
  get length(): number {
    return this.bits.length;
  }

  /** True if the field [start, start+length) lies inside the message. */
  has(start: number, length: number): boolean {
    return start + length <= this.bits.length;
  }

  /** Unsigned big-endian integer from `length` bits at `start`. */
  uint(start: number, length: number): number {
    if (!this.has(start, length)) {
      throw new RangeError(`field ${start}+${length} beyond ${this.bits.length} bits`);
    }
    let value = 0;
    for (let i = start; i < start + length; i++) {
      value = value * 2 + this.bits[i]; // NOT `value << 1 | bit` — see the 32-bit pitfall above
    }
    return value;
  }

  /** Signed (two's complement) integer. The first bit is the sign. */
  sint(start: number, length: number): number {
    let value = this.uint(start, length);
    if (this.bits[start] === 1) { // negative: subtract 2**length
      value -= 2 ** length;
    }
    return value;
  }

  /** A single bit as a boolean. */
  flag(start: number): boolean {
    return this.uint(start, 1) === 1;
  }

  /**
   * Six-bit ASCII text occupying `length` bits (a multiple of 6).
   *
   * Per ITU-R M.1371-5 Annex 8 §3.3.8, unused characters are '@'; in
   * practice names are often space-padded instead. We cut at the first
   * '@' and strip trailing spaces.
   */
  text(start: number, length: number): string {
    const chars: string[] = [];
    for (let pos = start; pos < start + length - 5; pos += 6) {
      if (!this.has(pos, 6)) {
        break; // tolerate short (truncated) fields
      }
      chars.push(SIXBIT_ASCII[this.uint(pos, 6)]);
    }
    let s = chars.join("");
    const at = s.indexOf("@");
    if (at >= 0) {
      s = s.slice(0, at);
    }
    // Python's rstrip(); only ' ' can occur because SIXBIT_ASCII has no other whitespace.
    return s.replace(/ +$/, "");
  }

  /** The field as a '0'/'1' string — for binary payloads we cannot decode. */
  raw(start: number, length: number): string {
    let s = "";
    for (const b of this.bits.subarray(start, start + length)) {
      s += b === 1 ? "1" : "0";
    }
    return s;
  }
}
