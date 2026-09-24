/** Compile-only assertions for the public DropdownMenu contract. */

import type {
  DropdownMenuRootExpose,
  DropdownMenuSelectEvent,
  DropdownMenuTriggerExpose,
} from "./dropdown-menu.ts";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuRoot,
  DropdownMenuTrigger,
} from "./dropdown-menu.ts";
import type { MenuRootExpose, MenuSelectEvent } from "../menu/menu.ts";
import { MenuContent, MenuItem } from "../menu/menu.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const trigger: DropdownMenuTriggerExpose;

type _RootExpose = Expect<Equal<DropdownMenuRootExpose, MenuRootExpose>>;
type _Select = Expect<Equal<DropdownMenuSelectEvent, MenuSelectEvent>>;
type _TriggerElement = Expect<Equal<typeof trigger.element, HTMLButtonElement | null>>;
type _SharedContent = Expect<Equal<typeof DropdownMenuContent, typeof MenuContent>>;
type _SharedItem = Expect<Equal<typeof DropdownMenuItem, typeof MenuItem>>;

const triggerProps: InstanceType<typeof DropdownMenuTrigger>["$props"] = {
  ariaLabel: "File",
  disabled: false,
  openOn: "click",
};
const rootProps: InstanceType<typeof DropdownMenuRoot>["$props"] = { modal: false, loop: true };

// @ts-expect-error openOn accepts only click and pointerdown.
const badTrigger: InstanceType<typeof DropdownMenuTrigger>["$props"] = { openOn: "hover" };

void DropdownMenu;
void [badTrigger, rootProps, triggerProps];
