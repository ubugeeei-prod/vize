/**
 * Pure rotary geometry shared by Knob and AnglePicker.
 *
 * Angles are degrees measured clockwise from 12 o'clock, which matches CSS
 * `rotate()` for an element whose indicator points up at 0deg.
 */

/** A point in client (viewport) coordinates. */
export interface RotaryPoint {
  /** Horizontal coordinate. */
  readonly x: number;

  /** Vertical coordinate (down is positive). */
  readonly y: number;
}

/** Normalized linear bounds for a rotary value. */
export interface RotaryBounds {
  /** Lower bound. */
  readonly min: number;

  /** Upper bound (greater than `min`). */
  readonly max: number;

  /** Positive step. */
  readonly step: number;

  /** Positive large step for Page Up / Page Down. */
  readonly largeStep: number;
}

/** Normalized sweep of a bounded knob. */
export interface RotarySweep {
  /** Angle of `min`, in degrees. */
  readonly start: number;

  /** Angle of `max`, in degrees (greater than `start`, at most `start + 360`). */
  readonly end: number;
}

function finite(value: number | undefined, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function decimals(value: number): number {
  const text = String(value);
  const exponent = text.indexOf("e-");
  if (exponent !== -1) return Number(text.slice(exponent + 2));
  const dot = text.indexOf(".");
  return dot === -1 ? 0 : text.length - dot - 1;
}

function round(value: number, places: number): number {
  const rounded = Number(value.toFixed(Math.min(Math.max(places, 0), 20)));
  return Object.is(rounded, -0) ? 0 : rounded;
}

/** Normalize raw bounds; invalid values fall back to 0–100 with step 1. */
export function normalizeRotaryBounds(options: {
  readonly min?: number | undefined;
  readonly max?: number | undefined;
  readonly step?: number | undefined;
  readonly largeStep?: number | undefined;
}): RotaryBounds {
  const min = finite(options.min, 0);
  const rawMax = finite(options.max, 100);
  const max = rawMax > min ? rawMax : min + 1;
  const step = finite(options.step, 1) > 0 ? finite(options.step, 1) : 1;
  const rawLarge = finite(options.largeStep, step * 10);
  return Object.freeze({ min, max, step, largeStep: rawLarge > 0 ? rawLarge : step * 10 });
}

/** Normalize a sweep; invalid or zero sweeps fall back to the default 270° arc. */
export function normalizeRotarySweep(options: {
  readonly startAngle?: number | undefined;
  readonly endAngle?: number | undefined;
}): RotarySweep {
  const start = finite(options.startAngle, -135);
  const end = finite(options.endAngle, 135);
  if (end <= start || end - start > 360) return Object.freeze({ start: -135, end: 135 });
  return Object.freeze({ start, end });
}

/** Snap a value to the step grid from `min` and clamp it into bounds. */
export function snapRotaryValue(value: number, bounds: RotaryBounds): number {
  const clamped = Math.min(Math.max(finite(value, bounds.min), bounds.min), bounds.max);
  const places = Math.max(decimals(bounds.min), decimals(bounds.step));
  const snapped = round(
    bounds.min + Math.round((clamped - bounds.min) / bounds.step) * bounds.step,
    places,
  );
  return Math.min(Math.max(snapped, bounds.min), bounds.max);
}

/** Angle of `point` around `center`, in degrees clockwise from 12 o'clock, in (-180, 180]. */
export function pointerAngle(center: RotaryPoint, point: RotaryPoint): number {
  const degrees = (Math.atan2(point.x - center.x, center.y - point.y) * 180) / Math.PI;
  return degrees === -180 ? 180 : round(degrees, 6);
}

/** Wrap any angle into `[0, 360)`. */
export function wrapAngle(angle: number): number {
  const wrapped = ((finite(angle, 0) % 360) + 360) % 360;
  return round(wrapped, 10) === 360 ? 0 : round(wrapped, 10);
}

/** Map a bounded value to its angle on the sweep. */
export function valueToAngle(value: number, bounds: RotaryBounds, sweep: RotarySweep): number {
  const ratio = (value - bounds.min) / (bounds.max - bounds.min);
  return round(sweep.start + ratio * (sweep.end - sweep.start), 6);
}

/**
 * Map a pointer angle onto the sweep. Angles inside the dead zone (outside
 * the sweep) resolve to the angularly closer end, so dragging past either end
 * never jumps to the opposite bound.
 */
export function angleToValue(angle: number, bounds: RotaryBounds, sweep: RotarySweep): number {
  // Express the angle relative to the sweep start in [0, 360).
  const relative = wrapAngle(angle - sweep.start);
  const span = sweep.end - sweep.start;
  let ratio: number;
  if (relative <= span) ratio = relative / span;
  else ratio = relative - span < 360 - relative ? 1 : 0;
  return snapRotaryValue(bounds.min + ratio * (bounds.max - bounds.min), bounds);
}

/** Snap a full-circle angle to `step` degrees and wrap it into `[0, 360)`. */
export function snapAngle(angle: number, step: number): number {
  const size = Number.isFinite(step) && step > 0 ? step : 1;
  return wrapAngle(round(Math.round(wrapAngle(angle) / size) * size, decimals(size)));
}
