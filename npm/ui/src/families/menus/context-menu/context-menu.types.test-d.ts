/** Compile-only assertions for the public ContextMenu contract. */

import type {
  ContextMenuPoint,
  ContextMenuRootExpose,
  ContextMenuTriggerExpose,
  ContextMenuTriggerSlotState,
} from "./context-menu.ts";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuRoot,
  ContextMenuTrigger,
} from "./context-menu.ts";
import { MenuContent } from "../menu/menu.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: ContextMenuRootExpose;
declare const trigger: ContextMenuTriggerExpose;
declare const slot: ContextMenuTriggerSlotState;

type _Point = Expect<Equal<ContextMenuPoint, { readonly x: number; readonly y: number }>>;
type _TriggerElement = Expect<Equal<typeof trigger.element, HTMLSpanElement | null>>;
type _SlotDisabled = Expect<Equal<typeof slot.disabled, boolean>>;
type _SharedContent = Expect<Equal<typeof ContextMenuContent, typeof MenuContent>>;

root.openAt({ x: 1, y: 2 });
const triggerProps: InstanceType<typeof ContextMenuTrigger>["$props"] = {
  disabled: false,
  longPressDelay: 500,
};
const rootProps: InstanceType<typeof ContextMenuRoot>["$props"] = { modal: false };

// @ts-expect-error openAt requires both coordinates.
root.openAt({ x: 1 });
// @ts-expect-error longPressDelay is numeric.
const badTrigger: InstanceType<typeof ContextMenuTrigger>["$props"] = { longPressDelay: "1s" };

void ContextMenu;
void [badTrigger, rootProps, triggerProps];
