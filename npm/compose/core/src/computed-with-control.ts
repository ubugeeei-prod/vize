import { customRef, watch } from "vue";
import type { Ref, WatchSource } from "vue";

/** Readonly cached value plus a manual invalidation trigger. */
export interface ComputedWithControl<Value> extends Readonly<Ref<Value>> {
  /** Mark the cached value stale; the next read recomputes it. */
  readonly trigger: () => void;
}

/** Explicit dependency accepted by {@link computedWithControl}: anything Vue's `watch` accepts. */
export type ComputedWithControlSource =
  | WatchSource<unknown>
  | object
  | readonly (WatchSource<unknown> | object)[];

/** Options for {@link computedWithControl}. */
export interface ComputedWithControlOptions {
  /**
   * Invalidate on nested changes of the explicit sources.
   *
   * @default false
   */
  readonly deep?: boolean;
}

/**
 * Computed value whose dependencies are declared explicitly.
 *
 * Only changes of `source` (or a manual {@link ComputedWithControl.trigger})
 * invalidate the cache; reactive reads inside `getter` do not. Useful when
 * the getter reads non-reactive data or should ignore some reactive reads.
 * The getter receives the previous value and runs lazily on the next read.
 * Pure derived state: SSR-safe; the invalidation watcher follows the owning
 * reactive scope.
 *
 * @example
 * ```ts
 * const snapshot = computedWithControl(version, () => expensiveRead(store));
 * snapshot.trigger(); // force a refresh
 * ```
 *
 * @param source Ref, getter, reactive object, or tuple of those.
 * @param getter Produces the value from the previous one.
 * @param options Depth of the source watch.
 * @default options {}
 * @returns The controlled computed ref.
 */
export function computedWithControl<Value>(
  source: ComputedWithControlSource,
  getter: (previous: Value | undefined) => Value,
  options: ComputedWithControlOptions = {},
): ComputedWithControl<Value> {
  let dirty = true;
  let cached: { readonly value: Value } | undefined;
  let notify = (): void => undefined;

  const invalidate = (): void => {
    dirty = true;
    notify();
  };

  watch(source, invalidate, { flush: "sync", deep: options.deep ?? false });

  const value = customRef<Value>((track, trigger) => {
    notify = trigger;
    return {
      get: () => {
        track();
        if (dirty || cached === undefined) {
          cached = { value: getter(cached?.value) };
          dirty = false;
        }
        return cached.value;
      },
      set: () => undefined,
    };
  });

  return Object.assign(value, { trigger: invalidate });
}
