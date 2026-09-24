import type { NumberFieldState } from "./number-field-types.ts";

/** Default NumberField step for decimal, currency, and unit styles. */
export const NUMBER_FIELD_DEFAULT_STEP = 1;

/** Default NumberField step when `formatOptions.style` is `"percent"` (one percent). */
export const NUMBER_FIELD_DEFAULT_PERCENT_STEP = 0.01;

/** Multiplier applied to `step` when `largeStep` is not supplied. */
export const NUMBER_FIELD_LARGE_STEP_MULTIPLIER = 10;

/** Normalized numeric constraints used by NumberField stepping and commits. */
export interface NumberFieldBounds {
  /** Lower bound, or `-Infinity` when unbounded. */
  readonly min: number;

  /** Upper bound, or `Infinity` when unbounded. */
  readonly max: number;

  /** Positive step used by arrows and triggers. */
  readonly step: number;

  /** Positive step used by Page Up and Page Down. */
  readonly largeStep: number;
}

/** Raw constraint values accepted by {@link normalizeNumberFieldBounds}. */
export interface NumberFieldBoundsOptions {
  /** Raw lower bound. Non-finite values mean unbounded. @default undefined */
  readonly min?: number | null | undefined;

  /** Raw upper bound. Non-finite values or values below `min` mean unbounded. @default undefined */
  readonly max?: number | null | undefined;

  /** Raw positive step. Invalid values fall back to `defaultStep`. @default undefined */
  readonly step?: number | null | undefined;

  /** Raw positive large step. Invalid values fall back to `step * 10`. @default undefined */
  readonly largeStep?: number | null | undefined;

  /** Step used when `step` is absent or invalid. @default NUMBER_FIELD_DEFAULT_STEP */
  readonly defaultStep?: number | undefined;
}

function isPositive(value: number | null | undefined): value is number {
  return typeof value === "number" && Number.isFinite(value) && value > 0;
}

function decimalPlaces(value: number): number {
  if (!Number.isFinite(value)) return 0;
  const text = String(value);
  const exponentIndex = text.indexOf("e-");
  if (exponentIndex !== -1) {
    const mantissa = text.slice(0, exponentIndex);
    const mantissaDecimals = mantissa.includes(".")
      ? mantissa.length - mantissa.indexOf(".") - 1
      : 0;
    return Number(text.slice(exponentIndex + 2)) + mantissaDecimals;
  }
  const decimalIndex = text.indexOf(".");
  return decimalIndex === -1 ? 0 : text.length - decimalIndex - 1;
}

function roundToPrecision(value: number, precision: number): number {
  const rounded = Number(value.toFixed(Math.min(Math.max(precision, 0), 20)));
  return Object.is(rounded, -0) ? 0 : rounded;
}

/** Normalize raw NumberField bounds so stepping never produces `NaN`. */
export function normalizeNumberFieldBounds(
  options: NumberFieldBoundsOptions = {},
): NumberFieldBounds {
  const min =
    typeof options.min === "number" && Number.isFinite(options.min)
      ? options.min
      : Number.NEGATIVE_INFINITY;
  const rawMax =
    typeof options.max === "number" && Number.isFinite(options.max)
      ? options.max
      : Number.POSITIVE_INFINITY;
  const max = rawMax >= min ? rawMax : Number.POSITIVE_INFINITY;
  const fallbackStep = isPositive(options.defaultStep)
    ? options.defaultStep
    : NUMBER_FIELD_DEFAULT_STEP;
  const step = isPositive(options.step) ? options.step : fallbackStep;
  const largeStep = isPositive(options.largeStep)
    ? options.largeStep
    : roundToPrecision(step * NUMBER_FIELD_LARGE_STEP_MULTIPLIER, decimalPlaces(step));
  return Object.freeze({ min, max, step, largeStep });
}

/** Clamp a value into normalized bounds. */
export function clampNumberFieldValue(value: number, bounds: NumberFieldBounds): number {
  return Math.min(Math.max(value, bounds.min), bounds.max);
}

function stepBase(bounds: NumberFieldBounds): number {
  return Number.isFinite(bounds.min) ? bounds.min : 0;
}

/** Snap a value to the nearest step from `min` (or zero when unbounded), then clamp. */
export function snapNumberFieldValue(value: number, bounds: NumberFieldBounds): number {
  const base = stepBase(bounds);
  const precision = Math.max(decimalPlaces(base), decimalPlaces(bounds.step));
  const snapped = base + Math.round((value - base) / bounds.step) * bounds.step;
  // Clamping after snapping keeps an off-grid `max` reachable, as native number inputs do.
  return clampNumberFieldValue(roundToPrecision(snapped, precision), bounds);
}

/**
 * Move a value along the step grid.
 *
 * The value first moves to the adjacent grid value in the requested direction
 * (so off-grid values land back on the grid, matching native `stepUp()` /
 * `stepDown()`), then continues by the rest of `amount`, snaps, and clamps.
 * `amount` defaults to one `step`; Page Up/Down pass `largeStep`. `null`
 * starts from `min` when incrementing, `max` when decrementing, else zero.
 */
export function stepNumberFieldValue(
  value: number | null,
  direction: 1 | -1,
  bounds: NumberFieldBounds,
  amount: number = bounds.step,
): number {
  if (value === null) {
    if (direction > 0 && Number.isFinite(bounds.min)) return bounds.min;
    if (direction < 0 && Number.isFinite(bounds.max)) return bounds.max;
    return clampNumberFieldValue(0, bounds);
  }
  const base = stepBase(bounds);
  const precision = Math.max(
    decimalPlaces(base),
    decimalPlaces(bounds.step),
    decimalPlaces(value),
    isPositive(amount) ? decimalPlaces(amount) : 0,
  );
  const offset = roundToPrecision((value - base) / bounds.step, 10);
  const count = direction > 0 ? Math.floor(offset) + 1 : Math.ceil(offset) - 1;
  const adjacent = base + count * bounds.step;
  const remainder = isPositive(amount) && amount > bounds.step ? amount - bounds.step : 0;
  const next = roundToPrecision(adjacent + direction * remainder, precision);
  const snapped =
    remainder === 0
      ? next
      : roundToPrecision(base + Math.round((next - base) / bounds.step) * bounds.step, precision);
  return clampNumberFieldValue(snapped, bounds);
}

/** Whether stepping in a direction can still change the value. */
export function canStepNumberFieldValue(
  value: number | null,
  direction: 1 | -1,
  bounds: NumberFieldBounds,
): boolean {
  if (value === null) return true;
  return direction > 0 ? value < bounds.max : value > bounds.min;
}

/** Stable state token published through `data-state`. */
export function getNumberFieldState(options: {
  readonly value: number | null;
  readonly bounds: NumberFieldBounds;
  readonly disabled: boolean;
  readonly readOnly: boolean;
  readonly invalid: boolean;
}): NumberFieldState {
  if (options.disabled) return "disabled";
  if (options.readOnly) return "readonly";
  if (options.invalid) return "invalid";
  if (options.value === null) return "empty";
  if (options.value <= options.bounds.min) return "min";
  if (options.value >= options.bounds.max) return "max";
  return "in-range";
}
