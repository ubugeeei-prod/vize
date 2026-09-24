import { toValue, watch } from "vue";
import type { MaybeRefOrGetter, WatchCallback, WatchOptions } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import type { TimedWatchHandle } from "./watch-debounced.ts";
import type { WatchCleanupRegistrar, WatchHelperCallback, WatchSources } from "./watch-source.ts";

/** Options for {@link watchThrottled}. */
export interface WatchThrottledOptions<
  Immediate extends boolean = false,
> extends WatchOptions<Immediate> {
  /**
   * Cooldown window in milliseconds. Reactive; read whenever a window opens.
   *
   * @default 0
   */
  readonly throttle?: MaybeRefOrGetter<number>;

  /**
   * Run the first change of a window immediately.
   *
   * @default true
   */
  readonly leading?: boolean;

  /**
   * Run the newest change collected during a window when it closes.
   *
   * @default true
   */
  readonly trailing?: boolean;

  /**
   * Applies the timing policy when no browser `window` is available. When
   * disabled, the callback runs synchronously on the server.
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Owns the cooldown timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

function resolveThrottleMs(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_WATCH_THROTTLED_INVALID_DELAY] throttle must be finite and at least zero; received ${String(value)}`,
    );
  }
  return Math.trunc(value);
}

/**
 * Watch sources like Vue's `watch`, but run the callback at most once per
 * cooldown window.
 *
 * With the defaults (`leading` and `trailing` both on) the first change runs
 * immediately and the newest change inside the window runs when it closes,
 * which opens a new window. Every call receives, as its old value, the value
 * of the previously delivered call, so consecutive calls chain without gaps. Source
 * typing mirrors `watch`.
 *
 * Without a browser `window` (and without `runOnServer`) the callback runs
 * synchronously. Timers are cleared when the watcher stops or the owning
 * scope is disposed.
 *
 * @example
 * ```ts
 * watchThrottled(scrollY, (y) => report(y), { throttle: 100 });
 * ```
 *
 * @param source Ref, getter, reactive object, or tuple of those.
 * @param callback Throttled watch callback.
 * @param options Watch options plus the throttle policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_WATCH_THROTTLED_INVALID_DELAY`
 * when `throttle` resolves to a negative or non-finite value.
 * @returns Stop, flush, and cancel controls.
 */
export function watchThrottled<
  const Sources extends WatchSources,
  Immediate extends Readonly<boolean> = false,
>(
  source: Sources,
  callback: WatchHelperCallback<Sources, Immediate>,
  options?: WatchThrottledOptions<Immediate>,
): TimedWatchHandle;
export function watchThrottled(
  source: WatchSources,
  callback: WatchCallback<unknown, unknown>,
  options: WatchThrottledOptions<boolean> = {},
): TimedWatchHandle {
  const {
    throttle = 0,
    leading = true,
    trailing = true,
    runOnServer = false,
    scheduler = defaultScheduler,
    ...watchOptions
  } = options;
  resolveThrottleMs(toValue(throttle));

  let timer: unknown;
  let pending: (() => void) | undefined;
  let lastDelivered: unknown;
  let pendingOld: unknown;
  let delivered = false;
  let cleanup: (() => void) | undefined;

  const invoke = (value: unknown, oldValue: unknown): void => {
    const runCleanup = cleanup;
    cleanup = undefined;
    runCleanup?.();
    lastDelivered = value;
    delivered = true;
    const onCleanup: WatchCleanupRegistrar = (fn) => {
      cleanup = fn;
    };
    callback(value, oldValue, onCleanup);
  };

  const closeWindow = (): void => {
    timer = undefined;
    const run = pending;
    pending = undefined;
    if (run === undefined) return;
    run();
    openWindow();
  };

  function openWindow(): void {
    timer = scheduler.setTimeout(closeWindow, resolveThrottleMs(toValue(throttle)));
  }

  const cancel = (): boolean => {
    if (timer !== undefined) scheduler.clearTimeout(timer);
    timer = undefined;
    const hadPending = pending !== undefined;
    pending = undefined;
    return hadPending;
  };

  const flush = (): boolean => {
    const run = pending;
    if (run === undefined) return false;
    pending = undefined;
    run();
    return true;
  };

  const schedule = (value: unknown, previous: unknown): void => {
    pending = () => {
      invoke(value, previous);
    };
    pendingOld = previous;
  };

  const handle = watch(
    source,
    (value, oldValue) => {
      if (typeof window === "undefined" && !runOnServer) {
        invoke(value, oldValue);
        return;
      }
      const previous = pending === undefined ? (delivered ? lastDelivered : oldValue) : pendingOld;
      if (timer === undefined) {
        if (leading) invoke(value, previous);
        else if (trailing) schedule(value, previous);
        openWindow();
        return;
      }
      if (trailing) schedule(value, previous);
    },
    watchOptions,
  );

  const stop = (): void => {
    handle.stop();
    cancel();
    const runCleanup = cleanup;
    cleanup = undefined;
    runCleanup?.();
  };

  tryOnScopeDispose(stop);

  return { stop, flush, cancel };
}
