/**
 * msg_22.ts — message 22: channel management (chapter 13).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.20, Table 75 (168 bits). A base station
 * tells mobile stations which two VHF channels, transmit/receive mode and
 * power level to use, either inside a rectangular region (broadcast form)
 * or for up to two individually addressed stations. Regions and the
 * transitional-zone behaviour are defined in Annex 2 §4.1; channel numbers
 * follow Recommendation ITU-R M.1084 Annex 4 (four-digit form).
 *
 * Bit layout (zero-based offsets; lengths from Table 75):
 *
 *      0- 5   6  message id (22)
 *      6- 7   2  repeat indicator
 *      8-37  30  station id (MMSI of the base station)
 *     38-39   2  spare
 *     40-51  12  channel A number (M.1084)
 *     52-63  12  channel B number (M.1084)
 *     64-67   4  Tx/Rx mode (0 = TxA/TxB RxA/RxB, 1 = TxA RxA/RxB, 2 = TxB RxA/RxB)
 *     68      1  power (0 = high, 1 = low)
 *     69- 86  18  longitude 1 (north-east corner), 1/10 min, 181° = n/a   | or 30-bit MMSI 1 at 69-98
 *     87-103  17  latitude 1 (north-east corner), 1/10 min, 91° = n/a     |    followed by 5 zero bits
 *    104-121  18  longitude 2 (south-west corner)                          | or 30-bit MMSI 2 at 104-133
 *    122-138  17  latitude 2 (south-west corner)                           |    followed by 5 zero bits
 *    139      1  addressed (1) or broadcast/geographical (0) — decides how 69-138 are read
 *    140      1  channel A bandwidth (0 = default; 1 = spare, formerly 12.5 kHz)
 *    141      1  channel B bandwidth
 *    142-144  3  transitional zone size, nautical miles = value + 1 (default 4 → 5 NM)
 *    145-167 23  spare
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `channelFrequencyKhz` returns `number | null`; the caller must test for
 *   `null` before using the value, and the compiler enforces that.
 * * `new Set<number>([...])` + `.has(x)` is the JavaScript set; `Math.floor`
 *   and `%` replace Python's `divmod`.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";

export const LON_NOT_AVAILABLE = 181 * 600; // 0x1A838, 1/10-minute units
export const LAT_NOT_AVAILABLE = 91 * 600; // 0x0D548

/** Table 75. The field is 4 bits wide but only 0–2 are defined. */
export const TXRX_MODE: { readonly [code: number]: string } = {
  0: "TxA/TxB, RxA/RxB",
  1: "TxA, RxA/RxB",
  2: "TxB, RxA/RxB",
};

export function txrxModeText(code: number): string {
  return code in TXRX_MODE ? TXRX_MODE[code] : "not used";
}

export const POWER: { readonly [code: number]: string } = { 0: "high", 1: "low" };

/**
 * RR Appendix 18 channels that are simplex (one frequency, "ship & coast"):
 * 6, 8–17 and 67–77. Everything else in 1–28 / 60–88 is a duplex pair.
 */
export const SIMPLEX_CHANNELS = new Set<number>([
  6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
  67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77,
]);

/**
 * Carrier frequency in kHz for an ITU-R M.1084 Annex 4 channel number.
 *
 * Two-digit numbers are the RR Appendix 18 channels 1–28 and 60–88; ship
 * frequencies are 156.050 + 0.050 (n-1) MHz for 1–28 and
 * 156.025 + 0.050 (n-60) MHz for 60–88; the coast frequency of a duplex
 * channel is 4.6 MHz higher. A leading 1 means "simplex use of the ship
 * frequency", a leading 2 "simplex use of the coast frequency" (M.1084
 * Annex 4 §3): 2087 = 161.975 MHz (AIS 1), 2088 = 162.025 MHz (AIS 2).
 * Returns null for numbers outside those ranges.
 */
export function channelFrequencyKhz(num: number): number | null {
  const prefix = Math.floor(num / 1000);
  const base = num % 1000;
  if (prefix < 0 || prefix > 2) {
    return null;
  }
  let ship: number;
  if (1 <= base && base <= 28) {
    ship = 156050 + 50 * (base - 1);
  } else if (60 <= base && base <= 88) {
    ship = 156025 + 50 * (base - 60);
  } else {
    return null;
  }
  const coast = SIMPLEX_CHANNELS.has(base) ? ship : ship + 4600;
  return prefix === 2 ? coast : ship;
}

/** `channelFrequencyKhz` as a Fixed with three decimals (MHz), or null. */
export function channelMhz(num: number): Fixed | null {
  const khz = channelFrequencyKhz(num);
  return khz === null ? null : new Fixed(khz / 1000.0, 3);
}

/** Decode a message 22. Throws DecodeError below 168 bits. */
export function decode(bits: Bits): JsonObject {
  requireBits(22, bits.length, 168);
  const channelA = bits.uint(40, 12);
  const channelB = bits.uint(52, 12);
  const txrx = bits.uint(64, 4);
  const power = bits.uint(68, 1);
  const addressed = bits.flag(139);
  const zone = bits.uint(142, 3);
  const out: JsonObject = {
    msg_type: bits.uint(0, 6),
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
    channel_a: channelA,
    channel_a_mhz: channelMhz(channelA),
    channel_b: channelB,
    channel_b_mhz: channelMhz(channelB),
    txrx_mode: txrx,
    txrx_mode_text: txrxModeText(txrx),
    power,
    power_text: POWER[power],
    addressed,
    band_a: bits.uint(140, 1),
    band_b: bits.uint(141, 1),
    zone_size: zone,
    zone_size_nm: zone + 1,
    spare2: bits.uint(145, 23),
  };
  if (addressed) {
    // Table 75: the 30-bit MMSI occupies the 18-bit longitude field and
    // the first 12 bits of the 17-bit latitude field; 5 zero bits follow.
    out["dest_mmsi_1"] = bits.uint(69, 30);
    out["dest_spare_1"] = bits.uint(99, 5);
    out["dest_mmsi_2"] = bits.uint(104, 30);
    out["dest_spare_2"] = bits.uint(134, 5);
  } else {
    const neLon = bits.sint(69, 18);
    const neLat = bits.sint(87, 17);
    const swLon = bits.sint(104, 18);
    const swLat = bits.sint(122, 17);
    out["ne_lon"] = new Fixed(neLon / 600.0, 6);
    out["ne_lat"] = new Fixed(neLat / 600.0, 6);
    out["sw_lon"] = new Fixed(swLon / 600.0, 6);
    out["sw_lat"] = new Fixed(swLat / 600.0, 6);
    out["region_available"] =
      neLon !== LON_NOT_AVAILABLE && neLat !== LAT_NOT_AVAILABLE &&
      swLon !== LON_NOT_AVAILABLE && swLat !== LAT_NOT_AVAILABLE;
  }
  return out;
}
