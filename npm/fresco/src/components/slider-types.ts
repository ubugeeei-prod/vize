/** Slider layout axis. */
export type SliderOrientation = "horizontal" | "vertical";

/** Public props for the controlled Fresco Slider. */
export interface SliderProps {
  /** Current controlled value. */
  modelValue?: number;
  /** Inclusive lower bound. */
  min?: number;
  /** Inclusive upper bound. */
  max?: number;
  /** Arrow-key and wheel increment. */
  step?: number;
  /** PageUp/PageDown increment. Defaults to ten steps. */
  pageStep?: number;
  /** Number of terminal cells in the track. */
  length?: number;
  /** Track direction. */
  orientation?: SliderOrientation;
  /** Whether the slider owns keyboard and mouse input. */
  focused?: boolean;
  /** Prevent value changes and render a dimmed track. */
  disabled?: boolean;
  /** Glyph before the thumb. */
  filled?: string;
  /** Glyph after the thumb. */
  empty?: string;
  /** Thumb glyph. */
  thumb?: string;
  /** Unfilled track color. */
  fg?: string;
  /** Filled track color. */
  filledFg?: string;
  /** Thumb color. */
  thumbFg?: string;
  /** Accessible label for the slider. */
  label?: string;
  /** Format the current value in the accessible label. */
  formatValue?: (value: number) => string;
}

/** Finite, ordered numeric bounds used internally by Slider. */
export interface SliderRange {
  readonly min: number;
  readonly max: number;
  readonly step: number;
}

function finiteOr(value: number | undefined, fallback: number): number {
  return value !== undefined && Number.isFinite(value) ? value : fallback;
}

/** Normalize malformed public range inputs without producing NaN or infinities. */
export function normalizeSliderRange(
  min: number | undefined,
  max: number | undefined,
  step: number | undefined,
): SliderRange {
  const lower = finiteOr(min, 0);
  const requestedUpper = finiteOr(max, 100);
  const upper = Math.max(lower, requestedUpper);
  const requestedStep = finiteOr(step, 1);
  return { min: lower, max: upper, step: requestedStep > 0 ? requestedStep : 1 };
}

function decimalPlaces(value: number): number {
  const text = value.toString().toLowerCase();
  const [coefficient = "", exponentText] = text.split("e");
  const exponent = Number(exponentText ?? 0);
  const fractionLength = coefficient.split(".")[1]?.length ?? 0;
  return Math.max(0, fractionLength - exponent);
}

/** Clamp and step-align a candidate while preserving exact inclusive endpoints. */
export function normalizeSliderValue(value: number, range: SliderRange): number {
  const finiteValue = Number.isFinite(value) ? value : range.min;
  if (finiteValue <= range.min || range.min === range.max) return range.min;
  if (finiteValue >= range.max) return range.max;

  const steps = Math.round((finiteValue - range.min) / range.step);
  const snapped = range.min + steps * range.step;
  const precision = Math.min(12, Math.max(decimalPlaces(range.min), decimalPlaces(range.step)));
  return Math.min(range.max, Math.max(range.min, Number(snapped.toFixed(precision))));
}

/** Map a normalized track fraction onto the public range. */
export function sliderValueAtRatio(ratio: number, range: SliderRange): number {
  const boundedRatio = Number.isFinite(ratio) ? Math.min(1, Math.max(0, ratio)) : 0;
  return normalizeSliderValue(range.min + boundedRatio * (range.max - range.min), range);
}
