# TypeScript version of the tutorial decoder

A line-for-line port of the Python reference in `../python/aistut/`. It
prints byte-identical JSON so that `tests/run_tests.sh typescript` can diff
it against `tests/expected/*.jsonl`.

No npm packages, no `package.json`, no network: the compiler is the
globally installed `tsc` (6.x) and the runtime is plain `node` (22+). Even
`@types/node` is absent, so the handful of Node APIs we touch are declared
by hand in `src/node-shim.d.ts`.

## Build and run

```sh
cd code/typescript
tsc -p tsconfig.json                 # src/*.ts -> dist/*.js (zero errors under "strict")
node dist/decode.js ../../data/ch03-position-reports.nmea
cat FILE | node dist/decode.js -     # '-' reads standard input
```

From the repository root: `tests/run_tests.sh typescript`.

## Module map

| Module | Mirrors | What it does | What it shows a TypeScript newcomer |
|---|---|---|---|
| `jsonout.ts` | `jsonout.py` | `Fixed` and `dumps()`: sorted keys, no whitespace, fixed decimals | union types (`JsonValue`), index signatures, `class` with `readonly` parameter properties, `toFixed` and its round-half-up quirk |
| `common.ts` | `common.py` | `DecodeError`, `NotImplementedError`, `requireBits`, EPFD and ship-type tables, Python-compatible `strip`/`int`/`float`/`repr` helpers | `extends Error`, `catch (e: unknown)` + `instanceof`, lookup objects vs `Map`, template literals, tuple types |
| `nmea.ts` | `nmea.py` | sentence splitting, checksum, GGA/GSA/GSV/RMC | `class` vs `interface`, getters, `number \| null`, `\|\|` vs `??`, destructuring with rest (`[address, ...fields]`), function type aliases |
| `bits.ts` | `bits.py` | armour decoding, `Bits` over a `Uint8Array` with `uint/sint/flag/text/raw/has/length` | typed arrays, `static` factories, **the 32-bit bitwise pitfall**, `2 ** n` |
| `vdm.ts` | `vdm.py` | `Vdm`, `Message`, `Assembler` | interfaces for plain data, `Map<string, Vdm[]>` with a string key built by `JSON.stringify` (tuples are not usable as keys) |
| `msg_1_2_3.ts` | `msg_1_2_3.py` | position reports 1/2/3 | object literals checked against an interface, `Object.assign` as `dict.update` |
| `tagblock.ts` | `tagblock.py` | NMEA 4.10 TAG blocks | getters, `Object.create(null)` for input-keyed dictionaries, tuple return types |
| `uscg.ts` | `uscg.py` | USCG metadata after the checksum | `try/catch` without exception filters, string slicing |
| `commstate.ts` | `commstate.py` | TDMA communication-state schedule and UTC clock helpers | `export { ... } from` re-exports, `Math.trunc`, default nullable parameters |
| `msg_4_11.ts` … `msg_27.ts` | the stubs | `decode()` throws `NotImplementedError` | a `throw` satisfies a non-`void` return type; type-only imports |
| `decode.ts` | `decode.py` | CLI: reads whole files as bytes, splits lines like Python, prints JSON lines | `import * as fs`, `Map` of functions, chunked `process.stdout.write`, EPIPE handling |
| `node-shim.d.ts` | — | minimal ambient typings for `fs`, `process`, `Buffer` | ambient module declarations, `declare const`, `never` |

## Language notes worth a paragraph in each chapter

* **Interfaces vs classes.** `Vdm` and `Message` are interfaces: object
  literals satisfy them structurally, nothing exists at run time. `Sentence`,
  `TagBlock` and `Bits` are classes because they carry behaviour (getters,
  methods). `instanceof` only works for classes.
* **`null`, `undefined` and union types.** "No data" is always `null`
  (`number | null`), matching JSON. `undefined` is what you get from a
  missing object key or `Map.get`; the writer prints both as `null` but the
  types keep them apart so that `x + 1` on a maybe-missing value is a
  compile error until you check.
* **`readonly`** is a compile-time promise only; `Object.freeze` would be
  the run-time version. `readonly string[]` is an array you may read but not
  `push` to.
* **Enums vs lookup objects.** The tutorial uses plain
  `{ readonly [code: number]: string }` objects (and `Map` where the key
  comes from input text) rather than TypeScript `enum`s, because the JSON
  needs both the integer *and* the text, and because an `enum` is an
  additional language feature that compiles to surprising JavaScript.
* **Template literals** (`` `message ${n} too short` ``) are f-strings.
* **`Map`** keys are compared by identity, so an array `[talker, channel,
  seq]` can never be looked up again; the `Assembler` stringifies the tuple.
  Plain objects are fine for fixed keys, but keys that come from the input
  (`"constructor"`, `"__proto__"`) collide with `Object.prototype`, hence
  `Map` for `GPS_PARSERS`/`RMC_MODE` and `Object.create(null)` for TAG-block
  parameters.
* **Bitwise operators are 32-bit.** `<<`, `>>`, `|`, `&`, `^` coerce to a
  signed 32-bit integer. A 30-bit MMSI fits, but `1 << 31` is negative and
  `x << 30` can lose bits, so `Bits.uint()` accumulates with
  `value * 2 + bit` and `sint()` subtracts `2 ** length`. Shifts are used
  only on values known to be narrow (6-bit armour values, the 14-bit SOTDMA
  sub-message).
* **`Number` vs `BigInt`.** `number` is a 64-bit double: exact for integers
  up to 2^53 − 1 (`Number.MAX_SAFE_INTEGER`), which covers every AIS field
  and every UNIX timestamp we meet. Values beyond that need `BigInt`
  (`123n`), which cannot be mixed with `number` in arithmetic and which
  `JSON.stringify` refuses to print — another reason the writer is
  hand-rolled.
* **`toFixed` rounding.** `x.toFixed(n)` prints the exact binary value
  correctly rounded, like C's `%.nf`, except that exact ties round *up*
  rather than to even (`(1.125).toFixed(2)` is `"1.13"`, Python prints
  `1.12`). `formatFixed` in `jsonout.ts` detects the tie
  (`x * 2**(n+1)` is an odd integer) and makes the last digit even, so the
  four languages agree. `-0.0` is normalised to `0.0`.
* **Python look-alikes that are not.** `Number("")` is `0`,
  `parseInt("12abc")` is `12`, `"a".trim()` strips a slightly different
  whitespace set than `str.strip()`, and `"x".split("\n")` ignores lone
  `\r`. `common.ts` and `decode.ts` reproduce the Python rules because the
  error text for malformed lines (`invalid literal for int() ...`) is part
  of the output contract.
