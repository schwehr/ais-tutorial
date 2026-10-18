/**
 * msg_1_2_3.ts — Class A position reports, messages 1, 2 and 3 (chapter 3).
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.1, Table 48 (field layout, 168 bits);
 * Table 49 (which communication state each message carries); Annex 2
 * §3.3.7.2.2 Table 18/19 (SOTDMA communication state) and §3.3.7.3.2
 * Table 20 (ITDMA communication state).
 *
 * Bit layout (zero-based offsets; lengths from Table 48):
 *
 *      0- 5   6  message id (1, 2 or 3)
 *      6- 7   2  repeat indicator
 *      8-37  30  user id (MMSI)
 *     38-41   4  navigational status
 *     42-49   8  rate of turn, signed, special encoding
 *     50-59  10  speed over ground, 1/10 knot; 1023 = n/a, 1022 = ≥102.2 kn
 *     60      1  position accuracy (1 = high, ≤10 m)
 *     61-88  28  longitude, signed, 1/10 000 minute; 181° = n/a
 *     89-115 27  latitude,  signed, 1/10 000 minute;  91° = n/a
 *    116-127 12  course over ground, 1/10 degree; 3600 = n/a
 *    128-136  9  true heading, degrees; 511 = n/a
 *    137-142  6  time stamp (UTC second); 60 = n/a, 61 manual, 62 dead reckoning, 63 inoperative
 *    143-144  2  special manoeuvre indicator
 *    145-147  3  spare
 *    148      1  RAIM flag
 *    149-167 19  communication state (SOTDMA for 1 and 2, ITDMA for 3)
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * The decoder returns an *object literal* whose property names are the
 *   JSON keys. TypeScript checks it against the `JsonObject` interface, so
 *   a typo like `new Fixd(...)` or a stray `undefined` is caught at compile
 *   time rather than in the diff against the Python output.
 * * `Object.assign(out, decodeRot(...))` merges one object into another —
 *   Python's `out.update(...)`.
 * * `sub >> 9` and `(sub >> 2) & 0x3f` are fine here because `sub` is only
 *   14 bits wide; see bits.ts for why `>>`/`<<` are dangerous above 31 bits.
 * * Chained ternaries (`a ? x : b ? y : z`) are JavaScript's conditional
 *   expression; indentation makes them readable.
 */

import { Bits } from "./bits";
import { requireBits } from "./common";
import { Fixed, JsonObject } from "./jsonout";

export const NAV_STATUS: { readonly [code: number]: string } = {
  0: "under way using engine",
  1: "at anchor",
  2: "not under command",
  3: "restricted manoeuvrability",
  4: "constrained by her draught",
  5: "moored",
  6: "aground",
  7: "engaged in fishing",
  8: "under way sailing",
  9: "reserved for HSC",
  10: "reserved for WIG",
  11: "power-driven vessel towing astern",
  12: "power-driven vessel pushing ahead or towing alongside",
  13: "reserved for future use",
  14: "AIS-SART / MOB-AIS / EPIRB-AIS active",
  15: "undefined (default)",
};

export const SYNC_STATE: { readonly [code: number]: string } = {
  0: "UTC direct",
  1: "UTC indirect",
  2: "synchronized to base station",
  3: "synchronized to another station",
};

export const SPECIAL_MANOEUVRE: { readonly [code: number]: string } = {
  0: "not available", 1: "not engaged", 2: "engaged", 3: "reserved",
};

/**
 * Interpret the 8-bit rate-of-turn field (Table 48).
 *
 * The sensor value is compressed with a square root:
 *     ROT_AIS = 4.733 * sqrt(ROT_sensor)   (sign preserved)
 * so to recover degrees per minute we square: (raw / 4.733)**2.
 */
export function decodeRot(raw: number): JsonObject {
  const out: JsonObject = { rot_raw: raw };
  if (raw === -128) {
    out["rot_deg_per_min"] = null;
    out["rot_text"] = "not available";
  } else if (raw === 127) {
    out["rot_deg_per_min"] = null;
    out["rot_text"] = "turning right > 5 deg per 30 s (no turn indicator)";
  } else if (raw === -127) {
    out["rot_deg_per_min"] = null;
    out["rot_text"] = "turning left > 5 deg per 30 s (no turn indicator)";
  } else {
    // x * x rather than x ** 2: both give the correctly rounded square, and
    // multiplication makes that obvious (C's pow() is what Python calls).
    const scaled = raw / 4.733;
    let value = scaled * scaled;
    if (raw < 0) {
      value = -value;
    }
    out["rot_deg_per_min"] = new Fixed(value, 1);
    out["rot_text"] = raw > 0 ? "turning right" : raw < 0 ? "turning left" : "not turning";
  }
  return out;
}

