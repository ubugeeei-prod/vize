import { computed, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Options for {@link useTimeoutFn}. */
export interface UseTimeoutFnOptions {
  /**
   * Start the timer as soon as the composable is created. Only available
   * for callbacks without arguments; pass `false` to schedule a callback
   * that takes arguments through {@link TimeoutFnControls.start}.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Starts host timers when no browser `window` is available.
   *
   * Keep this disabled during server rendering: `isPending` then reports the
   * requested state but no timer is created and the callback never runs.
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Owns the single-shot timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

/** Controls returned by {@link useTimeoutFn}. */
export interface TimeoutFnControls<Args extends readonly unknown[]> {
  /** Whether a call is scheduled and has not fired or been stopped yet. */
  readonly isPending: Readonly<ShallowRef<boolean>>;

  /**
   * (Re)start the timer. A pending call is replaced, and the callback later
   * receives exactly these arguments.
   */
  readonly start: (...args: Args) => void;

  /**
   * Cancel the pending call.
   *
   * @returns Whether a pending call was cancelled.
   */
  readonly stop: () => boolean;
}

/** Options for {@link useTimeout}. */
export interface UseTimeoutOptions<Controls extends boolean = false> extends UseTimeoutFnOptions {
  /**
   * Return the full control object instead of the bare readiness ref.
   *
   * @default false
   */
  readonly controls?: Controls;

  /**
   * Invoked when the timeout elapses.
   *
   * @default undefined
   */
  readonly callback?: () => void;
}

/** Readiness and controls returned by `useTimeout(ms, { controls: true })`. */
export interface TimeoutControls extends TimeoutFnControls<[]> {
  /** `true` once the timer elapsed or was stopped, `false` while pending. */
  readonly ready: ComputedRef<boolean>;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

function resolveDelayMs(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_TIMEOUT_INVALID_DELAY] delayMs must be finite and at least zero; received ${String(value)}`,
    );
  }
  return Math.trunc(value);
}

/**
 * Call a function once after a delay, with restart and cancel controls.
 *
 * The typing is arity-aware: a zero-argument callback may start
 * immediately, while a callback with parameters requires
 * `{ immediate: false }` so it can only ever run with arguments supplied to
 * `start(...)`. `delayMs` is reactive and read on every `start`.
 *
 * Server rendering is hydration-stable: without a browser `window` (and with
 * {@link UseTimeoutFnOptions.runOnServer} disabled) `isPending` reports the
 * requested state but no timer is created. A pending timer is cleared when
 * the owning reactive scope stops.
 *
 * @example
 * ```ts
 * const { start, stop } = useTimeoutFn((id: string) => save(id), 500, { immediate: false });
 * start("draft-1");
 * ```
 *
 * @param callback Invoked once the delay elapses.
 * @param delayMs Reactive delay in milliseconds; finite and at least zero.
 * @param options Start, server, and scheduler policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_TIMEOUT_INVALID_DELAY` for invalid
 * delays, synchronously at creation and on every `start`.
 * @returns Pending flag and start/stop controls.
 */
export function useTimeoutFn(
  callback: () => void,
  delayMs: MaybeRefOrGetter<number>,
  options?: UseTimeoutFnOptions,
): TimeoutFnControls<[]>;
export function useTimeoutFn<Args extends readonly unknown[]>(
  callback: (...args: Args) => void,
  delayMs: MaybeRefOrGetter<number>,
  options: UseTimeoutFnOptions & { readonly immediate: false },
): TimeoutFnControls<Args>;
export function useTimeoutFn(
  callback: (...args: readonly unknown[]) => void,
  delayMs: MaybeRefOrGetter<number>,
  options: UseTimeoutFnOptions = {},
): TimeoutFnControls<readonly unknown[]> {
  const scheduler = options.scheduler ?? defaultScheduler;
  const isPending = shallowRef(false);
  let handle: unknown;
  let scheduled = false;

  resolveDelayMs(toValue(delayMs));

  const clear = (): void => {
    if (!scheduled) return;
    scheduled = false;
    scheduler.clearTimeout(handle);
    handle = undefined;
  };

  const stop = (): boolean => {
    const wasPending = isPending.value;
    isPending.value = false;
    clear();
    return wasPending;
  };

  const start = (...args: readonly unknown[]): void => {
    const delay = resolveDelayMs(toValue(delayMs));
    clear();
    isPending.value = true;
    if (typeof window === "undefined" && !(options.runOnServer ?? false)) return;
    scheduled = true;
    handle = scheduler.setTimeout(() => {
      scheduled = false;
      handle = undefined;
      isPending.value = false;
      callback(...args);
    }, delay);
  };

  // Only the zero-argument overload may start immediately.
  if (options.immediate ?? true) start();

  tryOnScopeDispose(() => {
    stop();
  });

  return { isPending, start, stop };
}

/**
 * Reactive "has the delay elapsed yet?" flag.
 *
 * A wrapper over {@link useTimeoutFn}. Pass `{ controls: true }` to receive
 * start/stop alongside the flag; the return type follows the literal flag.
 * Stopping a pending timer marks it ready.
 *
 * @example
 * ```ts
 * const ready = useTimeout(300); // ComputedRef<boolean>
 * const { ready: done, start } = useTimeout(300, { controls: true, immediate: false });
 * ```
 *
 * @param delayMs Reactive delay in milliseconds.
 * @param options Timeout options plus the `controls` flag.
 * @default delayMs 1000
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_TIMEOUT_INVALID_DELAY` for invalid
 * delays (see {@link useTimeoutFn}).
 * @returns The readiness ref, or the full controls when `controls` is `true`.
 */
export function useTimeout(
  delayMs?: MaybeRefOrGetter<number>,
  options?: UseTimeoutOptions<false>,
): ComputedRef<boolean>;
export function useTimeout(
  delayMs: MaybeRefOrGetter<number>,
  options: UseTimeoutOptions<true>,
): TimeoutControls;
export function useTimeout(
  delayMs: MaybeRefOrGetter<number> = 1_000,
  options: UseTimeoutOptions<boolean> = {},
): ComputedRef<boolean> | TimeoutControls {
  const controls = useTimeoutFn(() => options.callback?.(), delayMs, options);
  const ready = computed(() => !controls.isPending.value);
  if (options.controls !== true) return ready;
  return { ...controls, ready };
}
