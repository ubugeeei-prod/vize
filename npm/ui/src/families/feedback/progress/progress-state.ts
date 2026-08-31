import type { ProgressSlotState } from "./progress-types.ts";

/** Native-friendly default maximum for determinate Progress. */
export const PROGRESS_DEFAULT_MAX = 100;

/** Raw values accepted by the Progress normalizer. */
export interface ProgressStateOptions {
  /**
   * Raw determinate value. `null`, `undefined`, and non-finite numbers are indeterminate.
   *
   * @default null
   */
  readonly value?: number | null;

  /**
   * Raw maximum. Non-positive and non-finite numbers fall back to `PROGRESS_DEFAULT_MAX`.
   *
   * @default PROGRESS_DEFAULT_MAX
   */
  readonly max?: number | null;
}

function normalizeMax(max: number | null | undefined): number {
  if (typeof max !== "number" || !Number.isFinite(max) || max <= 0) return PROGRESS_DEFAULT_MAX;
  return max;
}

function normalizeValue(value: number | null | undefined, max: number): number | null {
  if (typeof value !== "number" || !Number.isFinite(value)) return null;
  return Math.min(Math.max(value, 0), max);
}

/** Normalize raw Progress props into the public state contract. */
export function getProgressState(options: ProgressStateOptions = {}): ProgressSlotState {
  const max = normalizeMax(options.max);
  const value = normalizeValue(options.value, max);
  const indeterminate = value === null;
  const percent = indeterminate ? null : (value / max) * 100;
  const complete = value !== null && value >= max;

  return Object.freeze({
    value,
    max,
    percent,
    indeterminate,
    complete,
    state: indeterminate ? "indeterminate" : complete ? "complete" : "loading",
  });
}
