/**
 * decode.ts — command-line driver: NMEA/AIS log lines in, JSON lines out.
 *
 *     node dist/decode.js FILE [FILE...]        # or '-' for standard input
 *
 * For every input line we print at most one JSON object (see jsonout.ts for
 * why the JSON is hand-rolled). The same program exists in Python, C++ and
 * Rust, and the test-suite checks that all four print identical text.
 *
 * Output shapes
 * -------------
 * GPS sentence          {"line":7,"sentence":"GGA", ...fields...}
 * AIS message           {"line":9,"ais":{...decoded...},"channel":"A","talker":"AI",
 *                        "own_ship":false,"payload_bits":168,"fragments":1,
 *                        "uscg":{...},"tagblock":{...}}
 *                       ("uscg"/"tagblock" only when present)
 * unknown formatter     {"line":3,"address":"PMTK010","skipped":"no parser"}
 * problem               {"line":5,"error":"checksum mismatch","raw":"..."}
 * Logger comments (# ...) and blank lines print nothing.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `import * as fs from "fs"` pulls in Node's file-system module; the
 *   types come from our own node-shim.d.ts because `@types/node` is not
 *   installed. The compiled JavaScript uses `require("fs")` (CommonJS).
 * * The whole file is read as *bytes* (`readFileSync` returns a Buffer),
 *   then decoded as UTF-8 with invalid sequences replaced by U+FFFD — the
 *   same thing Python's `open(..., errors="replace")` does. Old logs
 *   contain stray bytes and both programs must survive them identically.
 * * Python iterates a text file in *universal newline* mode: `\n`, `\r\n`
 *   and a lone `\r` all end a line. `text.split(/\r\n|\r|\n/)` reproduces
 *   that; a plain `split("\n")` would not.
 * * The decoder table is a `Map<number, (bits: Bits) => JsonObject>` —
 *   functions are values, so a table of functions is just data.
 * * Output is accumulated and written in large chunks with
 *   `process.stdout.write`; `console.log` per line is correct but slow.
 */

import * as fs from "fs";

import { Bits } from "./bits";
import { DecodeError, NotImplementedError, ValueError, pyStrip } from "./common";
import { JsonObject, dumps } from "./jsonout";
import { GPS_PARSERS, NmeaError, parseSentence } from "./nmea";
import { splitTagBlock } from "./tagblock";
import { parseUscgFields } from "./uscg";
import { Assembler, Message, parseVdm } from "./vdm";

// One module per message type (or family). Each exposes decode(bits): JsonObject.
import * as msg_1_2_3 from "./msg_1_2_3";
import * as msg_4_11 from "./msg_4_11";
import * as msg_5 from "./msg_5";
import * as msg_6 from "./msg_6";
import * as msg_7_13 from "./msg_7_13";
import * as msg_8 from "./msg_8";
import * as msg_9 from "./msg_9";
import * as msg_10 from "./msg_10";
import * as msg_12 from "./msg_12";
import * as msg_14 from "./msg_14";
import * as msg_15 from "./msg_15";
import * as msg_16 from "./msg_16";
import * as msg_17 from "./msg_17";
import * as msg_18 from "./msg_18";
import * as msg_19 from "./msg_19";
import * as msg_20 from "./msg_20";
import * as msg_21 from "./msg_21";
import * as msg_22 from "./msg_22";
import * as msg_23 from "./msg_23";
import * as msg_24 from "./msg_24";
import * as msg_25 from "./msg_25";
import * as msg_26 from "./msg_26";
import * as msg_27 from "./msg_27";

type Decoder = (bits: Bits) => JsonObject;

export const DECODERS: ReadonlyMap<number, Decoder> = new Map<number, Decoder>([
  [1, msg_1_2_3.decode], [2, msg_1_2_3.decode], [3, msg_1_2_3.decode],
  [4, msg_4_11.decode], [11, msg_4_11.decode],
  [5, msg_5.decode],
  [6, msg_6.decode],
  [7, msg_7_13.decode], [13, msg_7_13.decode],
  [8, msg_8.decode],
  [9, msg_9.decode],
  [10, msg_10.decode],
  [12, msg_12.decode],
  [14, msg_14.decode],
  [15, msg_15.decode],
  [16, msg_16.decode],
  [17, msg_17.decode],
  [18, msg_18.decode],
  [19, msg_19.decode],
  [20, msg_20.decode],
  [21, msg_21.decode],
  [22, msg_22.decode],
  [23, msg_23.decode],
  [24, msg_24.decode],
  [25, msg_25.decode],
  [26, msg_26.decode],
  [27, msg_27.decode],
]);

