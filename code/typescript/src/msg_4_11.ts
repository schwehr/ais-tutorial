/**
 * msg_4_11.ts — messages 4 and 11: base station report / UTC-date response
 * (chapter 6).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.2, Table 51 (field layout, 168 bits). The
 * SOTDMA communication state is Annex 2 §3.3.7.2.2 Table 18/19 and is decoded
 * by `decodeSotdma` from msg_1_2_3.ts — a message 4 always carries SOTDMA,
 * never ITDMA [Table 51, last row].
 *
 * Message 4 is sent by a base station every 10 s (3 1/3 s when it acts as
 * the synchronisation "semaphore") [Annex 1 §4.2.1 Table 2 note 1; Annex 2
 * §3.1.3.3.1]. Message 11 is the *same layout* sent by a mobile station in
 * answer to a UTC-date inquiry (message 10) [Annex 8 §3.2, §3.9].
 *
 * Bit layout (zero-based offsets; lengths from Table 51):
 *
 *      0- 5   6  message id (4 or 11)
 *      6- 7   2  repeat indicator
 *      8-37  30  user id (MMSI)
 *     38-51  14  UTC year,   1-9999; 0 = n/a
 *     52-55   4  UTC month,  1-12;   0 = n/a; 13-15 not used
 *     56-60   5  UTC day,    1-31;   0 = n/a
 *     61-65   5  UTC hour,   0-23;  24 = n/a; 25-31 not used
 *     66-71   6  UTC minute, 0-59;  60 = n/a; 61-63 not used
 *     72-77   6  UTC second, 0-59;  60 = n/a; 61-63 not used
 *     78      1  position accuracy (1 = high, <= 10 m)
 *     79-106 28  longitude, signed, 1/10 000 minute; 181° (0x6791AC0) = n/a
 *    107-133 27  latitude,  signed, 1/10 000 minute;  91° (0x3412140) = n/a
 *    134-137  4  type of electronic position fixing device (EPFD)
 *    138      1  transmission control for long-range broadcast message
 *    139-147  9  spare
 *    148      1  RAIM flag
 *    149-167 19  SOTDMA communication state
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `import { decodeSotdma } from "./msg_1_2_3"` — any `export`ed function
 *   of a sibling module can be imported; the module path is relative and
 *   has no extension.
 * * `utcIso` returns `string | null`, a *union type*. The caller does not
 *   need a special case: `null` is a valid `JsonValue` and prints as `null`.
 * * `String(n).padStart(2, "0")` is JavaScript's zero padding; there is no
 *   printf-style format string in the language, so we build the text by
 *   hand with a template literal.
 */

import { Bits } from "./bits";
import { epfdText, requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";
import { decodeSotdma } from "./msg_1_2_3";

// Not-available sentinels for the six UTC fields, Table 51.
export const UTC_YEAR_NA = 0;
export const UTC_MONTH_NA = 0;
export const UTC_DAY_NA = 0;
export const UTC_HOUR_NA = 24;
export const UTC_MINUTE_NA = 60;
export const UTC_SECOND_NA = 60;

export const TRANSMISSION_CONTROL: { readonly [code: number]: string } = {
  0: "Class A stops message 27 within base station coverage (default)",
  1: "Class A requested to transmit message 27 within base station coverage",
};

/**
 * Join the six UTC fields into "YYYY-MM-DDTHH:MM:SSZ", or null.
 *
 * null is returned when *any* component is its not-available value or
 * outside its valid range (Table 51: month 13-15, hour 25-31, minute and
 * second 61-63 are "not used"). We do not check the calendar (31 April
 * passes) — that is left to the reader as an exercise.
 */
export function utcIso(
  year: number, month: number, day: number, hour: number, minute: number, second: number,
): string | null {
  if (year < 1 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31) {
    return null;
  }
  if (hour > 23 || minute > 59 || second > 59) {
    return null;
  }
  const p2 = (n: number): string => String(n).padStart(2, "0");
  return `${String(year).padStart(4, "0")}-${p2(month)}-${p2(day)}T${p2(hour)}:${p2(minute)}:${p2(second)}Z`;
}

/** Decode a message 4 or 11. Throws DecodeError if shorter than 168 bits. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 168);
  const year = bits.uint(38, 14);
  const month = bits.uint(52, 4);
  const day = bits.uint(56, 5);
  const hour = bits.uint(61, 5);
  const minute = bits.uint(66, 6);
  const second = bits.uint(72, 6);
  const lonRaw = bits.sint(79, 28);
  const latRaw = bits.sint(107, 27);
  const epfd = bits.uint(134, 4);
  const txControl = bits.uint(138, 1);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    utc_year: year,
    utc_year_available: year !== UTC_YEAR_NA,
    utc_month: month,
    utc_month_available: month !== UTC_MONTH_NA,
    utc_day: day,
    utc_day_available: day !== UTC_DAY_NA,
    utc_hour: hour,
    utc_hour_available: hour !== UTC_HOUR_NA,
    utc_minute: minute,
    utc_minute_available: minute !== UTC_MINUTE_NA,
    utc_second: second,
    utc_second_available: second !== UTC_SECOND_NA,
    utc_iso: utcIso(year, month, day, hour, minute, second),
    position_accuracy: bits.uint(78, 1),
    lon: new Fixed(lonRaw / 600000.0, 6),
    lat: new Fixed(latRaw / 600000.0, 6),
    position_available: lonRaw !== 0x6791ac0 && latRaw !== 0x3412140,
    epfd,
    epfd_text: epfdText(epfd),
    transmission_control: txControl,
    transmission_control_text: TRANSMISSION_CONTROL[txControl],
    spare: bits.uint(139, 9),
    raim: bits.flag(148),
    comm_state: decodeSotdma(bits, 149),
  };
  return out;
}
