/**
 * msg_17.ts — message 17: GNSS broadcast binary message (chapter 13).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.15, Table 68 (80-bit header plus 0–736
 * bits of differential-correction data) and Table 69 (layout of the data:
 * a 40-bit RTCM SC-104 style header followed by N 24-bit data words, parity
 * removed). The content follows Recommendation ITU-R M.823.
 *
 * Bit layout (zero-based offsets; lengths from Table 68):
 *
 *      0- 5   6  message id (17)
 *      6- 7   2  repeat indicator
 *      8-37  30  source id (MMSI of the base station)
 *     38-39   2  spare
 *     40-57  18  longitude of the DGNSS reference station, signed, 1/10 min; 181° = n/a
 *     58-74  17  latitude of the DGNSS reference station, signed, 1/10 min; 91° = n/a
 *     75-79   5  spare
 *     80-     0–736 differential correction data (Table 69):
 *       80- 85   6  RTCM message type
 *       86- 95  10  RTCM station id
 *       96-108  13  Z count, units of 0.6 s (0–3599.4 s)
 *      109-111   3  sequence number (cyclic 0–7)
 *      112-116   5  N, number of 24-bit data words following this header (≤ 29)
 *      117-119   3  reference station health
 *      120-     N × 24 bits of DGNSS data words
 *
 * Length rule: at least 80 bits (else DecodeError). With 40 or more data
 * bits we decode the RTCM header and hand back the remaining bits as a
 * string; `words_complete` says whether exactly N × 24 bits were present.
 * Fewer than 40 data bits gives `rtcm: null`.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * A nested object is just a `JsonObject` value stored under a key; the
 *   sorted stringifier in jsonout.ts recurses into it.
 * * `out["rtcm"] = cond ? decodeRtcmHeader(bits, 80) : null;` — the
 *   conditional expression may have the union type `JsonObject | null`,
 *   which is exactly what `JsonValue` allows.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";

export const LON_NOT_AVAILABLE = 181 * 600; // 0x1A838, 1/10-minute units
export const LAT_NOT_AVAILABLE = 91 * 600; // 0x0D548

/** Reference-station health, Table 69 → ITU-R M.823 (the RTCM SC-104 v2 code). */
export const HEALTH: { readonly [code: number]: string } = {
  0: "UDRE scale factor 1.0",
  1: "UDRE scale factor 0.75",
  2: "UDRE scale factor 0.5",
  3: "UDRE scale factor 0.3",
  4: "UDRE scale factor 0.2",
  5: "UDRE scale factor 0.1",
  6: "reference station transmission not monitored",
  7: "reference station not working",
};

/** The 40-bit RTCM header of Table 69 plus whatever data words follow. */
export function decodeRtcmHeader(bits: Bits, start: number): JsonObject {
  const nWords = bits.uint(start + 32, 5);
  const health = bits.uint(start + 37, 3);
  const zCount = bits.uint(start + 16, 13);
  const wordsStart = start + 40;
  const wordsBits = bits.length - wordsStart;
  return {
    message_type: bits.uint(start, 6),
    station_id: bits.uint(start + 6, 10),
    z_count: zCount,
    z_count_s: new Fixed(zCount * 0.6, 1),
    sequence: bits.uint(start + 22, 3),
    n_words: nWords,
    health,
    health_text: HEALTH[health],
    words: bits.raw(wordsStart, wordsBits),
    words_bits: wordsBits,
    words_complete: wordsBits === nWords * 24,
  };
}

/** Decode a message 17. Throws DecodeError below 80 bits. */
export function decode(bits: Bits): JsonObject {
  requireBits(17, bits.length, 80);
  const lonRaw = bits.sint(40, 18);
  const latRaw = bits.sint(58, 17);
  const dataBits = bits.length - 80;
  const out: JsonObject = {
    msg_type: bits.uint(0, 6),
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    spare: bits.uint(38, 2),
    lon: new Fixed(lonRaw / 600.0, 6),
    lat: new Fixed(latRaw / 600.0, 6),
    position_available: lonRaw !== LON_NOT_AVAILABLE && latRaw !== LAT_NOT_AVAILABLE,
    spare2: bits.uint(75, 5),
    data: bits.raw(80, dataBits),
    data_bits: dataBits,
  };
  // Table 69: the two-word (40-bit) header is only meaningful when complete.
  out["rtcm"] = dataBits >= 40 ? decodeRtcmHeader(bits, 80) : null;
  return out;
}
