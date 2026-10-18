/**
 * msg_21.ts — message 21: aids-to-navigation (AtoN) report (chapter 15).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.19, Table 73 (field layout, 272–360 bits);
 * Table 74 (the 32 AtoN types); Figure 41/41bis and §3.3.3 (dimension /
 * reference-point encoding, shared with message 5); Table 51 (EPFD types,
 * shared with message 4). The eight "AtoN status" bits are only *reserved*
 * by ITU; their page-7 layout (RACON / light / health) comes from IALA
 * Recommendation R0126 (A-126) ed. 2 §4.8.4 Figure 3.
 *
 * Bit layout (zero-based offsets; lengths from Table 73):
 *
 *      0- 5    6  message id (21)
 *      6- 7    2  repeat indicator
 *      8- 37  30  user id (MMSI; AtoN format 99MIDxxxx per ITU-R M.585)
 *     38- 42   5  type of aid to navigation (Table 74); 0 = not available
 *     43-162 120  name, 20 six-bit characters; all '@' = not available
 *    163       1  position accuracy (1 = high, ≤10 m)
 *    164-191  28  longitude, signed, 1/10 000 minute; 181° = n/a
 *    192-218  27  latitude,  signed, 1/10 000 minute;  91° = n/a
 *    219-227   9  dimension A (reference point to "bow" / true north), m
 *    228-236   9  dimension B (to "stern"), m
 *    237-242   6  dimension C (to port), m
 *    243-248   6  dimension D (to starboard), m
 *    249-252   4  type of EPFD (Table 51); 7 = surveyed is normal for fixed/virtual aids
 *    253-258   6  UTC second; 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
 *    259       1  off-position indicator (floating AtoN only; valid only if second <= 59)
 *    260-267   8  AtoN status — "reserved for the indication of the AtoN status"
 *    268       1  RAIM flag
 *    269       1  virtual AtoN flag (1 = does not physically exist)
 *    270       1  assigned mode flag
 *    271       1  spare
 *    272-      0..84  name extension: 0–14 more six-bit characters, then
 *                 0, 2, 4 or 6 spare bits to reach a byte boundary
 *
 * Variable length rule: everything after bit 272 is the name extension. We
 * decode floor((len - 272) / 6) whole characters (at most 14) and report the
 * raw number of extension bits as `name_extension_bits`.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `/` on numbers is always floating-point division in JavaScript;
 *   `Math.floor(extensionBits / 6)` is Python's `extensionBits // 6`.
 * * `Math.min(a, b)` works on plain numbers — no type juggling as in C++
 *   and Rust, because JavaScript has only one number type.
 * * `(status >> 3) & 0x3` is safe: `status` is 8 bits, far below the
 *   32-bit limit of the bitwise operators (see bits.ts).
 * * `const ATON_TYPES: readonly string[]` is an array indexed by the
 *   field value; `readonly` makes `ATON_TYPES[0] = "x"` a compile error.
 */

