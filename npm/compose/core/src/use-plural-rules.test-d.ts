/** Compile-only assertions for the `use-plural-rules` type contracts. */

import { usePluralRules } from "./use-plural-rules.ts";
import type { PluralCategory } from "./use-plural-rules.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const plural = usePluralRules(1, {
  "=0": "none",
  one: "# item",
  other: (count) => `${count} items`,
});
type _CategoryIsCldr = Expect<Equal<typeof plural.category.value, PluralCategory>>;
type _CategoryUnion = Expect<
  Equal<PluralCategory, "zero" | "one" | "two" | "few" | "many" | "other">
>;

// @ts-expect-error every message table needs `other`.
usePluralRules(1, { one: "# item" });

// @ts-expect-error unknown categories are rejected.
usePluralRules(1, { several: "#", other: "#" });

// @ts-expect-error rule types are cardinal or ordinal.
usePluralRules(1, { other: "#" }, { type: "nominal" });
