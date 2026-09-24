import { shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/**
 * Repeating timer host used by {@link useIntervalFn} and the clocks built
 * on it.
 *
 * Implement this interface to integrate a deterministic test clock, a native
 * runtime timer, or an application-owned scheduler. Handles are opaque: they
 * are only handed back to {@link IntervalScheduler.clearInterval}.
 */
export interface IntervalScheduler {
  /** Starts a repeating callback and returns its opaque cancellation handle. */
  readonly setInterval: (callback: () => void, intervalMs: number) => unknown;

  /** Cancels a handle previously returned by {@link IntervalScheduler.setInterval}. */
  readonly clearInterval: (handle: unknown) => void;
}

/** Pause/resume controls shared by the repeating timing composables. */
export interface PausableControls {
  /**
   * Whether the timer is logically running.
   *
   * On the server (without `runOnServer`) this reflects the requested state
   * without starting a host timer, so server and client render the same
   * initial value.
   */
  readonly isActive: Readonly<ShallowRef<boolean>>;

  /** Stop ticking. Idempotent. */
  readonly pause: () => void;

  /** Start (or keep) ticking. Idempotent. */
  readonly resume: () => void;
}

/** Options for {@link useIntervalFn}. */
export interface UseIntervalFnOptions {
  /**
   * Start ticking as soon as the composable is created.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Invoke the callback synchronously whenever the timer (re)starts, in
   * addition to every tick.
   *
   * @default false
   */
  readonly immediateCallback?: boolean;

  /**
   * Starts host timers when no browser `window` is available.
   *
   * Keep this disabled during server rendering: the controls then track the
   * requested state but never schedule work. Enable it for native, desktop,
   * worker, and test runtimes whose scheduler is lifecycle-bound.
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Owns the repeating timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: IntervalScheduler;
}

/** Options for {@link useInterval}. */
export interface UseIntervalOptions<Controls extends boolean = false> extends UseIntervalFnOptions {
  /**
   * Return the full control object instead of the bare counter ref.
   *
   * @default false
   */
  readonly controls?: Controls;

  /**
   * Invoked after every increment with the new count.
   *
   * @default undefined
   */
  readonly callback?: (count: number) => void;
}

/** Counter and controls returned by `useInterval(ms, { controls: true })`. */
export interface IntervalControls extends PausableControls {
  /** Number of ticks since creation or the last {@link IntervalControls.reset}. */
  readonly counter: Readonly<ShallowRef<number>>;

  /** Reset the counter to zero without touching the timer. */
  readonly reset: () => void;
}

const defaultScheduler: IntervalScheduler = {
  setInterval: (callback, intervalMs) => globalThis.setInterval(callback, intervalMs),
  clearInterval: (handle) => {
    globalThis.clearInterval(handle as ReturnType<typeof setInterval>);
  },
};

function resolveIntervalMs(value: number): number {
  if (!Number.isFinite(value) || value <= 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_INTERVAL_INVALID_DELAY] intervalMs must be finite and greater than zero; received ${String(value)}`,
    );
  }
  return Math.max(1, Math.trunc(value));
}

/**
 * Call a function on a fixed, pausable interval.
 *
 * `intervalMs` is reactive: while active, a change replaces the running timer
 * with one using the new period. The timer is cleared when the owning
 * reactive scope stops; call inside an active scope, or call `pause()`
 * yourself.
 *
 * Server rendering is hydration-stable: without a browser `window` (and
 * with {@link UseIntervalFnOptions.runOnServer} disabled) `isActive` reports
 * the requested state but no host timer is created, so nothing leaks and
 * the server snapshot matches the client's initial render.
 *
 * @example
 * ```ts
 * const { pause, resume, isActive } = useIntervalFn(() => poll(), 5_000);
 * ```
 *
 * @param callback Invoked on every tick.
 * @param intervalMs Reactive period in milliseconds; must be finite and
 * greater than zero. Fractions are truncated.
 * @param options Start, server, and scheduler policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_INTERVAL_INVALID_DELAY` when the
 * resolved period is invalid, synchronously at creation and whenever a timer
 * would start.
 * @returns Pause/resume controls.
 */
export function useIntervalFn(
  callback: () => void,
  intervalMs: MaybeRefOrGetter<number>,
  options: UseIntervalFnOptions = {},
): PausableControls {
  const scheduler = options.scheduler ?? defaultScheduler;
  const isActive = shallowRef(false);
  let handle: unknown;
  let running = false;

  resolveIntervalMs(toValue(intervalMs));

  const canSchedule = (): boolean =>
    typeof window !== "undefined" || (options.runOnServer ?? false);

  const clear = (): void => {
    if (!running) return;
    running = false;
    scheduler.clearInterval(handle);
    handle = undefined;
  };

  const start = (): void => {
    const period = resolveIntervalMs(toValue(intervalMs));
    clear();
    if (!canSchedule()) return;
    if (options.immediateCallback ?? false) callback();
    if (!isActive.value) return;
    running = true;
    handle = scheduler.setInterval(callback, period);
  };

  const pause = (): void => {
    isActive.value = false;
    clear();
  };

  const resume = (): void => {
    if (isActive.value) return;
    isActive.value = true;
    start();
  };

  watch(
    () => toValue(intervalMs),
    () => {
      if (isActive.value) start();
    },
  );

  if (options.immediate ?? true) resume();

  tryOnScopeDispose(pause);

  return { isActive, pause, resume };
}

/**
 * Count interval ticks reactively.
 *
 * A thin wrapper over {@link useIntervalFn} whose state is a counter. Pass
 * `{ controls: true }` to receive pause/resume/reset alongside the counter;
 * the return type follows the literal flag.
 *
 * @example
 * ```ts
 * const ticks = useInterval(1_000); // Readonly<ShallowRef<number>>
 * const { counter, pause, reset } = useInterval(1_000, { controls: true });
 * ```
 *
 * @param intervalMs Reactive period in milliseconds.
 * @param options Interval options plus the `controls` flag.
 * @default intervalMs 1000
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_INTERVAL_INVALID_DELAY` for
 * invalid periods (see {@link useIntervalFn}).
 * @returns The counter ref, or the full controls when `controls` is `true`.
 */
export function useInterval(
  intervalMs?: MaybeRefOrGetter<number>,
  options?: UseIntervalOptions<false>,
): Readonly<ShallowRef<number>>;
export function useInterval(
  intervalMs: MaybeRefOrGetter<number>,
  options: UseIntervalOptions<true>,
): IntervalControls;
export function useInterval(
  intervalMs: MaybeRefOrGetter<number> = 1_000,
  options: UseIntervalOptions<boolean> = {},
): Readonly<ShallowRef<number>> | IntervalControls {
  const counter = shallowRef(0);
  const controls = useIntervalFn(
    () => {
      counter.value += 1;
      options.callback?.(counter.value);
    },
    intervalMs,
    options,
  );
  if (options.controls !== true) return counter;
  return {
    ...controls,
    counter,
    reset: () => {
      counter.value = 0;
    },
  };
}
