import { shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

/**
 * Track the value a reactive source held before its latest change.
 *
 * The semantics are precise and arity-based:
 *
 * - Without an `initial` argument the ref holds `undefined` until the source
 *   changes for the first time.
 * - With an `initial` argument (including an explicit `undefined` when
 *   `Value` allows it) the ref holds that value until the first change, and
 *   the return type never widens with `undefined`.
 * - Every synchronous write is observed (`flush: "sync"`), so a sequence of
 *   writes in one tick shifts the previous value step by step instead of
 *   collapsing into one batch.
 * - Writes whose value is `Object.is`-equal to the current value do not
 *   count as changes, matching Vue's own change detection.
 * - Tracking is shallow: reassignments are observed, in-place mutations of
 *   object values are not.
 *
 * Safe during server rendering: no browser globals are read and no timers
 * start. The underlying watcher is bound to the current reactive scope and
 * stops with it; call inside an active scope, or the watcher lives as long
 * as the source. A plain non-reactive source never changes, so the ref stays
 * at its initial value.
 *
 * @example
 * ```ts
 * const route = shallowRef("/home");
 * const previousRoute = usePrevious(route, "/");
 * route.value = "/settings";
 * previousRoute.value; // "/home"
 * ```
 *
 * @param source Reactive source to observe.
 * @param initial Value reported before the first change.
 * @returns Readonly shallow ref holding the previous source value.
 */
export function usePrevious<Value>(
  source: MaybeRefOrGetter<Value>,
): Readonly<ShallowRef<Value | undefined>>;
export function usePrevious<Value>(
  source: MaybeRefOrGetter<Value>,
  initial: Value,
): Readonly<ShallowRef<Value>>;
export function usePrevious<Value>(
  source: MaybeRefOrGetter<Value>,
  ...initial: readonly [] | readonly [Value]
): Readonly<ShallowRef<Value | undefined>> {
  const previous = shallowRef<Value | undefined>(initial.length === 1 ? initial[0] : undefined);

  watch(
    () => toValue(source),
    (_next, replaced) => {
      previous.value = replaced;
    },
    { flush: "sync" },
  );

  return previous;
}
