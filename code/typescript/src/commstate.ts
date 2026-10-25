/**
 * commstate.ts — TDMA communication state schedule and timing helpers (chapter 17).
 *
 * Spec: ITU-R M.1371-5 Annex 2 §3.1 (synchronisation and frame structure),
 * §3.3.4 (SOTDMA and ITDMA slot allocation), §3.3.7.2.2 Tables 18/19 (SOTDMA
 * communication state), and §3.3.7.3.2 Table 20 (ITDMA communication state).
 *
 * Eight AIS message types end with a communication state:
 *
 *     1, 2      168 bits   bits 149-167 (19)   always SOTDMA
 *     3         168 bits   bits 149-167 (19)   always ITDMA
 *     4, 11     168 bits   bits 149-167 (19)   always SOTDMA
 *     9         168 bits   bits 148-167 (20)   selector at 148 + 19-bit state
 *     18        168 bits   bits 148-167 (20)   selector at 148 + 19-bit state
 *     26     <=1064 bits   last 20 bits        selector + 19-bit state
 *
 * `msg_1_2_3.ts` decodes the 19-bit SOTDMA and ITDMA bit fields into a
 * `JsonObject`; this module re-exports those decoders and adds the
 * frame-arithmetic and clock-recovery functions that turn raw slot numbers,
 * offsets, and UTC sub-messages into physical schedules and UTC arrival times.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `export { SYNC_STATE, decodeItdma, decodeSotdma } from "./msg_1_2_3"`
 *   re-exports symbols from a sibling module without copying them.
 * * `Math.trunc(unixTime)` strips the fractional part of a number the same
 *   way Python's `int(unix_time)` and C++/Rust's integer cast do.
 */

import { Fixed } from "./jsonout";
export { SYNC_STATE, decodeItdma, decodeSotdma } from "./msg_1_2_3";

// Annex 2 §3.2.1 Table 10: one 60 s frame has 2 250 slots (26.667 ms each).
export const SLOTS_PER_FRAME = 2250;
export const FRAME_DURATION_S = 60.0;
export const SLOT_DURATION_S = FRAME_DURATION_S / SLOTS_PER_FRAME;

// Annex 2 §3.3.4.4 Table 16: a newly allocated SOTDMA slot is assigned a
// random integer time-out between TMO_MIN (3) and TMO_MAX (7) frames.
export const TMO_MIN = 3;
export const TMO_MAX = 7;

// Annex 2 §3.3.7.2.2 Table 19: in the `slot_timeout == 1` UTC hour/minute
// sub-message, hour 0..23 and minute 0..59 are valid; 24 and 60 mean n/a.
export const UTC_HOUR_NA = 24;
export const UTC_MINUTE_NA = 60;

/**
 * Convert a slot number (`0..2249`) to seconds into the UTC minute.
 *
 * Each TDMA frame is 60 s divided into 2 250 slots, so slot `k` begins at
 * `k * 60 / 2250` seconds past the minute [Annex 2 §3.2.1]. Returns `null`
 * if `slotNumber` is outside `0..2249`.
 */
export function slotToSeconds(slotNumber: number): Fixed | null {
  if (slotNumber < 0 || slotNumber >= SLOTS_PER_FRAME) {
    return null;
  }
  return new Fixed((slotNumber * FRAME_DURATION_S) / SLOTS_PER_FRAME, 3);
}

/**
 * Return the slot in the *next* frame announced by a SOTDMA `slot_offset`.
 *
 * When a SOTDMA stream reaches `slot_timeout == 0`, its 14-bit sub-message
 * is a `slot_offset` relative to the current slot on the same channel
 * [Annex 2 §3.3.7.2.2 Table 19]:
 *   - `slotOffset === 0` means the slot is de-allocated after this
 *     transmission (`null`);
 *   - any non-zero offset selects `(slotNumber + slotOffset) % 2250` in
 *     the next frame.
 */
export function nextSotdmaSlot(slotNumber: number, slotOffset: number): number | null {
  if (slotNumber < 0 || slotNumber >= SLOTS_PER_FRAME || slotOffset <= 0) {
    return null;
  }
  return (slotNumber + slotOffset) % SLOTS_PER_FRAME;
}

/**
 * Return the slot of the next transmission announced by an ITDMA `slot_offset`.
 *
 * In ITDMA (`decodeItdma`), `slot_offset` is already expanded to
 * `slot_increment` (`0..8191`) or `slot_increment + 8192` when `slots_raw`
 * is 5, 6, or 7 [Annex 2 §3.3.7.3.2 Table 20]. An offset of `0` means this
 * is the last transmission for the slot (`null`); otherwise the next
 * transmission on the same channel lands in `(slotNumber + slotOffset) % 2250`.
 */
export function nextItdmaSlot(slotNumber: number, slotOffset: number): number | null {
  if (slotNumber < 0 || slotNumber >= SLOTS_PER_FRAME || slotOffset <= 0) {
    return null;
  }
  return (slotNumber + slotOffset) % SLOTS_PER_FRAME;
}

/**
 * Recover the UTC transmission timestamp from a logger clock and SOTDMA state.
 *
 * When `utcHour` (`0..23`) and `utcMinute` (`0..59`) are known — from a
 * nearby `slot_timeout == 1` sub-message — the full time-of-day within the
 * 86 400 s UTC day is fixed, resolving logger clock offsets of many minutes
 * or hours. Otherwise we fall back to the nearest minute within 30 s of
 * `unixTime` (the same rule as `timeOfArrivalToUtc` in `uscg.ts`).
 */
export function slotToUtc(
  unixTime: number,
  slotNumber: number,
  utcHour: number | null = null,
  utcMinute: number | null = null,
): Fixed | null {
  if (slotNumber < 0 || slotNumber >= SLOTS_PER_FRAME) {
    return null;
  }
  const slotSec = (slotNumber * FRAME_DURATION_S) / SLOTS_PER_FRAME;
  const whole = Math.trunc(unixTime);

  if (
    utcHour !== null &&
    utcMinute !== null &&
    utcHour >= 0 &&
    utcHour <= 23 &&
    utcMinute >= 0 &&
    utcMinute <= 59
  ) {
    const dayStart = whole - (((whole % 86400) + 86400) % 86400);
    const tod = utcHour * 3600 + utcMinute * 60 + slotSec;
    const candidates = [-1, 0, 1].map((k) => dayStart + k * 86400 + tod);
    let best = candidates[0];
    for (const t of candidates) {
      if (Math.abs(t - unixTime) < Math.abs(best - unixTime)) {
        best = t;
      }
    }
    if (Math.abs(best - unixTime) > 43200) {
      return null;
    }
    return new Fixed(best, 3);
  }

  const minuteStart = whole - (((whole % 60) + 60) % 60);
  const candidates = [-60, 0, 60].map((k) => minuteStart + k + slotSec);
  let best = candidates[0];
  for (const t of candidates) {
    if (Math.abs(t - unixTime) < Math.abs(best - unixTime)) {
      best = t;
    }
  }
  if (Math.abs(best - unixTime) > 30) {
    return null;
  }
  return new Fixed(best, 3);
}
