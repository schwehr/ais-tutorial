/**
 * msg_27.ts — message 27: long-range AIS broadcast message (chapter 16).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.25, Table 84 (96 bits); Annex 4 §3
 * (long-range applications by broadcast: channels 75/76, 3-minute interval,
 * Table 22 packet structure); Table 51 (message 4 "transmission control for
 * long-range broadcast message" bit).
 *
 * Bit layout (zero-based offsets; lengths from Table 84):
 *
 *      0- 5   6  message id (27)
 *      6- 7   2  repeat indicator ("always 3": do not repeat)
 *      8-37  30  user id (MMSI)
 *     38      1  position accuracy (as message 1)
 *     39      1  RAIM flag (as message 1)
 *     40-43   4  navigational status (as message 1)
 *     44-61  18  longitude, signed, 1/10 minute; 181° (0x1A838) = n/a
 *     62-78  17  latitude,  signed, 1/10 minute;  91° (0xD548)  = n/a
 *     79-84   6  speed over ground, whole knots 0–62; 63 = n/a
 *     85-93   9  course over ground, whole degrees 0–359; 511 = n/a
 *     94      1  position latency: 0 = < 5 s, 1 = > 5 s (default)
 *                (M.1371-4 called this bit "status of current GNSS position")
 *     95      1  spare
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * JavaScript has one number type, so there is no signed/unsigned
 *   distinction to worry about when comparing `lonRaw` with the sentinel
 *   `0x1a838`; the price is that `sint` had to be written with `2 ** n`
 *   rather than shifts (see bits.ts).
 * * `sog_knots` is a plain integer here (whole knots, Table 84), not a
 *   `Fixed` — the JSON prints `12`, not `12.0`.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";
import { NAV_STATUS } from "./msg_1_2_3";

export const LON_NOT_AVAILABLE = 0x1a838; // 181 degrees * 600
export const LAT_NOT_AVAILABLE = 0xd548; //  91 degrees * 600

export const POSITION_LATENCY: { readonly [code: number]: string } = {
  0: "less than 5 s",
  1: "greater than 5 s (default)",
};

/**
 * Decode a message 27. Throws DecodeError if shorter than 96 bits.
 * Payloads longer than 96 bits are decoded from their first 96 bits; the
 * driver's `payload_bits` tells the caller that something was odd.
 */
export function decode(bits: Bits): JsonObject {
  requireBits(27, bits.length, 96);
  const nav = bits.uint(40, 4);
  const lonRaw = bits.sint(44, 18);
  const latRaw = bits.sint(62, 17);
  const sog = bits.uint(79, 6);
  const cog = bits.uint(85, 9);
  const latency = bits.uint(94, 1);
  return {
    msg_type: 27,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    position_accuracy: bits.uint(38, 1),
    raim: bits.flag(39),
    nav_status: nav,
    nav_status_text: NAV_STATUS[nav],
    lon: new Fixed(lonRaw / 600.0, 6),
    lat: new Fixed(latRaw / 600.0, 6),
    position_available: lonRaw !== LON_NOT_AVAILABLE && latRaw !== LAT_NOT_AVAILABLE,
    sog_knots: sog,
    sog_available: sog !== 63,
    cog_deg: cog,
    cog_available: cog !== 511,
    position_latency: latency,
    position_latency_text: POSITION_LATENCY[latency],
    spare: bits.uint(95, 1),
  };
}
