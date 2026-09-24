import {
  computed,
  hasInjectionContext,
  readonly,
  shallowRef,
  toValue,
  watch,
  watchPostEffect,
} from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Deadline handed to idle callbacks. */
export interface IdleDeadlineLike {
  /** Whether the callback runs because its `timeout` elapsed. */
  readonly didTimeout: boolean;
  /** Milliseconds left in the current idle period. */
  timeRemaining(): number;
}

/** Minimal `requestIdleCallback` capability. */
export interface IdleCallbackHost {
  /** Schedule `callback` for the next idle period. */
  requestIdleCallback(
    callback: (deadline: IdleDeadlineLike) => void,
    options?: { readonly timeout?: number },
  ): number;
  /** Cancel a scheduled callback. */
  cancelIdleCallback(handle: number): void;
}

/** Scheduling options shared by {@link useRequestIdleCallback} and {@link idle}. */
export interface IdleSchedulingOptions {
  /**
   * `requestIdleCallback` capability for alternate runtimes and tests.
   *
   * @default window when it implements requestIdleCallback
   */
  readonly host?: MaybeRefOrGetter<IdleCallbackHost | null | undefined>;

  /**
   * Timers used by the fallback when `host` is missing. Without timers
   * (server rendering) nothing is scheduled.
   *
   * @default window.setTimeout and window.clearTimeout when a window exists
   */
  readonly timers?: MaybeRefOrGetter<TimeoutScheduler | null | undefined>;

  /**
   * Run the callback after this many milliseconds even when the browser is
   * never idle (`deadline.didTimeout` is then true).
   *
   * @default undefined (no timeout)
   */
  readonly timeout?: number;

  /**
   * Clock used by the fallback deadline, in milliseconds.
   *
   * @default Date.now
   */
  readonly now?: () => number;
}

/** Options for {@link useRequestIdleCallback}. */
export interface UseRequestIdleCallbackOptions extends IdleSchedulingOptions {
  /**
   * Schedule the callback as soon as the composable is created.
   *
   * @default true
   */
  readonly immediate?: boolean;
}

/** Options for {@link idle}. */
export interface IdleOptions extends IdleSchedulingOptions {
  /**
   * Cancels the wait; the promise rejects with `signal.reason`.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;
}

/** Reactive state and actions returned by {@link useRequestIdleCallback}. */
export interface RequestIdleCallbackControls {
  /** Whether the native `requestIdleCallback` API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Whether a callback is scheduled and has not run yet. */
  readonly isPending: Readonly<ShallowRef<boolean>>;

  /** Schedule the callback, replacing a pending one. No-op on the server. */
  readonly start: () => void;

