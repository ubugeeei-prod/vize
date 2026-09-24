/** Compile-only assertions for the `use-list-format` type contracts. */

import type { ComputedRef } from "vue";

import { useListFormat } from "./use-list-format.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const list = useListFormat(["a"], { locale: "en", type: "disjunction" });
type _FormattedIsString = Expect<Equal<typeof list.formatted, ComputedRef<string>>>;

// @ts-expect-error items are strings.
useListFormat([1, 2]);

// @ts-expect-error list types are the platform union.
useListFormat(["a"], { type: "either" });
