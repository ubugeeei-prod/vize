import type {
  SliderDirection,
  SliderOrientation,
  SliderSlotState,
  SliderState,
  SliderStep,
} from "./slider-types.ts";

/** Native-friendly default minimum for Slider. */
export const SLIDER_DEFAULT_MIN = 0;

/** Native-friendly default maximum for Slider. */
export const SLIDER_DEFAULT_MAX = 100;

/** Native range default step. */
export const SLIDER_DEFAULT_STEP = 1;

/** Raw values accepted by the Slider normalizer. */
export interface SliderStateOptions {
  /**
   * Raw current value. Non-finite values fall back to the normalized minimum.
   *
   * @default SLIDER_DEFAULT_MIN
   */
  readonly value?: number | null;

  /**
   * Raw lower bound. Non-finite values fall back to `SLIDER_DEFAULT_MIN`.
   *
   * @default SLIDER_DEFAULT_MIN
   */
  readonly min?: number | null;

  /**
   * Raw upper bound. Values less than or equal to `min` are repaired to `min + 1`.
   *
   * @default SLIDER_DEFAULT_MAX
   */
  readonly max?: number | null;

  /**
   * Raw step. Non-positive and non-finite numbers fall back to `SLIDER_DEFAULT_STEP`.
   *
   * @default SLIDER_DEFAULT_STEP
   */
  readonly step?: SliderStep | null;

  /**
   * Raw orientation.
   *
   * @default "horizontal"
   */
  readonly orientation?: SliderOrientation | null;

  /**
   * Raw text direction.
   *
   * @default "ltr"
   */
  readonly direction?: SliderDirection | null;

  /**
   * Whether native focus and form submission are disabled.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Whether user value changes are locked while focus remains available.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Whether the field is currently marked invalid for assistive technology.
   *
   * @default false
   */
  readonly invalid?: boolean;

  /**
   * Whether native required validation is requested.
   *
   * @default false
   */
  readonly required?: boolean;
}

function finiteOr(value: number | null | undefined, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function normalizeStep(step: SliderStep | null | undefined): SliderStep {
  if (step === "any") return "any";
  return typeof step === "number" && Number.isFinite(step) && step > 0 ? step : SLIDER_DEFAULT_STEP;
}

function decimalPlaces(value: number): number {
  const text = String(value);
  const exponentIndex = text.indexOf("e-");
  if (exponentIndex !== -1) return Number(text.slice(exponentIndex + 2));
  const decimalIndex = text.indexOf(".");
  return decimalIndex === -1 ? 0 : text.length - decimalIndex - 1;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function snapValue(value: number, min: number, max: number, step: SliderStep): number {
  const clamped = clamp(value, min, max);
  if (step === "any") return clamped;

  const precision = Math.max(decimalPlaces(min), decimalPlaces(max), decimalPlaces(step));
  const snapped = min + Math.round((clamped - min) / step) * step;
  return clamp(Number(snapped.toFixed(Math.min(precision, 20))), min, max);
}

function normalizeOrientation(
  orientation: SliderOrientation | null | undefined,
): SliderOrientation {
  return orientation === "vertical" ? "vertical" : "horizontal";
}

function normalizeDirection(direction: SliderDirection | null | undefined): SliderDirection {
  return direction === "rtl" ? "rtl" : "ltr";
}

function sliderState(options: {
  readonly disabled: boolean;
  readonly invalid: boolean;
  readonly max: number;
  readonly min: number;
  readonly readOnly: boolean;
  readonly value: number;
}): SliderState {
  if (options.disabled) return "disabled";
  if (options.readOnly) return "readonly";
  if (options.invalid) return "invalid";
  if (options.value <= options.min) return "min";
  if (options.value >= options.max) return "max";
  return "in-range";
}

/** Normalize raw Slider props into the public state contract. */
export function getSliderState(options: SliderStateOptions = {}): SliderSlotState {
  const min = finiteOr(options.min, SLIDER_DEFAULT_MIN);
  const rawMax = finiteOr(options.max, SLIDER_DEFAULT_MAX);
  const max = rawMax > min ? rawMax : min + 1;
  const step = normalizeStep(options.step);
  const value = snapValue(finiteOr(options.value, min), min, max, step);
  const percent = ((value - min) / (max - min)) * 100;
  const disabled = options.disabled === true;
  const readOnly = options.readOnly === true;
  const invalid = options.invalid === true;
  const required = options.required === true;
  const orientation = normalizeOrientation(options.orientation);
  const direction = normalizeDirection(options.direction);

  return Object.freeze({
    value,
    min,
    max,
    step,
    percent,
    orientation,
    direction,
    disabled,
    readOnly,
    required,
    invalid,
    state: sliderState({ disabled, invalid, max, min, readOnly, value }),
  });
}
