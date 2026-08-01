/**
 * POSIX path helpers with upstream `pathe` semantics.
 *
 * `@nuxt/eslint` builds every lint directory and glob with `pathe`, which
 * normalises Windows separators to `/` and then behaves like `node:path/posix`.
 * The generated globs are matched against POSIX-normalised paths on every
 * platform, so reproducing that normalisation is what makes the Windows output
 * identical to the Linux and macOS output.
 */
import { posix } from "node:path";

/** Replace Windows separators so POSIX path maths applies uniformly. */
function toPosix(value: string): string {
  return value.replace(/\\/g, "/");
}

/** `pathe.join` — POSIX join over separator-normalised segments. */
export function posixJoin(...segments: string[]): string {
  return posix.join(...segments.map(toPosix));
}

/** `pathe.resolve` — POSIX resolve over separator-normalised segments. */
export function posixResolve(...segments: string[]): string {
  return posix.resolve(...segments.map(toPosix));
}

/** `pathe.relative` — POSIX relative over separator-normalised paths. */
export function posixRelative(from: string, to: string): string {
  return posix.relative(toPosix(from), toPosix(to));
}