/** SOTDMA communication state, 19 bits (Annex 2 Table 18 and 19). */
export function decodeSotdma(bits: Bits, start: number): JsonObject {
  const sync = bits.uint(start, 2);
  const timeout = bits.uint(start + 2, 3);
  const sub = bits.uint(start + 5, 14);
  const out: JsonObject = {
    scheme: "SOTDMA",
    sync_state: sync,
    sync_state_text: SYNC_STATE[sync],
    slot_timeout: timeout,
  };
  // The meaning of the 14-bit sub-message depends on the slot time-out.
  if (timeout === 3 || timeout === 5 || timeout === 7) {
    out["received_stations"] = sub;
  } else if (timeout === 2 || timeout === 4 || timeout === 6) {
    out["slot_number"] = sub;
  } else if (timeout === 1) {
    out["utc_hour"] = sub >> 9;            // bits 13..9
    out["utc_minute"] = (sub >> 2) & 0x3f; // bits 8..2
  } else { // timeout === 0
    out["slot_offset"] = sub;
  }
  return out;
}

/** ITDMA communication state, 19 bits (Annex 2 Table 20). */
export function decodeItdma(bits: Bits, start: number): JsonObject {
  const sync = bits.uint(start, 2);
  const increment = bits.uint(start + 2, 13);
  const nslots = bits.uint(start + 15, 3);
  const keep = bits.flag(start + 18);
  const out: JsonObject = {
    scheme: "ITDMA",
    sync_state: sync,
    sync_state_text: SYNC_STATE[sync],
    slot_increment: increment,
    slots_raw: nslots,
    keep_flag: keep,
  };
  // Table 20: 0..4 mean 1..5 slots; 5..7 mean 1..3 slots with +8192 offset.
  if (nslots <= 4) {
    out["slots"] = nslots + 1;
    out["slot_offset"] = increment;
  } else {
    out["slots"] = nslots - 4;
    out["slot_offset"] = increment + 8192;
  }
  return out;
}

/** Decode a message 1, 2 or 3. Throws DecodeError if too short. */
export function decode(bits: Bits): JsonObject {
  const msgType = bits.uint(0, 6);
  requireBits(msgType, bits.length, 168);
  const nav = bits.uint(38, 4);
  const sogRaw = bits.uint(50, 10);
  const lonRaw = bits.sint(61, 28);
  const latRaw = bits.sint(89, 27);
  const cogRaw = bits.uint(116, 12);
  const heading = bits.uint(128, 9);
  const timestamp = bits.uint(137, 6);
  const manoeuvre = bits.uint(143, 2);

  const out: JsonObject = {
    msg_type: msgType,
    repeat_indicator: bits.uint(6, 2),
    mmsi: bits.uint(8, 30),
    nav_status: nav,
    nav_status_text: NAV_STATUS[nav],
    sog_knots: new Fixed(sogRaw / 10.0, 1),
    sog_available: sogRaw !== 1023,
    position_accuracy: bits.uint(60, 1),
    lon: new Fixed(lonRaw / 600000.0, 6),
    lat: new Fixed(latRaw / 600000.0, 6),
    position_available: lonRaw !== 0x6791ac0 && latRaw !== 0x3412140,
    cog_deg: new Fixed(cogRaw / 10.0, 1),
    cog_available: cogRaw !== 3600,
    true_heading: heading,
    heading_available: heading !== 511,
    timestamp,
    special_manoeuvre: manoeuvre,
    special_manoeuvre_text: SPECIAL_MANOEUVRE[manoeuvre],
    spare: bits.uint(145, 3),
    raim: bits.flag(148),
  };
  Object.assign(out, decodeRot(bits.sint(42, 8)));
  if (msgType === 3) {
    out["comm_state"] = decodeItdma(bits, 149);
  } else {
    out["comm_state"] = decodeSotdma(bits, 149);
  }
  return out;
}
