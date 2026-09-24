import { computed, shallowRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

/** Lifecycle of one queued task. */
export type AsyncQueueTaskState = "pending" | "running" | "fulfilled" | "rejected" | "aborted";

/** Outcome record of one queued task. */
export interface AsyncQueueTaskResult<Data> {
  /** Current lifecycle state. */
  readonly state: AsyncQueueTaskState;

  /** Resolved value once fulfilled. */
  readonly data: Data | undefined;

  /** Failure once rejected. */
  readonly error: unknown;
}

/** Context passed to every queued task. */
export interface AsyncQueueContext {
  /** Aborted when the queue is aborted. */
  readonly signal: AbortSignal;
}

/** Result of the task before position `Index` (`undefined` for the first). */
export type AsyncQueuePrevious<Results extends readonly unknown[], Index> = Index extends keyof [
  undefined,
  ...Results,
]
  ? [undefined, ...Results][Index]
  : never;

/** One queued task: receives the previous result and resolves the next. */
export type AsyncQueueTask<Previous, Result> = (
  previous: Previous,
  context: AsyncQueueContext,
) => Result | Promise<Result>;

/** Task tuple whose parameters chain the previous task's result. */
export type AsyncQueueTasks<Results extends readonly unknown[]> = {
  readonly [Index in keyof Results]: (
    previous: AsyncQueuePrevious<Results, Index>,
    context: AsyncQueueContext,
  ) => Results[Index] | Promise<Results[Index]>;
};

/**
 * Task with its result chain erased. Declared through a method signature so
 * its parameters are compared bivariantly: every typed task is assignable.
 */
type ErasedTask = {
  run(previous: unknown, context: AsyncQueueContext): unknown;
}["run"];

/** Options for {@link useAsyncQueue}. */
export interface UseAsyncQueueOptions {
  /**
   * Stop at the first rejection (remaining tasks become `"aborted"`).
   *
   * @default true
   */
  readonly interrupt?: boolean;

  /**
   * Start running right away.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * External cancellation; aborting it aborts the queue.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;

  /**
   * Called when a task rejects.
   *
   * @default undefined
   */
  readonly onError?: (error: unknown, index: number) => void;

  /**
   * Called once the queue settles.
   *
   * @default undefined
   */
  readonly onFinished?: () => void;
}

/** Queue state and controls returned by {@link useAsyncQueue}. */
export interface AsyncQueue<Results extends readonly unknown[]> {
  /** Index of the running task (`-1` before start, `length` when done). */
  readonly activeIndex: Readonly<ShallowRef<number>>;

  /** Per-task outcome records, typed per position. */
  readonly results: Readonly<
    ShallowRef<{ readonly [Index in keyof Results]: AsyncQueueTaskResult<Results[Index]> }>
  >;

  /** Whether the queue settled (all done, interrupted, or aborted). */
  readonly isFinished: ComputedRef<boolean>;

  /**
   * Run the queue (again) from the first task.
   *
   * @returns Settles with the final records.
   */
  readonly run: () => Promise<{
    readonly [Index in keyof Results]: AsyncQueueTaskResult<Results[Index]>;
  }>;

  /** Abort the running queue. */
  readonly abort: (reason?: unknown) => void;
}

/**
 * Run async tasks one after another, each receiving the previous result.
 *
 * Result types are inferred per position from the task tuple, and each
 * task's `previous` parameter is contextually typed as the preceding task's
 * result (up to six tasks; longer queues use the mapped-tuple signature with
 * annotated parameters).
 * Every task gets an `AbortSignal`; `abort()` (or the external signal)
 * stops the queue and marks unfinished tasks `"aborted"`. A new `run()`
 * aborts the one in flight. Starts on the server too when `immediate`, but
 * nothing awaits it there, so markup shows the pending state on both sides.
 *
 * @example
 * ```ts
 * const { results } = useAsyncQueue([
 *   async () => fetchUser(),
 *   async (user) => fetchOrders(user.id),
 * ]);
 * ```
 *
 * @param tasks Ordered task tuple.
 * @param options Interruption, start, cancellation, and callbacks.
 * @default options {}
 * @returns Queue state and controls.
 */
export function useAsyncQueue<R1>(
  tasks: readonly [AsyncQueueTask<undefined, R1>],
  options?: UseAsyncQueueOptions,
): AsyncQueue<[R1]>;
export function useAsyncQueue<R1, R2>(
  tasks: readonly [AsyncQueueTask<undefined, R1>, AsyncQueueTask<R1, R2>],
  options?: UseAsyncQueueOptions,
): AsyncQueue<[R1, R2]>;
export function useAsyncQueue<R1, R2, R3>(
  tasks: readonly [AsyncQueueTask<undefined, R1>, AsyncQueueTask<R1, R2>, AsyncQueueTask<R2, R3>],
  options?: UseAsyncQueueOptions,
): AsyncQueue<[R1, R2, R3]>;
export function useAsyncQueue<R1, R2, R3, R4>(
  tasks: readonly [
    AsyncQueueTask<undefined, R1>,
    AsyncQueueTask<R1, R2>,
    AsyncQueueTask<R2, R3>,
    AsyncQueueTask<R3, R4>,
  ],
  options?: UseAsyncQueueOptions,
): AsyncQueue<[R1, R2, R3, R4]>;
export function useAsyncQueue<R1, R2, R3, R4, R5>(
  tasks: readonly [
    AsyncQueueTask<undefined, R1>,
    AsyncQueueTask<R1, R2>,
    AsyncQueueTask<R2, R3>,
    AsyncQueueTask<R3, R4>,
    AsyncQueueTask<R4, R5>,
  ],
  options?: UseAsyncQueueOptions,
): AsyncQueue<[R1, R2, R3, R4, R5]>;
export function useAsyncQueue<R1, R2, R3, R4, R5, R6>(
  tasks: readonly [
    AsyncQueueTask<undefined, R1>,
    AsyncQueueTask<R1, R2>,
    AsyncQueueTask<R2, R3>,
    AsyncQueueTask<R3, R4>,
    AsyncQueueTask<R4, R5>,
    AsyncQueueTask<R5, R6>,
  ],
  options?: UseAsyncQueueOptions,
): AsyncQueue<[R1, R2, R3, R4, R5, R6]>;
export function useAsyncQueue<Results extends readonly unknown[]>(
  tasks: AsyncQueueTasks<Results>,
  options?: UseAsyncQueueOptions,
): AsyncQueue<Results>;
export function useAsyncQueue(
  tasks: readonly ErasedTask[],
  options: UseAsyncQueueOptions = {},
): AsyncQueue<readonly unknown[]> {
  const pendingRecords = (): AsyncQueueTaskResult<unknown>[] =>
    tasks.map(() => ({ state: "pending", data: undefined, error: undefined }));
  const activeIndex = shallowRef(-1);
  const results = shallowRef<readonly AsyncQueueTaskResult<unknown>[]>(pendingRecords());
  const isFinished = computed(() => activeIndex.value >= tasks.length);
  let controller: AbortController | undefined;

  const update = (index: number, record: AsyncQueueTaskResult<unknown>): void => {
    results.value = results.value.map((current, position) =>
      position === index ? record : current,
    );
  };
  const abortRemaining = (from: number): void => {
    results.value = results.value.map((current, position) =>
      position >= from && (current.state === "pending" || current.state === "running")
        ? { state: "aborted", data: undefined, error: current.error }
        : current,
    );
  };

  const finish = (): void => {
    activeIndex.value = tasks.length;
    options.onFinished?.();
  };

  const abortRun = (run: AbortController, reason?: unknown): void => {
    if (run.signal.aborted) return;
    run.abort(reason);
    if (controller !== run) return;
    abortRemaining(0);
    finish();
  };

  const run = async (): Promise<readonly AsyncQueueTaskResult<unknown>[]> => {
    controller?.abort();
    const own = new AbortController();
    controller = own;
    results.value = pendingRecords();
    const external = options.signal;
    const onExternalAbort = (): void => abortRun(own, external?.reason);
    if (external?.aborted === true) {
      abortRun(own, external.reason);
      return results.value;
    }
    external?.addEventListener("abort", onExternalAbort, { once: true });

    let previous: unknown;
    try {
      for (const [index, task] of tasks.entries()) {
        activeIndex.value = index;
        update(index, { state: "running", data: undefined, error: undefined });
        try {
          previous = await task(previous, { signal: own.signal });
          if (own.signal.aborted) return results.value;
          update(index, { state: "fulfilled", data: previous, error: undefined });
        } catch (error) {
          if (own.signal.aborted) return results.value;
          update(index, { state: "rejected", data: undefined, error });
          options.onError?.(error, index);
          if (options.interrupt ?? true) {
            abortRemaining(index + 1);
            break;
          }
          previous = undefined;
        }
      }
      finish();
      return results.value;
    } finally {
      external?.removeEventListener("abort", onExternalAbort);
    }
  };

  if (options.immediate ?? true) void run();

  return {
    activeIndex,
    results,
    isFinished,
    run,
    abort: (reason) => {
      if (controller !== undefined) abortRun(controller, reason);
    },
  };
}
