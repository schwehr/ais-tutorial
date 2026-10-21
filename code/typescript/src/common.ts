/**
 * common.ts — small things shared by several message decoders.
 *
 * Besides the error classes and the EPFD / ship-type tables that the Python
 * version has, this module also holds a few helpers that make TypeScript
 * behave like the Python reference on *odd* input: Python's `str.strip()`,
 * `int()`, `float()` and `repr()` are stricter (or looser) than their
 * JavaScript look-alikes, and since the two programs must print identical
 * error text for the same garbage line, we spell the Python rules out.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `class DecodeError extends Error` is how you define your own exception
 *   type. There is no checked-exception list; any value can be thrown, and
 *   in a `catch (e)` block `e` has type `unknown` under `strict`, so you
 *   narrow it with `instanceof` before touching its fields.
 * * A `{ readonly [code: number]: string }` type is a lookup object keyed
 *   by number. Indexing with a key that is missing yields `undefined` at
 *   run time even though the type says `string` — that is a well-known gap
 *   in TypeScript's type system; we use `??` (nullish coalescing) to supply
 *   a default where the Python code used `dict.get(key, default)`.
 * * Template literals — `` `message ${n} too short` `` — are the
 *   equivalent of Python f-strings.
 */

/** Mirrors Python's ValueError: "the input text is not what I expected". */
export class ValueError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ValueError";
  }
}

/**
 * A message could not be decoded (usually: too few bits).
 *
 * All four language versions print the same text for this situation so
 * that their outputs can be compared:  "message 5 too short: 160 bits < 424"
 */
export class DecodeError extends ValueError {
  constructor(
    readonly msgType: number,
    readonly needBits: number,
    readonly haveBits: number,
  ) {
    super(`message ${msgType} too short: ${haveBits} bits < ${needBits}`);
    this.name = "DecodeError";
  }
}

/** Thrown by the stub decoders; the driver reports `not_implemented`. */
export class NotImplementedError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "NotImplementedError";
  }
}

export function requireBits(msgType: number, bitsLen: number, need: number): void {
  if (bitsLen < need) {
    throw new DecodeError(msgType, need, bitsLen);
  }
}

// Type of electronic position fixing device (EPFD). ITU-R M.1371-5 Annex 8
// Table 51 (message 4); the same code list is used in messages 5, 9, 11,
// 19, 21 and 24.
export const EPFD_TYPES: { readonly [code: number]: string } = {
  0: "undefined",
  1: "GPS",
  2: "GLONASS",
  3: "combined GPS/GLONASS",
  4: "Loran-C",
  5: "Chayka",
  6: "integrated navigation system",
  7: "surveyed",
  8: "Galileo",
  15: "internal GNSS",
};

export function epfdText(code: number): string {
  return EPFD_TYPES[code] ?? "not used";
}

// Type of ship and cargo type. ITU-R M.1371-5 Annex 8 Table 53. The first
// digit is the vessel category, the second digit (for 2x–9x) the hazard
// category (1–4 = DG/HS/MP category X, Y, Z, OS — formerly A–D, which is
// what the `_text` strings still say; 5–8 reserved, 9 = no additional
// information). We spell out the common ones and derive the rest.
export const SHIP_TYPE_FIRST_DIGIT: { readonly [digit: number]: string } = {
  2: "WIG", 3: "", 4: "HSC", 5: "", 6: "passenger", 7: "cargo", 8: "tanker", 9: "other",
};
export const SHIP_TYPE_SPECIAL: { readonly [code: number]: string } = {
  0: "not available (default)",
  30: "fishing", 31: "towing", 32: "towing, length > 200 m or breadth > 25 m",
  33: "dredging or underwater operations", 34: "diving operations",
  35: "military operations", 36: "sailing", 37: "pleasure craft",
  38: "reserved", 39: "reserved",
  50: "pilot vessel", 51: "search and rescue vessel", 52: "tug", 53: "port tender",
  54: "anti-pollution equipment", 55: "law enforcement", 56: "spare - local vessel",
  57: "spare - local vessel", 58: "medical transport", 59: "noncombatant ship (RR Res. 18)",
};
export const HAZARD_DIGIT: { readonly [digit: number]: string } = {
  0: "all ships of this type", 1: "hazardous category A", 2: "hazardous category B",
  3: "hazardous category C", 4: "hazardous category D", 5: "reserved", 6: "reserved",
  7: "reserved", 8: "reserved", 9: "no additional information",
};

