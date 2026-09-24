import type { ImageCompareDirection, ImageCompareOrientation } from "./image-compare-types.ts";

/** Axis-aligned box read from `getBoundingClientRect()`. */
export interface ImageCompareRect {
  readonly left: number;
  readonly top: number;
  readonly width: number;
  readonly height: number;
}

/** Axis options shared by the helpers. */
export interface ImageCompareAxis {
  readonly orientation: ImageCompareOrientation;
  readonly dir: ImageCompareDirection;
}

/** Keyboard step options for {@link imageComparePositionForKey}. */
export interface ImageCompareKeyOptions extends ImageCompareAxis {
  readonly step: number;
  readonly pageStep: number;
}

function decimals(value: number): number {
  const text = String(value);
  const exponent = /e-(\d+)$/i.exec(text);
  if (exponent?.[1] !== undefined) return Number(exponent[1]);
  return text.split(".")[1]?.length ?? 0;
}

/**
 * Clamp a position into `[0, 100]` and snap it to `step`. Non-finite input
 * resolves to `0`; a non-positive or non-finite step disables snapping.
 */
export function normalizeImageComparePosition(position: number, step = 0): number {
  if (!Number.isFinite(position)) return 0;
  const clamped = Math.min(100, Math.max(0, position));
  if (!(step > 0) || !Number.isFinite(step)) return clamped;
  const snapped = Math.round(clamped / step) * step;
  return Math.min(100, Math.max(0, Number(snapped.toFixed(decimals(step)))));
}

/**
 * Position under a pointer. Horizontal dividers measure from the reading-start
 * edge (left in LTR, right in RTL); vertical dividers measure from the top.
 * An unmeasured (zero-size) rect returns `null`.
 */
export function imageComparePositionFromPoint(
  rect: ImageCompareRect,
  clientX: number,
  clientY: number,
  axis: ImageCompareAxis,
): number | null {
  if (axis.orientation === "vertical") {
    if (!(rect.height > 0)) return null;
    return ((clientY - rect.top) / rect.height) * 100;
  }
  if (!(rect.width > 0)) return null;
  const fromLeft = ((clientX - rect.left) / rect.width) * 100;
  return axis.dir === "rtl" ? 100 - fromLeft : fromLeft;
}

/**
 * Next position for a slider key, following the WAI-ARIA slider pattern:
 * arrows move by `step`, Page keys by `pageStep`, Home/End jump to the ends.
 * Horizontal arrows follow the reading direction. Returns `null` for other keys.
 */
export function imageComparePositionForKey(
  key: string,
  current: number,
  options: ImageCompareKeyOptions,
): number | null {
  const rtl = options.orientation === "horizontal" && options.dir === "rtl";
  switch (key) {
    case "ArrowRight":
      return current + (rtl ? -options.step : options.step);
    case "ArrowLeft":
      return current - (rtl ? -options.step : options.step);
    case "ArrowUp":
      return current - options.step;
    case "ArrowDown":
      return current + options.step;
    case "PageUp":
      return current + options.pageStep;
    case "PageDown":
      return current - options.pageStep;
    case "Home":
      return 0;
    case "End":
      return 100;
    default:
      return null;
  }
}
