/**
 * msg_24.ts — message 24: static data report, parts A and B (chapter 14).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.22, Table 78 (part A, 160 bits) and
 * Table 79 (part B, 168 bits), Table 79A (vendor ID sub-fields). Message 24
 * was introduced in M.1371-2 (Annex 7 Tables 60–61), together with the rule
 * that an unregistered daughter vessel puts the mother ship's MMSI in the
 * dimension field; M.1371-4 (Tables 75, 76, 76A) added the vendor-ID split;
 * M.1371-5 dropped the mother-ship sentence and redefined the last six bits
 * of part B as a 4-bit type of EPFD plus 2 spare bits. Dimensions: §3.3.3
 * Fig. 41. Ship type: Table 53. EPFD: Table 51. Auxiliary-craft MMSI format
 * 98MIDxxxx: ITU-R M.585 Annex 1.
 *
 * Bit layout (zero-based offsets):
 *
 *   common   0- 5   6  message id (24)
 *            6- 7   2  repeat indicator
 *            8-37  30  user id (MMSI)
 *           38-39   2  part number: 0 = part A, 1 = part B, 2-3 not allowed
 *   part A  40-159 120 name, 20 six-bit characters; all '@' = n/a
 *          160-167  8  (spare) — present only on units that send 168 bits
 *   part B  40- 47  8  type of ship and cargo type; 0 = n/a
 *           48- 89 42  vendor ID; since M.1371-4 split into
 *                         48-65 18  manufacturer's ID, 3 six-bit characters
 *                         66-69  4  unit model code
 *                         70-89 20  unit serial number
 *           90-131 42  call sign, 7 six-bit characters; all '@' = n/a
 *          132-161 30  dimensions / reference point (A 9, B 9, C 6, D 6), or
 *                      for an auxiliary craft (MMSI 98MIDxxxx) the MMSI of the
 *                      mother ship (M.1371-2 to -4 rule)
 *          162-165  4  type of EPFD (Table 79; spare in M.1371-2 to -4)
 *          166-167  2  spare
 *
 * EPFD on old units: a unit built to M.1371-2/-3/-4 sends six spare bits,
 * which are zero, so it decodes as `epfd: 0` = "undefined" — the same value
 * a -5 unit sends when it does not know its fix type. `epfd: 0` therefore
 * says nothing about the edition the unit was built to; a non-zero `epfd`
 * is only possible on a -5 unit (or a bit error).
 *
 * Length rule: part A needs 160 bits; anything beyond 160 is reported as
 * `spare` (at most 8 bits are decoded) with its width in `spare_bits`.
 * Part B needs 168 bits. Part numbers 2 and 3 return the remainder of the
 * payload undecoded as `data`/`data_bits`.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `980_000_000` — numeric separators are allowed in JavaScript too.
 * * `Math.min(spareBits, 8)` — there is no integer type; `number` is a
 *   double, which is exact for every value we meet here.
 */

import { Bits } from "./bits";
import { epfdText, requireBits, shipTypeText } from "./common";
import { JsonObject } from "./jsonout";
import { decodeDimensions } from "./msg_19";

export const PART_TEXT = ["part A", "part B", "reserved", "reserved"];

/** True for MMSIs of the form 98MIDxxxx (craft associated with a parent ship). */
export function isAuxiliaryCraft(mmsi: number): boolean {
  return 980_000_000 <= mmsi && mmsi < 990_000_000;
}

/** Decode a message 24 part A or part B. Throws DecodeError if too short. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 40);
  const mmsi = bits.uint(8, 30);
  const part = bits.uint(38, 2);
  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi,
    part_number: part,
    part_text: PART_TEXT[part],
  };

  if (part === 0) {
    requireBits(msgType, bits.length, 160);
    out["name"] = bits.text(40, 120);
    const spareBits = bits.length - 160;
    out["spare_bits"] = spareBits;
    out["spare"] = spareBits > 0 ? bits.uint(160, Math.min(spareBits, 8)) : 0;
  } else if (part === 1) {
    requireBits(msgType, bits.length, 168);
    const shipType = bits.uint(40, 8);
    const epfd = bits.uint(162, 4);
    Object.assign(out, {
      ship_type: shipType,
      ship_type_text: shipTypeText(shipType),
      vendor_id: bits.text(48, 42),
      manufacturer_id: bits.text(48, 18),
      unit_model_code: bits.uint(66, 4),
      unit_serial_number: bits.uint(70, 20),
      call_sign: bits.text(90, 42),
      auxiliary_craft: isAuxiliaryCraft(mmsi),
      epfd,
      epfd_text: epfdText(epfd),
      spare: bits.uint(166, 2),
    });
    if (isAuxiliaryCraft(mmsi)) {
      out["mothership_mmsi"] = bits.uint(132, 30);
    } else {
      Object.assign(out, decodeDimensions(bits, 132));
    }
  } else {
    out["data"] = bits.raw(40, bits.length - 40);
    out["data_bits"] = bits.length - 40;
  }
  return out;
}
