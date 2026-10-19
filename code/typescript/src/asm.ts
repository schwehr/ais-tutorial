/**
 * asm.ts — application-specific message (ASM) payloads, keyed by (DAC, FI).
 *
 * Messages 6 and 8 carry a 16-bit application identifier — a 10-bit
 * designated area code (DAC) and a 6-bit function identifier (FI) — followed
 * by data whose layout is defined by whoever owns that DAC
 * [ITU-R M.1371-5 Annex 5 §2.1 Table 23]. This module is the registry that
 * turns a known (dac, fi) pair into decoded fields; msg_6.ts and msg_8.ts
 * call `decodeAsm(dac, fi, bits, offset)` and attach a non-null result as
 * "application". The Python original (aistut/asm.py) documents every bit
 * offset.
 *
 * Implemented (chapter 9):
 *   (1, 0)    IFM 0 text telegram                 M.1371-5 Annex 5 §5.1 Tables 25/26
 *   (1, 11)   Met/hydro, IMO SN/Circ.236 (2004)   Circ.236 trial message 1 (AIVDM "Type 8(1/11)")
 *   (1, 31)   Met/hydro, IMO SN.1/Circ.289 (2010) Circ.289 Annex Table 1.1
 *   (366, 34) USCG/NOAA right-whale zone notice   reverse-engineered, see chapter 9
 *
 * Output conventions: a not-available sensor value prints as null (no
 * `_available` companion); enumerations keep their `_text` companion; a
 * payload that is too short yields {"application_name", "error"}.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * A `type AsmDecoder = (bits: Bits, o: number, dataBits: number) => JsonObject`
 *   is a *function type*; functions are values, so a `Map` can hold them.
 *   JavaScript `Map` keys are compared by identity, so a tuple `[dac, fi]`
 *   would never match — we key by the string `${dac}/${fi}` instead.
 * * `number | null` is a union type: the value is a number or it is `null`.
 *   Helpers that may return "not available" are typed that way, and the
 *   compiler makes callers deal with the `null`.
 * * `0x7fffff` and friends are ordinary numbers; there is no signed/unsigned
 *   distinction, which is why `sint` in bits.ts avoids the 32-bit operators.
 */

import { Bits } from "./bits";
import { Fixed, JsonObject, JsonValue } from "./jsonout";

// --- small helpers ---------------------------------------------------------

/** `raw` unless it equals the not-available sentinel, in which case null. */
function na(raw: number, sentinel: number): number | null {
  return raw === sentinel ? null : raw;
}

/** raw / divisor + offset with `decimals` places, or null if raw == sentinel. */
function scaled(raw: number, sentinel: number, divisor: number, decimals: number, offset = 0): Fixed | null {
  if (raw === sentinel) return null;
  return new Fixed(raw / divisor + offset, decimals);
}

export const TENDENCY: { readonly [code: number]: string } = {
  0: "steady", 1: "decreasing", 2: "increasing", 3: "not available",
};

/** WMO 306 code table 4.201 as reproduced in Circ.289 Table 1.1. */
export const PRECIPITATION: { readonly [code: number]: string } = {
  0: "reserved", 1: "rain", 2: "thunderstorm", 3: "freezing rain",
  4: "mixed/ice", 5: "snow", 6: "reserved", 7: "not available",
};

/** Beaufort scale, Circ.289 Table 1.2. */
export const SEA_STATE: { readonly [code: number]: string } = {
  0: "calm (flat)", 1: "light air (ripples)", 2: "light breeze (small wavelets)",
  3: "gentle breeze (large wavelets)", 4: "moderate breeze (small waves)",
  5: "fresh breeze (moderate waves)", 6: "strong breeze (large waves)",
  7: "near gale (sea heaps up)", 8: "gale (moderately high waves)",
  9: "strong gale (high waves)", 10: "storm (very high waves)",
  11: "violent storm (exceptionally high waves)", 12: "hurricane (huge waves)",
  13: "not available", 14: "reserved", 15: "reserved",
};

export const ICE: { readonly [code: number]: string } = {
  0: "no", 1: "yes", 2: "reserved", 3: "not available",
};

// --- (1, 0) IFM 0: text telegram (Annex 5 Tables 25/26) --------------------

