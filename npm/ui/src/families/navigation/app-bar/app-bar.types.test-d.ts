/** Compile-only assertions for the public AppBar contract. */

import type { AppBarExpose, AppBarSlotState, AppBarState, AppBarTarget } from "./app-bar.ts";
import { AppBar } from "./app-bar.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: AppBarExpose;
declare const slot: AppBarSlotState;

type _State = Expect<Equal<AppBarState, "visible" | "hidden">>;
type _Target = Expect<Equal<AppBarTarget, HTMLElement | string | null>>;
type _SlotElevated = Expect<Equal<typeof slot.elevated, boolean>>;
type _ExposeElement = Expect<Equal<typeof exposed.element, HTMLElement | null>>;
type _ExposeRefresh = Expect<Equal<typeof exposed.refresh, () => number>>;

const props: InstanceType<typeof AppBar>["$props"] = {
  as: "div",
  elevateThreshold: 4,
  hideOnScroll: true,
  hideThreshold: 80,
  target: "#main",
  onVisibilityChange: (hidden: boolean) => {
    void hidden;
  },
};

// @ts-expect-error the host is a header or a div.
const badHost: InstanceType<typeof AppBar>["$props"] = { as: "nav" };

void [badHost, props];
