/** Compile-only assertions for the `use-worker-fn` type contracts. */

import { useWorkerFn } from "./use-worker-fn.ts";
import type { WorkerFnStatus } from "./use-worker-fn.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const sort = useWorkerFn((values: readonly number[], descending: boolean) =>
  descending ? [...values].sort((a, b) => b - a) : [...values].sort((a, b) => a - b),
);
type _ArgumentsAreInferred = Expect<
  Equal<Parameters<typeof sort.run>, [values: readonly number[], descending: boolean]>
>;
type _ResultIsInferred = Expect<Equal<Awaited<ReturnType<typeof sort.run>>, number[]>>;

const asyncTask = useWorkerFn(async (id: string) => ({ id }));
type _AsyncResultIsAwaited = Expect<
  Equal<Awaited<ReturnType<typeof asyncTask.run>>, { id: string }>
>;
type _StatusIsClosed = Expect<Equal<typeof sort.status.value, WorkerFnStatus>>;

// @ts-expect-error run keeps the function's parameter types.
void sort.run(["1"], false);

// @ts-expect-error the timeout is numeric.
useWorkerFn(() => 1, { timeoutMs: "1s" });
