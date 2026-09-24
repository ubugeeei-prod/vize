import { computed, shallowRef, watchEffect } from "vue";
import type { Ref } from "vue";

/** Registers a callback run when the current evaluation becomes stale. */
export type ComputedAsyncCancel = (onCancel: () => void) => void;

/** Context handed to a {@link computedAsync} evaluator. */
export interface ComputedAsyncContext {
  /** Aborted as soon as a newer evaluation starts or the scope stops. */
  readonly signal: AbortSignal;

  /** Register a callback for when this evaluation becomes stale. */
  readonly onCancel: ComputedAsyncCancel;
}

/** Options for {@link computedAsync}. */
export interface ComputedAsyncOptions {
  /**
   * Defer the first evaluation until the value is read.
   *
   * @default false
   */
  readonly lazy?: boolean;

  /**
   * Ref that is `true` while an evaluation is in flight.
   *
   * @default an internal ref
   */
  readonly evaluating?: Ref<boolean>;

  /**
   * Receives evaluator failures; the previous value is kept.
   *
   * @default undefined (failures are dropped)
   */
  readonly onError?: (error: unknown) => void;
}

/**
 * Computed value backed by an async evaluator.
 *
 * Reactive dependencies read synchronously in the evaluator (before its
 * first `await`) are tracked; when one changes, the previous evaluation is
 * cancelled (its `signal` aborts and `onCancel` callbacks run) and a new one
 * starts. Only the newest evaluation may write the result, so out-of-order
 * responses can never win. The value holds `initial` until the first
 * evaluation settles, which also keeps server and client markup identical
 * during hydration (the server never waits for the promise). Results are
 * stored shallowly: replace the value instead of mutating it.
 *
 * @example
 * ```ts
 * const user = computedAsync(
 *   async ({ signal }) => (await fetch(`/users/${id.value}`, { signal })).json(),
 *   null,
 * );
 * ```
 *
 * @param evaluator Async (or sync) producer of the value.
 * @param initial Value before the first evaluation settles.
 * @param options Laziness, evaluating flag, and error hook.
 * @default options {}
 * @returns Readonly ref holding the newest settled value.
 */
export function computedAsync<Value>(
  evaluator: (context: ComputedAsyncContext) => Value | Promise<Value>,
  initial: Value,
  options?: ComputedAsyncOptions,
): Readonly<Ref<Value>>;
export function computedAsync<Value>(
  evaluator: (context: ComputedAsyncContext) => Value | Promise<Value>,
  initial?: undefined,
  options?: ComputedAsyncOptions,
): Readonly<Ref<Value | undefined>>;
export function computedAsync<Value>(
  evaluator: (context: ComputedAsyncContext) => Value | Promise<Value>,
  initial?: Value,
  options: ComputedAsyncOptions = {},
): Readonly<Ref<Value | undefined>> {
  const started = shallowRef(!(options.lazy ?? false));
  const current = shallowRef<Value | undefined>(initial);
  const evaluating = options.evaluating ?? shallowRef(false);
  let generation = 0;

  watchEffect(async (onCleanup) => {
    if (!started.value) return;
    generation += 1;
    const own = generation;
    const controller = new AbortController();
    const cancelCallbacks: (() => void)[] = [];
    let settled = false;
    onCleanup(() => {
      if (settled) return;
      controller.abort();
      for (const callback of cancelCallbacks) callback();
    });

    evaluating.value = true;
    try {
      const result = await evaluator({
        signal: controller.signal,
        onCancel: (callback) => {
          cancelCallbacks.push(callback);
        },
      });
      if (own === generation) current.value = result;
    } catch (error) {
      if (own === generation) options.onError?.(error);
    } finally {
      settled = true;
      if (own === generation) evaluating.value = false;
    }
  });

  if (options.lazy ?? false) {
    return computed(() => {
      started.value = true;
      return current.value;
    });
  }
  return current;
}
