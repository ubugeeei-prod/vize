/** Compile-only assertions for the `use-sorted-locale` type contracts. */

import type { ComputedRef } from "vue";

import { useSortedLocale } from "./use-sorted-locale.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const strings = useSortedLocale(["b", "a"]);
type _StringsStayStrings = Expect<Equal<typeof strings.sorted, ComputedRef<readonly string[]>>>;

const users = useSortedLocale([{ name: "Ada", id: 1 }], { key: (user) => user.name });
type _ItemsKeepTheirType = Expect<
  Equal<typeof users.sorted.value, readonly { name: string; id: number }[]>
>;

// @ts-expect-error non-string items need a key.
useSortedLocale([{ name: "Ada" }]);

// @ts-expect-error keys must return strings.
useSortedLocale([{ id: 1 }], { key: (item) => item.id });
