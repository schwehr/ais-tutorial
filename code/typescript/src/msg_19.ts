/**
 * msg_19.ts — message 19: extended Class B equipment position report (chapter 14).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.17, Table 71 (312 bits, two slots; Table
 * 68 in M.1371-4). Dimension/reference-point field: §3.3.3 Fig. 41. Ship
 * type: §3.3.2 Table 53. EPFD: Table 51. DTE: §3.3.1.
 *
 * M.1371-5 says this message "is not needed and should not be used" by new
 * equipment — a Class B unit should send message 18 plus message 24 A/B
 * instead — but legacy units still send it once every 6 minutes.
 *
 * Bit layout (zero-based offsets; lengths from Table 71). Bits 0-138 are
 * laid out exactly as in message 18, so we reuse that module's helper.
 *
 *      0- 5   6  message id (19)
 *      6- 7   2  repeat indicator
 *      8-37  30  user id (MMSI)
 *     38-45   8  spare
 *     46-55  10  speed over ground, 1/10 knot; 1023 = n/a
 *     56      1  position accuracy
 *     57-84  28  longitude, signed, 1/10 000 minute; 181° = n/a
 *     85-111 27  latitude,  signed, 1/10 000 minute;  91° = n/a
 *    112-123 12  course over ground, 1/10 degree; 3600 = n/a
 *    124-132  9  true heading; 511 = n/a
 *    133-138  6  time stamp (UTC second); 60 = n/a
 *    139-142  4  spare
 *    143-262 120 name, 20 six-bit characters; all '@' = n/a
 *    263-270  8  type of ship and cargo type; 0 = n/a
 *    271-300 30  dimensions / reference point: A 9, B 9, C 6, D 6 (Fig. 41)
 *    301-304  4  type of EPFD; 0 = undefined
 *    305      1  RAIM flag
 *    306      1  DTE: 0 = data terminal available, 1 = not available (default)
 *    307      1  assigned mode flag
 *    308-311  4  spare
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `decodeDimensions` is exported for msg_24.ts. A module's exports form
 *   its public surface; everything else stays file-private.
 */

import { Bits } from "./bits";
import { epfdText, requireBits, shipTypeText } from "./common";
import { JsonObject } from "./jsonout";
import { ASSIGNED_MODE, decodeCommonPosition } from "./msg_18";

export const DTE = ["data terminal available", "data terminal not available"];

/**
 * The 30-bit dimension / reference-point field (M.1371-5 §3.3.3 Fig. 41).
 *
 * A = distance from the reference point (the GNSS antenna) to the bow,
 * B = to the stern, C = to the port side, D = to the starboard side, all in
 * metres. A and B saturate at 511, C and D at 63. All four zero means
 * "nothing available"; A = C = 0 with B and D non-zero means "dimensions
 * known but reference point not". Keys and availability rules are exactly
 * those of message 5 (msg_5.ts); message 21 reuses this helper as well.
 */
export function decodeDimensions(bits: Bits, start: number): JsonObject {
  const a = bits.uint(start, 9);
  const b = bits.uint(start + 9, 9);
  const c = bits.uint(start + 18, 6);
  const d = bits.uint(start + 24, 6);
  return {
    dim_bow_m: a,
    dim_stern_m: b,
    dim_port_m: c,
    dim_starboard_m: d,
    // Figure 41: all four zero = neither reference point nor dimensions;
    // A = C = 0 with B, D non-zero = dimensions but no reference point.
    dimensions_available: !(a === 0 && b === 0 && c === 0 && d === 0),
    reference_point_available: !(a === 0 && c === 0),
    length_m: a + b,
    beam_m: c + d,
  };
}

/** Decode a message 19. Throws DecodeError if shorter than 312 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 312);
  const shipType = bits.uint(263, 8);
  const epfd = bits.uint(301, 4);
  const dte = bits.uint(306, 1);
  const assigned = bits.uint(307, 1);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 8),
  };
  Object.assign(out, decodeCommonPosition(bits, 46));
  Object.assign(out, {
    spare2: bits.uint(139, 4),
    name: bits.text(143, 120),
    ship_type: shipType,
    ship_type_text: shipTypeText(shipType),
  });
  Object.assign(out, decodeDimensions(bits, 271));
  Object.assign(out, {
    epfd,
    epfd_text: epfdText(epfd),
    raim: bits.flag(305),
    dte,
    dte_text: DTE[dte],
    assigned_mode: assigned,
    assigned_mode_text: ASSIGNED_MODE[assigned],
    spare3: bits.uint(308, 4),
  });
  return out;
}
