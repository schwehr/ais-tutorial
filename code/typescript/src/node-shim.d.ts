/**
 * node-shim.d.ts — the few pieces of Node.js we use, declared by hand.
 *
 * This tutorial has no npm dependencies, and that includes `@types/node`
 * (the community-maintained type declarations for Node's built-in modules).
 * Without it TypeScript knows nothing about `process` or the `fs` module.
 * A `.d.ts` file contains *only* type declarations (no code is emitted for
 * it), so here we describe the handful of things decode.ts touches.
 *
 * TypeScript notes for readers new to the language
 * ------------------------------------------------
 * * `declare module "fs" { ... }` is an *ambient module declaration*: it
 *   tells the compiler "when somebody imports "fs", these are its exports".
 *   The real implementation is supplied by Node at run time.
 * * `declare const process: {...}` declares a global variable that exists
 *   at run time but is defined somewhere the compiler cannot see.
 * * `readonly` on a property means "cannot be assigned to", nothing more; it
 *   is a compile-time check, not run-time immutability.
 * * `never` as a return type means the function does not return at all
 *   (it exits the process or throws).
 */

/** The subset of Node's `Buffer` that we rely on: it *is* a Uint8Array. */
interface NodeBuffer extends Uint8Array {
  /** Decode the bytes as text. Invalid UTF-8 becomes U+FFFD, like Python's errors="replace". */
  toString(encoding?: "utf8" | "latin1"): string;
}

declare module "fs" {
  /** Read a whole file. A number is a file descriptor (0 = standard input). */
  export function readFileSync(pathOrFd: string | number): NodeBuffer;
}

declare const process: {
  /** ["node", "/path/to/decode.js", ...user arguments] */
  readonly argv: readonly string[];
  /** Set this instead of calling exit() so buffered output is flushed first. */
  exitCode: number | undefined;
  exit(code?: number): never;
  readonly stdout: {
    write(chunk: string): boolean;
    /** Node streams report problems (e.g. EPIPE) through an "error" event. */
    on(event: "error", listener: (err: { code?: string }) => void): void;
  };
  readonly stderr: {
    write(chunk: string): boolean;
  };
};
