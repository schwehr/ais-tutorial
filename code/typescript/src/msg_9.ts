/**
 * msg_9.ts — message 9: standard search and rescue (SAR) aircraft position report.
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.7, Table 59 (168 bits); the 19-bit
 * communication state is SOTDMA (Annex 2 §3.3.7.2.2 Tables 18/19) or ITDMA
 * (Annex 2 §3.3.7.3.2 Table 20) according to the selector bit, exactly as
 * in message 18 (chapter 14), so we reuse `decodeSotdma` / `decodeItdma`
 * from msg_1_2_3.ts.
 *
 * Bit layout (zero-based offsets; lengths from Table 59):
 *
 *      0- 5   6  message id (9)
 *      6- 7   2  repeat indicator
 *      8-37  30  user id (MMSI; SAR aircraft use 111MIDxxx, ITU-R M.585-9 Annex 1 §3)
 *     38-49  12  altitude in metres; 4095 = n/a, 4094 = 4094 m or higher
 *     50-59  10  speed over ground in whole knots; 1023 = n/a, 1022 = 1022 kn or higher
 *     60      1  position accuracy (1 = high, <=10 m)
 *     61-88  28  longitude, signed, 1/10 000 minute; 181° (0x6791AC0) = n/a
 *     89-115 27  latitude,  signed, 1/10 000 minute;  91° (0x3412140) = n/a
 *    116-127 12  course over ground, 1/10 degree; 3600 = n/a
 *    128-133  6  time stamp (UTC second); 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
 *    134      1  altitude sensor (0 = GNSS, 1 = barometric)
 *    135-141  7  spare
 *    142      1  DTE (0 = data terminal available, 1 = not available = default)
 *    143-145  3  spare
 *    146      1  assigned mode flag (0 = autonomous, 1 = assigned)
 *    147      1  RAIM flag
 *    148      1  communication state selector (0 = SOTDMA, 1 = ITDMA)
 *    149-167 19  communication state
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * JavaScript has a single `number` type, so "integer SOG" versus
 *   "SOG in tenths" is purely a matter of what we hand to the JSON writer:
 *   a bare `number` that is integral prints as `150`; a `new Fixed(x, 1)`
 *   prints `15.0`. Here SOG is whole knots, so no `Fixed`.
 * * `const ALTITUDE_SENSOR = ["GNSS", "barometric"]` indexed by a 0/1
 *   field is the same trick as the Class B flag tables in msg_18.ts.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";
import { decodeItdma, decodeSotdma } from "./msg_1_2_3";

export const ALTITUDE_NOT_AVAILABLE = 4095;
export const ALTITUDE_MAX = 4094; // "4 094 m or higher"
export const SOG_NOT_AVAILABLE = 1023;
export const SOG_MAX = 1022; // "1 022 knots or higher"

export const ALTITUDE_SENSOR = ["GNSS", "barometric"];
export const ASSIGNED_MODE = ["autonomous and continuous", "assigned"];

/** Decode a message 9. Throws DecodeError if shorter than 168 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 168);
  const altitude = bits.uint(38, 12);
  const sog = bits.uint(50, 10);
  const lonRaw = bits.sint(61, 28);
  const latRaw = bits.sint(89, 27);
  const cogRaw = bits.uint(116, 12);
  const sensor = bits.uint(134, 1);
  const assigned = bits.uint(146, 1);
  const selector = bits.uint(148, 1);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    altitude_m: altitude,
    altitude_available: altitude !== ALTITUDE_NOT_AVAILABLE,
    altitude_at_max: altitude === ALTITUDE_MAX,
    sog_knots: sog, // whole knots: an integer, not Fixed
    sog_available: sog !== SOG_NOT_AVAILABLE,
    sog_at_max: sog === SOG_MAX,
    position_accuracy: bits.uint(60, 1),
    lon: new Fixed(lonRaw / 600000.0, 6),
    lat: new Fixed(latRaw / 600000.0, 6),
    position_available: lonRaw !== 0x6791ac0 && latRaw !== 0x3412140,
    cog_deg: new Fixed(cogRaw / 10.0, 1),
    cog_available: cogRaw !== 3600,
    timestamp: bits.uint(128, 6),
    altitude_sensor: sensor,
    altitude_sensor_text: ALTITUDE_SENSOR[sensor],
    spare: bits.uint(135, 7),
    dte: bits.uint(142, 1),
    spare2: bits.uint(143, 3),
    assigned_mode: assigned,
    assigned_mode_text: ASSIGNED_MODE[assigned],
    raim: bits.flag(147),
    comm_state_selector: selector,
  };
  if (selector === 1) {
    out["comm_state"] = decodeItdma(bits, 149);
  } else {
    out["comm_state"] = decodeSotdma(bits, 149);
  }
  return out;
}
