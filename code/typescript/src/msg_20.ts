/**
 * msg_20.ts — message 20: data link management message (chapter 13).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.18, Table 72. A base station announces the
 * slots it has reserved for itself (FATDMA — fixed access TDMA) so that
 * mobile stations within 120 NM keep out of them (Annex 2 §3.3.4.3).
 *
 * Bit layout (zero-based offsets; lengths from Table 72):
 *
 *      0- 5   6  message id (20)
 *      6- 7   2  repeat indicator
 *      8-37  30  source station id (MMSI of the base station)
 *     38-39   2  spare
 *     40-69  30  reservation 1:  offset number (12), number of slots (4),
 *                                time-out in minutes (3), increment (11)
 *     70-99  30  reservation 2 (optional)
 *    100-129 30  reservation 3 (optional)
 *    130-159 30  reservation 4 (optional)
 *    then 0, 2, 4 or 6 spare bits to reach a byte boundary; 72–160 bits total.
 *
 * Length rule: at least 72 bits (else DecodeError). Reservation groups are
 * read while 30 bits remain (at most four); whatever is left over is
 * reported as `spare2` with its width in `spare2_bits` (null if wider than
 * 32 bits — that is a broken message, not a spare field).
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `let spare2: number | null = null;` declares a variable whose type is a
 *   union; later assignments must be one of the two.
 * * `bits.uint(pos, spareBits)` with up to 32 bits is safe: `uint` builds
 *   its result by multiplication, not with the 32-bit `<<` operator
 *   (chapter 2).
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";

export const RESERVATION_BITS = 30;
export const MAX_RESERVATIONS = 4;

/** One offset/slots/time-out/increment group (Table 72). */
export function decodeReservation(bits: Bits, start: number): JsonObject {
  const offset = bits.uint(start, 12);
  const slots = bits.uint(start + 12, 4);
  const timeout = bits.uint(start + 16, 3);
  const increment = bits.uint(start + 19, 11);
  return {
    offset,
    slots,
    timeout_min: timeout,
    increment,
    // Table 72 footnote: an interrogated base station with nothing to
    // announce sends one group with offset, slots and time-out all zero.
    available: !(offset === 0 && slots === 0 && timeout === 0),
  };
}

/** Decode a message 20. Throws DecodeError below 72 bits. */
export function decode(bits: Bits): JsonObject {
  requireBits(20, bits.length, 72);
  const reservations: JsonObject[] = [];
  let pos = 40;
  while (reservations.length < MAX_RESERVATIONS && bits.has(pos, RESERVATION_BITS)) {
    reservations.push(decodeReservation(bits, pos));
    pos += RESERVATION_BITS;
  }
  const spareBits = bits.length - pos;
  // The spec allows 0, 2, 4 or 6 trailing bits. We print whatever is there
  // as an integer so that odd lengths show up; anything wider than 32 bits
  // is not a spare field but a broken message and is reported as null.
  let spare2: number | null = null;
  if (spareBits === 0) {
    spare2 = 0;
  } else if (spareBits <= 32) {
    spare2 = bits.uint(pos, spareBits);
  }
  return {
    msg_type: bits.uint(0, 6),
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
    reservations,
    spare2,
    spare2_bits: spareBits,
  };
}