export function decodeTextTelegram(bits: Bits, o: number, dataBits: number): JsonObject {
  const textBits = Math.floor((dataBits - 12) / 6) * 6;
  return {
    ack_required: bits.flag(o),
    text_sequence_number: bits.uint(o + 1, 11),
    text: bits.text(o + 12, textBits),
    text_chars: textBits / 6,
    spare_bits: dataBits - 12 - textBits,
  };
}

// --- (1, 11) IMO SN/Circ.236 met/hydro, 296 data bits ----------------------
// Latitude before longitude; temperatures and water level are offset-coded
// (value/10 - 60, - 20, - 10); every not-available value is the field maximum.

export const MET_HYDRO_236_BITS = 296;

export function decodeMetHydro236(bits: Bits, o: number, _dataBits: number): JsonObject {
  void _dataBits;
  const latRaw = bits.sint(o, 24);
  const lonRaw = bits.sint(o + 24, 25);
  const tendency = bits.uint(o + 134, 2);
  const levelTrend = bits.uint(o + 153, 2);
  const seaState = bits.uint(o + 262, 4);
  const precip = bits.uint(o + 276, 3);
  const ice = bits.uint(o + 288, 2);
  const pressureRaw = bits.uint(o + 125, 9);
  return {
    lat: latRaw === 0x7fffff ? null : new Fixed(latRaw / 60000.0, 6),
    lon: lonRaw === 0xffffff ? null : new Fixed(lonRaw / 60000.0, 6),
    day: na(bits.uint(o + 49, 5), 31),
    hour: na(bits.uint(o + 54, 5), 31),
    minute: na(bits.uint(o + 59, 6), 63),
    wind_speed_knots: na(bits.uint(o + 65, 7), 127),
    wind_gust_knots: na(bits.uint(o + 72, 7), 127),
    wind_dir_deg: na(bits.uint(o + 79, 9), 511),
    wind_gust_dir_deg: na(bits.uint(o + 88, 9), 511),
    air_temp_c: scaled(bits.uint(o + 97, 11), 2047, 10.0, 1, -60.0),
    humidity_pct: na(bits.uint(o + 108, 7), 127),
    dew_point_c: scaled(bits.uint(o + 115, 10), 1023, 10.0, 1, -20.0),
    pressure_hpa: pressureRaw === 511 ? null : pressureRaw + 800,
    pressure_tendency: tendency,
    pressure_tendency_text: TENDENCY[tendency],
    visibility_nm: scaled(bits.uint(o + 136, 8), 255, 10.0, 1),
    water_level_m: scaled(bits.uint(o + 144, 9), 511, 10.0, 1, -10.0),
    water_level_trend: levelTrend,
    water_level_trend_text: TENDENCY[levelTrend],
    current_speed_knots: scaled(bits.uint(o + 155, 8), 255, 10.0, 1),
    current_dir_deg: na(bits.uint(o + 163, 9), 511),
    current_2_speed_knots: scaled(bits.uint(o + 172, 8), 255, 10.0, 1),
    current_2_dir_deg: na(bits.uint(o + 180, 9), 511),
    current_2_depth_m: na(bits.uint(o + 189, 5), 31),
    current_3_speed_knots: scaled(bits.uint(o + 194, 8), 255, 10.0, 1),
    current_3_dir_deg: na(bits.uint(o + 202, 9), 511),
    current_3_depth_m: na(bits.uint(o + 211, 5), 31),
    wave_height_m: scaled(bits.uint(o + 216, 8), 255, 10.0, 1),
    wave_period_s: na(bits.uint(o + 224, 6), 63),
    wave_dir_deg: na(bits.uint(o + 230, 9), 511),
    swell_height_m: scaled(bits.uint(o + 239, 8), 255, 10.0, 1),
    swell_period_s: na(bits.uint(o + 247, 6), 63),
    swell_dir_deg: na(bits.uint(o + 253, 9), 511),
    sea_state: seaState,
    sea_state_text: SEA_STATE[seaState],
    water_temp_c: scaled(bits.uint(o + 266, 10), 1023, 10.0, 1, -10.0),
    precipitation: precip,
    precipitation_text: PRECIPITATION[precip],
    salinity_ppt: scaled(bits.uint(o + 279, 9), 511, 10.0, 1),
    ice,
    ice_text: ICE[ice],
    spare: bits.uint(o + 290, 6),
  };
}

