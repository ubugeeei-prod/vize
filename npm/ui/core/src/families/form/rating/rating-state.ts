import type { RatingDirection, RatingSlotState, RatingState, RatingValue } from "./rating-types.ts";

/** Native-friendly default minimum for Rating. */
export const RATING_DEFAULT_MIN = 1;

/** Default number of choices when Rating `max` is omitted. */
export const RATING_DEFAULT_COUNT = 5;

/** Raw bounds accepted by the Rating normalizer. */
export interface RatingBoundsOptions {
  /**
   * Raw lower bound. Non-finite values fall back to `RATING_DEFAULT_MIN`.
   *
   * @default RATING_DEFAULT_MIN
   */
  readonly min?: number | null | undefined;

  /**
   * Raw upper bound. When omitted, `count` derives the upper bound.
   *
   * @default undefined
   */
  readonly max?: number | null | undefined;

  /**
   * Raw item count used when `max` is omitted.
   *
   * @default RATING_DEFAULT_COUNT
   */
  readonly count?: number | null | undefined;
}

/** Raw values accepted by the Rating normalizer. */
export interface RatingStateOptions extends RatingBoundsOptions {
  /**
   * Raw current value. `null`, `undefined`, and non-finite values represent no rating.
   *
   * @default null
   */
  readonly value?: RatingValue | undefined;

  /**
   * Raw text direction.
   *
   * @default "ltr"
   */
  readonly direction?: RatingDirection | null | undefined;

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

  /**
   * Whether user gestures may clear the selected value.
   *
   * @default false
   */
  readonly clearable?: boolean;
}

/** Normalized Rating bounds. */
export interface RatingBounds {
  /** Normalized lower bound. */
  readonly min: number;

  /** Normalized upper bound. */
  readonly max: number;

  /** Number of generated choices. */
  readonly count: number;

  /** Generated values in DOM order. */
  readonly items: readonly number[];
}

function integerOr(value: number | null | undefined, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? Math.trunc(value) : fallback;
}

function positiveIntegerOr(value: number | null | undefined, fallback: number): number {
  const integer = integerOr(value, fallback);
  return integer > 0 ? integer : fallback;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function normalizeDirection(direction: RatingDirection | null | undefined): RatingDirection {
  return direction === "rtl" ? "rtl" : "ltr";
}

function ratingState(options: {
  readonly disabled: boolean;
  readonly invalid: boolean;
  readonly readOnly: boolean;
  readonly value: RatingValue;
}): RatingState {
  if (options.disabled) return "disabled";
  if (options.readOnly) return "readonly";
  if (options.invalid) return "invalid";
  return options.value === null ? "empty" : "selected";
}

/** Normalize raw Rating min/max/count props into a finite generated range. */
export function getRatingBounds(options: RatingBoundsOptions = {}): RatingBounds {
  const min = integerOr(options.min, RATING_DEFAULT_MIN);
  const count = positiveIntegerOr(options.count, RATING_DEFAULT_COUNT);
  const rawMax =
    typeof options.max === "number" && Number.isFinite(options.max)
      ? Math.trunc(options.max)
      : min + count - 1;
  const max = rawMax >= min ? rawMax : min;
  const normalizedCount = max - min + 1;
  const items = Array.from({ length: normalizedCount }, (_item, index) => min + index);

  return Object.freeze({ min, max, count: normalizedCount, items });
}

/** Normalize a raw Rating value into the generated range, or `null` for empty. */
export function normalizeRatingValue(
  value: RatingValue | undefined,
  options: RatingBoundsOptions = {},
): RatingValue {
  if (value === null || value === undefined) return null;
  if (!Number.isFinite(value)) return null;
  const { min, max } = getRatingBounds(options);
  return clamp(Math.round(value), min, max);
}

/** Normalize raw Rating props into the public state contract. */
export function getRatingState(options: RatingStateOptions = {}): RatingSlotState {
  const bounds = getRatingBounds(options);
  const value = normalizeRatingValue(options.value, bounds);
  const percent = value === null ? 0 : ((value - bounds.min + 1) / bounds.count) * 100;
  const disabled = options.disabled === true;
  const readOnly = options.readOnly === true;
  const required = options.required === true;
  const invalid = options.invalid === true;
  const clearable = options.clearable === true;
  const direction = normalizeDirection(options.direction);

  return Object.freeze({
    value,
    min: bounds.min,
    max: bounds.max,
    count: bounds.count,
    items: bounds.items,
    percent,
    direction,
    disabled,
    readOnly,
    required,
    invalid,
    clearable,
    state: ratingState({ disabled, invalid, readOnly, value }),
  });
}
