/**
 * msg_10.ts — message 10: UTC and date inquiry.
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.8, Table 60 (72 bits). The answer is a
 * message 11 (same layout as message 4, chapter 6), which "is only
 * transmitted as a result of a UTC request message (Message 10)" and goes
 * out "on the channel where the UTC request message was received"
 * [Annex 8 §3.2].
 *
 * Bit layout (zero-based offsets; lengths from Table 60):
 *
 *      0- 5   6  message id (10)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI of the station that wants to know the time)
 *     38-39   2  spare
 *     40-69  30  destination id (MMSI of the station being asked)
 *     70-71   2  spare
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * Nothing new: a function that returns an object literal. The property
 *   names are unquoted because they are valid identifiers; the JSON writer
 *   sorts them when printing.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";

/** Decode a message 10. Throws DecodeError if shorter than 72 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 72);
  return {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
    dest_mmsi: bits.uint(40, 30),
    spare2: bits.uint(70, 2),
  };
}
