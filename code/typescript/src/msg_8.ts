/**
 * msg_8.ts — message 8: binary broadcast message (chapter 8).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.6, Table 57 (layout), Table 58 (slot
 * budget); Annex 5 §2.1 Table 23 (application identifier).
 *
 * Bit layout (zero-based offsets; lengths from Table 57):
 *
 *      0- 5   6  message id (8)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI)
 *     38-39   2  spare
 *     40-49  10  designated area code (DAC)          } 16-bit application
 *     50-55   6  function identifier (FI)             } identifier, Annex 5 Table 23
 *     56-    ≤952 application data
 *
 * Length rule: at least the 56-bit header is required; otherwise
 * DecodeError. The rest is `data`/`data_bits`, plus `application` when
 * asm.ts knows the (dac, fi).
 */

import { decodeAsm } from "./asm";
import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";

export const HEADER_BITS = 56; // message id .. function identifier

/** Decode a message 8. Throws DecodeError below 56 bits. */
export function decode(bits: Bits): JsonObject {
  requireBits(8, bits.length, HEADER_BITS);
  const dac = bits.uint(40, 10);
  const fi = bits.uint(50, 6);
  const out: JsonObject = {
    msg_type: 8,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
    dac,
    fi,
    data: bits.raw(HEADER_BITS, bits.length - HEADER_BITS),
    data_bits: bits.length - HEADER_BITS,
  };
  const app = decodeAsm(dac, fi, bits, HEADER_BITS);
  if (app !== null) {
    out["application"] = app;
  }
  return out;
}
