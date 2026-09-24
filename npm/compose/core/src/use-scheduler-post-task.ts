import { computed, readonly, shallowRef, toValue, unref } from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Priorities defined by the Prioritized Task Scheduling API. */
export type SchedulerTaskPriority = "user-blocking" | "user-visible" | "background";

/** Options passed to {@link SchedulerHost.postTask}. */
export interface SchedulerPostTaskInit {
  /** Fixed task priority; overrides the signal's priority. */
  readonly priority?: SchedulerTaskPriority;
  /** Delay before the task is queued, in milliseconds. */
  readonly delay?: number;
  /** Aborts the task (a `TaskSignal` also carries a mutable priority). */
  readonly signal?: AbortSignal;
}

/** Minimal `scheduler` global. */
export interface SchedulerHost {
  /** Queue `callback`; resolves with its result, rejects with the abort reason. */
  postTask<Result>(
    callback: () => Result | PromiseLike<Result>,
    options?: SchedulerPostTaskInit,
  ): Promise<Result>;
  /** Yield to the event loop, continuing with inherited priority. */
  yield?(): Promise<void>;
}

/** Minimal `TaskController` instance. */
export interface TaskControllerLike {
  /** Signal handed to `postTask`. */
  readonly signal: AbortSignal;
  /** Abort every task using this controller's signal. */
  abort(reason?: unknown): void;
  /** Change the priority of every task using this controller's signal. */
  setPriority(priority: SchedulerTaskPriority): void;
}

/** Minimal `TaskController` constructor. */
export type TaskControllerHost = new (init?: {
  readonly priority?: SchedulerTaskPriority;
}) => TaskControllerLike;

/** Options for {@link useSchedulerPostTask}. */
export interface UseSchedulerPostTaskOptions {
  /**
   * Prioritized Task Scheduling capability for alternate runtimes and tests.
   *
   * @default window.scheduler when it implements postTask
   */
  readonly scheduler?: MaybeRefOrGetter<SchedulerHost | null | undefined>;

  /**
   * `TaskController` constructor. A ref (not a getter) because the host is a
   * constructor function. Without it tasks use a plain `AbortController`
   * and receive the priority explicitly, so `setPriority` cannot re-prioritize
   * them.
   *
   * @default window.TaskController when it exists
   */
  readonly TaskController?: MaybeRef<TaskControllerHost | null | undefined>;

  /**
   * Timers used when `scheduler` is missing. Without timers (server
   * rendering) `postTask` rejects.
   *
   * @default window.setTimeout and window.clearTimeout when a window exists
   */
  readonly timers?: MaybeRefOrGetter<TimeoutScheduler | null | undefined>;

  /**
   * Initial priority of tasks posted without an explicit priority.
   *
   * @default "user-visible"
   */
  readonly priority?: SchedulerTaskPriority;
}

/** Per-call options of {@link SchedulerPostTaskControls.postTask}. */
export interface PostTaskCallOptions {
  /**
   * Fixed priority for this task; `setPriority` no longer affects it.
   *
   * @default the composable priority
   */
  readonly priority?: SchedulerTaskPriority;

  /**
   * Delay before the task is queued, in milliseconds.
   *
   * @default 0
   */
  readonly delay?: number;

  /**
   * Aborts this task only.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;
}

/** Reactive state and actions returned by {@link useSchedulerPostTask}. */
export interface SchedulerPostTaskControls {
  /** Whether the native `scheduler.postTask` API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Priority applied to tasks posted without an explicit priority. */
  readonly priority: Readonly<ShallowRef<SchedulerTaskPriority>>;

  /** Number of posted tasks that have not settled. */
  readonly pending: Readonly<ShallowRef<number>>;

  /**
   * Post a task.
   *
   * @param callback Task body; may return a value or a promise.
   * @param options Priority, delay and abort signal.
   * @default options {}
   * @returns The callback result; rejects with the abort reason when
   * aborted, or with `[VIZE_COMPOSE_POST_TASK_UNAVAILABLE]` when neither the
   * scheduler nor timers exist (server rendering).
   */
  readonly postTask: <Result>(
    callback: () => Result | PromiseLike<Result>,
    options?: PostTaskCallOptions,
  ) => Promise<Result>;

