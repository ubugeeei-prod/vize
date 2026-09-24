import { computed, isReadonly, isRef, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, WritableComputedRef } from "vue";

function clampNumber(value: number, min: number, max: number): number {
  if (min > max) {
    throw new RangeError(
      `[VIZE_COMPOSE_MATH_INVALID_RANGE] min must not exceed max; received [${String(min)}, ${String(max)}]`,
    );
  }
  return Math.min(max, Math.max(min, value));
}

function isWritableRef(value: MaybeRefOrGetter<number>): value is Ref<number> {
  return isRef(value) && !isReadonly(value);
}

/**
 * Number kept inside `[min, max]`.
 *
 * A writable source (a number or a writable ref) yields a writable ref whose
 * reads and writes are clamped; writes go through to the source ref. A
 * getter or computed source yields a readonly computed. Bounds are reactive.
 * Pure derived state: SSR-safe and nothing to dispose.
 *
 * @example
 * ```ts
 * const volume = useClamp(50, 0, 100);
 * volume.value = 150; // 100
 * const progress = useClamp(() => loaded.value / total.value, 0, 1);
 * ```
 *
 * @param value Source number, writable ref, or getter.
 * @param min Reactive inclusive lower bound.
 * @param max Reactive inclusive upper bound.
 * @throws `RangeError` tagged `VIZE_COMPOSE_MATH_INVALID_RANGE` on read or
 * write when `min > max`.
 * @returns Writable clamped ref, or a readonly computed for getters.
 */
export function useClamp(
  value: (() => number) | ComputedRef<number>,
  min: MaybeRefOrGetter<number>,
  max: MaybeRefOrGetter<number>,
): ComputedRef<number>;
export function useClamp(
  value: number | Ref<number>,
  min: MaybeRefOrGetter<number>,
  max: MaybeRefOrGetter<number>,
): WritableComputedRef<number>;
export function useClamp(
  value: MaybeRefOrGetter<number>,
  min: MaybeRefOrGetter<number>,
  max: MaybeRefOrGetter<number>,
): ComputedRef<number> | WritableComputedRef<number> {
  let source: Ref<number>;
  if (isWritableRef(value)) source = value;
  else if (typeof value === "number") source = shallowRef(value);
  else return computed(() => clampNumber(toValue(value), toValue(min), toValue(max)));
  return computed({
    get: () => clampNumber(source.value, toValue(min), toValue(max)),
    set: (next) => {
      source.value = clampNumber(next, toValue(min), toValue(max));
    },
  });
}

/** Rounding strategy used by {@link useRound}. */
export type RoundingMethod = "round" | "floor" | "ceil" | "trunc";

/** Options for {@link useRound}. */
export interface UseRoundOptions {
  /**
   * Decimal places to keep; negative values round to tens, hundreds, …
   * Reactive.
   *
   * @default 0
   */
  readonly precision?: MaybeRefOrGetter<number>;

  /**
   * Rounding strategy.
   *
   * @default "round"
   */
  readonly method?: RoundingMethod;
}

/** Multiply by `10 ** places` exactly, by editing the decimal exponent. */
function shiftExponent(value: number, places: number): number {
  const [mantissa, exponent = "0"] = String(value).split("e");
  return Number(`${mantissa}e${Number(exponent) + places}`);
}

/**
 * Reactive rounding with decimal precision.
 *
 * Uses exponent shifting instead of multiplying by powers of ten, so values
 * such as `1.005` round to `1.01` at two decimals. Pure derived state.
 *
 * @example
 * ```ts
 * const price = useRound(rawPrice, { precision: 2 });
 * ```
 *
 * @param value Reactive number.
 * @param options Precision and method.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_MATH_INVALID_PRECISION` on read
 * when the precision is not an integer.
 * @returns Computed rounded number.
 */
export function useRound(
  value: MaybeRefOrGetter<number>,
  options: UseRoundOptions = {},
): ComputedRef<number> {
  return computed(() => {
    const precision = toValue(options.precision ?? 0);
    if (!Number.isInteger(precision)) {
      throw new RangeError(
        `[VIZE_COMPOSE_MATH_INVALID_PRECISION] precision must be an integer; received ${String(precision)}`,
      );
    }
    const number = toValue(value);
    if (!Number.isFinite(number)) return number;
    const round = Math[options.method ?? "round"];
    return shiftExponent(round(shiftExponent(number, precision)), -precision);
  });
}

/** A `[start, end]` numeric interval. */
export type ProjectionDomain = readonly [start: number, end: number];

/** Options for {@link useProjection}. */
export interface UseProjectionOptions {
  /**
   * Clamp the result into the target domain.
   *
   * @default false
   */
  readonly clamp?: boolean;
}

/**
 * Linearly map a value from one domain to another.
 *
 * Domains are reactive tuples and may be reversed (`[100, 0]`). A degenerate
 * source domain maps everything to the target start. Pure derived state.
 *
 * @example
 * ```ts
 * const percent = useProjection(scrollY, () => [0, maxScroll.value], [0, 100], { clamp: true });
 * ```
 *
 * @param value Reactive number in the source domain.
 * @param from Reactive source domain.
 * @param to Reactive target domain.
 * @param options Clamping.
 * @default options {}
 * @returns Computed projected number.
 */
export function useProjection(
  value: MaybeRefOrGetter<number>,
  from: MaybeRefOrGetter<ProjectionDomain>,
  to: MaybeRefOrGetter<ProjectionDomain>,
  options: UseProjectionOptions = {},
): ComputedRef<number> {
  return computed(() => {
    const [fromStart, fromEnd] = toValue(from);
    const [toStart, toEnd] = toValue(to);
    const span = fromEnd - fromStart;
    const ratio = span === 0 ? 0 : (toValue(value) - fromStart) / span;
    const projected = toStart + ratio * (toEnd - toStart);
    if (!(options.clamp ?? false)) return projected;
    return Math.min(Math.max(projected, Math.min(toStart, toEnd)), Math.max(toStart, toEnd));
  });
}
