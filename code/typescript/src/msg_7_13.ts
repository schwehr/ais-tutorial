/**
 * msg_7_13.ts — messages 7 and 13: binary / safety related acknowledgement.
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.5 (message 7, and by reference §3.11
 * message 13), Table 56. Annex 2 §5.3.1 describes when the acknowledgement
 * is sent (after an addressed message 6 or 12) and what the sequence number
 * means.
 *
 * Bit layout (zero-based offsets; lengths from Table 56):
 *
 *      0- 5   6  message id (7 = binary ack, 13 = safety related ack)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI of the station that acknowledges)
 *     38-39   2  spare
 *     40-69  30  destination id 1 (MMSI whose message is acknowledged)
 *     70-71   2  sequence number of the acknowledged message, 0-3
 *     72-101 30  destination id 2       } "should be omitted if no
 *    102-103  2  sequence number 2      }  destination ID2"
 *    104-135 32  destination id 3 + sequence number 3
 *    136-167 32  destination id 4 + sequence number 4
 *
 * Total 72, 104, 136 or 168 bits. The repeated 32-bit groups are output as
 * a JSON array `acks`, one object per acknowledged station.
 *
 * Length rule: at least 72 bits (one destination) are required; otherwise
 * DecodeError. After the first group we take as many complete 32-bit groups
 * as are present, up to four. Bits left over after the last complete group
 * are counted in `trailing_bits` and never interpreted.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `JsonObject[]` is "array of JsonObject"; `push` appends like Python's
 *   `list.append`. Declared with `const` because the *binding* never
 *   changes — the array contents still can.
 * * `let pos = HEADER_BITS;` uses `let` because `pos` is reassigned in the
 *   loop; everything that is never reassigned is `const`.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { JsonObject } from "./jsonout";

const HEADER_BITS = 40; // message id .. spare
const GROUP_BITS = 32;  // destination id (30) + sequence number (2)
const MAX_GROUPS = 4;

/** Decode a message 7 or 13. Throws DecodeError below 72 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, HEADER_BITS + GROUP_BITS);

  const acks: JsonObject[] = [];
  let pos = HEADER_BITS;
  // Loop over the repeated groups. `bits.has` is the guard that stops us
  // at the end of a 72-, 104-, 136- or 168-bit message.
  while (acks.length < MAX_GROUPS && bits.has(pos, GROUP_BITS)) {
    acks.push({
      mmsi: bits.uint(pos, 30),
      sequence_number: bits.uint(pos + 30, 2),
    });
    pos += GROUP_BITS;
  }

  return {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
    acks,
    trailing_bits: bits.length - pos,
  };
}
