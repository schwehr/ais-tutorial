/**
 * msg_12.ts — message 12: addressed safety related message.
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.10, Table 61 (layout), Table 62 (how
 * many characters fit in 1–5 slots), Table 47 (six-bit ASCII); Annex 2
 * §5.3.1 (sequence numbers, retransmission, acknowledgement by message 13).
 *
 * Bit layout (zero-based offsets; lengths from Table 61):
 *
 *      0- 5   6  message id (12)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI)
 *     38-39   2  sequence number, 0-3
 *     40-69  30  destination id (MMSI)
 *     70      1  retransmit flag (0 = first transmission, 1 = retransmitted)
 *     71      1  spare
 *     72-     ≤936  safety related text, six-bit ASCII, 0-156 characters
 *
 * Length rule: the 72-bit header is required (DecodeError otherwise). Every
 * complete six-bit group after it is one character. The 0-5 bits left over
 * at the end are *not* text: the radio layer pads messages to a byte
 * boundary (Annex 2 §5.2.1), and the receiver's armouring pads again to six
 * bits, so a real message often carries 2 or 4 stray bits after the last
 * character. We count them in `trailing_bits` and ignore them.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * JavaScript has one number type (a double). `text_bits / 6` would give
 *   18.666…, so integer division is spelled `Math.floor(text_bits / 6)`;
 *   `%` is fine because both operands are non-negative integers.
 * * `export function decodeTextField` makes the helper importable from
 *   msg_14.ts with `import { decodeTextField } from "./msg_12";`.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";

const HEADER_BITS = 72;

/**
 * Decode a variable-length six-bit text field that runs to the end.
 *
 * Shared by messages 12 and 14. `Bits.text` already stops before a partial
 * six-bit group, cuts the text at the first '@' (the Table 47 padding /
 * not-available character) and strips trailing spaces; here we add the
 * bookkeeping numbers.
 */
export function decodeTextField(bits: Bits, start: number): JsonObject {
  const textBits = bits.length - start;
  return {
    text: bits.text(start, textBits),
    text_bits: textBits,
    text_chars: Math.floor(textBits / 6), // characters on the air, '@' padding included
    trailing_bits: textBits % 6,          // byte-alignment leftovers, not text
  };
}

/** Decode a message 12. Throws DecodeError below 72 bits. */
export function decode(bits: Bits): JsonObject {
  requireBits(12, bits.length, HEADER_BITS);
  const out: JsonObject = {
    msg_type: bits.uint(0, 6),
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    sequence_number: bits.uint(38, 2),
    dest_mmsi: bits.uint(40, 30),
    retransmit: bits.flag(70),
    spare: bits.uint(71, 1),
  };
  Object.assign(out, decodeTextField(bits, HEADER_BITS));
  return out;
}
