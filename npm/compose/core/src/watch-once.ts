import { watch } from "vue";
import type { WatchCallback, WatchHandle, WatchOptions } from "vue";

import type { WatchHelperCallback, WatchSources } from "./watch-source.ts";

/** Options for {@link watchOnce}: Vue watch options without `once`. */
export type WatchOnceOptions<Immediate extends boolean = false> = Omit<
  WatchOptions<Immediate>,
  "once"
>;

/**
 * Watch sources like Vue's `watch`, delivering exactly one change and then
 * stopping.
 *
 * A thin, typed shorthand for `watch(source, callback, { once: true })`:
 * with `immediate: true` the single call is the immediate one. The returned
 * handle can stop the watcher before it fires. Safe during server rendering
 * and bound to the owning reactive scope.
 *
 * @example
 * ```ts
 * watchOnce(user, (loaded) => track("first-user", loaded.id));
 * ```
 *
 * @param source Ref, getter, reactive object, or tuple of those.
 * @param callback Called at most once.
 * @param options Vue watch options except `once`.
 * @default options {}
 * @returns Vue's watch handle.
 */
export function watchOnce<
  const Sources extends WatchSources,
  Immediate extends Readonly<boolean> = false,
>(
  source: Sources,
  callback: WatchHelperCallback<Sources, Immediate>,
  options?: WatchOnceOptions<Immediate>,
): WatchHandle;
export function watchOnce(
  source: WatchSources,
  callback: WatchCallback<unknown, unknown>,
  options: WatchOnceOptions<boolean> = {},
): WatchHandle {
  return watch(source, callback, { ...options, once: true });
}
