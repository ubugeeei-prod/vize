import { toValue, watch } from "vue";
import type { MaybeRefOrGetter, WatchCallback, WatchHandle, WatchOptions } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import type { WatchCleanupRegistrar, WatchHelperCallback, WatchSources } from "./watch-source.ts";

/** Options for {@link watchDebounced}. */
export interface WatchDebouncedOptions<
  Immediate extends boolean = false,
> extends WatchOptions<Immediate> {
  /**
   * Quiet period in milliseconds after the last change before the callback
   * runs. Reactive; read whenever a call is scheduled.
   *
   * @default 0
   */
  readonly debounce?: MaybeRefOrGetter<number>;

  /**
   * Longest time a call may be postponed by continuous changes. When it
   * elapses, the newest pending call runs even if changes keep coming.
   *
   * @default undefined (no ceiling)
   */
  readonly maxWait?: MaybeRefOrGetter<number | undefined>;

  /**
   * Applies the timing policy when no browser `window` is available. When
   * disabled, the callback runs synchronously on the server.
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Owns the single-shot timers.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

/** Stop handle plus timing controls returned by the debounced/throttled watchers. */
export interface TimedWatchHandle {
  /** Stop watching and discard any pending call. */
  readonly stop: () => void;

  /**
   * Run the pending call now instead of waiting.
   *
   * @returns Whether a pending call was run.
   */
  readonly flush: () => boolean;

  /**
   * Discard the pending call; later changes schedule again.
   *
   * @returns Whether a pending call was discarded.
   */
  readonly cancel: () => boolean;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

function resolveMs(value: number, name: string): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_WATCH_DEBOUNCED_INVALID_DELAY] ${name} must be finite and at least zero; received ${String(value)}`,
    );
  }
  return Math.trunc(value);
}

/**
 * Watch sources like Vue's `watch`, but run the callback only after the
 * sources stayed quiet for `debounce` milliseconds.
 *
 * The callback receives the newest value together with the old value from
 * before the first change of the burst, so it always describes the whole
 * settled transition. `onCleanup` registered by a callback runs before the
 * next call, as with `watch`. `maxWait` bounds starvation under continuous
 * changes. Source typing mirrors `watch`, including tuples and the
 * `immediate` flag (an immediate first call is debounced too).
 *
 * Without a browser `window` (and without `runOnServer`) the callback runs
 * synchronously, so server-side effects are not silently dropped. Timers are
 * cleared when the watcher stops or the owning scope is disposed.
 *
 * @example
 * ```ts
 * watchDebounced(query, (value) => search(value), { debounce: 300, maxWait: 1_000 });
 * ```
 *
 * @param source Ref, getter, reactive object, or tuple of those.
 * @param callback Debounced watch callback.
 * @param options Watch options plus the debounce policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_WATCH_DEBOUNCED_INVALID_DELAY`
 * when `debounce` or `maxWait` resolve to a negative or non-finite value,
 * synchronously at creation and whenever a call is scheduled.
 * @returns Stop, flush, and cancel controls.
 */
export function watchDebounced<
  const Sources extends WatchSources,
  Immediate extends Readonly<boolean> = false,
>(
  source: Sources,
  callback: WatchHelperCallback<Sources, Immediate>,
  options?: WatchDebouncedOptions<Immediate>,
): TimedWatchHandle;
export function watchDebounced(
  source: WatchSources,
  callback: WatchCallback<unknown, unknown>,
  options: WatchDebouncedOptions<boolean> = {},
): TimedWatchHandle {
  const {
    debounce = 0,
    maxWait,
    runOnServer = false,
    scheduler = defaultScheduler,
    ...watchOptions
  } = options;
  resolveMs(toValue(debounce), "debounce");
  const initialMaxWait = toValue(maxWait);
  if (initialMaxWait !== undefined) resolveMs(initialMaxWait, "maxWait");

  let timer: unknown;
  let maxTimer: unknown;
  let pending: (() => void) | undefined;
  let burstOld: unknown;
  let cleanup: (() => void) | undefined;

  const clearTimers = (): void => {
    if (timer !== undefined) scheduler.clearTimeout(timer);
    if (maxTimer !== undefined) scheduler.clearTimeout(maxTimer);
    timer = undefined;
    maxTimer = undefined;
  };

  const invoke = (value: unknown, oldValue: unknown): void => {
    const runCleanup = cleanup;
    cleanup = undefined;
    runCleanup?.();
    const onCleanup: WatchCleanupRegistrar = (fn) => {
      cleanup = fn;
    };
    callback(value, oldValue, onCleanup);
  };

  const flush = (): boolean => {
    const run = pending;
    if (run === undefined) return false;
    clearTimers();
    pending = undefined;
    run();
    return true;
  };

  const cancel = (): boolean => {
    if (pending === undefined) return false;
    clearTimers();
    pending = undefined;
    return true;
  };

  const handle: WatchHandle = watch(
    source,
    (value, oldValue) => {
      if (typeof window === "undefined" && !runOnServer) {
        invoke(value, oldValue);
        return;
      }
      if (pending === undefined) burstOld = oldValue;
      const settledOld = burstOld;
      pending = () => {
        invoke(value, settledOld);
      };
      const delay = resolveMs(toValue(debounce), "debounce");
      if (timer !== undefined) scheduler.clearTimeout(timer);
      timer = scheduler.setTimeout(flush, delay);
      const ceiling = toValue(maxWait);
      if (ceiling !== undefined && maxTimer === undefined) {
        maxTimer = scheduler.setTimeout(flush, resolveMs(ceiling, "maxWait"));
      }
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
