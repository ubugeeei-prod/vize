/** Feature-detected wrapper around the EyeDropper API (not yet in every TypeScript DOM lib). */

/** Structural view of an `EyeDropper` instance. */
export interface EyeDropperLike {
  readonly open: (options?: { readonly signal?: AbortSignal }) => Promise<unknown>;
}

function isEyeDropperLike(value: unknown): value is EyeDropperLike {
  return (
    typeof value === "object" && value !== null && typeof Reflect.get(value, "open") === "function"
  );
}

/** Whether the current global scope exposes a constructible `EyeDropper`. SSR-safe. */
export function isEyeDropperSupported(scope: object = globalThis): boolean {
  return typeof Reflect.get(scope, "EyeDropper") === "function";
}

/** Construct an EyeDropper when supported, otherwise `null`. */
export function createEyeDropper(scope: object = globalThis): EyeDropperLike | null {
  const constructor: unknown = Reflect.get(scope, "EyeDropper");
  if (typeof constructor !== "function") return null;
  const instance: unknown = Reflect.construct(constructor, []);
  return isEyeDropperLike(instance) ? instance : null;
}

/** Read the `sRGBHex` string from an EyeDropper result, or `null` when malformed. */
export function readEyeDropperResult(result: unknown): string | null {
  if (typeof result !== "object" || result === null) return null;
  const hex: unknown = Reflect.get(result, "sRGBHex");
  return typeof hex === "string" ? hex : null;
}

/** Whether a rejection means the user dismissed the picker (or it was aborted). */
export function isEyeDropperAbort(error: unknown): boolean {
  return typeof error === "object" && error !== null && Reflect.get(error, "name") === "AbortError";
}