import { Bits } from "./bits";
import { epfdText, requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";

/** Table 73: the fixed part (two slots). */
export const MIN_BITS = 272;
/** Table 73: "up to 14 additional 6-bit-ASCII characters". */
export const MAX_EXTENSION_CHARS = 14;

/**
 * Type of aid to navigation, Table 74. Codes 5–19 are listed under
 * "Fixed AtoN", 20–31 under "Floating AtoN"; 1–4 are general. Code 4 was
 * "spare" in M.1371-4 and became "Emergency Wreck Marking Buoy" in M.1371-5.
 */
export const ATON_TYPES: readonly string[] = [
  "not specified (default)",
  "reference point",
  "RACON",
  "fixed structure off-shore",
  "emergency wreck marking buoy",
  "light, without sectors",
  "light, with sectors",
  "leading light front",
  "leading light rear",
  "beacon, cardinal N",
  "beacon, cardinal E",
  "beacon, cardinal S",
  "beacon, cardinal W",
  "beacon, port hand",
  "beacon, starboard hand",
  "beacon, preferred channel port hand",
  "beacon, preferred channel starboard hand",
  "beacon, isolated danger",
  "beacon, safe water",
  "beacon, special mark",
  "cardinal mark N",
  "cardinal mark E",
  "cardinal mark S",
  "cardinal mark W",
  "port hand mark",
  "starboard hand mark",
  "preferred channel port hand",
  "preferred channel starboard hand",
  "isolated danger",
  "safe water",
  "special mark",
  "light vessel / LANBY / rig",
];

/** IALA R0126 Figure 3, status page 7 ("111 RR LL H"). */
export const RACON_STATUS: readonly string[] = [
  "no RACON installed",
  "RACON installed but not monitored",
  "RACON operational",
  "RACON error",
];
export const LIGHT_STATUS: readonly string[] = [
  "no light or no monitoring",
  "light on",
  "light off",
  "light fail or at reduced range",
];

export function atonTypeText(code: number): string {
  return ATON_TYPES[code];
}

/**
 * True for the Table 74 codes that describe a floating aid (a buoy).
 * Table 74 heads codes 20–31 "Floating AtoN"; code 4, the emergency wreck
 * marking buoy, is listed above that heading but is a buoy as well.
 */
export function isFloating(atonType: number): boolean {
  return atonType === 4 || (20 <= atonType && atonType <= 31);
}

/**
 * Interpret the 8 AtoN status bits.
 *
 * ITU only reserves them (Table 73). IALA R0126 §4.8.4 splits them into a
 * 3-bit page id (the most significant bits) and 5 page-dependent bits;
 * page 0 is "no data" (the all-zero default) and page 7 carries the
 * RACON / light / health summary that monitoring systems key on.
 */
export function decodeStatus(status: number): JsonObject {
  const page = status >> 5;
  const out: JsonObject = { aton_status_page: page };
  if (page === 7) {
    const racon = (status >> 3) & 0x3;
    const light = (status >> 1) & 0x3;
    out["racon_status"] = racon;
    out["racon_status_text"] = RACON_STATUS[racon];
    out["light_status"] = light;
    out["light_status_text"] = LIGHT_STATUS[light];
    out["health_alarm"] = (status & 0x1) === 1;
  }
  return out;
}

/** Decode a message 21. Throws DecodeError if shorter than 272 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, MIN_BITS);
  const atonType = bits.uint(38, 5);
  const lonRaw = bits.sint(164, 28);
  const latRaw = bits.sint(192, 27);
  const epfd = bits.uint(249, 4);
  const timestamp = bits.uint(253, 6);
  const status = bits.uint(260, 8);

  // --- the variable-length name extension -------------------------------
  // Table 73: "0, 6, 12, … 84" bits of six-bit characters followed by 0, 2,
  // 4 or 6 spare bits so that the message ends on a byte boundary. Only
  // whole characters are decoded; a partial character is ignored.
  const extensionBits = bits.length - MIN_BITS;
  const extensionChars = Math.min(Math.floor(extensionBits / 6), MAX_EXTENSION_CHARS);
  const name = bits.text(43, 120);
  const nameExtension = bits.text(MIN_BITS, extensionChars * 6);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    aton_type: atonType,
    aton_type_text: atonTypeText(atonType),
    name,
    name_extension: nameExtension,
    name_extension_bits: extensionBits,
    full_name: name + nameExtension,
    position_accuracy: bits.uint(163, 1),
    lon: new Fixed(lonRaw / 600000.0, 6),
    lat: new Fixed(latRaw / 600000.0, 6),
    position_available: lonRaw !== 0x6791ac0 && latRaw !== 0x3412140,
    dim_a_m: bits.uint(219, 9),
    dim_b_m: bits.uint(228, 9),
    dim_c_m: bits.uint(237, 6),
    dim_d_m: bits.uint(243, 6),
    epfd,
    epfd_text: epfdText(epfd),
    timestamp,
    off_position: bits.flag(259),
    // Table 73 note 1: the flag means something only for a floating aid
    // whose time stamp is a real second (0–59).
    off_position_valid: isFloating(atonType) && timestamp <= 59,
    aton_status: status,
    aton_status_bits: bits.raw(260, 8),
    raim: bits.flag(268),
    virtual_aton: bits.flag(269),
    assigned_mode: bits.flag(270),
    spare: bits.uint(271, 1),
  };
  Object.assign(out, decodeStatus(status));
  return out;
}
