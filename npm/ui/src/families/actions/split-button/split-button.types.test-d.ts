/** Compile-only assertions for the public SplitButton contract. */

import type {
  SplitButtonActionExpose,
  SplitButtonAnchor,
  SplitButtonRootExpose,
  SplitButtonSelectEvent,
  SplitButtonSlotState,
  SplitButtonTriggerExpose,
} from "./split-button.ts";
import {
  SplitButton,
  SplitButtonAction,
  SplitButtonContent,
  SplitButtonItem,
  SplitButtonRoot,
  SplitButtonTrigger,
} from "./split-button.ts";
import type {
  MenuRootExpose,
  MenuSelectEvent,
  MenuSlotState,
  MenuTriggerExpose,
} from "../../menus/menu/menu.ts";
import { MenuContent, MenuItem } from "../../menus/menu/menu.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: SplitButtonRootExpose;
declare const action: SplitButtonActionExpose;
declare const slot: SplitButtonSlotState;

type _Anchor = Expect<Equal<SplitButtonAnchor, "group" | "trigger">>;
type _RootExtendsMenu = Expect<
  Equal<Omit<SplitButtonRootExpose, "disabled" | "element">, MenuRootExpose>
>;
type _RootElement = Expect<Equal<typeof root.element, HTMLDivElement | null>>;
type _RootDisabled = Expect<Equal<typeof root.disabled, boolean>>;
type _ActionElement = Expect<Equal<typeof action.element, HTMLButtonElement | null>>;
type _TriggerExpose = Expect<Equal<SplitButtonTriggerExpose, MenuTriggerExpose>>;
type _SlotExtendsMenu = Expect<Equal<Omit<SplitButtonSlotState, "disabled">, MenuSlotState>>;
type _SlotDisabled = Expect<Equal<typeof slot.disabled, boolean>>;
type _Select = Expect<Equal<SplitButtonSelectEvent, MenuSelectEvent>>;
type _SharedContent = Expect<Equal<typeof SplitButtonContent, typeof MenuContent>>;
type _SharedItem = Expect<Equal<typeof SplitButtonItem, typeof MenuItem>>;

const rootProps: InstanceType<typeof SplitButtonRoot>["$props"] = {
  anchor: "trigger",
  ariaLabel: "Save options",
  loop: true,
  modal: false,
};
const actionProps: InstanceType<typeof SplitButtonAction>["$props"] = {
  ariaLabel: "Save",
  disabled: false,
  type: "submit",
};
const triggerProps: InstanceType<typeof SplitButtonTrigger>["$props"] = {
  ariaLabel: "More save options",
  disabled: false,
  openOn: "click",
};

// @ts-expect-error anchor accepts only group and trigger.
const badRoot: InstanceType<typeof SplitButtonRoot>["$props"] = { anchor: "action" };
// @ts-expect-error type accepts only native button types.
const badAction: InstanceType<typeof SplitButtonAction>["$props"] = { type: "menu" };
// @ts-expect-error openOn accepts only click and pointerdown.
const badTrigger: InstanceType<typeof SplitButtonTrigger>["$props"] = { openOn: "hover" };

void SplitButton;
void [actionProps, badAction, badRoot, badTrigger, rootProps, triggerProps];
