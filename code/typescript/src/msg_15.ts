/**
 * msg_15.ts — message 15: interrogation.
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.13, Table 66 (88-160 bits) and Table 65
 * (which station class may ask which other class for which messages).
 * Annex 2 §3.3.2.3 ("polled" mode) and §4.3.3.3.3 describe the response:
 * automatic, on the channel where the interrogation was received.
 *
 * Bit layout (zero-based offsets; lengths from Table 66):
 *
 *      0- 5   6  message id (15)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI of the interrogating station)
 *     38-39   2  spare
 *     40-69  30  destination id 1 (first interrogated station)
 *     70-75   6  message id 1.1 (first message type requested from station 1)
 *     76-87  12  slot offset 1.1 (0 = let the responder choose a slot)
 *     ------- 88 bits: one station, one message -------
 *     88-89   2  spare
 *     90-95   6  message id 1.2 (second message type requested from station 1)
 *     96-107 12  slot offset 1.2
 *     108-109 2  spare
 *     ------- 110 bits as written in Table 66; 112 when padded to a byte -------
 *     110-139 30 destination id 2 (second interrogated station)
 *     140-145 6  message id 2.1
 *     146-157 12 slot offset 2.1
 *     158-159 2  spare
 *     ------- 160 bits: two stations -------
 *
 * Output and length rules are those of the Python module (msg_15.py):
 * `interrogations` is an array of {mmsi, msg_type, msg_type_text,
 * slot_offset}; the 1.2 group is included only when present and its
 * message id is non-zero; spare fields appear as spare/spare2/spare3/spare4
 * when present; `trailing_bits` counts what is left (2 for the 112-bit
 * form).
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `MESSAGE_NAMES[id] ?? "undefined"` — the nullish-coalescing operator
 *   `??` supplies a default when the lookup yields `undefined`, like
 *   Python's `dict.get(key, default)`.
 * * The helper `request()` returns a `JsonObject`; pushing it into a
 *   `JsonObject[]` and assigning that array to a property is all it takes
 *   to produce a JSON array of objects.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";

/**
 * Table 46 message names, used for the `_text` companion of a requested
 * message type. The names are abbreviated from the "Name" column.
 */
export const MESSAGE_NAMES: { readonly [code: number]: string } = {
  0: "none",
  1: "position report",
  2: "position report (assigned)",
  3: "position report (interrogated)",
  4: "base station report",
  5: "static and voyage related data",
  6: "binary addressed message",
  7: "binary acknowledgement",
  8: "binary broadcast message",
  9: "standard SAR aircraft position report",
  10: "UTC/date inquiry",
  11: "UTC/date response",
  12: "addressed safety related message",
  13: "safety related acknowledgement",
  14: "safety related broadcast message",
  15: "interrogation",
  16: "assignment mode command",
  17: "DGNSS broadcast binary message",
  18: "standard Class B equipment position report",
  19: "extended Class B equipment position report",
  20: "data link management message",
  21: "aids-to-navigation report",
  22: "channel management",
  23: "group assignment command",
  24: "static data report",
  25: "single slot binary message",
  26: "multiple slot binary message with communications state",
  27: "position report for long-range applications",
};

export function messageName(msgId: number): string {
  return MESSAGE_NAMES[msgId] ?? "undefined";
}

/** One (station, message, slot) triple of the `interrogations` array. */
function request(mmsi: number, msgId: number, slotOffset: number): JsonObject {
  return {
    mmsi,
    msg_type: msgId,
    msg_type_text: messageName(msgId),
    slot_offset: slotOffset,
  };
}

/** Decode a message 15. Throws DecodeError below 88 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 88);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
  };
  const mmsi1 = bits.uint(40, 30);
  const interrogations: JsonObject[] = [request(mmsi1, bits.uint(70, 6), bits.uint(76, 12))];
  let pos = 88;

  // Second request to the first station (bits 88-107, plus its spare).
  if (bits.has(88, 20)) {
    out["spare2"] = bits.uint(88, 2);
    const msgId12 = bits.uint(90, 6);
    if (msgId12 !== 0) {
      interrogations.push(request(mmsi1, msgId12, bits.uint(96, 12)));
    }
    pos = 108;
  }

  // A second station (bits 108-157, plus the trailing spare).
  if (bits.has(108, 50)) {
    out["spare3"] = bits.uint(108, 2);
    interrogations.push(request(bits.uint(110, 30), bits.uint(140, 6), bits.uint(146, 12)));
    pos = 158;
    if (bits.has(158, 2)) {
      out["spare4"] = bits.uint(158, 2);
      pos = 160;
    }
  } else if (bits.has(108, 2)) {
    // The 110/112-bit form: the spare after group 1.2, then 0 or 2
    // padding bits.
    out["spare3"] = bits.uint(108, 2);
    pos = 110;
  }

  out["interrogations"] = interrogations;
  out["trailing_bits"] = bits.length - pos;
  return out;
}
