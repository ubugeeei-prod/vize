/** Compile-only assertions for the public Collapsible contract. */

import type {
  CollapsibleContentExpose,
  CollapsibleContentRole,
  CollapsibleRootExpose,
  CollapsibleSlotState,
  CollapsibleState,
  CollapsibleTriggerExpose,
} from "./collapsible.ts";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleRoot,
  CollapsibleTrigger,
} from "./collapsible.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: CollapsibleRootExpose;
declare const trigger: CollapsibleTriggerExpose;
declare const content: CollapsibleContentExpose;
declare const slot: CollapsibleSlotState;

type _StateIsLiteral = Expect<Equal<CollapsibleState, "closed" | "open">>;
type _RoleIsLiteral = Expect<Equal<CollapsibleContentRole, "group" | "region">>;
type _RootOpenIsBoolean = Expect<Equal<typeof root.open, boolean>>;
type _RootDisabledIsBoolean = Expect<Equal<typeof root.disabled, boolean>>;
type _TriggerElementIsButton = Expect<Equal<typeof trigger.element, HTMLButtonElement | null>>;
type _ContentElementIsDiv = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly open: boolean;
      readonly disabled: boolean;
      readonly state: CollapsibleState;
    }
  >
>;

const rootProps: InstanceType<typeof CollapsibleRoot>["$props"] = {
  defaultOpen: true,
  disabled: false,
  id: "navigation",
  open: false,
  "onUpdate:open": (value: boolean) => value,
};
const triggerProps: InstanceType<typeof CollapsibleTrigger>["$props"] = {
  ariaLabel: "Navigation",
  ariaLabelledby: "navigation-label",
  disabled: false,
  type: "button",
};
const contentProps: InstanceType<typeof CollapsibleContent>["$props"] = {
  ariaDescribedby: "navigation-help",
  ariaLabelledby: null,
  role: "region",
};
const slotState: CollapsibleSlotState = { disabled: false, open: true, state: "open" };

root.expand();
root.collapse();
root.toggle();
root.setOpen(true);
trigger.focus();
content.focusContent();

// @ts-expect-error Collapsible state has a closed token contract.
const invalidState: CollapsibleState = "opening";

// @ts-expect-error Collapsible content role is intentionally limited.
const invalidRole: CollapsibleContentRole = "navigation";

// @ts-expect-error open is boolean-only.
const badRootProps: InstanceType<typeof CollapsibleRoot>["$props"] = { open: "true" };

// @ts-expect-error trigger type remains a native button type.
const badTriggerProps: InstanceType<typeof CollapsibleTrigger>["$props"] = { type: "menu" };

// @ts-expect-error content role must be a CollapsibleContentRole or null.
const badContentProps: InstanceType<typeof CollapsibleContent>["$props"] = { role: "status" };

void Collapsible;
void badContentProps;
void badRootProps;
void badTriggerProps;
void contentProps;
void invalidRole;
void invalidState;
void rootProps;
void slotState;
void triggerProps;
