/**
 * nmea.ts — the NMEA 0183 sentence layer (chapter 1).
 *
 * An NMEA 0183 sentence (IEC 61162-1 §7) looks like
 *
 *     $GPGGA,000429.000,3601.5522,N,10947.9359,W,1,7,4.21,12491.0,M,-20.5,M,,*6D
 *     ^^     ^^^                                                              ^^
 *     talker formatter                 comma-separated data fields        checksum
 *
 * * `$` (or `!` for encapsulated data such as AIS) starts the sentence.
 * * The next two characters are the *talker* (GP = GPS, AI = AIS mobile
 *   station, BS = base station, AN = AtoN, …), then a 3-letter *formatter*
 *   (GGA, RMC, VDM, …). Proprietary sentences start with `P` + a 3-letter
 *   manufacturer code (`PMTK`, `PSRT`, `PFEC`) and have no fixed formatter.
 * * Fields are separated by commas; an empty field is allowed and means
 *   "no data".
 * * `*` followed by two hex digits is the checksum: the XOR of every byte
 *   between `$`/`!` and `*` (exclusive). IEC 61162-1 §7.2.3.
 * * The sentence ends with CR LF and may be at most 82 characters including
 *   `$` and CR LF (§7.1.1). Real logs often break this; we do not enforce it.
 *
 * The USCG / serial_logger convention of appending extra comma fields *after*
 * the checksum (`…*6D,rplane_to_sd,1197072048.14`) is also handled here:
 * those fields are returned separately as `afterChecksum`.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `class Sentence` with `readonly` fields plays the role of Python's
 *   `@dataclass`: a bundle of named, typed values. `get address()` is a
 *   computed property, like Python's `@property`.
 * * `number | null` is TypeScript's `Optional[int]`: the compiler will not
 *   let you do arithmetic on it until you have checked for `null`. We use
 *   `null` (not `undefined`) for "no data" throughout, because `null` is
 *   what JSON has.
 * * `||` and `??` both provide defaults, but differently: `f[13] || null`
 *   treats the empty string as missing (Python's `f[13] or None`), while
 *   `x ?? d` only replaces `null`/`undefined`.
 * * Lookup tables keyed by *input text* are `Map`s, not plain objects: a
 *   plain object inherits keys such as "constructor" and "toString" from
 *   `Object.prototype`, so `{}["constructor"]` is not `undefined`! Tables
 *   keyed by small integers are safe as plain objects.
 */

import { Fixed, JsonObject, JsonValue } from "./jsonout";
import { ValueError, partition, pyFloat, pyHexInt, pyInt, pyRepr, pyStrip } from "./common";

/** XOR of all characters in `body` (the text between '$' and '*'). */
export function checksum(body: string): number {
  let value = 0;
  for (const ch of body) {
    value ^= ch.codePointAt(0) ?? 0;
  }
  return value;
}

/** One parsed NMEA sentence, before any formatter-specific decoding. */
export class Sentence {
  constructor(
    readonly start: string,                  // '$' or '!'
    readonly talker: string,                 // 'GP', 'AI', 'P' for proprietary ...
    readonly formatter: string,              // 'GGA', 'VDM', 'MTK010' ...
    readonly fields: readonly string[],      // data fields *after* the address field
    readonly checksumGiven: number | null,   // value written in the sentence, or null
    readonly checksumOk: boolean,            // true only if present and matching
    readonly afterChecksum: readonly string[] = [],   // USCG extras
    readonly raw: string = "",
  ) {}

  /** The address field, e.g. 'GPGGA' or 'AIVDM'. */
  get address(): string {
    return this.talker + this.formatter;
  }
}

/** Thrown for text that is not an NMEA sentence at all. */
export class NmeaError extends ValueError {
  constructor(message: string) {
    super(message);
    this.name = "NmeaError";
  }
}

/**
 * Split one line into its NMEA parts and verify the checksum.
 *
 * Leading/trailing whitespace and CR/LF are ignored. Characters before the
 * `$`/`!` (such as a `file-name:` prefix from grep) are discarded; TAG
 * blocks (chapter 4) must be stripped by the caller first.
 */
