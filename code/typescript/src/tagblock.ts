/**
 * tagblock.ts — NMEA 0183 v4.10 TAG blocks (chapter 4).
 *
 * A TAG block is metadata that *precedes* a sentence on the same line:
 *
 *     \s:r003669945,c:1224006993*2B\!AIVDM,1,1,,A,15N4cJ`005Jrek0H@9n`DW5608EP,0*13
 *     ^                            ^ ^
 *     backslash  parameter list   checksum of the text between the backslashes
 *
 * Defined in IEC 61162-1 ed. 4 / NMEA 0183 v4.10 §7.1.2 "TAG block".
 * Parameters are `code:value` pairs separated by commas:
 *
 *     c:  UNIX time (seconds, or milliseconds if > 10^10 or configured so)
 *     d:  destination identification (≤ 15 chars)
 *     g:  sentence grouping "n-total-group_id" for multi-sentence groups
 *     n:  line count (an incrementing serial number)
 *     r:  relative time
 *     s:  source identification (≤ 15 chars) — the receiving station
 *     t:  free text (≤ 15 chars)
 *
 * The checksum is computed exactly like a sentence checksum: XOR of all
 * characters between the two backslashes, excluding the `*hh`.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `get time(): number | null { ... }` defines a *getter*: it is read like
 *   a field (`tag.time`) but computed on each access — Python's `@property`.
 * * The parameter dictionary is created with `Object.create(null)`, an
 *   object with *no prototype*. A normal `{}` inherits `toString`,
 *   `constructor` and friends, and assigning to the key `"__proto__"` would
 *   change the prototype instead of storing a value. Input-derived keys
 *   deserve the paranoia; our own fixed keys do not.
 * * `[number, number, number] | null` is a *tuple type* or null.
 * * `splitTagBlock` returns a two-element tuple so the caller can write
 *   `const [tag, rest] = splitTagBlock(line)`.
 */

import { JsonObject } from "./jsonout";
import { checksum } from "./nmea";
import { ValueError, isDigits, partition, pyHexInt, pyStrip } from "./common";

export class TagBlock {
  constructor(
    readonly params: { readonly [code: string]: string },
    readonly checksumOk: boolean = false,
    readonly raw: string = "",
  ) {}

  // Typed views of the common parameters. Missing -> null.
  get time(): number | null {
    const c = this.params["c"];
    return c !== undefined && c !== "" && isDigits(c) ? Number(c) : null;
  }

  get source(): string | null {
    return this.params["s"] ?? null;
  }

  /** (sentence number, total sentences, group id) from the g: parameter. */
  get group(): [number, number, number] | null {
    const g = this.params["g"];
    if (g === undefined || g === "") {
      return null;
    }
    const parts = g.split("-");
    if (parts.length !== 3 || !parts.every(isDigits)) {
      return null;
    }
    return [Number(parts[0]), Number(parts[1]), Number(parts[2])];
  }

  toDict(): JsonObject {
    // Copy the params into a fresh object so the JSON writer sees plain data.
    const params: JsonObject = Object.create(null);
    for (const code of Object.keys(this.params)) {
      params[code] = this.params[code];
    }
    const d: JsonObject = { params, checksum_ok: this.checksumOk };
    const time = this.time;
    if (time !== null) {
      d["time"] = time;
    }
    const group = this.group;
    if (group !== null) {
      d["group"] = [...group];
    }
    return d;
  }
}

/**
 * Separate a leading TAG block from the sentence that follows it.
 *
 * Returns [TagBlock or null, remainder]. Lines without a leading
 * backslash are returned unchanged.
 */
export function splitTagBlock(rawLine: string): [TagBlock | null, string] {
  const line = pyStrip(rawLine);
  if (!line.startsWith("\\")) {
    return [null, line];
  }
  const end = line.indexOf("\\", 1);
  if (end < 0) {
    throw new ValueError("unterminated TAG block");
  }
  const inner = line.slice(1, end);
  const [body, star, hexpart] = partition(inner, "*");
  let ok = false;
  if (star && hexpart.length >= 2) {
    try {
      ok = pyHexInt(hexpart.slice(0, 2)) === checksum(body);
    } catch (e) {
      if (!(e instanceof ValueError)) {
        throw e;
      }
      ok = false;
    }
  }
  const params: { [code: string]: string } = Object.create(null);
  for (const item of body.split(",")) {
    const [code, colon, value] = partition(item, ":");
    if (colon) {
      params[code] = value;
    }
  }
  return [new TagBlock(params, ok, inner), line.slice(end + 1)];
}

/**
 * Build a TAG block string from parameters (used in chapter 4 to convert
 * USCG-format lines into TAG block lines).
 */
export function formatTagBlock(params: { readonly [code: string]: string }): string {
  const body = Object.keys(params).map((k) => `${k}:${params[k]}`).join(",");
  return `\\${body}*${checksum(body).toString(16).toUpperCase().padStart(2, "0")}\\`;
}
