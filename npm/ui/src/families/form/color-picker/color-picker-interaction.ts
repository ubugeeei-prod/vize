/** Pure pointer and keyboard helpers shared by ColorPickerArea and ColorPickerChannelSlider. */

import type { ColorChannelRange } from "./color-picker-color.ts";
import type { ColorPickerDirection, ColorPickerOrientation } from "./color-picker-types.ts";

/** Minimal rectangle read from `getBoundingClientRect()`. */
export interface ColorPickerRect {
  readonly left: number;
  readonly top: number;
  readonly width: number;
  readonly height: number;
}

/** Pointer position normalized to fractions of a rectangle, clamped to `[0, 1]`. */
export interface ColorPickerFraction {
  /** Horizontal fraction in reading direction (`0` = inline start). */
  readonly x: number;

  /** Vertical fraction from the bottom edge (`0` = bottom, `1` = top). */
  readonly y: number;
}

function clampUnit(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.min(1, Math.max(0, value));
}

/** Convert client coordinates into clamped fractions of `rect`. */
export function pointerFraction(
  rect: ColorPickerRect,
  clientX: number,
  clientY: number,
  dir: ColorPickerDirection = "ltr",
): ColorPickerFraction {
  const rawX = rect.width > 0 ? (clientX - rect.left) / rect.width : 0;
  const rawY = rect.height > 0 ? (clientY - rect.top) / rect.height : 0;
  const x = clampUnit(rawX);
  return {
    x: dir === "rtl" ? 1 - x : x,
    y: 1 - clampUnit(rawY),
  };
}

/** Map a unit fraction onto a channel range. */
export function fractionToValue(range: ColorChannelRange, fraction: number): number {
  return range.min + (range.max - range.min) * clampUnit(fraction);
}

/** Map a channel value onto a `0`–`100` percentage of its range. */
export function valueToPercent(range: ColorChannelRange, value: number): number {
  const span = range.max - range.min;
  if (span <= 0) return 0;
  return Math.round(clampUnit((value - range.min) / span) * 10000) / 100;
}

/** Keyboard intent resolved from one keydown on a color slider thumb. */
export type ColorPickerKeyIntent =
  | { readonly kind: "delta"; readonly axis: "x" | "y"; readonly amount: number }
  | { readonly kind: "edge"; readonly axis: "x" | "y"; readonly edge: "max" | "min" };

/**
 * Resolve a keydown into a slider intent, or `null` for unrelated keys.
 *
 * - Arrow keys move one `step` (Shift: one `pageStep`) on their axis; for a
 *   one-dimensional slider every arrow maps to its single axis.
 * - Page Up / Page Down move one `pageStep` on the vertical axis (2D) or the
 *   only axis (1D).
 * - Home / End jump to the minimum / maximum of the horizontal axis (2D) or
 *   the only axis (1D).
 * - Horizontal arrows invert in right-to-left layouts.
 */
export function resolveKeyIntent(
  event: Pick<KeyboardEvent, "key" | "shiftKey">,
  options: {
    readonly dimensions: 1 | 2;
    readonly orientation?: ColorPickerOrientation;
    readonly dir?: ColorPickerDirection;
    readonly xRange: ColorChannelRange;
    readonly yRange?: ColorChannelRange;
  },
): ColorPickerKeyIntent | null {
  const singleAxis = options.orientation === "vertical" ? "y" : "x";
  const horizontalAxis = options.dimensions === 2 ? "x" : singleAxis;
  const verticalAxis = options.dimensions === 2 ? "y" : singleAxis;
  const rangeFor = (axis: "x" | "y"): ColorChannelRange =>
    axis === "y" && options.dimensions === 2 && options.yRange !== undefined
      ? options.yRange
      : options.xRange;
  const stepFor = (axis: "x" | "y"): number => {
    const range = rangeFor(axis);
    return event.shiftKey ? range.pageStep : range.step;
  };
  const rtl = options.dir === "rtl" ? -1 : 1;
  switch (event.key) {
    case "ArrowRight":
      return { kind: "delta", axis: horizontalAxis, amount: stepFor(horizontalAxis) * rtl };
    case "ArrowLeft":
      return { kind: "delta", axis: horizontalAxis, amount: -stepFor(horizontalAxis) * rtl };
    case "ArrowUp":
      return { kind: "delta", axis: verticalAxis, amount: stepFor(verticalAxis) };
    case "ArrowDown":
      return { kind: "delta", axis: verticalAxis, amount: -stepFor(verticalAxis) };
    case "PageUp":
      return { kind: "delta", axis: verticalAxis, amount: rangeFor(verticalAxis).pageStep };
    case "PageDown":
      return { kind: "delta", axis: verticalAxis, amount: -rangeFor(verticalAxis).pageStep };
    case "Home":
      return { kind: "edge", axis: horizontalAxis, edge: "min" };
    case "End":
      return { kind: "edge", axis: horizontalAxis, edge: "max" };
    default:
      return null;
  }
}

/** Read an element rectangle without throwing for detached nodes. */
export function readElementRect(element: Element): ColorPickerRect {
  const rect = element.getBoundingClientRect();
  return { left: rect.left, top: rect.top, width: rect.width, height: rect.height };
}

/** Capture the pointer on `element` when the platform supports it. */
export function capturePointer(element: Element, pointerId: number): void {
  if (typeof element.setPointerCapture !== "function") return;
  try {
    element.setPointerCapture(pointerId);
  } catch {
    // Synthetic or already-released pointers cannot be captured; dragging still works.
  }
}

/** Release a captured pointer when the platform supports it. */
export function releasePointer(element: Element, pointerId: number): void {
  if (typeof element.releasePointerCapture !== "function") return;
  try {
    if (typeof element.hasPointerCapture !== "function" || element.hasPointerCapture(pointerId)) {
      element.releasePointerCapture(pointerId);
    }
  } catch {
    // Released pointers are already free.
  }
}
