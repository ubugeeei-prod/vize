/** Compile-only assertions for the `use-stack` type contracts. */

import { useStack } from "./use-stack.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const stack = useStack(["a"]);
type _ItemsInferred = Expect<Equal<typeof stack.items.value, readonly string[]>>;
type _PopMayBeEmpty = Expect<Equal<ReturnType<typeof stack.pop>, string | undefined>>;
type _PushReturnsAccepted = Expect<Equal<ReturnType<typeof stack.push>, number>>;

// @ts-expect-error pushed items keep the inferred type.
stack.push(1);

// @ts-expect-error the overflow policy is a closed union.
useStack([], { overflow: "grow" });

// @ts-expect-error the size is read-only.
stack.size.value = 0;
