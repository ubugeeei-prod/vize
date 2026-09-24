/** Compile-only assertions for the `use-scheduler-post-task` type contracts. */

import { useSchedulerPostTask } from "./use-scheduler-post-task.ts";
import type { SchedulerTaskPriority } from "./use-scheduler-post-task.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const tasks = useSchedulerPostTask();

const sync = tasks.postTask(() => 42);
type _Sync = Expect<Equal<typeof sync, Promise<number>>>;

const async = tasks.postTask(async () => ({ ok: true as const }));
type _Async = Expect<Equal<Awaited<typeof async>, { ok: true }>>;

type _Priority = Expect<Equal<typeof tasks.priority.value, SchedulerTaskPriority>>;
type _Yield = Expect<Equal<ReturnType<typeof tasks.yield>, Promise<void>>>;

// @ts-expect-error unknown priorities are rejected.
void tasks.postTask(() => 1, { priority: "urgent" });

// @ts-expect-error the priority is changed through setPriority.
tasks.priority.value = "background";
