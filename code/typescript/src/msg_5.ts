/**
 * msg_5.ts — message 5: ship static and voyage related data (chapter 7).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.3, Table 52 (field layout, 424 bits, two
 * slots); §3.3.1 (DTE); §3.3.2 Table 53 (type of ship and cargo type);
 * §3.3.3 Figure 41 (reference point and dimensions A/B/C/D); Table 47
 * (six-bit ASCII). The EPFD code list is Table 51 (shared via common.ts).
 *
 * Bit layout (zero-based offsets; lengths from Table 52):
 *
 *       0-  5   6  message id (5)
 *       6-  7   2  repeat indicator
 *       8- 37  30  user id (MMSI)
 *      38- 39   2  AIS version indicator (0 = M.1371-1, 1 = -3, 2 = -5, 3 = future)
 *      40- 69  30  IMO number; 0 = n/a; 1..999999 not used;
 *                  1000000..9999999 IMO number; 10000000.. flag-state number
 *      70-111  42  call sign, 7 six-bit characters, "@@@@@@@" = n/a
 *     112-231 120  name, 20 six-bit characters, all '@' = n/a
 *     232-239   8  type of ship and cargo type (Table 53); 0 = n/a
 *     240-248   9  dimension A, metres from reference point to bow;   511 = ≥511 m
 *     249-257   9  dimension B, metres to stern;                      511 = ≥511 m
 *     258-263   6  dimension C, metres to port side;                   63 = ≥63 m
 *     264-269   6  dimension D, metres to starboard side;              63 = ≥63 m
 *     270-273   4  type of electronic position fixing device (EPFD)
 *     274-277   4  ETA month  1..12;  0 = n/a
 *     278-282   5  ETA day    1..31;  0 = n/a
 *     283-287   5  ETA hour   0..23; 24 = n/a
 *     288-293   6  ETA minute 0..59; 60 = n/a
 *     294-301   8  maximum present static draught, 1/10 m; 0 = n/a; 255 = ≥25.5 m
 *     302-421 120  destination, 20 six-bit characters, all '@' = n/a
 *     422       1  DTE: 0 = data terminal available, 1 = not available (default)
 *     423       1  spare
 *
 * Length rule: we require the full 424 bits and ignore anything after bit
 * 423 (426-bit messages from senders that under-count the fill bits decode
 * normally). Shorter messages are reported as `message 5 too short`.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `String(imo).split("").map(Number)` turns 9074729 into [9,0,7,4,7,2,9]
 *   — the JavaScript way to look at decimal digits one at a time.
 * * `String(v).padStart(2, "0")` is the zero-padding that Python writes
 *   as `f"{v:02d}"`.
 * * `reduce` folds an array into one value; here it adds up the weighted
 *   digits. The second argument (0) is the starting total.
 * * Numbers in JavaScript are 64-bit floats, so a 30-bit IMO field is
 *   exact; `%` on them behaves like Python's for non-negative values.
 */

import { Bits } from "./bits";
import { epfdText, requireBits, shipTypeText } from "./common";
import { Fixed, JsonObject } from "./jsonout";

/** Table 52, "AIS version indicator". */
export const AIS_VERSION: { readonly [code: number]: string } = {
  0: "ITU-R M.1371-1",
  1: "ITU-R M.1371-3 or later",
  2: "ITU-R M.1371-5 or later",
  3: "future edition",
};

/** Table 52, "DTE" (data terminal equipment ready), §3.3.1. */
export const DTE_TEXT: { readonly [code: number]: string } = {
  0: "available",
  1: "not available (default)",
};

/** Which of the four ranges of Table 52 the 30-bit IMO field falls in. */
export function imoRangeText(imo: number): string {
  if (imo === 0) {
    return "not available (default)";
  }
  if (imo <= 999999) {
    return "not used";
  }
  if (imo <= 9999999) {
    return "IMO number";
  }
  return "official flag state number";
}

/**
 * True if `imo` is a seven-digit IMO number whose check digit is right.
 *
 * Multiply the first six digits by 7, 6, 5, 4, 3, 2, add the products; the
 * last digit of the sum is the check digit:
 *     9074729:  9*7 + 0*6 + 7*5 + 4*4 + 7*3 + 2*2 = 139  ->  9  ok
 */
export function imoCheckDigitOk(imo: number): boolean {
  if (imo < 1000000 || imo > 9999999) {
    return false;
  }
  const digits = String(imo).split("").map(Number);
  const weights = [7, 6, 5, 4, 3, 2];
  const total = weights.reduce((sum, w, i) => sum + digits[i] * w, 0);
  return total % 10 === digits[6];
}

/** 'MM-DD HH:MM' in UTC with '??' for each part that is not available. */
export function etaText(month: number, day: number, hour: number, minute: number): string {
  const two = (v: number): string => String(v).padStart(2, "0");
  const mm = 1 <= month && month <= 12 ? two(month) : "??";
  const dd = 1 <= day && day <= 31 ? two(day) : "??";
  const hh = hour <= 23 ? two(hour) : "??";
  const mi = minute <= 59 ? two(minute) : "??";
  return `${mm}-${dd} ${hh}:${mi}`;
}

/** Decode a message 5. Throws DecodeError if shorter than 424 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 424);
  const aisVersion = bits.uint(38, 2);
  const imo = bits.uint(40, 30);
  const shipType = bits.uint(232, 8);
  const dimA = bits.uint(240, 9); // to bow
  const dimB = bits.uint(249, 9); // to stern
  const dimC = bits.uint(258, 6); // to port
  const dimD = bits.uint(264, 6); // to starboard
  const epfd = bits.uint(270, 4);
  const month = bits.uint(274, 4);
  const day = bits.uint(278, 5);
  const hour = bits.uint(283, 5);
  const minute = bits.uint(288, 6);
  const draughtRaw = bits.uint(294, 8);
  const dte = bits.uint(422, 1);

  return {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    ais_version: aisVersion,
    ais_version_text: AIS_VERSION[aisVersion],
    imo,
    imo_available: imo !== 0,
    imo_range_text: imoRangeText(imo),
    imo_valid: imoCheckDigitOk(imo),
    call_sign: bits.text(70, 42),
    name: bits.text(112, 120),
    ship_type: shipType,
    ship_type_text: shipTypeText(shipType),
    dim_bow_m: dimA,
    dim_stern_m: dimB,
    dim_port_m: dimC,
    dim_starboard_m: dimD,
    // Figure 41: all four zero = neither reference point nor dimensions;
    // A = C = 0 with B, D non-zero = dimensions but no reference point.
    dimensions_available: !(dimA === 0 && dimB === 0 && dimC === 0 && dimD === 0),
    reference_point_available: !(dimA === 0 && dimC === 0),
    length_m: dimA + dimB,
    beam_m: dimC + dimD,
    epfd,
    epfd_text: epfdText(epfd),
    eta_month: month,
    eta_day: day,
    eta_hour: hour,
    eta_minute: minute,
    eta_text: etaText(month, day, hour, minute),
    eta_available: 1 <= month && month <= 12 && 1 <= day && day <= 31 && hour <= 23 && minute <= 59,
    draught_m: new Fixed(draughtRaw / 10.0, 1),
    draught_available: draughtRaw !== 0,
    destination: bits.text(302, 120),
    dte,
    dte_text: DTE_TEXT[dte],
    spare: bits.uint(423, 1),
  };
}