/** Turn an assembled VDM payload into an object (or an error entry). */
export function decodeAis(msg: Message): JsonObject {
  let bits: Bits;
  try {
    bits = Bits.fromPayload(msg.payload, msg.fillBits);
  } catch (e) {
    if (e instanceof ValueError) {
      return { error: e.message };
    }
    throw e;
  }
  if (bits.length < 6) {
    return { error: `payload too short: ${bits.length} bits`, payload_bits: bits.length };
  }
  const msgType = bits.uint(0, 6);
  const decoder = DECODERS.get(msgType);
  const out: JsonObject = { payload_bits: bits.length };
  if (decoder === undefined) {
    out["error"] = `unknown message type ${msgType}`;
    out["msg_type"] = msgType;
    return out;
  }
  try {
    out["ais"] = decoder(bits);
  } catch (e) {
    if (e instanceof DecodeError) {
      out["error"] = e.message;
      out["msg_type"] = msgType;
    } else if (e instanceof NotImplementedError) {
      out["ais"] = { msg_type: msgType, not_implemented: true };
    } else {
      throw e;
    }
  }
  return out;
}

/** Handle one text line. Returns an object to print, or null to stay quiet. */
export function processLine(line: string, lineno: number, asm: Assembler): JsonObject | null {
  const stripped = pyStrip(line);
  if (!stripped || stripped.startsWith("#")) {
    return null;
  }
  const out: JsonObject = { line: lineno };

  let rest: string;
  try {
    const [tag, remainder] = splitTagBlock(stripped);
    if (tag !== null) {
      out["tagblock"] = tag.toDict();
    }
    rest = remainder;
  } catch (e) {
    if (!(e instanceof ValueError)) {
      throw e;
    }
    out["error"] = e.message;
    out["raw"] = stripped;
    return out;
  }

  let s;
  try {
    s = parseSentence(rest);
  } catch (e) {
    if (!(e instanceof NmeaError)) {
      throw e;
    }
    out["error"] = e.message;
    out["raw"] = stripped;
    return out;
  }
  if (s.afterChecksum.length > 0) {
    out["uscg"] = parseUscgFields(s.afterChecksum);
  }
  if (!s.checksumOk) {
    out["error"] = s.checksumGiven !== null ? "checksum mismatch" : "no checksum";
    out["raw"] = s.raw;
    return out;
  }

  if (s.formatter === "VDM" || s.formatter === "VDO") {
    let msg: Message | null;
    try {
      msg = asm.add(parseVdm(s));
    } catch (e) {
      if (!(e instanceof ValueError)) {
        throw e;
      }
      out["error"] = e.message;
      out["raw"] = s.raw;
      return out;
    }
    if (msg === null) {
      return null; // waiting for more fragments
    }
    Object.assign(out, decodeAis(msg));
    out["channel"] = msg.channel;
    out["talker"] = msg.talker;
    out["own_ship"] = msg.ownShip;
    out["fragments"] = msg.fragments.length;
    return out;
  }

  const parser = s.talker !== "P" ? GPS_PARSERS.get(s.formatter) : undefined;
  if (parser === undefined) {
    out["address"] = s.address;
    out["skipped"] = "no parser";
    return out;
  }
  Object.assign(out, parser(s));
  return out;
}

const USAGE = `decode.ts — command-line driver: NMEA/AIS log lines in, JSON lines out.

    node dist/decode.js FILE [FILE...]        # or '-' for standard input
`;

/** Read a whole file (or stdin for "-") as UTF-8 text, lossily. */
function readText(name: string): string {
  const buffer = fs.readFileSync(name === "-" ? 0 : name);
  return buffer.toString("utf8");
}

/** Collects output lines and writes them to stdout in big chunks. */
class Output {
  private lines: string[] = [];

  add(line: string): void {
    this.lines.push(line);
    if (this.lines.length >= 4096) {
      this.flush();
    }
  }

  flush(): void {
    if (this.lines.length > 0) {
      process.stdout.write(this.lines.join("\n") + "\n");
      this.lines = [];
    }
  }
}

/**
 * Split text into lines the way Python's text-mode file iteration does:
 * `\n`, `\r\n` and a lone `\r` all terminate a line, and a terminator at the
 * very end does not start an extra empty line (the line count must match
 * Python's across several input files).
 */
export function splitLines(text: string): string[] {
  const lines = text.split(/\r\n|\r|\n/);
  if (lines.length > 0 && lines[lines.length - 1] === "") {
    lines.pop();
  }
  return lines;
}

export function main(argv: readonly string[]): number {
  if (argv.length === 0) {
    process.stdout.write(USAGE);
    return 2;
  }
  // When piped into `head`, the reader may go away; exit quietly (Python
  // handles the equivalent BrokenPipeError the same way).
  process.stdout.on("error", (err) => {
    if (err.code === "EPIPE") {
      process.exit(0);
    }
    throw err;
  });
  const asm = new Assembler();
  const output = new Output();
  let lineno = 0;
  for (const name of argv) {
    for (const line of splitLines(readText(name))) {
      lineno += 1;
      const result = processLine(line, lineno, asm);
      if (result !== null) {
        output.add(dumps(result));
      }
    }
  }
  output.flush();
  return 0;
}

// `require.main === module` is the CommonJS idiom for Python's
// `if __name__ == "__main__"`, but it needs Node's typings. Since this file
// is only ever run as the program (never imported for its side effects), we
// simply run main.
process.exitCode = main(process.argv.slice(2));
