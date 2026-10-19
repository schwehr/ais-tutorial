/**
 * msg_23.ts — message 23: group assignment command (chapter 13).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.21, Table 76 (160 bits) and Table 77
 * (reporting-interval codes). A base station acting as controlling entity
 * selects mobile stations by region, station type and ship type, and
 * commands their Tx/Rx mode, reporting interval and a quiet time.
 *
 * Bit layout (zero-based offsets; lengths from Table 76):
 *
 *      0- 5   6  message id (23)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI of the base station)
 *     38-39   2  spare
 *     40-57  18  longitude 1 (north-east corner), 1/10 min
 *     58-74  17  latitude 1 (north-east corner), 1/10 min
 *     75-92  18  longitude 2 (south-west corner), 1/10 min
 *     93-109 17  latitude 2 (south-west corner), 1/10 min
 *    110-113  4  station type
 *    114-121  8  type of ship and cargo type (Table 53; 0 = all)
 *    122-143 22  spare
 *    144-145  2  Tx/Rx mode (0 = TxA/TxB RxA/RxB, 1 = TxA RxA/RxB, 2 = TxB RxA/RxB, 3 = reserved)
 *    146-149  4  reporting interval (Table 77)
 *    150-153  4  quiet time in minutes (0 = none)
 *    154-159  6  spare
 *
 * Table 76 gives no not-available value for the region; we apply the
 * 181°/91° sentinels of message 22 (the same 1/10-minute fields) for
 * `region_available`.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * Three lookup objects typed `{ readonly [code: number]: string }` are
 *   complete for their field widths (16, 4 and 16 entries), so a plain
 *   index never yields `undefined` — unlike msg_22's TXRX_MODE, which
 *   needs a fallback because codes 3–15 are "not used".
 */

import { Bits } from "./bits";
import { requireBits, shipTypeText } from "./common";
import { Fixed, JsonObject } from "./jsonout";

export const LON_NOT_AVAILABLE = 181 * 600; // 0x1A838, 1/10-minute units
export const LAT_NOT_AVAILABLE = 91 * 600; // 0x0D548

/** Table 76. */
export const STATION_TYPE: { readonly [code: number]: string } = {
  0: "all types of mobiles (default)",
  1: "Class A mobile stations only",
  2: "all types of Class B mobile stations",
  3: "SAR airborne mobile station",
  4: "Class B SO mobile stations only",
  5: "Class B CS shipborne mobile station only",
  6: "inland waterways",
  7: "regional use",
  8: "regional use",
  9: "regional use",
  10: "base station coverage area",
  11: "for future use",
  12: "for future use",
  13: "for future use",
  14: "for future use",
  15: "for future use",
};

/** Table 76. */
export const TXRX_MODE: { readonly [code: number]: string } = {
  0: "TxA/TxB, RxA/RxB",
  1: "TxA, RxA/RxB",
  2: "TxB, RxA/RxB",
  3: "reserved for future use",
};

/** Table 77. */
export const REPORTING_INTERVAL: { readonly [code: number]: string } = {
  0: "as given by the autonomous mode",
  1: "10 min",
  2: "6 min",
  3: "3 min",
  4: "1 min",
  5: "30 s",
  6: "15 s",
  7: "10 s",
  8: "5 s",
  9: "next shorter reporting interval",
  10: "next longer reporting interval",
  11: "2 s (not applicable to Class B CS and SO)",
  12: "reserved for future use",
  13: "reserved for future use",
  14: "reserved for future use",
  15: "reserved for future use",
};

/** Decode a message 23. Throws DecodeError below 160 bits. */
export function decode(bits: Bits): JsonObject {
  requireBits(23, bits.length, 160);
  const neLon = bits.sint(40, 18);
  const neLat = bits.sint(58, 17);
  const swLon = bits.sint(75, 18);
  const swLat = bits.sint(93, 17);
  const stationType = bits.uint(110, 4);
  const shipType = bits.uint(114, 8);
  const txrx = bits.uint(144, 2);
  const interval = bits.uint(146, 4);
  return {
    msg_type: bits.uint(0, 6),
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
    ne_lon: new Fixed(neLon / 600.0, 6),
    ne_lat: new Fixed(neLat / 600.0, 6),
    sw_lon: new Fixed(swLon / 600.0, 6),
    sw_lat: new Fixed(swLat / 600.0, 6),
    region_available:
      neLon !== LON_NOT_AVAILABLE && neLat !== LAT_NOT_AVAILABLE &&
      swLon !== LON_NOT_AVAILABLE && swLat !== LAT_NOT_AVAILABLE,
    station_type: stationType,
    station_type_text: STATION_TYPE[stationType],
    ship_type: shipType,
    // Table 76 redefines 0 as "all types"; 1–255 follow Table 53.
    ship_type_text: shipType === 0 ? "all types (default)" : shipTypeText(shipType),
    spare2: bits.uint(122, 22),
    txrx_mode: txrx,
    txrx_mode_text: TXRX_MODE[txrx],
    reporting_interval: interval,
    reporting_interval_text: REPORTING_INTERVAL[interval],
    quiet_time_min: bits.uint(150, 4),
    spare3: bits.uint(154, 6),
  };
}
