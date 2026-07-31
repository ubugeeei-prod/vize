/**
 * A module shape the JS side did not derive itself -- reported by the native
 * compiler or restored from a `.vpc` cache entry -- and the two things such a
 * shape needs before `generateOutput` can act on it (#3425). Kept out of
 * `module-output.ts` so that module stays inside the per-file source-length
 * budget.
 */

import type { ModuleOutputInfo } from "./module-output.ts";

/** The offsets, the only fields the native compiler reports as optional. */
type OffsetField = {
  [Field in keyof ModuleOutputInfo]: ModuleOutputInfo[Field] extends number | null ? Field : never;
}[keyof ModuleOutputInfo];

/**
 * A shape as the native compiler reports it: `Option<u32>` crosses the NAPI
 * boundary as an absent field -- not a missing one that happens to read
 * `undefined`, an optional one -- where this module's own analysis reports
 * `null`. The flags are always present, so they are not widened.
 */
type ReportedModuleShape = Omit<ModuleOutputInfo, OffsetField> & {
  [Field in OffsetField]?: number | null;
};

/**
 * Restate a reported shape with this module's absent value, so consumers test
 * one thing rather than both `null` and `undefined` (#3425).
 */
export function normalizeModuleShape(
  shape: ReportedModuleShape | null | undefined,
): ModuleOutputInfo | undefined {
  if (shape == null) {
    return undefined;
  }

  return {
    ...shape,
    defaultExportStart: shape.defaultExportStart ?? null,
    defaultExportKeywordEnd: shape.defaultExportKeywordEnd ?? null,
    defaultExportEnd: shape.defaultExportEnd ?? null,
  };
}

/**
 * Whether a shape reported by someone else describes `code`.
 *
 * A caller that did not derive the shape itself -- `generateOutput` reading the
 * shape the native compiler reported, or one restored from a `.vpc` cache entry
 * -- cannot know it belongs to the module in hand. Every offset below is spliced
 * at verbatim, so one that is merely plausible corrupts the module with no error
 * at all. This costs a slice of two keywords rather than the parse the reported
 * shape exists to avoid, and a shape that fails it is discarded for one derived
 * from `code`.
 *
 * Absent offsets pass: `analyzeModuleOutput`'s own fast path reports a default
 * export with no end offset, and the consumers re-derive what they are missing.
 */
export function describesModule(code: string, info: ModuleOutputInfo): boolean {
  const { defaultExportStart: start, defaultExportKeywordEnd: keywordEnd } = info;
  if (start == null || keywordEnd == null) {
    return true;
  }
  if (start < 0 || keywordEnd > code.length || keywordEnd <= start) {
    return false;
  }
  if (!/^export\s+default\b/.test(code.slice(start, keywordEnd))) {
    return false;
  }

  const end = info.defaultExportEnd;
  return end == null || (end >= keywordEnd && end <= code.length);
}
