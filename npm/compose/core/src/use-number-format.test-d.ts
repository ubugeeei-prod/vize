/** Compile-only assertions for the `use-number-format` type contracts. */

import type { ComputedRef } from "vue";

import { useNumberFormat } from "./use-number-format.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const number = useNumberFormat(1, { locale: "en-US", style: "percent" });
type _FormattedIsString = Expect<Equal<typeof number.formatted, ComputedRef<string>>>;
number.format(1n) satisfies string;
number.formatRange(1, 2) satisfies string;

// @ts-expect-error values are numeric.
useNumberFormat("1");

// @ts-expect-error styles are the platform union.
useNumberFormat(1, { style: "money" });