// --- (1, 31) IMO SN.1/Circ.289 met/hydro, 304 data bits --------------------

export const MET_HYDRO_289_BITS = 304;
export const LON_1000_NOT_AVAILABLE = 181 * 60000; // 10 860 000
export const LAT_1000_NOT_AVAILABLE = 91 * 60000; //  5 460 000

/** Circ.289 air pressure: 1-401 map to 800-1200 hPa; the ends are clamped. */
function pressure289(raw: number): number | null {
  if (raw === 511) return null;
  if (raw === 0) return 799; // "pressure 799 hPa or less"
  if (raw >= 402) return 1201; // "pressure 1201 hPa or greater"
  return raw + 799;
}

export function decodeMetHydro289(bits: Bits, o: number, _dataBits: number): JsonObject {
  void _dataBits;
  const lonRaw = bits.sint(o, 25);
  const latRaw = bits.sint(o + 25, 24);
  const tendency = bits.uint(o + 135, 2);
  const visRaw = bits.uint(o + 137, 8);
  const levelTrend = bits.uint(o + 157, 2);
  const seaState = bits.uint(o + 266, 4);
  const precip = bits.uint(o + 280, 3);
  const salinityRaw = bits.uint(o + 283, 9);
  const ice = bits.uint(o + 292, 2);
  return {
    lon: lonRaw === LON_1000_NOT_AVAILABLE ? null : new Fixed(lonRaw / 60000.0, 6),
    lat: latRaw === LAT_1000_NOT_AVAILABLE ? null : new Fixed(latRaw / 60000.0, 6),
    position_accuracy: bits.uint(o + 49, 1),
    day: na(bits.uint(o + 50, 5), 0),
    hour: na(bits.uint(o + 55, 5), 24),
    minute: na(bits.uint(o + 60, 6), 60),
    wind_speed_knots: na(bits.uint(o + 66, 7), 127),
    wind_gust_knots: na(bits.uint(o + 73, 7), 127),
    wind_dir_deg: na(bits.uint(o + 80, 9), 360),
    wind_gust_dir_deg: na(bits.uint(o + 89, 9), 360),
    air_temp_c: scaled(bits.sint(o + 98, 11), -1024, 10.0, 1),
    humidity_pct: na(bits.uint(o + 109, 7), 101),
    dew_point_c: scaled(bits.sint(o + 116, 10), 501, 10.0, 1),
    pressure_hpa: pressure289(bits.uint(o + 126, 9)),
    pressure_tendency: tendency,
    pressure_tendency_text: TENDENCY[tendency],
    visibility_nm: scaled(visRaw % 128, 127, 10.0, 1), // low 7 bits
    visibility_greater_than: visRaw >= 128, // the MSB
    water_level_m: scaled(bits.uint(o + 145, 12), 4001, 100.0, 2, -10.0),
    water_level_trend: levelTrend,
    water_level_trend_text: TENDENCY[levelTrend],
    current_speed_knots: scaled(bits.uint(o + 159, 8), 255, 10.0, 1),
    current_dir_deg: na(bits.uint(o + 167, 9), 360),
    current_2_speed_knots: scaled(bits.uint(o + 176, 8), 255, 10.0, 1),
    current_2_dir_deg: na(bits.uint(o + 184, 9), 360),
    current_2_depth_m: na(bits.uint(o + 193, 5), 31),
    current_3_speed_knots: scaled(bits.uint(o + 198, 8), 255, 10.0, 1),
    current_3_dir_deg: na(bits.uint(o + 206, 9), 360),
    current_3_depth_m: na(bits.uint(o + 215, 5), 31),
    wave_height_m: scaled(bits.uint(o + 220, 8), 255, 10.0, 1),
    wave_period_s: na(bits.uint(o + 228, 6), 63),
    wave_dir_deg: na(bits.uint(o + 234, 9), 360),
    swell_height_m: scaled(bits.uint(o + 243, 8), 255, 10.0, 1),
    swell_period_s: na(bits.uint(o + 251, 6), 63),
    swell_dir_deg: na(bits.uint(o + 257, 9), 360),
    sea_state: seaState,
    sea_state_text: SEA_STATE[seaState],
    water_temp_c: scaled(bits.sint(o + 270, 10), 501, 10.0, 1),
    precipitation: precip,
    precipitation_text: PRECIPITATION[precip],
    salinity_ppt: salinityRaw >= 510 ? null : new Fixed(salinityRaw / 10.0, 1),
    salinity_sensor_available: salinityRaw !== 511,
    ice,
    ice_text: ICE[ice],
    spare: bits.uint(o + 294, 10),
  };
}

