import { watch } from "vue";
import type { WatchHandle, WatchOptions, WatchSource } from "vue";

import type { WatchCleanupRegistrar } from "./watch-source.ts";

/** Values treated as "not yet" by {@link whenever}. */
export type Falsy = false | 0 | 0n | "" | null | undefined;

/** Options for {@link whenever}. */
export type WheneverOptions = Omit<WatchOptions<boolean>, "once"> & {
  /**
   * Stop after the first truthy delivery instead of after the first change.
   *
   * @default false
   */
  readonly once?: boolean;
};

/**
 * Run a callback whenever a source becomes (or changes while) truthy.
 *
 * The callback's value is narrowed with `Exclude<Value, Falsy>`, so a
 * `Ref<User | null>` delivers `User`. With `once`, the watcher stops after
 * the first truthy delivery, not after the first change (unlike Vue's
 * `once`). With `immediate`, an already-truthy source is delivered right
 * away. Safe during server rendering and bound to the owning scope.
 *
 * @example
 * ```ts
 * whenever(currentUser, (user) => greet(user.name), { immediate: true, once: true });
 * ```
 *
 * @param source Ref or getter.
 * @param callback Receives the truthy value, the previous value, and
 * `onCleanup`.
 * @param options Vue watch options; `once` counts truthy deliveries only.
 * @default options {}
 * @returns Vue's watch handle.
 */
export function whenever<Value>(
  source: WatchSource<Value>,
  callback: (
    value: Exclude<Value, Falsy>,
    oldValue: Value | undefined,
    onCleanup: WatchCleanupRegistrar,
  ) => void,
  options: WheneverOptions = {},
): WatchHandle {
  const { once = false, ...watchOptions } = options;
  let handle: WatchHandle | undefined;
  let stopRequested = false;
  handle = watch(
    source,
    (value, oldValue, onCleanup) => {
      if (!isTruthy(value)) return;
      if (once) {
        if (handle === undefined) stopRequested = true;
        else handle.stop();
      }
      callback(value, oldValue, onCleanup);
    },
    watchOptions,
  );
  if (stopRequested) handle.stop();
  return handle;
}

function isTruthy<Value>(value: Value): value is Exclude<Value, Falsy> {
  return Boolean(value);
}
