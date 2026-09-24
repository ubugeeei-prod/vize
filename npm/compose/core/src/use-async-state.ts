import { shallowRef } from "vue";
import type { ShallowRef } from "vue";

/** Options for {@link useAsyncState}. */
export interface UseAsyncStateOptions<Data> {
  /**
   * Execute once right away. Only available for producers without
   * parameters; pass `false` to run a producer that takes arguments through
   * {@link AsyncState.execute}.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Restore the initial state at the start of every execution.
   *
   * @default false
   */
  readonly resetOnExecute?: boolean;

  /**
   * Re-throw failures from `execute` in addition to storing them.
   *
   * @default false
   */
  readonly throwError?: boolean;

  /**
   * Called with every failure of the newest execution.
   *
   * @default undefined
   */
  readonly onError?: (error: unknown) => void;

  /**
   * Called with every successful result of the newest execution.
   *
   * @default undefined
   */
  readonly onSuccess?: (data: Data) => void;
}

/** Reactive state and controls returned by {@link useAsyncState}. */
export interface AsyncState<Data, Initial, Parameters extends readonly unknown[]> {
  /** Newest result, or the initial state. */
  readonly state: Readonly<ShallowRef<Data | Initial>>;

  /** Whether at least one execution succeeded. */
  readonly isReady: Readonly<ShallowRef<boolean>>;

  /** Whether an execution is in flight. */
  readonly isLoading: Readonly<ShallowRef<boolean>>;

  /** Failure of the newest execution, cleared when a new one starts. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Run the producer. Only the newest execution may update the state.
   *
   * @returns The resulting state (the previous state for failed or
   * superseded executions unless `throwError` is set).
   */
  readonly execute: (...args: Parameters) => Promise<Data | Initial>;
}

/**
 * Reactive state for an async producer.
 *
 * Tracks the result, loading, readiness, and error of the newest execution;
 * out-of-order completions of older executions are ignored, so the state
 * always reflects the latest request. The typing is arity-aware: producers
 * with parameters require `{ immediate: false }` and `execute` keeps their
 * parameter tuple. On the server an immediate execution starts but is not
 * awaited, so both sides render the initial state with `isLoading: true`.
 *
 * @example
 * ```ts
 * const { state, isLoading, execute } = useAsyncState((id: string) => fetchUser(id), null, {
 *   immediate: false,
 * });
 * await execute("u-1");
 * ```
 *
 * @param producer Async function producing the data.
 * @param initial State before the first success.
 * @param options Immediate execution, reset, error, and success policy.
 * @default options {}
 * @returns Reactive state and the `execute` control.
 */
export function useAsyncState<Data, Initial>(
  producer: () => Promise<Data>,
  initial: Initial,
  options?: UseAsyncStateOptions<Data>,
): AsyncState<Data, Initial, []>;
export function useAsyncState<Data, Initial, Parameters extends readonly unknown[]>(
  producer: (...args: Parameters) => Promise<Data>,
  initial: Initial,
  options: UseAsyncStateOptions<Data> & { readonly immediate: false },
): AsyncState<Data, Initial, Parameters>;
export function useAsyncState(
  producer: (...args: readonly unknown[]) => Promise<unknown>,
  initial: unknown,
  options: UseAsyncStateOptions<unknown> = {},
): AsyncState<unknown, unknown, readonly unknown[]> {
  const state = shallowRef(initial);
  const isReady = shallowRef(false);
  const isLoading = shallowRef(false);
  const error = shallowRef<unknown>(undefined);
  let generation = 0;

  const execute = async (...args: readonly unknown[]): Promise<unknown> => {
    generation += 1;
    const own = generation;
    if (options.resetOnExecute ?? false) state.value = initial;
    error.value = undefined;
    isLoading.value = true;
    try {
      const data = await producer(...args);
      if (own !== generation) return state.value;
      state.value = data;
      isReady.value = true;
      options.onSuccess?.(data);
    } catch (failure) {
      if (own !== generation) return state.value;
      error.value = failure;
      options.onError?.(failure);
      if (options.throwError ?? false) throw failure;
    } finally {
      if (own === generation) isLoading.value = false;
    }
    return state.value;
  };

  // Only the zero-parameter overload may execute immediately.
  if (options.immediate ?? true) void execute().catch(() => undefined);

  return { state, isReady, isLoading, error, execute };
}
