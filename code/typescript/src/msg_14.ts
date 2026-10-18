/**
 * msg_14.ts — message 14: safety related broadcast message.
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.12, Table 63 (layout), Table 64 (how
 * many characters fit in 1–5 slots), Table 47 (six-bit ASCII). Annex 1
 * §2.1.6–2.1.8 prescribe the fixed texts sent by AIS-SART, MOB-AIS and
 * EPIRB-AIS devices ("SART ACTIVE", "SART TEST", "MOB ACTIVE", "MOB TEST",
 * "EPIRB ACTIVE", "EPIRB TEST"); §3.12 repeats the list.
 *
 * Bit layout (zero-based offsets; lengths from Table 63):
 *
 *      0- 5   6  message id (14)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI)
 *     38-39   2  spare
 *     40-     ≤968  safety related text, six-bit ASCII, 0-161 characters
 *
 * Length rule: the 40-bit header is required (DecodeError otherwise); the
 * text handling is shared with message 12 (decodeTextField in msg_12.ts).
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `BEACON_TEXTS` is a plain object used as a string-keyed table. Looking
 *   up a key that is not there yields `undefined`, not an error; `?? null`
 *   (the nullish-coalescing operator) turns that into the JSON null that the
 *   other three languages print.
 * * `out["text"] as string` is a *type assertion*: the JsonObject index
 *   type says "some JsonValue", and we tell the compiler we know it is a
 *   string. Python just did `out["text"]`.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";
import { decodeTextField } from "./msg_12";

const HEADER_BITS = 40;

/**
 * The six texts that distress beacons are required to send (Annex 1
 * §2.1.6-2.1.8). A watch-keeping application matches on these.
 */
export const BEACON_TEXTS: { readonly [text: string]: string } = {
  "SART ACTIVE": "AIS-SART active",
  "SART TEST": "AIS-SART test",
  "MOB ACTIVE": "MOB-AIS active",
  "MOB TEST": "MOB-AIS test",
  "EPIRB ACTIVE": "EPIRB-AIS active",
  "EPIRB TEST": "EPIRB-AIS test",
};

/** Decode a message 14. Throws DecodeError below 40 bits. */
export function decode(bits: Bits): JsonObject {
  requireBits(14, bits.length, HEADER_BITS);
  const out: JsonObject = {
    msg_type: bits.uint(0, 6),
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
  };
  Object.assign(out, decodeTextField(bits, HEADER_BITS));
  out["beacon_text"] = BEACON_TEXTS[out["text"] as string] ?? null;
  return out;
}
