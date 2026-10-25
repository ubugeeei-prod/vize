import { PNG } from "pngjs";
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

/** Exact baseline/current RGBA, left/right, with transparent bottom padding. */
export function sideBySidePng(baseline: PNG, current: PNG): PNG {
  const output = new PNG({
    width: baseline.width + current.width,
    height: Math.max(baseline.height, current.height),
  });
  for (const [image, offset] of [
    [baseline, 0],
    [current, baseline.width],
  ] as const) {
    for (let y = 0; y < image.height; y++) {
      image.data.copy(
        output.data,
        (y * output.width + offset) * 4,
        y * image.width * 4,
        (y + 1) * image.width * 4,
      );
    }
  }
  return output;
}
