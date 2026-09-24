/** Compile-only assertions for the `use-set` type contracts. */

import { useSet } from "./use-set.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const tags = useSet(["a", "b"]);
type _ItemInferred = Expect<Equal<typeof tags.values.value, readonly string[]>>;
type _ViewIsReadonly = Expect<Equal<typeof tags.set, ReadonlySet<string>>>;
type _AlgebraReturnsPlainSets = Expect<Equal<ReturnType<typeof tags.union>, Set<string>>>;

const ids = useSet<number>();
ids.toggle(1, true) satisfies boolean;

// @ts-expect-error members keep the inferred type.
tags.add(1);

// @ts-expect-error set algebra accepts the same member type.
tags.intersection([1]);

// @ts-expect-error the reactive view is read-only.
tags.set.add("c");
