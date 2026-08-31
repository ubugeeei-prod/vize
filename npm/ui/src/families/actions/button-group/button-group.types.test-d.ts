/** Compile-only assertions for the public ButtonGroup contract. */

import type { Component } from "vue";

import {
  ButtonGroup,
  ButtonGroupItem,
  type ButtonGroupExpose,
  type ButtonGroupItemExpose,
  type ButtonGroupItemSlotState,
  type ButtonGroupItemState,
  type ButtonGroupOrientation,
  type ButtonGroupRole,
  type ButtonGroupSlotState,
  type ButtonGroupState,
} from "./button-group.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const group: ButtonGroupExpose;
declare const item: ButtonGroupItemExpose;
declare const slot: ButtonGroupSlotState;
declare const itemSlot: ButtonGroupItemSlotState;
declare const componentTarget: Component;

type GroupProps = InstanceType<typeof ButtonGroup>["$props"];
type ItemProps = InstanceType<typeof ButtonGroupItem>["$props"];

type _RoleIsLiteral = Expect<Equal<ButtonGroupRole, "group" | "toolbar">>;
type _OrientationIsLiteral = Expect<Equal<ButtonGroupOrientation, "horizontal" | "vertical">>;
type _StateIsLiteral = Expect<Equal<ButtonGroupState, "disabled" | "idle">>;
type _ItemStateIsLiteral = Expect<Equal<ButtonGroupItemState, "disabled" | "idle">>;
type _GroupActiveValueIsPublic = Expect<Equal<typeof group.activeValue, string | null>>;
type _ItemElementIsPrimitive = Expect<Equal<typeof item.element, ButtonGroupItemExpose["element"]>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly disabled: boolean;
      readonly orientation: ButtonGroupOrientation;
      readonly role: ButtonGroupRole;
      readonly rovingFocus: boolean;
      readonly state: ButtonGroupState;
    }
  >
>;
type _ItemSlotStateIsExact = Expect<
  Equal<
    typeof itemSlot,
    {
      readonly value: string;
      readonly disabled: boolean;
      readonly orientation: ButtonGroupOrientation;
      readonly state: ButtonGroupItemState;
    }
  >
>;

const groupProps: GroupProps = {
  ariaDescribedby: "actions-help",
  ariaLabel: "Actions",
  ariaLabelledby: "actions-label",
  as: componentTarget,
  disabled: false,
  loop: false,
  onPress: (value: string, event: MouseEvent) => {
    void value;
    void event;
  },
  orientation: "vertical",
  role: "toolbar",
  rovingFocus: true,
};
const itemProps: ItemProps = {
  ariaDescribedby: "save-help",
  ariaLabel: "Save",
  ariaLabelledby: "save-label",
  as: componentTarget,
  disabled: false,
  native: false,
  onPress: (value: string, event: MouseEvent) => {
    void value;
    void event;
  },
  type: "button",
  value: "save",
};

group.focus();
group.focusValue("save");
item.focus();

// @ts-expect-error role is a closed ARIA grouping contract.
const invalidRole: ButtonGroupRole = "menubar";

// @ts-expect-error orientation is a closed keyboard-navigation contract.
const invalidOrientation: ButtonGroupOrientation = "block";

// @ts-expect-error item values are always native strings.
const badItemValue: ItemProps = { value: null };

// @ts-expect-error native button type is limited to platform submit modes.
const badItemButtonType: ItemProps = { type: "menu", value: "save" };

void badItemButtonType;
void badItemValue;
void groupProps;
void invalidOrientation;
void invalidRole;
void itemProps;
