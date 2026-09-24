import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { useSchedulerPostTask } from "./use-scheduler-post-task.ts";
import type {
  SchedulerHost,
  SchedulerPostTaskInit,
  SchedulerTaskPriority,
  TaskControllerLike,
} from "./use-scheduler-post-task.ts";

class FakeTaskController extends AbortController implements TaskControllerLike {
  static readonly instances: FakeTaskController[] = [];
  priority: SchedulerTaskPriority;

  constructor(init?: { readonly priority?: SchedulerTaskPriority }) {
    super();
    this.priority = init?.priority ?? "user-visible";
    FakeTaskController.instances.push(this);
  }

  setPriority(priority: SchedulerTaskPriority): void {
    this.priority = priority;
  }
}

interface QueuedTask {
  readonly run: () => void;
  readonly init: SchedulerPostTaskInit | undefined;
}

function createScheduler(withYield = true): {
  scheduler: SchedulerHost;
  queue: QueuedTask[];
  flush: () => void;
  yields: () => number;
} {
  const queue: QueuedTask[] = [];
  let yieldCount = 0;
  const scheduler: SchedulerHost = {
    postTask: (callback, init) =>
      new Promise((resolve, reject) => {
        const signal = init?.signal;
        if (signal?.aborted) {
          reject(signal.reason);
          return;
        }
        const task: QueuedTask = {
          init,
          run: () => {
            try {
              resolve(callback());
            } catch (cause) {
              reject(cause);
            }
          },
        };
        signal?.addEventListener("abort", () => {
          queue.splice(queue.indexOf(task), 1);
          reject(signal.reason);
        });
        queue.push(task);
      }),
  };
  const host: SchedulerHost = withYield
    ? {
        ...scheduler,
        yield: () => {
          yieldCount += 1;
          return Promise.resolve();
        },
      }
    : scheduler;
  return {
    scheduler: host,
    queue,
    flush: () => {
      for (const task of queue.splice(0)) task.run();
    },
    yields: () => yieldCount,
  };
}

function createTimers(): {
  timers: TimeoutScheduler;
  queue: Map<number, { callback: () => void; delay: number }>;
  flush: () => void;
} {
  const queue = new Map<number, { callback: () => void; delay: number }>();
  let next = 0;
  return {
    queue,
    timers: {
      setTimeout: (callback, delay) => {
        next += 1;
        queue.set(next, { callback, delay });
        return next;
      },
      clearTimeout: (handle) => {
        if (typeof handle === "number") queue.delete(handle);
      },
    },
    flush: () => {
      const tasks = [...queue.values()];
      queue.clear();
      for (const task of tasks) task.callback();
    },
  };
}

void test("posts tasks with a task controller and resolves their results", async () => {
  const { scheduler, queue, flush } = createScheduler();
  const tasks = useSchedulerPostTask({ scheduler, TaskController: FakeTaskController });
  const first = tasks.postTask(() => 21 * 2);
  const second = tasks.postTask(async () => "done", { priority: "background", delay: 5 });

  assert.equal(tasks.supported.value, true);
  assert.equal(tasks.pending.value, 2);
  assert.equal(queue[0]?.init?.priority, undefined, "the task signal carries the priority");
  assert.equal(queue[1]?.init?.priority, "background");
  assert.equal(queue[1]?.init?.delay, 5);
  flush();
  assert.equal(await first, 42);
  assert.equal(await second, "done");
  assert.equal(tasks.pending.value, 0);
});

void test("setPriority re-prioritizes pending tasks without a fixed priority", () => {
  const { scheduler } = createScheduler();
  FakeTaskController.instances.length = 0;
  const tasks = useSchedulerPostTask({
    scheduler,
    TaskController: FakeTaskController,
    priority: "background",
  });
  void tasks.postTask(() => undefined).catch(() => undefined);
  void tasks.postTask(() => undefined, { priority: "background" }).catch(() => undefined);
  tasks.setPriority("user-blocking");

  assert.equal(tasks.priority.value, "user-blocking");
  assert.deepEqual(
    FakeTaskController.instances.map((controller) => controller.priority),
    ["user-blocking", "background"],
  );
  tasks.abort();
});

