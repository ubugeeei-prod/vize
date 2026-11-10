import type { DiffStyle } from "../types/index.js";

/** A requested output format cannot be produced by the PNG comparison pipeline. */
export class UnsupportedDiffStyleError extends Error {
  readonly code = "ERR_MUSEA_UNSUPPORTED_DIFF_STYLE";

  constructor(readonly style: unknown) {
    super(`Unsupported VRT diff style: ${String(style)}. Use overlay, sideBySide, or diffOnly.`);
    this.name = "UnsupportedDiffStyleError";
  }
}

/** Validate before file access, baseline writes, or identical-image shortcuts. */
export function resolveDiffStyle(style: unknown): DiffStyle {
  if (style === undefined) return "overlay";
  if (style === "overlay" || style === "sideBySide" || style === "diffOnly") return style;
  throw new UnsupportedDiffStyleError(style);
}
