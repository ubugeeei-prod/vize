import { computed, shallowRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

/** Options for {@link useCounter}. */
export interface UseCounterOptions {
  /**
   * Inclusive lower bound applied to every value the counter takes.
   *
   * @default Number.NEGATIVE_INFINITY
   */
  readonly min?: number;

  /**
   * Inclusive upper bound applied to every value the counter takes.
   *
   * @default Number.POSITIVE_INFINITY
   */
  readonly max?: number;
}

/** Reactive controls returned by {@link useCounter}. */
export interface CounterControls {
  /** Current count. Changes only through the controls, never by assignment. */
  readonly count: Readonly<ShallowRef<number>>;

  /** Whether the count currently sits on the configured lower bound. */
  readonly atMin: ComputedRef<boolean>;

  /** Whether the count currently sits on the configured upper bound. */
  readonly atMax: ComputedRef<boolean>;

  /**
   * Add `delta` (default `1`) to the count and clamp into the bounds.
   *
   * @param delta Amount added; may be negative or infinite.
   * @returns The count after clamping.
   */
  readonly increment: (delta?: number) => number;

  /**
   * Subtract `delta` (default `1`) from the count and clamp into the bounds.
   *
   * @param delta Amount subtracted; may be negative or infinite.
   * @returns The count after clamping.
   */
  readonly decrement: (delta?: number) => number;

  /**
   * Assign a value directly, clamped into the bounds.
   *
   * @returns The count after clamping.
   */
  readonly set: (value: number) => number;

  /**
   * Restore the reset baseline, or establish a new one.
   *
   * Without an argument the count returns to the creation-time initial value
   * (after its original clamping). With an argument, the clamped value
   * becomes both the new count and the baseline used by later `reset()`
   * calls.
   *
   * @param value Replacement baseline.
   * @returns The count after clamping.
   */
  readonly reset: (value?: number) => number;
}

/**
 * Create a clamped counter whose every transition stays inside `[min, max]`.
 *
 * All operations clamp instead of failing, including the initial value, so
 * the count is inside the bounds at every observable moment. Only `NaN` is
 * rejected — silently corrupting the count is never an option. Purely
 * synchronous state: safe during server rendering (no browser globals, no
 * timers) and nothing to dispose, so it works inside and outside reactive
 * scopes alike. Bounds are fixed at creation and not reactive.
 *
 * @example
 * ```ts
 * const { count, increment, atMax } = useCounter(9, { min: 0, max: 10 });
 * increment(); // 10
 * increment(); // 10 (clamped)
 * atMax.value; // true
 * ```
 *
 * @param initial Count before any operation, clamped into the bounds.
 * @default initial 0
 * @param options Inclusive bounds for every value the counter takes.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_COUNTER_INVALID_RANGE` when a
 * bound is `NaN` or `min` exceeds `max`.
 * @throws `RangeError` tagged `VIZE_COMPOSE_COUNTER_INVALID_VALUE` when an
 * initial value, operand, or arithmetic result is `NaN` (for example
 * incrementing `-Infinity` by `Infinity`); the count is left unchanged.
 * @returns Reactive count, bound flags, and mutation controls.
 */
export function useCounter(initial = 0, options: UseCounterOptions = {}): CounterControls {
  const min = requireBound(options.min ?? Number.NEGATIVE_INFINITY, "min");
  const max = requireBound(options.max ?? Number.POSITIVE_INFINITY, "max");
  if (min > max) {
    throw new RangeError(
      `[VIZE_COMPOSE_COUNTER_INVALID_RANGE] min must not exceed max; received min ${String(min)} and max ${String(max)}`,
    );
  }

  const clamp = (value: number): number => Math.min(max, Math.max(min, value));
  const count = shallowRef(clamp(requireValue(initial)));
  let baseline = count.value;

  const setClamped = (next: number): number => {
    count.value = clamp(requireValue(next));
    return count.value;
  };

  const reset = (value?: number): number => {
    const applied = setClamped(value ?? baseline);
    if (value !== undefined) baseline = applied;
    return applied;
  };

  return {
    count,
    atMin: computed(() => count.value === min),
    atMax: computed(() => count.value === max),
    increment: (delta = 1) => setClamped(count.value + requireValue(delta)),
    decrement: (delta = 1) => setClamped(count.value - requireValue(delta)),
    set: setClamped,
    reset,
  };
}

function requireBound(value: number, label: "min" | "max"): number {
  if (Number.isNaN(value)) {
    throw new RangeError(`[VIZE_COMPOSE_COUNTER_INVALID_RANGE] ${label} must not be NaN`);
  }
  return value;
}

function requireValue(value: number): number {
  if (Number.isNaN(value)) {
    throw new RangeError(
      "[VIZE_COMPOSE_COUNTER_INVALID_VALUE] the value is NaN; counter state was left unchanged",
    );
  }
  return value;
}
