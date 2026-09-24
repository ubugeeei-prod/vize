/** Compile-only assertions for the public Menubar contract. */

import type {
  MenubarMenuExpose,
  MenubarRootExpose,
  MenubarSlotState,
  MenubarTriggerSlotState,
} from "./menubar.ts";
import { Menubar, MenubarContent, MenubarMenu, MenubarRoot, MenubarTrigger } from "./menubar.ts";
import { MenuContent } from "../menu/menu.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: MenubarRootExpose;
declare const menu: MenubarMenuExpose;
declare const slot: MenubarSlotState;
declare const triggerSlot: MenubarTriggerSlotState;

type _Value = Expect<Equal<typeof slot.value, string | null>>;
type _MenuValue = Expect<Equal<typeof menu.value, string>>;
type _TriggerHighlight = Expect<Equal<typeof triggerSlot.highlighted, boolean>>;
type _RootElement = Expect<Equal<typeof root.element, HTMLDivElement | null>>;
type _SharedContent = Expect<Equal<typeof MenubarContent, typeof MenuContent>>;

root.setValue("file");
root.setValue(null);
menu.setOpen(true);
const rootProps: InstanceType<typeof MenubarRoot>["$props"] = {
  defaultValue: null,
  dir: "rtl",
  loop: false,
  modelValue: "file",
};
const menuProps: InstanceType<typeof MenubarMenu>["$props"] = { value: "file", modal: false };
const triggerProps: InstanceType<typeof MenubarTrigger>["$props"] = { textValue: "File" };

// @ts-expect-error menubar values are strings.
const badRoot: InstanceType<typeof MenubarRoot>["$props"] = { modelValue: 1 };
// @ts-expect-error menu values are strings.
const badMenu: InstanceType<typeof MenubarMenu>["$props"] = { value: 2 };

void Menubar;
void [badMenu, badRoot, menuProps, rootProps, triggerProps];