// --- (366, 34) USCG/NOAA right-whale zone notice, 2008 form ----------------
// Reverse-engineered (chapter 9): a sequence of 112-bit records, each a
// 36-bit header (zone id 5, zone type 7, day 5, hour 5, minute 6, duration
// in 10-minute units 8) and one 76-bit circle (shape 1, lon 28, lat 27,
// radius 16, spare 4).

export const WHALE_RECORD_BITS = 112;

export const WHALE_ZONE_TYPE: { readonly [code: number]: string } = {
  0: "right whales not observed",
  1: "right whales present",
};

export function decodeWhaleNotice(bits: Bits, o: number, dataBits: number): JsonObject {
  const notices: JsonObject[] = [];
  let r = o;
  while (bits.has(r, WHALE_RECORD_BITS)) {
    const zoneType = bits.uint(r + 5, 7);
    const durationRaw = bits.uint(r + 28, 8);
    const z = r + 36;
    notices.push({
      zone_id: bits.uint(r, 5),
      zone_type: zoneType,
      zone_type_text: WHALE_ZONE_TYPE[zoneType] ?? "unknown",
      start_day: bits.uint(r + 12, 5),
      start_hour: bits.uint(r + 17, 5),
      start_minute: bits.uint(r + 22, 6),
      duration_raw: durationRaw,
      duration_minutes: durationRaw * 10,
      shape: bits.uint(z, 1),
      lon: new Fixed(bits.sint(z + 1, 28) / 600000.0, 6),
      lat: new Fixed(bits.sint(z + 29, 27) / 600000.0, 6),
      radius_m: bits.uint(z + 56, 16),
      spare: bits.uint(z + 72, 4),
    });
    r += WHALE_RECORD_BITS;
  }
  return { notices, trailing_bits: o + dataBits - r };
}

// --- the registry -----------------------------------------------------------

export type AsmDecoder = (bits: Bits, o: number, dataBits: number) => JsonObject;

interface AsmEntry {
  readonly name: string;
  readonly needBits: number;
  readonly func: AsmDecoder;
}

/** Keyed by "dac/fi" because Map compares array keys by identity, not value. */
export const DECODERS: ReadonlyMap<string, AsmEntry> = new Map<string, AsmEntry>([
  ["1/0", { name: "IFM 0 text telegram", needBits: 12, func: decodeTextTelegram }],
  ["1/11", { name: "IMO Circ.236 meteorological and hydrological data", needBits: MET_HYDRO_236_BITS, func: decodeMetHydro236 }],
  ["1/31", { name: "IMO Circ.289 meteorological and hydrographic data", needBits: MET_HYDRO_289_BITS, func: decodeMetHydro289 }],
  ["366/34", { name: "USCG/NOAA right whale zone notice (2008)", needBits: WHALE_RECORD_BITS, func: decodeWhaleNotice }],
]);

/**
 * Decode the application data that starts at `offset`. Returns null when
 * (dac, fi) is unknown. For a known pair the object always carries
 * `application_name`; too few bits yield an `error` text instead of fields.
 */
export function decodeAsm(dac: number, fi: number, bits: Bits, offset: number): JsonObject | null {
  const entry = DECODERS.get(`${dac}/${fi}`);
  if (entry === undefined) return null;
  const dataBits = bits.length - offset;
  const out: JsonObject = { application_name: entry.name };
  if (dataBits < entry.needBits) {
    out["error"] = `needs ${entry.needBits} data bits, have ${dataBits}`;
    return out;
  }
  const fields = entry.func(bits, offset, dataBits);
  for (const key of Object.keys(fields)) {
    out[key] = fields[key] as JsonValue;
  }
  return out;
}