  /** Cancel the pending callback. Repeated calls are safe. */
  readonly cancel: () => void;
}

/** Idle period length fabricated by the fallback, in milliseconds (the spec maximum). */
const FALLBACK_IDLE_PERIOD = 50;

function browserIdleHost(): IdleCallbackHost | undefined {
  return typeof window !== "undefined" && typeof window.requestIdleCallback === "function"
    ? window
    : undefined;
}

function browserTimers(): TimeoutScheduler | undefined {
  if (typeof window === "undefined") return undefined;
  const target = window;
  return {
    setTimeout: (callback, delayMs) => target.setTimeout(callback, delayMs),
    clearTimeout: (handle) => {
      if (typeof handle === "number") target.clearTimeout(handle);
    },
  };
}

function validateTimeout(timeout: number | undefined): void {
  if (timeout !== undefined && (!Number.isFinite(timeout) || timeout < 0)) {
    throw new RangeError(
      `[VIZE_COMPOSE_IDLE_CALLBACK_INVALID_TIMEOUT] timeout must be a finite, non-negative number, received ${String(timeout)}`,
    );
  }
}

/** Schedule once; returns a canceller, or undefined when nothing could be scheduled. */
function scheduleIdle(
  options: IdleSchedulingOptions,
  callback: (deadline: IdleDeadlineLike) => void,
): (() => void) | undefined {
  const host = options.host === undefined ? browserIdleHost() : toValue(options.host);
  if (host) {
    const handle = host.requestIdleCallback(
      callback,
      options.timeout === undefined ? undefined : { timeout: options.timeout },
    );
    return () => host.cancelIdleCallback(handle);
  }
  const timers = options.timers === undefined ? browserTimers() : toValue(options.timers);
  if (!timers) return undefined;
  const now = options.now ?? Date.now;
  const handle = timers.setTimeout(() => {
    const start = now();
    callback({
      didTimeout: false,
      timeRemaining: () => Math.max(0, FALLBACK_IDLE_PERIOD - (now() - start)),
    });
  }, 1);
  return () => timers.clearTimeout(handle);
}

/**
 * Run a callback in the next idle period with `requestIdleCallback`.
 *
 * When the API is missing, a `setTimeout` fallback fabricates a deadline
 * with a 50 ms budget and `didTimeout: false`. The pending callback is
 * cancelled when the owning reactive scope stops; outside a scope the
 * caller owns `cancel()`.
 *
 * Server rendering: nothing is scheduled, `isPending` and `supported` are
 * false. Inside a component the first callback is scheduled after mounting,
 * so hydration renders the server state first.
 *
 * @example
 * ```ts
 * const { start } = useRequestIdleCallback((deadline) => {
 *   while (deadline.timeRemaining() > 1 && queue.length) queue.shift()?.();
 * }, { timeout: 2000, immediate: false });
 * ```
 *
 * @param callback Receives the idle deadline.
 * @param options Hosts, timeout and start policy.
 * @default options {}
 * @returns Scheduling state and actions.
 */
export function useRequestIdleCallback(
  callback: (deadline: IdleDeadlineLike) => void,
  options: UseRequestIdleCallbackOptions = {},
): RequestIdleCallbackControls {
  validateTimeout(options.timeout);
  const isPending = shallowRef(false);
  let cancelPending: (() => void) | undefined;

  const cancel = (): void => {
    cancelPending?.();
    cancelPending = undefined;
    isPending.value = false;
  };

  const start = (): void => {
    cancel();
    cancelPending = scheduleIdle(options, (deadline) => {
      cancelPending = undefined;
      isPending.value = false;
      callback(deadline);
    });
    isPending.value = cancelPending !== undefined;
  };

  // Inside a component the first callback is scheduled (and support is
  // reported) only after mounting, so a hydrating client renders the same
  // idle state as the server. Outside components it starts synchronously.
  const hydrated = shallowRef(!hasInjectionContext());
  if (hydrated.value) {
    if (options.immediate ?? true) start();
  } else {
    const stopReady = watch(
      hydrated,
      (ready) => {
        if (!ready) return;
        stopReady();
        if (options.immediate ?? true) start();
      },
      { flush: "sync" },
    );
    watchPostEffect(() => {
      hydrated.value = true;
    });
  }
  tryOnScopeDispose(cancel);

  return {
    supported: computed(
      () =>
        hydrated.value &&
        (options.host === undefined
          ? browserIdleHost() !== undefined
          : Boolean(toValue(options.host))),
    ),
    isPending: readonly(isPending),
    start,
    cancel,
  };
}

/**
 * Resolve in the next idle period (or after `timeout`).
 *
 * Uses the same hosts and fallback as {@link useRequestIdleCallback}. When
 * neither an idle host nor timers resolve (server rendering) it resolves on
 * the next microtask with an exhausted deadline instead of hanging.
 *
 * @example
 * ```ts
 * await idle({ timeout: 1000 });
 * ```
 *
 * @param options Hosts, timeout and abort signal.
 * @default options {}
 * @returns The idle deadline; rejects with `signal.reason` when aborted.
 */
export function idle(options: IdleOptions = {}): Promise<IdleDeadlineLike> {
  validateTimeout(options.timeout);
  const { signal } = options;
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(signal.reason);
      return;
    }
    const onAbort = (): void => {
      cancel?.();
      reject(signal?.reason);
    };
    const cancel = scheduleIdle(options, (deadline) => {
      signal?.removeEventListener("abort", onAbort);
      resolve(deadline);
    });
    if (!cancel) {
      resolve({ didTimeout: false, timeRemaining: () => 0 });
      return;
    }
    signal?.addEventListener("abort", onAbort, { once: true });
  });
}
