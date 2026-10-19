/**
 * msg_16.ts — message 16: assignment mode command.
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.14, Table 67 (96 or 144 bits) and the
 * note (1) and paragraph below it that give the meaning of offset and
 * increment. Annex 2 §3.3.6 describes what the assigned station does.
 *
 * Bit layout (zero-based offsets; lengths from Table 67):
 *
 *      0- 5   6  message id (16)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI of the assigning base station)
 *     38-39   2  spare
 *     40-69  30  destination id A
 *     70-81  12  offset A
 *     82-91  10  increment A
 *     ------- then EITHER
 *     92-95   4  spare (padding to a byte boundary)   -> 96 bits
 *     ------- OR
 *     92-121 30  destination id B
 *    122-133 12  offset B
 *    134-143 10  increment B                          -> 144 bits
 *
 * Meaning of (offset, increment) [Table 67 note 1]: increment 0 = reporting
 * rate assignment, offset = reports per 10 min; increment 1-6 = slot
 * assignment with 1125/375/225/125/75/45 slots between transmissions;
 * 7 = undefined (disregard); 8-1023 not defined in M.1371-5.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `number | null` is a union type: the variable holds a number or the
 *   JSON `null`. Under `strict` the compiler will not let you forget the
 *   `null` case when you later use the value.
 * * `SLOT_INCREMENTS[increment] ?? null` turns a missing table entry
 *   (`undefined`) into `null`, which is what the JSON writer prints.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";

/** Table 67 text under note (1): increment code -> slots between assigned slots. */
export const SLOT_INCREMENTS: { readonly [code: number]: number } = {
  1: 1125, 2: 375, 3: 225, 4: 125, 5: 75, 6: 45,
};

/** Interpret one (destination, offset, increment) triple. */
function assignment(mmsi: number, offset: number, increment: number): JsonObject {
  let reports: number | null = null;
  const slots: number | null = SLOT_INCREMENTS[increment] ?? null;
  let text: string;
  if (increment === 0) {
    text = "reporting rate";
    reports = offset;
  } else if (slots !== null) {
    text = "slot assignment";
  } else if (increment === 7) {
    text = "undefined increment (disregard)";
  } else {
    text = "increment not defined in M.1371-5";
  }
  return {
    mmsi,
    offset,
    increment,
    assignment_text: text,
    reports_per_10_min: reports,
    slot_increment: slots,
  };
}

/** Decode a message 16. Throws DecodeError below 96 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 96);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
  };
  const assignments: JsonObject[] = [assignment(bits.uint(40, 30), bits.uint(70, 12), bits.uint(82, 10))];
  let pos: number;
  if (bits.has(92, 52)) {
    assignments.push(assignment(bits.uint(92, 30), bits.uint(122, 12), bits.uint(134, 10)));
    pos = 144;
  } else {
    out["spare2"] = bits.uint(92, 4);
    pos = 96;
  }
  out["assignments"] = assignments;
  out["trailing_bits"] = bits.length - pos;
  return out;
}