void test("passes the priority explicitly without TaskController", () => {
  const { scheduler, queue } = createScheduler();
  const tasks = useSchedulerPostTask({ scheduler, TaskController: null, priority: "background" });
  void tasks.postTask(() => undefined).catch(() => undefined);
  assert.equal(queue[0]?.init?.priority, "background");
  tasks.abort();
});

void test("abort rejects pending tasks and per-call signals abort one task", async () => {
  const { scheduler, queue } = createScheduler();
  const tasks = useSchedulerPostTask({ scheduler, TaskController: FakeTaskController });
  const controller = new AbortController();
  const single = tasks.postTask(() => 1, { signal: controller.signal });
  const other = tasks.postTask(() => 2);
  controller.abort(new Error("one"));
  await assert.rejects(single, /one/);
  assert.equal(queue.length, 1);

  tasks.abort(new Error("all"));
  await assert.rejects(other, /all/);
  assert.equal(tasks.pending.value, 0);
});

void test("falls back to timers when the scheduler is missing", async () => {
  const { timers, queue, flush } = createTimers();
  const tasks = useSchedulerPostTask({ scheduler: null, timers });
  const result = tasks.postTask(() => "late", { delay: 30 });
  const failing = tasks.postTask(() => {
    throw new Error("boom");
  });
  const aborted = tasks.postTask(() => "never");

  assert.equal(tasks.supported.value, false);
  assert.equal([...queue.values()][0]?.delay, 30);
  tasks.abort(new Error("cancelled"));
  await assert.rejects(result, /cancelled/);
  await assert.rejects(failing, /cancelled/);
  await assert.rejects(aborted, /cancelled/);
  assert.equal(queue.size, 0);

  const ok = tasks.postTask(() => 7);
  const broken = tasks.postTask(() => {
    throw new Error("boom");
  });
  flush();
  assert.equal(await ok, 7);
  await assert.rejects(broken, /boom/);
});

void test("yield uses scheduler.yield, then timers", async () => {
  const native = createScheduler();
  await useSchedulerPostTask({ scheduler: native.scheduler }).yield();
  assert.equal(native.yields(), 1);

  const { timers, flush, queue } = createTimers();
  const waiting = useSchedulerPostTask({ scheduler: null, timers }).yield();
  assert.equal([...queue.values()][0]?.delay, 0);
  flush();
  await waiting;

  await useSchedulerPostTask({ scheduler: null, timers: null }).yield();
});

void test("rejects invalid arguments and unavailable hosts", async () => {
  assert.throws(
    () => useSchedulerPostTask({ priority: "urgent" as SchedulerTaskPriority }),
    /VIZE_COMPOSE_POST_TASK_INVALID_PRIORITY/,
  );
  const tasks = useSchedulerPostTask({ scheduler: null, timers: null });
  assert.throws(
    () => tasks.postTask(() => 1, { delay: -5 }),
    /VIZE_COMPOSE_POST_TASK_INVALID_DELAY/,
  );
  await assert.rejects(
    tasks.postTask(() => 1),
    /VIZE_COMPOSE_POST_TASK_UNAVAILABLE/,
  );
});

void test("synchronous scheduler failures settle and remove pending tasks", async () => {
  const failure = new Error("scheduler rejected before returning a promise");
  const tasks = useSchedulerPostTask({
    scheduler: {
      postTask: () => {
        throw failure;
      },
    },
  });

  await assert.rejects(
    tasks.postTask(() => 1),
    (cause) => cause === failure,
  );
  assert.equal(tasks.pending.value, 0);
});

void test("aborts pending tasks with the scope", async () => {
  const { scheduler, queue } = createScheduler();
  const scope = effectScope();
  const tasks = scope.run(() => useSchedulerPostTask({ scheduler }));
  assert.ok(tasks);
  const task = tasks.postTask(() => 1);
  scope.stop();
  await assert.rejects(task, { name: "AbortError" });
  assert.equal(queue.length, 0);
});

void test("server rendering schedules nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const tasks = useSchedulerPostTask({ priority: "background" });
    return { supported: tasks.supported, priority: tasks.priority, pending: tasks.pending };
  });
  assert.equal(state, '{"supported":false,"priority":"background","pending":0}');
});
