// Import through the Slider entry so bundlers keep Slider's module order in shared chunks.
import { getSliderState } from "../slider/slider.ts";
import type { RangeSliderValue } from "./range-slider-types.ts";

/** Normalized numeric constraints of a RangeSlider. */
export interface RangeSliderBounds {
  /** Normalized lower bound. */
  readonly min: number;

  /** Normalized upper bound, always greater than `min`. */
  readonly max: number;

  /** Normalized positive step. */
  readonly step: number;

  /** Normalized positive large step. */
  readonly largeStep: number;

  /** Minimum distance between neighboring thumbs, in value units. */
  readonly minDistance: number;
}

/** Raw constraints accepted by {@link normalizeRangeSliderBounds}. */
export interface RangeSliderBoundsOptions {
  /** Raw lower bound. @default 0 */
  readonly min?: number | undefined;

  /** Raw upper bound. @default 100 */
  readonly max?: number | undefined;

  /** Raw positive step. @default 1 */
  readonly step?: number | undefined;

  /** Raw positive large step. @default step * 10 */
  readonly largeStep?: number | undefined;

  /** Raw minimum number of steps between thumbs. @default 0 */
  readonly minStepsBetweenThumbs?: number | undefined;
}

/** Normalize RangeSlider bounds with the same repairs as Slider. */
export function normalizeRangeSliderBounds(
  options: RangeSliderBoundsOptions = {},
): RangeSliderBounds {
  const base = getSliderState({
    min: options.min ?? null,
    max: options.max ?? null,
    step: options.step ?? null,
  });
  // Slider allows `"any"`; RangeSlider always needs a numeric grid for keyboard steps.
  const step = base.step === "any" ? 1 : base.step;
  const largeStep =
    typeof options.largeStep === "number" &&
    Number.isFinite(options.largeStep) &&
    options.largeStep > 0
      ? options.largeStep
      : step * 10;
  const steps =
    typeof options.minStepsBetweenThumbs === "number" &&
    Number.isFinite(options.minStepsBetweenThumbs) &&
    options.minStepsBetweenThumbs > 0
      ? Math.floor(options.minStepsBetweenThumbs)
      : 0;
  return Object.freeze({
    min: base.min,
    max: base.max,
    step,
    largeStep,
    minDistance: Math.min(steps * step, base.max - base.min),
  });
}

/** Snap and clamp one raw value onto the slider grid. */
export function snapRangeSliderValue(value: number, bounds: RangeSliderBounds): number {
  return getSliderState({ value, min: bounds.min, max: bounds.max, step: bounds.step }).value;
}

/** Default thumbs when no values are supplied: one at each bound. */
export function defaultRangeSliderValue(bounds: RangeSliderBounds): RangeSliderValue {
  return [bounds.min, bounds.max];
}

/** Snap, sort, and space raw values so every thumb respects `minDistance`. */
export function normalizeRangeSliderValue(
  values: RangeSliderValue | undefined,
  bounds: RangeSliderBounds,
): RangeSliderValue {
  const source =
    values === undefined || values.length === 0 ? defaultRangeSliderValue(bounds) : values;
  const sorted = source
    .map((value) => snapRangeSliderValue(Number.isFinite(value) ? value : bounds.min, bounds))
    .sort((left, right) => left - right);
  // Push thumbs apart from the lowest upward, then pull back from the top if needed.
  for (let index = 1; index < sorted.length; index += 1) {
    const previous = sorted[index - 1] ?? bounds.min;
    const current = sorted[index] ?? bounds.min;
    if (current - previous < bounds.minDistance) {
      sorted[index] = Math.min(previous + bounds.minDistance, bounds.max);
    }
  }
  for (let index = sorted.length - 2; index >= 0; index -= 1) {
    const next = sorted[index + 1] ?? bounds.max;
    const current = sorted[index] ?? bounds.max;
    if (next - current < bounds.minDistance) {
      sorted[index] = Math.max(next - bounds.minDistance, bounds.min);
    }
  }
  return sorted;
}

/** Lowest and highest value one thumb may take without crossing its neighbors. */
export function getRangeSliderThumbBounds(
  values: RangeSliderValue,
  index: number,
  bounds: RangeSliderBounds,
): { readonly min: number; readonly max: number } {
  const previous = values[index - 1];
  const next = values[index + 1];
  return {
    min: previous === undefined ? bounds.min : previous + bounds.minDistance,
    max: next === undefined ? bounds.max : next - bounds.minDistance,
  };
}

/** Move one thumb, snapping and clamping it between its neighbors. */
export function setRangeSliderThumb(
  values: RangeSliderValue,
  index: number,
  value: number,
  bounds: RangeSliderBounds,
): RangeSliderValue {
  if (index < 0 || index >= values.length || !Number.isFinite(value)) return values;
  const limits = getRangeSliderThumbBounds(values, index, bounds);
  const next = Math.min(Math.max(snapRangeSliderValue(value, bounds), limits.min), limits.max);
  if (next === values[index]) return values;
  return values.map((current, position) => (position === index ? next : current));
}

/** Index of the thumb closest to a value; ties prefer the thumb that can move toward it. */
export function closestRangeSliderThumb(values: RangeSliderValue, value: number): number {
  let closest = 0;
  let distance = Number.POSITIVE_INFINITY;
  values.forEach((current, index) => {
    const next = Math.abs(current - value);
    if (next < distance || (next === distance && value > current)) {
      closest = index;
      distance = next;
    }
  });
  return closest;
}

/** Position of a value between the bounds, from 0 to 100. */
export function rangeSliderPercent(value: number, bounds: RangeSliderBounds): number {
  return ((value - bounds.min) / (bounds.max - bounds.min)) * 100;
}

/** Whether two value lists are equal by position. */
export function rangeSliderValuesEqual(left: RangeSliderValue, right: RangeSliderValue): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}