export function shipTypeText(code: number): string {
  // `in` tests for the presence of a key, like Python's `code in dict`.
  if (code in SHIP_TYPE_SPECIAL) {
    return SHIP_TYPE_SPECIAL[code];
  }
  if (1 <= code && code <= 19) {
    return "reserved for future use";
  }
  if (100 <= code && code <= 199) {
    return "reserved for regional use";
  }
  if (code >= 200) {
    return "reserved for future use";
  }
  const tens = Math.floor(code / 10); // Python's divmod(code, 10)
  const ones = code % 10;
  if (tens in SHIP_TYPE_FIRST_DIGIT && tens !== 3 && tens !== 5) {
    return `${SHIP_TYPE_FIRST_DIGIT[tens]}, ${HAZARD_DIGIT[ones]}`;
  }
  return "reserved";
}

// --- Python-compatibility helpers -------------------------------------------
//
// The Python reference is the specification for what gets printed, so the
// TypeScript port must copy Python's text-handling quirks exactly.

/**
 * The characters Python's `str.strip()` removes. JavaScript's `trim()` is
 * close but not identical: it also removes U+FEFF and does *not* remove the
 * ASCII separators U+001C–U+001F or U+0085, so we write our own.
 */
const PY_WHITESPACE = "\t\n\v\f\r \x1c\x1d\x1e\x1f\x85\xa0\u1680\u2000\u2001\u2002" +
  "\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200a\u2028\u2029\u202f\u205f\u3000";

/** Python's `s.strip()`. */
export function pyStrip(s: string): string {
  let start = 0;
  let end = s.length;
  while (start < end && PY_WHITESPACE.includes(s[start])) {
    start++;
  }
  while (end > start && PY_WHITESPACE.includes(s[end - 1])) {
    end--;
  }
  return s.slice(start, end);
}

/**
 * Python's `repr()` of a string, used in a few error messages. Python
 * prefers single quotes, switches to double quotes if the text contains a
 * single quote but no double quote, and escapes control characters as
 * `\xNN`.
 */
export function pyRepr(s: string): string {
  const quote = s.includes("'") && !s.includes('"') ? '"' : "'";
  let out = quote;
  for (const ch of s) {
    const cp = ch.codePointAt(0) ?? 0;
    if (ch === quote || ch === "\\") {
      out += "\\" + ch;
    } else if (ch === "\n") {
      out += "\\n";
    } else if (ch === "\r") {
      out += "\\r";
    } else if (ch === "\t") {
      out += "\\t";
    } else if (cp < 0x20 || (0x7f <= cp && cp <= 0xa0) || cp === 0xad) {
      out += "\\x" + cp.toString(16).padStart(2, "0");
    } else {
      out += ch;
    }
  }
  return out + quote;
}

/**
 * Python's `int(s)`: optional surrounding whitespace, optional sign, ASCII
 * digits (underscores allowed between digit groups). Anything else raises
 * ValueError with Python's exact wording — that text can end up in the JSON
 * output, so it has to match. Beware: `Number("")` is 0 and `parseInt("12abc")`
 * is 12 in JavaScript; neither is what we want.
 */
export function pyInt(s: string): number {
  const t = pyStrip(s);
  if (!/^[+-]?[0-9]+(_[0-9]+)*$/.test(t)) {
    throw new ValueError(`invalid literal for int() with base 10: ${pyRepr(s)}`);
  }
  return Number(t.replace(/_/g, ""));
}

/** Python's `int(s, 16)` restricted to what the checksum code needs. */
export function pyHexInt(s: string): number {
  const t = pyStrip(s);
  if (!/^[+-]?[0-9a-fA-F]+(_[0-9a-fA-F]+)*$/.test(t)) {
    throw new ValueError(`invalid literal for int() with base 16: ${pyRepr(s)}`);
  }
  return parseInt(t.replace(/_/g, ""), 16);
}

/** Python's `float(s)`. Rejects what Python rejects (e.g. "0x10", "abc"). */
export function pyFloat(s: string): number {
  const t = pyStrip(s).replace(/_/g, "");
  if (/^[+-]?(inf|infinity)$/i.test(t)) {
    return t.startsWith("-") ? -Infinity : Infinity;
  }
  if (/^[+-]?nan$/i.test(t)) {
    return NaN;
  }
  if (!/^[+-]?([0-9]+\.?[0-9]*|\.[0-9]+)([eE][+-]?[0-9]+)?$/.test(t)) {
    throw new ValueError(`could not convert string to float: ${pyRepr(s)}`);
  }
  return Number(t);
}

/** Python's `str.isdigit()` for the ASCII digits we care about. */
export function isDigits(s: string): boolean {
  return /^[0-9]+$/.test(s);
}

/**
 * Python's `s.partition(sep)`: split at the *first* occurrence into
 * [before, sep, after]; if `sep` is absent, [s, "", ""]. Returned as a
 * *tuple type* `[string, string, string]` — an array whose length and
 * element types are fixed, handy for destructuring.
 */
export function partition(s: string, sep: string): [string, string, string] {
  const at = s.indexOf(sep);
  if (at < 0) {
    return [s, "", ""];
  }
  return [s.slice(0, at), sep, s.slice(at + sep.length)];
}