export function parseSentence(rawLine: string): Sentence {
  let line = pyStrip(rawLine);
  // The earliest of '$' and '!' that is actually present.
  const candidates = [line.indexOf("$"), line.indexOf("!")].filter((i) => i >= 0);
  if (candidates.length === 0) {
    throw new NmeaError("no '$' or '!' in line");
  }
  line = line.slice(Math.min(...candidates));
  const start = line[0];

  const [body, star, tail] = partition(line.slice(1), "*");
  let given: number | null = null;
  let ok = false;
  let extras: string[] = [];
  if (star) {
    const [hexpart, , rest] = partition(tail, ",");
    try {
      given = pyHexInt(hexpart.slice(0, 2));
    } catch (e) {
      if (!(e instanceof ValueError)) {
        throw e;
      }
      given = null;
    }
    ok = given !== null && given === checksum(body);
    if (rest) {
      extras = rest.split(",");
    }
  }
  // If there was no '*' we still parse the fields but checksumOk is false.

  const [address, ...fields] = body.split(",");
  if (address.length < 3) {
    throw new NmeaError(`address field too short: ${pyRepr(address)}`);
  }
  let talker: string;
  let formatter: string;
  if (address.startsWith("P")) { // proprietary: P + 3-char maker code
    talker = "P";
    formatter = address.slice(1);
  } else {
    talker = address.slice(0, 2);
    formatter = address.slice(2);
  }
  return new Sentence(start, talker, formatter, fields, given, ok, extras, line);
}

// --- Helpers for the GPS formatters ----------------------------------------

/**
 * '4312.2805','N' -> 43.204675 ; empty -> null.
 *
 * NMEA writes latitude as DDMM.MMMM: two digits of degrees followed by
 * minutes with a decimal fraction. Longitude uses three degree digits
 * (DDDMM.MMMM); see parseLongitude.
 */
export function parseLatitude(ddmm: string, hemisphere: string): number | null {
  if (!ddmm) {
    return null;
  }
  const degrees = pyInt(ddmm.slice(0, 2));
  const minutes = pyFloat(ddmm.slice(2));
  const value = degrees + minutes / 60.0;
  return hemisphere === "S" ? -value : value;
}

/** '07052.8474','W' -> -70.88079 */
export function parseLongitude(dddmm: string, hemisphere: string): number | null {
  if (!dddmm) {
    return null;
  }
  const degrees = pyInt(dddmm.slice(0, 3));
  const minutes = pyFloat(dddmm.slice(3));
  const value = degrees + minutes / 60.0;
  return hemisphere === "W" ? -value : value;
}

/** '000429.000' -> '00:04:29.000' (kept as text; no date is available). */
export function parseUtcTime(hhmmss: string): string | null {
  if (hhmmss.length < 6) {
    return null;
  }
  return `${hhmmss.slice(0, 2)}:${hhmmss.slice(2, 4)}:${hhmmss.slice(4)}`;
}

/**
 * '081207' -> '2007-12-08'. Two-digit years: NMEA predates Y2K, we assume
 * 2000–2099 which is right for every file in this tutorial.
 */
export function parseDate(ddmmyy: string): string | null {
  if (ddmmyy.length !== 6) {
    return null;
  }
  return `20${ddmmyy.slice(4, 6)}-${ddmmyy.slice(2, 4)}-${ddmmyy.slice(0, 2)}`;
}

export function optFloat(s: string): number | null {
  return s ? pyFloat(s) : null;
}

/** Wrap a float for printing with `decimals` places (see jsonout.ts). */
export function fixed(value: number | null, decimals: number): Fixed | null {
  return value === null ? null : new Fixed(value, decimals);
}

export function optInt(s: string): number | null {
  return s ? pyInt(s) : null;
}

/** Python's `fields + [""] * (n - len(fields))`: pad short sentences. */
function padFields(fields: readonly string[], n: number): string[] {
  const out = [...fields]; // spread syntax copies the array
  while (out.length < n) {
    out.push("");
  }
  return out;
}

// --- The four GPS sentences of chapter 1 -----------------------------------
//
// Field numbers below are 1-based positions *after* the address field, which
// is how the NMEA standard numbers them.

export const GGA_FIX_QUALITY: { readonly [code: number]: string } = {
  0: "invalid", 1: "GPS fix (SPS)", 2: "DGPS fix", 3: "PPS fix",
  4: "RTK fixed", 5: "RTK float", 6: "estimated (dead reckoning)",
  7: "manual input", 8: "simulation",
};

/**
 * GGA — Global positioning system fix data (IEC 61162-1 §8.3.x / NMEA 0183 v3 §GGA).
 *
 *     1 UTC time hhmmss.ss        6 fix quality (table above)
 *     2 latitude ddmm.mmmm        7 number of satellites in use
 *     3 N/S                        8 HDOP
 *     4 longitude dddmm.mmmm      9 antenna altitude, 10 'M'
 *     5 E/W                       11 geoidal separation, 12 'M'
 *                                 13 age of DGPS data (s), 14 DGPS station id
 */
