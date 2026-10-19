/**
 * msg_18.ts — message 18: standard Class B equipment position report (chapter 14).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.16, Table 70 (168 bits; Table 67 in
 * M.1371-3 and -4). Communication states: Annex 2 §3.3.7.2.2 Table 18/19 (SOTDMA)
 * and §3.3.7.3.2 Table 20 (ITDMA). Class B "CS" behaviour: Annex 7.
 *
 * Bit layout (zero-based offsets; lengths from Table 70):
 *
 *      0- 5   6  message id (18)
 *      6- 7   2  repeat indicator (should be 0 for "CS" units)
 *      8-37  30  user id (MMSI)
 *     38-45   8  spare ("regional reserved" in older editions); should be 0
 *     46-55  10  speed over ground, 1/10 knot; 1023 = n/a, 1022 = ≥102.2 kn
 *     56      1  position accuracy (1 = high, ≤10 m)
 *     57-84  28  longitude, signed, 1/10 000 minute; 181° = n/a
 *     85-111 27  latitude,  signed, 1/10 000 minute;  91° = n/a
 *    112-123 12  course over ground, 1/10 degree; 3600 = n/a
 *    124-132  9  true heading, degrees; 511 = n/a
 *    133-138  6  time stamp (UTC second); 60 = n/a; 61-63 not used by "CS"
 *    139-140  2  spare (regional reserved in older editions); should be 0
 *    141      1  Class B unit flag: 0 = SOTDMA ("SO") unit, 1 = "CS" unit
 *    142      1  Class B display flag: 1 = can display messages 12 and 14
 *    143      1  Class B DSC flag: 1 = has DSC function
 *    144      1  Class B band flag: 1 = whole marine band, 0 = upper 525 kHz only
 *    145      1  Class B message 22 flag: 1 = accepts channel management
 *    146      1  assigned mode flag: 1 = assigned mode
 *    147      1  RAIM flag
 *    148      1  communication state selector: 0 = SOTDMA follows, 1 = ITDMA follows
 *    149-167 19  communication state; a "CS" unit fills it with 1100000000000000110
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * One-bit enumerations are plain two-element arrays indexed by the bit:
 *   `CLASS_B_UNIT[unit]`. TypeScript infers `string[]`; `as const` would
 *   make it a readonly tuple, but a plain array is enough here.
 * * `decodeCommonPosition` returns an object that the caller merges with
 *   `Object.assign`; msg_19.ts imports it, so it is `export`ed.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";
import { decodeItdma, decodeSotdma } from "./msg_1_2_3";

/** Table 70: the 19-bit value every Class B "CS" unit puts in its communication state field. */
export const CS_FIXED_COMM_STATE = "1100000000000000110";

export const CLASS_B_UNIT = ["Class B SOTDMA unit", "Class B CS unit"];
export const CLASS_B_DISPLAY = [
  "no display for messages 12 and 14", "integrated display for messages 12 and 14",
];
export const CLASS_B_DSC = ["no DSC function", "DSC function (dedicated or time-shared)"];
export const CLASS_B_BAND = ["upper 525 kHz of the marine band only", "whole marine band"];
export const CLASS_B_MSG22 = [
  "AIS 1 and AIS 2 only, no message 22 frequency management",
  "frequency management via message 22",
];
export const ASSIGNED_MODE = ["autonomous and continuous", "assigned"];

/**
 * SOG…time stamp as laid out in messages 18 and 19.
 *
 * Both messages share the block that starts with the 10-bit SOG at `start`
 * (46 in both cases) and ends with the 6-bit time stamp, so the same code
 * reads both. Offsets below are relative to `start`.
 */
export function decodeCommonPosition(bits: Bits, start: number): JsonObject {
  const sogRaw = bits.uint(start, 10);
  const lonRaw = bits.sint(start + 11, 28);
  const latRaw = bits.sint(start + 39, 27);
  const cogRaw = bits.uint(start + 66, 12);
  const heading = bits.uint(start + 78, 9);
  const timestamp = bits.uint(start + 87, 6);
  return {
    sog_knots: new Fixed(sogRaw / 10.0, 1),
    sog_available: sogRaw !== 1023,
    position_accuracy: bits.uint(start + 10, 1),
    lon: new Fixed(lonRaw / 600000.0, 6),
    lat: new Fixed(latRaw / 600000.0, 6),
    position_available: lonRaw !== 0x6791ac0 && latRaw !== 0x3412140,
    cog_deg: new Fixed(cogRaw / 10.0, 1),
    cog_available: cogRaw !== 3600,
    true_heading: heading,
    heading_available: heading !== 511,
    timestamp,
  };
}

/** Decode a message 18. Throws DecodeError if shorter than 168 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 168);
  const unit = bits.uint(141, 1);
  const display = bits.uint(142, 1);
  const dsc = bits.uint(143, 1);
  const band = bits.uint(144, 1);
  const msg22 = bits.uint(145, 1);
  const assigned = bits.uint(146, 1);
  const selector = bits.uint(148, 1);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 8),
  };
  Object.assign(out, decodeCommonPosition(bits, 46));
  Object.assign(out, {
    spare2: bits.uint(139, 2),
    class_b_unit: unit,
    class_b_unit_text: CLASS_B_UNIT[unit],
    class_b_display: display,
    class_b_display_text: CLASS_B_DISPLAY[display],
    class_b_dsc: dsc,
    class_b_dsc_text: CLASS_B_DSC[dsc],
    class_b_band: band,
    class_b_band_text: CLASS_B_BAND[band],
    class_b_msg22: msg22,
    class_b_msg22_text: CLASS_B_MSG22[msg22],
    assigned_mode: assigned,
    assigned_mode_text: ASSIGNED_MODE[assigned],
    raim: bits.flag(147),
    comm_state_selector: selector,
    cs_fixed_comm_state: bits.raw(149, 19) === CS_FIXED_COMM_STATE,
  });
  if (selector === 1) {
    out["comm_state"] = decodeItdma(bits, 149);
  } else {
    out["comm_state"] = decodeSotdma(bits, 149);
  }
  return out;
}
