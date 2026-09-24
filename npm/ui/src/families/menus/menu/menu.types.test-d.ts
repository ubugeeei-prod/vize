/** Compile-only assertions for the public Menu contract. */

import type {
  MenuCheckedState,
  MenuContentExpose,
  MenuDirection,
  MenuEntryFocus,
  MenuItemCheckedState,
  MenuItemExpose,
  MenuKind,
  MenuRadioGroupExpose,
  MenuRadioGroupSlotState,
  MenuRootExpose,
  MenuSelectEvent,
  MenuState,
} from "./menu.ts";
import {
  Menu,
  MenuCheckboxItem,
  MenuContent,
  MenuItem,
  MenuRadioGroup,
  MenuRadioItem,
  MenuRoot,
  MenuSub,
  MenuSubContent,
  MenuTrigger,
} from "./menu.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: MenuRootExpose;
declare const content: MenuContentExpose;
declare const menuItem: MenuItemExpose;
declare const select: MenuSelectEvent;
declare const radioGroup: MenuRadioGroupExpose<{ readonly id: number }>;
declare const radioSlot: MenuRadioGroupSlotState<"a" | "b">;

type _State = Expect<Equal<MenuState, "closed" | "open">>;
type _Direction = Expect<Equal<MenuDirection, "ltr" | "rtl">>;
type _Kind = Expect<Equal<MenuKind, "context-menu" | "dropdown-menu" | "menu" | "menubar">>;
type _Entry = Expect<Equal<MenuEntryFocus, "content" | "first" | "last" | "none">>;
type _Checked = Expect<Equal<MenuCheckedState, boolean | "indeterminate">>;
type _CheckedToken = Expect<Equal<MenuItemCheckedState, "checked" | "indeterminate" | "unchecked">>;
type _RootOpen = Expect<Equal<typeof root.open, boolean>>;
type _ContentElement = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _ItemId = Expect<Equal<typeof menuItem.id, string>>;
type _SelectPrevented = Expect<Equal<typeof select.defaultPrevented, boolean>>;
type _SelectType = Expect<Equal<typeof select.type, "select">>;
type _RadioValue = Expect<Equal<typeof radioGroup.value, { readonly id: number } | null>>;
type _RadioSlotValue = Expect<Equal<typeof radioSlot.value, "a" | "b" | null>>;

const rootProps: InstanceType<typeof MenuRoot>["$props"] = {
  defaultOpen: false,
  dir: "rtl",
  disabled: false,
  id: "menu",
  loop: true,
  modal: false,
  open: true,
  "onUpdate:open": (value: boolean) => value,
};
const contentProps: InstanceType<typeof MenuContent>["$props"] = {
  closeOnEscape: false,
  forceMount: true,
  offset: 2,
  placement: "right-start",
  portalDisabled: true,
  strategy: "absolute",
};
const itemProps: InstanceType<typeof MenuItem>["$props"] = {
  closeOnSelect: false,
  disabled: true,
  textValue: "Save",
  onSelect: (event: MenuSelectEvent) => event.preventDefault(),
};
const checkboxProps: InstanceType<typeof MenuCheckboxItem>["$props"] = {
  modelValue: "indeterminate",
};
const subContentProps: InstanceType<typeof MenuSubContent>["$props"] = { offset: 0 };
const subProps: InstanceType<typeof MenuSub>["$props"] = { defaultOpen: true };

// Generic radio groups and items infer their value type from props.
type RadioGroupProps<Value> = Parameters<typeof MenuRadioGroup<Value>>[0];
type RadioItemProps<Value> = Parameters<typeof MenuRadioItem<Value>>[0];
const typedGroup: RadioGroupProps<"small" | "large"> = {
  modelValue: "small",
  equals: (left, right) => left === right,
  "onUpdate:modelValue": (value) => value,
};
const typedItem: RadioItemProps<"small" | "large"> = { value: "large" };

root.openMenu("last");
root.toggle();
content.focusFirst();
menuItem.select();
radioGroup.setValue({ id: 1 });

// @ts-expect-error Menu state is a closed token contract.
const badState: MenuState = "opening";
// @ts-expect-error dir accepts only ltr and rtl.
const badRoot: InstanceType<typeof MenuRoot>["$props"] = { dir: "auto" };
// @ts-expect-error placement is limited to positioner placements.
const badContent: InstanceType<typeof MenuContent>["$props"] = { placement: "middle" };
// @ts-expect-error checkbox values are boolean or "indeterminate".
const badCheckbox: InstanceType<typeof MenuCheckboxItem>["$props"] = { modelValue: "mixed" };
// @ts-expect-error radio item values must match the inferred group type.
const badItem: RadioItemProps<"small" | "large"> = { value: "medium" };
// @ts-expect-error radio group equality receives typed values.
const badEquals: RadioGroupProps<number> = { equals: (left: string) => left.length > 0 };
// @ts-expect-error trigger props do not include menu content options.
const badTrigger: InstanceType<typeof MenuTrigger>["$props"] = { placement: "top" };

void Menu;
void [badCheckbox, badContent, badEquals, badItem, badRoot, badState, badTrigger];
void [checkboxProps, contentProps, itemProps, rootProps, subContentProps, subProps];
void [typedGroup, typedItem];
