import { PNG } from "pngjs";
export { UnsupportedDiffStyleError, resolveDiffStyle } from "./diff-style-option.js";

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
