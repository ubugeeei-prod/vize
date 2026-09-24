/** Compile-only assertions for the `use-queue` type contracts. */

import { useQueue } from "./use-queue.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const queue = useQueue([1, 2]);
type _ItemsInferred = Expect<Equal<typeof queue.items.value, readonly number[]>>;
type _DequeueMayBeEmpty = Expect<Equal<ReturnType<typeof queue.dequeue>, number | undefined>>;
type _DrainIsMutableCopy = Expect<Equal<ReturnType<typeof queue.drain>, number[]>>;

const explicit = useQueue<{ readonly id: string }>();
explicit.enqueue({ id: "a" });

// @ts-expect-error enqueued items keep the inferred type.
queue.enqueue("3");

// @ts-expect-error the overflow policy is a closed union.
useQueue([], { overflow: "drop-newest" });

// @ts-expect-error the snapshot is read-only.
queue.items.value.push(3);
