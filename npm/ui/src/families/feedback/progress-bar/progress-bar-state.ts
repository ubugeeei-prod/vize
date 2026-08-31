import type { ProgressBarDirection, ProgressBarSlotState } from "./progress-bar-types.ts";

/** Native-friendly default minimum for determinate ProgressBar. */
export const PROGRESS_BAR_DEFAULT_MIN = 0;

/** Native-friendly default maximum for determinate ProgressBar. */
export const PROGRESS_BAR_DEFAULT_MAX = 100;

/** Raw values accepted by the ProgressBar normalizer. */
export interface ProgressBarStateOptions {
  /**
   * Raw determinate value. `null`, `undefined`, and non-finite numbers are indeterminate.
   *
   * @default null
   */
  readonly value?: number | null;

  /**
   * Raw minimum. Non-finite values fall back to `PROGRESS_BAR_DEFAULT_MIN`.
   *
   * @default PROGRESS_BAR_DEFAULT_MIN
   */
  readonly min?: number | null;

  /**
   * Raw maximum. Values less than or equal to `min` are repaired to `min + 100`.
   *
   * @default PROGRESS_BAR_DEFAULT_MAX
   */
  readonly max?: number | null;

  /**
   * Reading direction for inline-start fill and indeterminate motion.
   *
   * @default "ltr"
   */
  readonly dir?: ProgressBarDirection;
}

function finite(value: number | null | undefined): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function normalizeNumber(value: number): string {
  return Number.isInteger(value) ? String(value) : String(Number(value.toPrecision(12)));
}

function normalizeDirection(dir: ProgressBarDirection | undefined): ProgressBarDirection {
  return dir === "rtl" ? "rtl" : "ltr";
}

/** Normalize raw ProgressBar props into the public ARIA and CSS state contract. */
export function getProgressBarState(
  options: ProgressBarStateOptions = {},
): Omit<ProgressBarSlotState, "id" | "labelId" | "style" | "valueId"> {
  const min = finite(options.min) ? options.min : PROGRESS_BAR_DEFAULT_MIN;
  let invalid = options.min != null && !finite(options.min);
  let max = PROGRESS_BAR_DEFAULT_MAX;
  if (finite(options.max) && options.max > min) {
    max = options.max;
  } else {
    invalid = invalid || options.max != null;
    max = min + PROGRESS_BAR_DEFAULT_MAX;
  }

  const value = finite(options.value) ? clamp(options.value, min, max) : null;
  invalid =
    invalid || (options.value != null && (!finite(options.value) || options.value !== value));
  const indeterminate = value === null;
  const ratio = indeterminate ? null : (value - min) / (max - min);
  const percent = ratio === null ? null : ratio * 100;
  const complete = value !== null && value >= max;

  return Object.freeze({
    value,
    min,
    max,
    percent,
    ratio,
    dir: normalizeDirection(options.dir),
    indeterminate,
    complete,
    invalid,
    state: indeterminate ? "indeterminate" : complete ? "complete" : "loading",
  });
}

/** Convert normalized state into stable ProgressBar CSS custom properties. */
export function getProgressBarStyle(
  state: Pick<ProgressBarSlotState, "max" | "min" | "percent" | "ratio" | "value">,
): ProgressBarSlotState["style"] {
  const ratio = state.ratio ?? 0;
  const percent = state.percent ?? 0;
  return Object.freeze({
    "--vize-ui-progress-bar-max": normalizeNumber(state.max),
    "--vize-ui-progress-bar-min": normalizeNumber(state.min),
    "--vize-ui-progress-bar-percent": `${normalizeNumber(percent)}%`,
    "--vize-ui-progress-bar-ratio": normalizeNumber(ratio),
    "--vize-ui-progress-bar-value": normalizeNumber(state.value ?? state.min),
  });
}
