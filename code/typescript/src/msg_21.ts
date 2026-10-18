/**
 * msg_21.ts — message 21: aids-to-navigation report.
 *
 * Spec: ITU-R M.1371-5 Annex 8 §3.19, Table 71.
 *
 * STUB: this decoder has not been written yet. The chapter that covers this
 * message replaces this file. Keep the signature `decode(bits: Bits): JsonObject`.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * A function whose declared return type is `JsonObject` may end in a
 *   `throw` without returning anything: the compiler sees that the end of
 *   the body is unreachable. The driver (decode.ts) catches
 *   `NotImplementedError` and reports `{"msg_type":N,"not_implemented":true}`.
 * * `import { Bits } from "./bits"` only imports a *type* here. Under
 *   `verbatimModuleSyntax` you would write `import type`; with the default
 *   settings the compiler simply drops the import from the emitted JS.
 */

import { Bits } from "./bits";
import { NotImplementedError } from "./common";
import { JsonObject } from "./jsonout";

export function decode(bits: Bits): JsonObject {
  void bits; // unused until the decoder is written
  throw new NotImplementedError("message 21 decoder not written yet");
}
