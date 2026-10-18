/**
 * uscg.ts — the US Coast Guard "N-AIS" receive metadata (chapter 5).
 *
 * The USCG Research & Development Center's AIS network (and Kurt Schwehr's
 * `serial_logger.py`, which copied the idea) appends extra comma-separated
 * fields *after* the NMEA checksum:
 *
 *     !AIVDM,1,1,,A,18153:h02BruMNFHJPhhe0Th0D05,0*02,s34486,d-074,T25.49701619,x78124,r003669946,1223901985
 *                                                    └ s: signal strength, raw receiver units (0–65535)
 *                                                           └ d: signal strength in dBm
 *                                                                  └ T: time of arrival, seconds into the UTC minute
 *                                                                               └ x: receiver's sentence counter
 *                                                                                       └ r: receiving station id
 *                                                                                                    └ UNIX time (s)
 *
 * The format was never published as a standard; it is described in the GPSD
 * AIVDM.adoc "U.S. Coast Guard Extended AIVDM" section and in the AISUser
 * manual (output format 2, "proprietary USCG R&D Center format"). Field
 * order varies and most fields are optional; only the station id and the
 * final time are always present. Known prefixes:
 *
 *     s   received signal strength indicator (RSSI), receiver-specific units
 *     d   signal strength, dBm (negative number)
 *     T   time of arrival within the minute (fractional seconds)
 *     S   slot number (0–2249) in which the message was received
 *     x   sentence counter from the receiver
 *     r   receiving station id (`b` is also seen for base stations)
 *     (none) a bare number: UNIX time in seconds, possibly with a fraction
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `try { ... } catch (e) { ... }` works like Python's, but there is no
 *   `except ValueError:` filter. The catch receives *everything*, typed
 *   `unknown`; we re-throw whatever is not a `ValueError` so that genuine
 *   bugs still surface.
 * * `item.slice(1)` is Python's `item[1:]`; `item[0]` indexes a string the
 *   same way in both languages (for the ASCII text seen here).
 * * The number parsers come from common.ts because `Number("")` is `0` and
 *   `Number("1e3")` is `1000` — JavaScript is more forgiving than Python
 *   and we need Python's answers.
 */

import { Fixed, JsonObject } from "./jsonout";
import { ValueError, isDigits, pyFloat, pyInt } from "./common";

/**
 * Interpret the fields that followed the checksum.
 *
 * Unknown or malformed items are kept verbatim in `unparsed` so that no
 * information is silently lost.
 */
export function parseUscgFields(extras: readonly string[]): JsonObject {
  const out: JsonObject = {};
  const unparsed: string[] = [];
  for (const item of extras) {
    if (!item) {
      continue;
    }
    const code = item[0];
    const rest = item.slice(1);
    try {
      // Python: rest.lstrip("-").isdigit() — strip *all* leading minus signs,
      // then require digits; int() then rejects "--5" and lands in `unparsed`.
      if (code === "s" && isDigits(rest.replace(/^-+/, ""))) {
        out["rssi"] = pyInt(rest);
      } else if (code === "d" && isDigits(rest.replace(/^-+/, ""))) {
        out["signal_dbm"] = pyInt(rest);
      } else if (code === "T") {
        out["time_of_arrival_s"] = new Fixed(pyFloat(rest), 8);
      } else if (code === "S" && isDigits(rest)) {
        out["slot"] = pyInt(rest);
      } else if (code === "x" && isDigits(rest)) {
        out["sentence_counter"] = pyInt(rest);
      } else if ((code === "r" || code === "b") && rest) {
        out["station"] = rest;
        out["station_kind"] = code === "b" ? "base" : "receiver";
      } else if (isDigits(item.replace(".", ""))) { // replace() without /g replaces the first only
        // A bare number is the UNIX receive time.
        out["time"] = item.includes(".") ? new Fixed(pyFloat(item), 2) : pyInt(item);
      } else {
        unparsed.push(item);
      }
    } catch (e) {
      if (!(e instanceof ValueError)) {
        throw e;
      }
      unparsed.push(item);
    }
  }
  if (unparsed.length > 0) {
    out["unparsed"] = unparsed;
  }
  return out;
}

/**
 * Combine the coarse UNIX time with the fine `T` field.
 *
 * `T` is seconds past the start of the UTC minute measured by the
 * receiver's clock, so the precise arrival time is "start of some UTC
 * minute + T". The logging computer's `unixTime` tells us *which*
 * minute: we try the minute containing `unixTime` and its neighbours
 * and keep the candidate closest to `unixTime` (the log time is a few
 * seconds late, and may have rolled into the next minute).
 */
export function timeOfArrivalToUtc(unixTime: number, toaS: number | null): Fixed | null {
  if (toaS === null) {
    return null;
  }
  const minuteStart = unixTime - (unixTime % 60);
  const candidates = [-60, 0, 60].map((k) => minuteStart + k + toaS);
  let precise = candidates[0];
  for (const t of candidates) {
    if (Math.abs(t - unixTime) < Math.abs(precise - unixTime)) {
      precise = t; // like Python's min(key=...): the first minimum wins
    }
  }
  if (Math.abs(precise - unixTime) > 30) {
    return null;
  }
  return new Fixed(precise, 3);
}
