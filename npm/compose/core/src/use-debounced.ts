import { shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Options for {@link useDebounced}. */
export interface UseDebouncedOptions {
  /**
   * Applies the timing policy when no browser `window` is available.
   *
   * Keep this disabled during server rendering, where the debounced view
   * mirrors the source synchronously instead of starting timers. Enable it
   * for native, desktop, worker, and test runtimes whose scheduler is
   * lifecycle-bound.
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

/** Reactive debounced view and controls returned by {@link useDebounced}. */
export interface DebouncedControls<Value> {
  /** Readonly view of the source that settles `waitMs` after the last change. */
  readonly debounced: Readonly<ShallowRef<Value>>;

  /** Whether a trailing update is currently scheduled. */
  readonly pending: Readonly<ShallowRef<boolean>>;

  /**
   * Discard the scheduled trailing update and keep the last settled value.
   * Later source changes debounce again as usual.
   *
   * @returns Whether a scheduled update was discarded.
   */
  readonly cancel: () => boolean;

  /**
   * Apply the current source value immediately instead of waiting out the
   * delay.
   *
   * @returns Whether a scheduled update was applied.
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
 * Create a readonly debounced view of a reactive source.
 *
 * The view starts at the current source value. Each source change (observed
 * with `flush: "sync"`, so every synchronous write counts) restarts a
 * single-shot timer of `waitMs` milliseconds; when it fires, the view takes
 * the source value current at that moment. `waitMs` is reactive and is read
 * when a timer is scheduled; changing it does not restart an already-pending
 * timer. A wait of `0` still defers to the next scheduler tick.
 *
 * Server rendering is explicit: without a browser `window` (and with
 * {@link UseDebouncedOptions.runOnServer} disabled) no timer ever starts and
 * the view mirrors the source synchronously, so server-rendered output shows
 * current values and nothing leaks. `pending` stays `false` and the controls
 * report `false` in that mode.
 *
 * Cleanup rule: the watcher and any pending timer are released when the
 * owning reactive scope stops; call inside an active scope. Outside one, the
 * watcher lives as long as the source and `cancel()` only clears the pending
 * timer.
 *
 * @example
 * ```ts
 * const query = shallowRef("");
 * const { debounced, flush } = useDebounced(query, 300);
 * query.value = "vize"; // debounced.value still "" for 300ms
 * flush(); // debounced.value === "vize" immediately
 * ```
 *
 * @param source Reactive source to debounce.
 * @param waitMs Reactive delay in milliseconds; must be finite and at least
 * zero. Fractions are truncated.
 * @param options Runtime scheduling overrides.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_DEBOUNCE_INVALID_WAIT` when the
 * resolved wait is not finite or is negative, both synchronously at creation
 * (even in mirror mode) and again for every scheduled delay.
 * @returns Readonly debounced view, pending flag, and cancel/flush controls.
 */
export function useDebounced<Value>(
  source: MaybeRefOrGetter<Value>,
  waitMs: MaybeRefOrGetter<number>,
  options: UseDebouncedOptions = {},
): DebouncedControls<Value> {
  const scheduler = options.scheduler ?? defaultScheduler;
  const debounced = shallowRef(toValue(source));
  const pending = shallowRef(false);
  let handle: unknown;

  // Fail synchronously for invalid initial input, even in mirror mode.
  resolveWaitMs(toValue(waitMs));

  const apply = (): void => {
    handle = undefined;
    pending.value = false;
    debounced.value = toValue(source);
  };

  const cancel = (): boolean => {
    if (!pending.value) return false;
    scheduler.clearTimeout(handle);
    handle = undefined;
    pending.value = false;
    return true;
  };

  const flush = (): boolean => {
    if (!pending.value) return false;
    scheduler.clearTimeout(handle);
    apply();
    return true;
  };

  watch(
    () => toValue(source),
    (next) => {
      if (typeof window === "undefined" && !(options.runOnServer ?? false)) {
        debounced.value = next;
        return;
      }
      const delayMs = resolveWaitMs(toValue(waitMs));
      if (pending.value) scheduler.clearTimeout(handle);
      pending.value = true;
      handle = scheduler.setTimeout(apply, delayMs);
    },
    { flush: "sync" },
  );

  tryOnScopeDispose(() => {
    cancel();
  });

  return { debounced, pending, cancel, flush };
}

function resolveWaitMs(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_DEBOUNCE_INVALID_WAIT] waitMs must be finite and at least zero; received ${String(value)}`,
    );
  }
  return Math.trunc(value);
}
