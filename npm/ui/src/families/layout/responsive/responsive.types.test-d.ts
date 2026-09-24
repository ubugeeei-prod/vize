/** Compile-only assertions for breakpoint-driven rendering. */

import type { ComputedRef, Ref } from "vue";

import { defaultBreakpoints, resolveActiveBreakpoint, useBreakpoint } from "./responsive.ts";
import type { BreakpointMap, BreakpointState, ResponsiveSwitchSlotState } from "./responsive.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const defaults = useBreakpoint({ ssrWidth: 800 });
type _DefaultNamesAreTailwind = Expect<
  Equal<typeof defaults.active, ComputedRef<"sm" | "md" | "lg" | "xl" | "2xl" | null>>
>;
type _WidthIsNullable = Expect<Equal<typeof defaults.width, Readonly<Ref<number | null>>>>;
defaults.isAbove("md");
// @ts-expect-error unknown default breakpoint names are rejected.
defaults.isAbove("tablet");

const custom = useBreakpoint({ phone: 0, tablet: 600, desktop: 1024 });
type _CustomNamesAreInferred = Expect<
  Equal<typeof custom, BreakpointState<"phone" | "tablet" | "desktop">>
>;
custom.isBetween("tablet", "desktop");
// @ts-expect-error custom maps only accept their own names.
custom.isBelow("md");
// @ts-expect-error breakpoint widths are numbers.
useBreakpoint({ tablet: "600px" });

const active = resolveActiveBreakpoint(defaultBreakpoints, 1000);
type _ResolveKeepsNames = Expect<Equal<typeof active, "sm" | "md" | "lg" | "xl" | "2xl" | null>>;
const _map: BreakpointMap<"a"> = { a: 1 };
const _slot: ResponsiveSwitchSlotState = { active: null, width: null };
