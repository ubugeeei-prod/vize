/**
 * Shared erasable types and JSON narrowing guards for the benchmark scripts.
 *
 * `JSON.parse`, `createRequire()` and dynamic `import()` all return `any`.
 * Typed benchmark scripts assign those results to `unknown` and narrow them
 * with these guards instead of casting them (#8840), so a malformed artifact
 * is rejected where it is read rather than typed as whatever it claims to be.
 */

/** Options parsed from `--key value` pairs; a bare `--flag` maps to `"true"`. */
export type CliArgs = Record<string, string | undefined>;

/** A JSON object: neither `null` nor an array. */
export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** An array whose every element is a string. */
export function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((entry: unknown) => typeof entry === "string");
}

/** `undefined`, or a value that `guard` accepts. */
export function isOptional(value: unknown, guard: (value: unknown) => boolean): boolean {
  return value === undefined || guard(value);
}
