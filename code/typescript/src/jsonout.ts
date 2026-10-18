/**
 * jsonout.ts — a tiny, deterministic JSON writer.
 *
 * Why not `JSON.stringify`? Because the Python, C++ and Rust versions of
 * this tutorial must print *byte-identical* output so that the test-suite
 * can diff them. `JSON.stringify` keeps object keys in insertion order and
 * prints numbers with the shortest round-trip representation, neither of
 * which the other languages reproduce. So we follow two rules everywhere:
 *
 *   1. Object keys are written in sorted order.
 *   2. Every non-integer number is wrapped in `Fixed(value, decimals)` and
 *      printed with exactly that many decimals, rounding the exact binary
 *      value the way C's printf and Python's `format(x, '.6f')` do.
 *
 * Supported values: null, boolean, integer number, Fixed, string, array,
 * plain object. (`undefined` is also printed as `null` for convenience.)
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `type JsonValue = ... | JsonValue[] | JsonObject` is a *union type*: a
 *   value may be any one of the listed alternatives. The compiler forces
 *   you to check which one you have before using it (`typeof`, `instanceof`,
 *   `Array.isArray`) — a pattern called *narrowing*.
 * * `interface JsonObject { [key: string]: JsonValue }` is an *index
 *   signature*: an object with any string keys, each holding a JsonValue.
 * * `class Fixed` with `constructor(readonly value: number, ...)` uses a
 *   *parameter property*: declaring the constructor argument with
 *   `readonly` (or `public`/`private`) creates and assigns the field in one
 *   step.
 * * JavaScript has one number type, `number`, a 64-bit IEEE double. There
 *   is no separate integer type, so `dumps` decides "is this an integer?"
 *   with `Number.isInteger` at run time.
 * * `x.toFixed(n)` is JavaScript's `%.nf`. It rounds exact ties *up*,
 *   whereas C and Python round them to *even*; `formatFixed` below corrects
 *   that rare case so the four languages agree.
 */

/** Any value the writer knows how to print. */
export type JsonValue =
  | null
  | boolean
  | number
  | string
  | Fixed
  | JsonValue[]
  | JsonObject;

/** A plain object with string keys — what every decoder returns. */
export interface JsonObject {
  [key: string]: JsonValue;
}

/** A float to be printed with a fixed number of decimals. */
export class Fixed {
  constructor(
    readonly value: number,
    readonly decimals: number,
  ) {}

  toString(): string {
    return formatFixed(this.value, this.decimals);
  }
}

/**
 * Format `x` with `decimals` places exactly like C's `printf("%.*f")`.
 *
 * `toFixed` already prints the *exact* decimal expansion of the double,
 * correctly rounded, with one difference: when the value sits exactly
 * half-way between two candidates (e.g. 1.125 printed with 2 decimals)
 * JavaScript picks the larger magnitude ("1.13") while C/Python/Rust pick
 * the even digit ("1.12"). Such exact ties can only happen when
 * `x * 2**(decimals+1)` is an odd integer, which is cheap to test, and in
 * that case the fix is to make the last digit even.
 */
export function formatFixed(x: number, decimals: number): string {
  if (!Number.isFinite(x)) {
    // Python prints inf/nan this way; JavaScript would say "Infinity"/"NaN".
    return Number.isNaN(x) ? "nan" : x < 0 ? "-inf" : "inf";
  }
  let s = x.toFixed(decimals);
  const scaled = x * 2 ** (decimals + 1); // exact: multiplying by a power of 2
  if (Number.isInteger(scaled) && Math.abs(scaled) % 2 === 1) {
    const last = s.charCodeAt(s.length - 1) - 48; // '0' is 48
    if (last % 2 === 1) {
      s = s.slice(0, -1) + String(last - 1); // round half to even
    }
  }
  if (s.startsWith("-") && Number(s) === 0) {
    s = s.slice(1); // avoid "-0.0"
  }
  return s;
}

function escape(s: string): string {
  let out = '"';
  // `for...of` on a string iterates Unicode code points, like Python does.
  for (const ch of s) {
    if (ch === '"') {
      out += '\\"';
    } else if (ch === "\\") {
      out += "\\\\";
    } else if (ch === "\n") {
      out += "\\n";
    } else if (ch === "\r") {
      out += "\\r";
    } else if (ch === "\t") {
      out += "\\t";
    } else if ((ch.codePointAt(0) ?? 0) < 0x20) {
      out += "\\u" + (ch.codePointAt(0) ?? 0).toString(16).padStart(4, "0");
    } else {
      out += ch;
    }
  }
  return out + '"';
}

/** Serialise `value` to compact JSON with sorted keys. */
export function dumps(value: JsonValue | undefined): string {
  if (value === null || value === undefined) {
    return "null";
  }
  if (value === true) {
    return "true";
  }
  if (value === false) {
    return "false";
  }
  if (typeof value === "number") {
    if (!Number.isInteger(value)) {
      throw new TypeError("wrap floats in new Fixed(value, decimals) for deterministic output");
    }
    return String(value);
  }
  if (value instanceof Fixed) {
    return value.toString();
  }
  if (typeof value === "string") {
    return escape(value);
  }
  if (Array.isArray(value)) {
    return "[" + value.map(dumps).join(",") + "]";
  }
  if (typeof value === "object") {
    // Python sorts by code point; the default string comparison in JS sorts
    // by UTF-16 code unit. For ASCII keys (all of ours) they agree.
    const keys = Object.keys(value).sort();
    return "{" + keys.map((k) => escape(k) + ":" + dumps(value[k])).join(",") + "}";
  }
  throw new TypeError(`cannot serialise ${typeof value}`);
}