export function parseGga(s: Sentence): JsonObject {
  const f = padFields(s.fields, 14);
  const quality = optInt(f[5]);
  return {
    sentence: "GGA",
    time_utc: parseUtcTime(f[0]),
    lat: fixed(parseLatitude(f[1], f[2]), 6),
    lon: fixed(parseLongitude(f[3], f[4]), 6),
    fix_quality: quality,
    fix_quality_text: quality !== null ? (GGA_FIX_QUALITY[quality] ?? "unknown") : null,
    satellites_used: optInt(f[6]),
    hdop: fixed(optFloat(f[7]), 2),
    altitude_m: fixed(optFloat(f[8]), 1),
    geoid_separation_m: fixed(optFloat(f[10]), 1),
    dgps_age_s: fixed(optFloat(f[12]), 1),
    dgps_station: f[13] || null,
  };
}

export const GSA_MODE2: { readonly [code: number]: string } = { 1: "no fix", 2: "2D", 3: "3D" };

/**
 * GSA — GNSS DOP and active satellites.
 *
 *     1 mode: M = manual, A = automatic 2D/3D switching
 *     2 mode: 1 = fix not available, 2 = 2D, 3 = 3D
 *     3–14 PRN numbers of the (up to 12) satellites used in the solution
 *     15 PDOP  16 HDOP  17 VDOP
 */
export function parseGsa(s: Sentence): JsonObject {
  const f = padFields(s.fields, 17);
  const fixMode = optInt(f[1]);
  return {
    sentence: "GSA",
    selection_mode: f[0] || null,
    fix_mode: fixMode,
    fix_mode_text: fixMode !== null ? (GSA_MODE2[fixMode] ?? null) : null,
    satellites_used: f.slice(2, 14).filter((p) => p).map((p) => pyInt(p)),
    pdop: fixed(optFloat(f[14]), 2),
    hdop: fixed(optFloat(f[15]), 2),
    vdop: fixed(optFloat(f[16]), 2),
  };
}

/**
 * GSV — GNSS satellites in view. One sentence carries up to 4 satellites.
 *
 *     1 total number of GSV sentences in this group
 *     2 this sentence's number (1-based)
 *     3 total satellites in view
 *     then repeating groups of 4: PRN, elevation (deg), azimuth (deg), SNR (dB-Hz)
 *     Elevation/azimuth/SNR are empty when not tracked.
 */
export function parseGsv(s: Sentence): JsonObject {
  const f = s.fields;
  const sats: JsonValue[] = [];
  for (let i = 3; i < f.length - 3; i += 4) {
    const [prn, elev, az, snr] = f.slice(i, i + 4);
    if (prn) {
      sats.push({ prn: pyInt(prn), elevation_deg: optInt(elev),
                  azimuth_deg: optInt(az), snr_db: optInt(snr) });
    }
  }
  return {
    sentence: "GSV",
    total_sentences: f.length > 0 ? optInt(f[0]) : null,
    sentence_number: f.length > 1 ? optInt(f[1]) : null,
    satellites_in_view: f.length > 2 ? optInt(f[2]) : null,
    satellites: sats,
  };
}

export const RMC_MODE: ReadonlyMap<string, string> = new Map([
  ["A", "autonomous"], ["D", "differential"], ["E", "estimated"],
  ["M", "manual"], ["S", "simulator"], ["N", "not valid"],
]);

/**
 * RMC — Recommended minimum specific GNSS data.
 *
 *     1 UTC time   2 status A = valid, V = warning (no fix)
 *     3,4 latitude   5,6 longitude   7 SOG knots   8 COG degrees true
 *     9 date ddmmyy   10,11 magnetic variation deg, E/W
 *     12 mode indicator (NMEA 2.3+)   13 navigational status (NMEA 4.1+)
 */
export function parseRmc(s: Sentence): JsonObject {
  const f = padFields(s.fields, 13);
  let variation = optFloat(f[9]);
  if (variation !== null && f[10] === "W") {
    variation = -variation;
  }
  return {
    sentence: "RMC",
    time_utc: parseUtcTime(f[0]),
    status: f[1] || null,
    valid: f[1] === "A",
    lat: fixed(parseLatitude(f[2], f[3]), 6),
    lon: fixed(parseLongitude(f[4], f[5]), 6),
    sog_knots: fixed(optFloat(f[6]), 2),
    cog_deg: fixed(optFloat(f[7]), 2),
    date: parseDate(f[8]),
    magnetic_variation_deg: fixed(variation, 1),
    mode: f[11] || null,
    mode_text: f[11] ? (RMC_MODE.get(f[11]) ?? null) : null,
  };
}

/** A function type alias: every GPS parser has this exact signature. */
export type GpsParser = (s: Sentence) => JsonObject;

export const GPS_PARSERS: ReadonlyMap<string, GpsParser> = new Map([
  ["GGA", parseGga], ["GSA", parseGsa], ["GSV", parseGsv], ["RMC", parseRmc],
]);
