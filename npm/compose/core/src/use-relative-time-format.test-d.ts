/** Compile-only assertions for the `use-relative-time-format` type contracts. */

import { selectRelativeTimeUnit, useRelativeTimeFormat } from "./use-relative-time-format.ts";
import type { RelativeTimeUnitSelection } from "./use-relative-time-format.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const selection = selectRelativeTimeUnit(1000);
type _UnitIsClosed = Expect<Equal<typeof selection.unit, RelativeTimeUnitSelection>>;

const relative = useRelativeTimeFormat(1, "day", { locale: "en", numeric: "auto" });
relative.formatFrom(new Date(), { now: 0 }) satisfies string;

// @ts-expect-error units are the platform union.
useRelativeTimeFormat(1, "fortnight");
