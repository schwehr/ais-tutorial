/**
 * msg_25.ts — message 25: single slot binary message (chapter 16).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.23, Table 80 (layout) and Table 81
 * (maximum data bits per flag combination); Annex 5 §1.1 for the 16-bit
 * application identifier (10-bit DAC + 6-bit FI).
 *
 * Bit layout (zero-based offsets). Two flags decide which fields exist, so
 * the offsets after bit 39 are *variable*:
 *
 *      0- 5   6  message id (25)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI)
 *     38      1  destination indicator: 0 = broadcast, 1 = addressed
 *     39      1  binary data flag: 0 = unstructured, 1 = application identifier follows
 *     40-69  30  destination MMSI        (only if destination indicator = 1)
 *     70-71   2  spare                   (only if destination indicator = 1)
 *     next   16  application identifier  (only if binary data flag = 1): DAC 10 + FI 6
 *     rest    n  application data, to the end of the payload (Table 81: ≤ 128/112/96/80)
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `decodeBinaryHeader` returns a *tuple type* `[JsonObject, number]`.
 *   The caller destructures it: `const [out, offset] = decodeBinaryHeader(...)`.
 *   Without the explicit return type annotation TypeScript would infer the
 *   looser array type `(number | JsonObject)[]`.
 * * `null` is a real value in JSON and in TypeScript's `JsonValue` union;
 *   `undefined` is not, which is why the optional fields are set to `null`
 *   up front rather than left out.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";

/**
 * The part of messages 25 and 26 that precedes the application data.
 * Returns the decoded fields and the offset of the first data bit. Throws
 * DecodeError if the optional fields announced by the flags are missing.
 */
export function decodeBinaryHeader(bits: Bits, msgType: number): [JsonObject, number] {
  requireBits(msgType, bits.length, 40);
  const addressed = bits.flag(38);
  const structured = bits.flag(39);
  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    addressed, // destination indicator, bit 38
    structured, // binary data flag, bit 39
    dest_mmsi: null,
    dac: null,
    fi: null,
  };
  let offset = 40;
  if (addressed) {
    requireBits(msgType, bits.length, offset + 32);
    out["dest_mmsi"] = bits.uint(offset, 30);
    out["spare"] = bits.uint(offset + 30, 2);
    offset += 32;
  }
  if (structured) {
    requireBits(msgType, bits.length, offset + 16);
    out["dac"] = bits.uint(offset, 10);
    out["fi"] = bits.uint(offset + 10, 6);
    offset += 16;
  }
  out["data_offset"] = offset;
  return [out, offset];
}

/** Decode a message 25. Throws DecodeError if a required part is missing. */
export function decode(bits: Bits): JsonObject {
  const [out, offset] = decodeBinaryHeader(bits, 25);
  // Everything after the header is application data; we cannot know its
  // internal structure, so we hand it on as a bit string (CONVENTIONS §3).
  out["data"] = bits.raw(offset, bits.length - offset);
  out["data_bits"] = bits.length - offset;
  return out;
}