  /**
   * Re-prioritize the composable and every pending task without a fixed priority.
   *
   * @param priority New priority.
   */
  readonly setPriority: (priority: SchedulerTaskPriority) => void;

  /**
   * Abort every pending task of this composable.
   *
   * @param reason Rejection reason (an AbortError by default).
   */
  readonly abort: (reason?: unknown) => void;

  /**
   * Yield to the event loop: `scheduler.yield()`, else a zero-delay timer,
   * else a resolved promise.
   *
   * @returns Resolves when the continuation may run.
   */
  readonly yield: () => Promise<void>;
}

interface PendingTask {
  readonly controller: TaskControllerLike | AbortController;
  readonly fixed: boolean;
}

const PRIORITIES: readonly string[] = ["user-blocking", "user-visible", "background"];

function isScheduler(candidate: unknown): candidate is SchedulerHost {
  return (
    typeof candidate === "object" &&
    candidate !== null &&
    typeof Reflect.get(candidate, "postTask") === "function"
  );
}

function isTaskControllerHost(candidate: unknown): candidate is TaskControllerHost {
  return typeof candidate === "function";
}

function isTaskController(
  controller: TaskControllerLike | AbortController,
): controller is TaskControllerLike {
  return "setPriority" in controller && typeof controller.setPriority === "function";
}

function browserGlobal(name: string): unknown {
  return typeof window === "undefined" ? undefined : Reflect.get(window, name);
}

function browserTimers(): TimeoutScheduler | undefined {
  if (typeof window === "undefined") return undefined;
  const target = window;
  return {
    setTimeout: (callback, delayMs) => target.setTimeout(callback, delayMs),
    clearTimeout: (handle) => {
      if (typeof handle === "number") target.clearTimeout(handle);
    },
  };
}

function validatePriority(priority: SchedulerTaskPriority | undefined): void {
  if (priority !== undefined && !PRIORITIES.includes(priority)) {
    throw new TypeError(
      `[VIZE_COMPOSE_POST_TASK_INVALID_PRIORITY] priority must be one of ${PRIORITIES.join(", ")}, received ${String(priority)}`,
    );
  }
}

/**
 * Post prioritized tasks with the Prioritized Task Scheduling API.
 *
 * Every task gets its own controller (a `TaskController` when available),
 * so `setPriority` re-prioritizes pending tasks and `abort` rejects them;
 * pending tasks are aborted when the owning reactive scope stops (their
 * promises reject with an AbortError; outside a scope the caller owns
 * `abort()`).
 *
 * Fallback without `scheduler`: tasks run from `setTimeout(delay)` in
 * timer (FIFO) order. Priorities are ignored, so a later "user-blocking"
 * task does not overtake an earlier "background" one, and tasks do not
 * yield to rendering or input the way native tasks do.
 *
 * Server rendering: nothing is scheduled, `supported` is false and
 * `postTask` rejects.
 *
 * @example
 * ```ts
 * const { postTask } = useSchedulerPostTask();
 * const total = await postTask(() => expensiveSum(rows), { priority: "background" });
 * ```
 *
 * @param options Hosts and the initial priority.
 * @default options {}
 * @returns Task state and actions.
 */
export function useSchedulerPostTask(
  options: UseSchedulerPostTaskOptions = {},
): SchedulerPostTaskControls {
  validatePriority(options.priority);
  const priority = shallowRef<SchedulerTaskPriority>(options.priority ?? "user-visible");
  const pending = shallowRef(0);
  const tasks = new Set<PendingTask>();

  const resolveScheduler = (): SchedulerHost | undefined => {
    const candidate: unknown =
      options.scheduler === undefined ? browserGlobal("scheduler") : toValue(options.scheduler);
    return isScheduler(candidate) ? candidate : undefined;
  };
  const resolveTaskController = (): TaskControllerHost | undefined => {
    const candidate: unknown =
      options.TaskController === undefined
        ? browserGlobal("TaskController")
        : unref(options.TaskController);
    return isTaskControllerHost(candidate) ? candidate : undefined;
  };
  const resolveTimers = (): TimeoutScheduler | undefined =>
    options.timers === undefined ? browserTimers() : (toValue(options.timers) ?? undefined);

  const postTask = <Result>(
    callback: () => Result | PromiseLike<Result>,
    callOptions: PostTaskCallOptions = {},
  ): Promise<Result> => {
    const { delay, signal } = callOptions;
    validatePriority(callOptions.priority);
    if (delay !== undefined && (!Number.isFinite(delay) || delay < 0)) {
      throw new RangeError(
        `[VIZE_COMPOSE_POST_TASK_INVALID_DELAY] delay must be a finite, non-negative number, received ${String(delay)}`,
      );
    }
    const scheduler = resolveScheduler();
    const timers = scheduler ? undefined : resolveTimers();
    if (!scheduler && !timers) {
      return Promise.reject(
        new Error(
          "[VIZE_COMPOSE_POST_TASK_UNAVAILABLE] neither scheduler.postTask nor timers are available",
        ),
      );
    }

    const TaskController = resolveTaskController();
    const taskPriority = callOptions.priority ?? priority.value;
    const controller = TaskController
      ? new TaskController({ priority: taskPriority })
      : new AbortController();
    const task: PendingTask = { controller, fixed: callOptions.priority !== undefined };
    const forwardAbort = (): void => controller.abort(signal?.reason);
    if (signal?.aborted) forwardAbort();
    else signal?.addEventListener("abort", forwardAbort, { once: true });
    tasks.add(task);
    pending.value = tasks.size;

    let result: Promise<Result>;
    if (scheduler) {
      const explicitPriority = task.fixed || !TaskController ? { priority: taskPriority } : {};
      try {
        result = scheduler.postTask(callback, {
          signal: controller.signal,
          ...explicitPriority,
          ...(delay === undefined ? {} : { delay }),
        });
      } catch (cause) {
        signal?.removeEventListener("abort", forwardAbort);
        tasks.delete(task);
        pending.value = tasks.size;
        return Promise.reject(cause);
      }
    } else {
      result = new Promise<Result>((resolve, reject) => {
        const taskSignal = controller.signal;
        if (taskSignal.aborted) {
          reject(taskSignal.reason);
          return;
        }
        const onAbort = (): void => {
          timers?.clearTimeout(handle);
          reject(taskSignal.reason);
        };
        const handle = timers?.setTimeout(() => {
          taskSignal.removeEventListener("abort", onAbort);
          try {
            resolve(callback());
          } catch (cause) {
            reject(cause);
          }
        }, delay ?? 0);
        taskSignal.addEventListener("abort", onAbort, { once: true });
      });
    }

    const settle = (): void => {
      signal?.removeEventListener("abort", forwardAbort);
      tasks.delete(task);
      pending.value = tasks.size;
    };
    return result.then(
      (value) => {
        settle();
        return value;
      },
      (cause: unknown) => {
        settle();
        throw cause;
      },
    );
  };

  const setPriority = (next: SchedulerTaskPriority): void => {
    validatePriority(next);
    priority.value = next;
    for (const task of tasks) {
      if (task.fixed || !isTaskController(task.controller)) continue;
      try {
        task.controller.setPriority(next);
      } catch {
        // NotAllowedError: a priority change is already being dispatched.
      }
    }
  };

  const abort = (reason?: unknown): void => {
    for (const task of Array.from(tasks)) task.controller.abort(reason);
  };

  const yieldTask = (): Promise<void> => {
    const scheduler = resolveScheduler();
    if (scheduler?.yield) return scheduler.yield();
    const timers = resolveTimers();
    if (!timers) return Promise.resolve();
    return new Promise((resolve) => {
      timers.setTimeout(resolve, 0);
    });
  };

  tryOnScopeDispose(() => abort());

  return {
    supported: computed(() => resolveScheduler() !== undefined),
    priority: readonly(priority),
    pending: readonly(pending),
    postTask,
    setPriority,
    abort,
    yield: yieldTask,
  };
}
