/** Compile-only assertions for the `use-display-names` type contracts. */

import { useDisplayNames } from "./use-display-names.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const names = useDisplayNames("JP", { type: "region" });
type _NameMayBeMissing = Expect<Equal<typeof names.name.value, string | undefined>>;

// @ts-expect-error the code kind is required.
useDisplayNames("JP", {});

// @ts-expect-error kinds are a closed union.
useDisplayNames("JP", { type: "country" });
