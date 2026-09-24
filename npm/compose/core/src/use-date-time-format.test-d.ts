/** Compile-only assertions for the `use-date-time-format` type contracts. */

import type { ComputedRef } from "vue";

import { useDateTimeFormat } from "./use-date-time-format.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const date = useDateTimeFormat(new Date(), { locale: "en-US", dateStyle: "short" });
type _FormattedIsString = Expect<Equal<typeof date.formatted, ComputedRef<string>>>;
date.formatRange(0, new Date()) satisfies string;

// @ts-expect-error date strings must be parsed by the caller.
useDateTimeFormat("2026-01-01");

// @ts-expect-error date styles are the platform union.
useDateTimeFormat(0, { dateStyle: "tiny" });
