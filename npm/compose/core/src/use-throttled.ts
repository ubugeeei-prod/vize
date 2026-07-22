import { shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Options for {@link useThrottled}. */
export interface UseThrottledOptions {
  /**
   * Apply the first change of a cooldown window immediately.
   *
   * @default true
   */
  readonly leading?: boolean;

  /**
   * Apply the newest change collected during a cooldown window when the
   * window ends. When disabled, changes inside a window are dropped.
   *
   * @default true
   */
  readonly trailing?: boolean;

  /**
   * Applies the timing policy when no browser `window` is available.
   *
   * Keep this disabled during server rendering, where the throttled view
   * mirrors the source synchronously instead of starting timers. Enable it
   * for native, desktop, worker, and test runtimes whose scheduler is
   * lifecycle-bound.
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Owns the single-shot cooldown timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

/** Reactive throttled view and controls returned by {@link useThrottled}. */
export interface ThrottledControls<Value> {
  /** Readonly view of the source updated at most once per cooldown window. */
  readonly throttled: Readonly<ShallowRef<Value>>;

  /** Whether a trailing update is waiting for the current window to end. */
  readonly pending: Readonly<ShallowRef<boolean>>;

  /**
   * Discard the waiting trailing update and close the cooldown window, so
   * the next change starts fresh on a leading edge.
   *
   * @returns Whether a waiting trailing update was discarded.
   */
  readonly cancel: () => boolean;

  /**
   * Apply the waiting trailing update immediately and close the cooldown
   * window. Without a waiting update the window is left untouched.
   *
   * @returns Whether a waiting trailing update was applied.
   */
  readonly flush: () => boolean;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

/**
 * Create a readonly throttled view of a reactive source.
 *
 * Changes are observed with `flush: "sync"`, so every synchronous write
 * counts. Outside a cooldown window, a change applies immediately when
 * {@link UseThrottledOptions.leading} is enabled (otherwise it waits as a
 * trailing update) and opens a window of `waitMs` milliseconds. Changes
 * inside a window are collected as the trailing candidate; when the window
 * ends with a candidate waiting, the source value current at that moment is
 * applied and the next window opens back to back, keeping applications
 * spaced by `waitMs`. A window that ends without a candidate closes silently.
 * `waitMs` is reactive and is read each time a window opens; changing it
 * never disturbs an already-open window. A wait of `0` still defers trailing
 * updates to the next scheduler tick.
 *
 * Server rendering is explicit: without a browser `window` (and with
 * {@link UseThrottledOptions.runOnServer} disabled) no timer ever starts and
 * the view mirrors the source synchronously, so server-rendered output shows
 * current values and nothing leaks. `pending` stays `false` and the controls
 * report `false` in that mode.
 *
 * Cleanup rule: the watcher and any open window timer are released when the
 * owning reactive scope stops; call inside an active scope. Outside one, the
 * watcher lives as long as the source and `cancel()` only clears the window.
 *
 * @example
 * ```ts
 * const scrollY = shallowRef(0);
 * const { throttled } = useThrottled(scrollY, 100);
 * scrollY.value = 40; // applied immediately (leading edge)
 * scrollY.value = 80; // applied when the 100ms window ends
 * ```
 *
 * @param source Reactive source to throttle.
 * @param waitMs Reactive cooldown in milliseconds; must be finite and at
 * least zero. Fractions are truncated.
 * @param options Edge policy and runtime scheduling overrides.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_THROTTLE_INVALID_WAIT` when the
 * resolved wait is not finite or is negative, both synchronously at creation
 * (even in mirror mode) and again each time a window opens.
 * @throws `TypeError` tagged `VIZE_COMPOSE_THROTTLE_INVALID_EDGES` when both
 * `leading` and `trailing` are disabled, because updates could then never
 * propagate.
 * @returns Readonly throttled view, pending flag, and cancel/flush controls.
 */
export function useThrottled<Value>(
  source: MaybeRefOrGetter<Value>,
  waitMs: MaybeRefOrGetter<number>,
  options: UseThrottledOptions = {},
): ThrottledControls<Value> {
  const leading = options.leading ?? true;
  const trailing = options.trailing ?? true;
  if (!leading && !trailing) {
    throw new TypeError(
      "[VIZE_COMPOSE_THROTTLE_INVALID_EDGES] at least one of leading or trailing must be enabled",
    );
  }
  const scheduler = options.scheduler ?? defaultScheduler;
  const throttled = shallowRef(toValue(source));
  const pending = shallowRef(false);
  let windowHandle: unknown;
  let windowOpen = false;

  // Fail synchronously for invalid initial input, even in mirror mode.
  resolveWaitMs(toValue(waitMs));

  const openWindow = (): void => {
    windowOpen = true;
    windowHandle = scheduler.setTimeout(
      () => {
        windowHandle = undefined;
        if (!pending.value) {
          windowOpen = false;
          return;
        }
        pending.value = false;
        throttled.value = toValue(source);
        openWindow();
      },
      resolveWaitMs(toValue(waitMs)),
    );
  };

  const closeWindow = (): void => {
    if (windowOpen) scheduler.clearTimeout(windowHandle);
    windowHandle = undefined;
    windowOpen = false;
    pending.value = false;
  };

  const cancel = (): boolean => {
    const hadTrailing = pending.value;
    closeWindow();
    return hadTrailing;
  };

  const flush = (): boolean => {
    if (!pending.value) return false;
    closeWindow();
    throttled.value = toValue(source);
    return true;
  };

  watch(
    () => toValue(source),
    (next) => {
      if (typeof window === "undefined" && !(options.runOnServer ?? false)) {
        throttled.value = next;
        return;
      }
      if (windowOpen) {
        if (trailing) pending.value = true;
        return;
      }
      if (leading) {
        throttled.value = next;
      } else {
        pending.value = true;
      }
      openWindow();
    },
    { flush: "sync" },
  );

  tryOnScopeDispose(() => {
    cancel();
  });

  return { throttled, pending, cancel, flush };
}

function resolveWaitMs(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_THROTTLE_INVALID_WAIT] waitMs must be finite and at least zero; received ${String(value)}`,
    );
  }
  return Math.trunc(value);
}
