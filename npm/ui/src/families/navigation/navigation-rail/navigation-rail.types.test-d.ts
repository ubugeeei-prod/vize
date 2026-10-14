/** Compile-only assertions for the public NavigationRail contract. */

import type { NavigationRailExpose, NavigationRailSlotState } from "./navigation-rail.ts";
import { NavigationRail, NavigationRailItem, NavigationRailToggle } from "./navigation-rail.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type RailProps<Value extends string> = Parameters<typeof NavigationRail<Value>>[0];

/** Infers destination values exactly as a template usage would. */
declare function inferDestination<Value extends string>(props: RailProps<Value>): Value;

const inferred = inferDestination({ destinations: ["home", "search"] });
declare const exposed: NavigationRailExpose;
declare const slot: NavigationRailSlotState<"home" | "search">;

type _InfersUnion = Expect<Equal<typeof inferred, "home" | "search">>;
type _SlotActive = Expect<Equal<typeof slot.active, "home" | "search" | undefined>>;
type _SlotExpanded = Expect<Equal<typeof slot.expanded, boolean>>;
type _ExposeToggle = Expect<Equal<typeof exposed.toggle, () => void>>;
type _ExposeSetExpanded = Expect<Equal<typeof exposed.setExpanded, (value: boolean) => void>>;

const props: RailProps<"home" | "search"> = {
  destinations: ["home", "search"],
  expanded: true,
  modelValue: "home",
  onSelect: (value: "home" | "search", event: MouseEvent) => {
    void value;
    void event;
  },
  "onUpdate:expanded": (value: boolean) => {
    void value;
  },
};
const itemProps: InstanceType<typeof NavigationRailItem>["$props"] = { value: "home", badge: 2 };
const toggleProps: InstanceType<typeof NavigationRailToggle>["$props"] = {
  ariaLabel: "Toggle navigation",
  disabled: false,
};

const wrongModel: RailProps<"home" | "search"> = {
  destinations: ["home", "search"],
  // @ts-expect-error the model must be a destination.
  modelValue: "x",
};

// @ts-expect-error items need a value.
const missingValue: InstanceType<typeof NavigationRailItem>["$props"] = {};

void [itemProps, missingValue, props, toggleProps, wrongModel];
