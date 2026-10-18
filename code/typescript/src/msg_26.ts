/**
 * msg_26.ts — message 26: multiple slot binary message with communications
 * state (chapter 16).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.24, Table 82 (layout) and Table 83
 * (maximum data bits per flag combination and slot count); Annex 2
 * §3.3.7.2.2 Tables 18/19 (SOTDMA) and §3.3.7.3.2 Table 20 (ITDMA) for the
 * communication state, decoded by the same functions as messages 1–3.
 *
 * Bit layout (zero-based offsets):
 *
 *      0- 5   6  message id (26)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI)
 *     38      1  destination indicator: 0 = broadcast, 1 = addressed
 *     39      1  binary data flag: 0 = unstructured, 1 = application identifier follows
 *     40-69  30  destination MMSI        (only if destination indicator = 1)
 *     70-71   2  spare                   (only if destination indicator = 1)
 *     next   16  application identifier  (only if binary data flag = 1): DAC 10 + FI 6
 *     next    n  application data (Table 83: 104 … 1 000 bits)
 *     next    4  spare "needed for byte alignment" (Table 82)
 *     last 20     communication state: 1 selector bit (0 = SOTDMA, 1 = ITDMA)
 *                 followed by the 19-bit SOTDMA or ITDMA state
 *
 * The communication state is found by counting back 20 bits from the end of
 * the payload; the data length is "whatever is left" and includes the
 * alignment spare, which a receiver cannot tell apart from data.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `export const COMM_STATE_BITS = 20;` — `const` makes the *binding*
 *   immutable, and for a number literal TypeScript narrows the type to the
 *   literal `20`, so it behaves like a true constant.
 * * Imports from sibling modules (`./msg_25`, `./msg_1_2_3`) are how code
 *   is reused; nothing is copied.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";
import { decodeItdma, decodeSotdma } from "./msg_1_2_3";
import { decodeBinaryHeader } from "./msg_25";

/** 1 selector bit + 19 bits of SOTDMA/ITDMA state. */
export const COMM_STATE_BITS = 20;

/** Decode a message 26. Throws DecodeError if header or comm state is missing. */
export function decode(bits: Bits): JsonObject {
  const [out, offset] = decodeBinaryHeader(bits, 26);
  // The communication state is the *last* 20 bits, whatever the data length.
  requireBits(26, bits.length, offset + COMM_STATE_BITS);
  const commStart = bits.length - COMM_STATE_BITS;
  out["data"] = bits.raw(offset, commStart - offset);
  out["data_bits"] = commStart - offset;
  const selector = bits.uint(commStart, 1);
  out["comm_state_selector"] = selector;
  if (selector === 1) {
    out["comm_state"] = decodeItdma(bits, commStart + 1);
  } else {
    out["comm_state"] = decodeSotdma(bits, commStart + 1);
  }
  return out;
}
