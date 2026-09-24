import { computed, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { useIntervalFn } from "./use-interval.ts";
import type { IntervalScheduler, PausableControls } from "./use-interval.ts";

/** Options for {@link useCountdown}. */
export interface UseCountdownOptions {
  /**
   * Milliseconds between two decrements. Reactive.
   *
   * @default 1000
   */
  readonly intervalMs?: MaybeRefOrGetter<number>;

  /**
   * Start counting down as soon as the composable is created.
   *
   * @default false
   */
  readonly immediate?: boolean;

  /**
   * Invoked after every decrement with the remaining count.
   *
   * @default undefined
   */
  readonly onTick?: (remaining: number) => void;

  /**
   * Invoked once when the count reaches zero.
   *
   * @default undefined
   */
  readonly onComplete?: () => void;

  /**
   * Starts host timers when no browser `window` is available.
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Repeating timer host.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: IntervalScheduler;
}

/** State and controls returned by {@link useCountdown}. */
export interface CountdownControls extends PausableControls {
  /** Remaining whole ticks; never negative. */
  readonly remaining: Readonly<ShallowRef<number>>;

  /** Whether the countdown reached zero. */
  readonly isComplete: ComputedRef<boolean>;

  /**
   * Reset to `count` (default: the initial count) and start ticking.
   *
   * @param count Replacement starting count.
   */
  readonly start: (count?: number) => void;

  /** Pause and reset to `count` (default: the initial count). */
  readonly reset: (count?: number) => void;

  /** Pause and jump to zero without firing `onComplete`. */
  readonly stop: () => void;
}

function resolveCount(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_COUNTDOWN_INVALID_COUNT] count must be finite and at least zero; received ${String(value)}`,
    );
  }
  return Math.trunc(value);
}

/**
 * Count down from a number of ticks to zero.
 *
 * Builds on {@link useIntervalFn}: pause/resume keep the remaining count,
 * `start` and `reset` restore it, and the timer pauses itself at zero after
 * calling `onComplete`. The initial count is reactive only for later
 * `start()`/`reset()` calls without an argument. Server rendering is
 * hydration-stable (no timer, remaining stays at the initial count).
 *
 * @example
 * ```ts
 * const { remaining, start } = useCountdown(60, { onComplete: resendAvailable });
 * start();
 * ```
 *
 * @param initialCount Reactive starting count in ticks.
 * @param options Cadence, callbacks, and scheduler policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_COUNTDOWN_INVALID_COUNT` when a
 * count is negative or not finite.
 * @returns Remaining count, completion flag, and controls.
 */
export function useCountdown(
  initialCount: MaybeRefOrGetter<number>,
  options: UseCountdownOptions = {},
): CountdownControls {
  const remaining = shallowRef(resolveCount(toValue(initialCount)));
  const isComplete = computed(() => remaining.value === 0);

  const interval = useIntervalFn(
    () => {
      if (remaining.value === 0) return;
      remaining.value -= 1;
      options.onTick?.(remaining.value);
      if (remaining.value === 0) {
        interval.pause();
        options.onComplete?.();
      }
    },
    options.intervalMs ?? 1_000,
    {
      immediate: false,
      runOnServer: options.runOnServer ?? false,
      ...(options.scheduler === undefined ? {} : { scheduler: options.scheduler }),
    },
  );

  const reset = (count?: number): void => {
    interval.pause();
    remaining.value = resolveCount(count ?? toValue(initialCount));
  };

  const start = (count?: number): void => {
    reset(count);
    if (remaining.value > 0) interval.resume();
  };

  const stop = (): void => {
    interval.pause();
    remaining.value = 0;
  };

  const resume = (): void => {
    if (remaining.value > 0) interval.resume();
  };

  if ((options.immediate ?? false) && remaining.value > 0) interval.resume();

  return {
    isActive: interval.isActive,
    pause: interval.pause,
    resume,
    remaining,
    isComplete,
    start,
    reset,
    stop,
  };
}
