import { watch } from "vue";
import type { WatchCallback, WatchOptions } from "vue";

import type { WatchHelperCallback, WatchSources } from "./watch-source.ts";

/** Controls returned by {@link watchIgnorable}. */
export interface IgnorableWatchHandle {
  /**
   * Run `updater` so that the source changes it makes synchronously do not
   * trigger the callback.
   */
  readonly ignoreUpdates: (updater: () => void) => void;

  /**
   * Swallow the changes already queued for the next (non-sync) flush. A
   * no-op for `flush: "sync"` watchers, which have nothing queued.
   */
  readonly ignorePrevAsyncUpdates: () => void;

  /** Stop watching permanently. */
  readonly stop: () => void;
}

/**
 * Watch sources like Vue's `watch`, with a way to make changes the callback
 * should not see.
 *
 * Typical use is two-way syncing: writes performed inside `ignoreUpdates`
 * (for example applying a server echo) do not bounce back. Works with every
 * `flush` mode: for `"pre"`/`"post"`, a synchronous side watcher counts
 * changes so that a batch made only of ignored writes is skipped, while a
 * batch that mixes ignored and regular writes is still delivered. Stops
 * with the owning reactive scope; safe during server rendering.
 *
 * @example
 * ```ts
 * const { ignoreUpdates } = watchIgnorable(draft, (value) => save(value));
 * ignoreUpdates(() => {
 *   draft.value = fromServer; // not saved again
 * });
 * ```
 *
 * @param source Ref, getter, reactive object, or tuple of those.
 * @param callback Watch callback for non-ignored changes.
 * @param options Vue watch options.
 * @default options {}
 * @returns Ignore helpers and the stop handle.
 */
export function watchIgnorable<
  const Sources extends WatchSources,
  Immediate extends Readonly<boolean> = false,
>(
  source: Sources,
  callback: WatchHelperCallback<Sources, Immediate>,
  options?: WatchOptions<Immediate>,
): IgnorableWatchHandle;
export function watchIgnorable(
  source: WatchSources,
  callback: WatchCallback<unknown, unknown>,
  options: WatchOptions<boolean> = {},
): IgnorableWatchHandle {
  if (options.flush === "sync") {
    let ignoring = false;
    const handle = watch(
      source,
      (value, oldValue, onCleanup) => {
        if (!ignoring) callback(value, oldValue, onCleanup);
      },
      options,
    );
    return {
      ignoreUpdates: (updater) => {
        const previous = ignoring;
        ignoring = true;
        try {
          updater();
        } finally {
          ignoring = previous;
        }
      },
      ignorePrevAsyncUpdates: () => undefined,
      stop: () => {
        handle.stop();
      },
    };
  }

  let ignoredCount = 0;
  let changeCount = 0;

  const counter = watch(
    source,
    () => {
      changeCount += 1;
    },
    { ...options, flush: "sync", immediate: false, once: false },
  );

  const handle = watch(
    source,
    (value, oldValue, onCleanup) => {
      const ignore = ignoredCount > 0 && ignoredCount === changeCount;
      ignoredCount = 0;
      changeCount = 0;
      if (options.once === true) counter.stop();
      if (!ignore) callback(value, oldValue, onCleanup);
    },
    options,
  );

  return {
    ignoreUpdates: (updater) => {
      const before = changeCount;
      try {
        updater();
      } finally {
        ignoredCount += changeCount - before;
      }
    },
    ignorePrevAsyncUpdates: () => {
      ignoredCount = changeCount;
    },
    stop: () => {
      counter.stop();
      handle.stop();
    },
  };
}
