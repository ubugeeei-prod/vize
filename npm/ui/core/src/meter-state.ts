import type { MeterRange, MeterSlotState, MeterState } from "./meter-types.ts";

/** Native default lower bound for Meter. */
export const METER_DEFAULT_MIN = 0;

/** Native default upper bound for Meter. */
export const METER_DEFAULT_MAX = 1;

/** Raw values accepted by the Meter normalizer. */
export interface MeterStateOptions {
  /**
   * Raw value. Non-finite values fall back to the normalized minimum.
   *
   * @default METER_DEFAULT_MIN
   */
  readonly value?: number | null;

  /**
   * Raw minimum.
   *
   * @default METER_DEFAULT_MIN
   */
  readonly min?: number | null;

  /**
   * Raw maximum. Values less than or equal to `min` are repaired to `min + 1`.
   *
   * @default METER_DEFAULT_MAX
   */
  readonly max?: number | null;

  /**
   * Optional low threshold.
   *
   * @default null
   */
  readonly low?: number | null;

  /**
   * Optional high threshold.
   *
   * @default null
   */
  readonly high?: number | null;

  /**
   * Optional optimum threshold.
   *
   * @default null
   */
  readonly optimum?: number | null;
}

function finite(value: number | null | undefined): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function readThreshold(
  value: number | null | undefined,
  min: number,
  max: number,
): { readonly invalid: boolean; readonly value: number | null } {
  if (value == null) return { invalid: false, value: null };
  if (!finite(value)) return { invalid: true, value: null };
  return { invalid: value < min || value > max, value: clamp(value, min, max) };
}

function rangeFor(value: number, low: number | null, high: number | null): MeterRange {
  if (low !== null && value < low) return "low";
  if (high !== null && value > high) return "high";
  return "medium";
}

function stateFor(
  value: number,
  min: number,
  max: number,
  range: MeterRange,
  optimal: boolean,
): MeterState {
  if (value <= min) return "empty";
  if (value >= max) return "full";
  return optimal ? "optimum" : range;
}

/** Normalize raw Meter props into the public native-safe state contract. */
export function getMeterState(options: MeterStateOptions = {}): MeterSlotState {
  const min = finite(options.min) ? options.min : METER_DEFAULT_MIN;
  let invalid = options.min != null && !finite(options.min);
  let max = METER_DEFAULT_MAX;
  if (finite(options.max) && options.max > min) {
    max = options.max;
  } else {
    invalid = invalid || options.max != null;
    max = min + 1;
  }
  const value = finite(options.value) ? clamp(options.value, min, max) : min;
  invalid =
    invalid || (options.value != null && (!finite(options.value) || options.value !== value));

  const lowResult = readThreshold(options.low, min, max);
  const highResult = readThreshold(options.high, min, max);
  const optimumResult = readThreshold(options.optimum, min, max);
  let low = lowResult.value;
  let high = highResult.value;

  invalid = invalid || lowResult.invalid || highResult.invalid || optimumResult.invalid;
  if (low !== null && high !== null && low > high) {
    invalid = true;
    [low, high] = [high, low];
  }

  const percent = ((value - min) / (max - min)) * 100;
  const range = rangeFor(value, low, high);
  const optimumRange =
    optimumResult.value === null ? null : rangeFor(optimumResult.value, low, high);
  const optimal = optimumRange !== null && range === optimumRange;

  return Object.freeze({
    value,
    min,
    max,
    low,
    high,
    optimum: optimumResult.value,
    percent,
    range,
    optimal,
    invalid,
    state: stateFor(value, min, max, range, optimal),
  });
}
