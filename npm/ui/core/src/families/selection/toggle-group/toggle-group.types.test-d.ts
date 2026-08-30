/** Compile-only assertions for the public ToggleGroup contract. */

import type { Component } from "vue";

import {
  ToggleGroup,
  ToggleGroupItem,
  type ToggleGroupExpose,
  type ToggleGroupItemExpose,
  type ToggleGroupItemSlotState,
  type ToggleGroupItemState,
  type ToggleGroupOrientation,
  type ToggleGroupSlotState,
  type ToggleGroupState,
  type ToggleGroupType,
  type ToggleGroupValue,
} from "./toggle-group.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const group: ToggleGroupExpose;
declare const item: ToggleGroupItemExpose;
declare const slot: ToggleGroupSlotState;
declare const itemSlot: ToggleGroupItemSlotState;
declare const componentTarget: Component;

type GroupProps = InstanceType<typeof ToggleGroup>["$props"];
type ItemProps = InstanceType<typeof ToggleGroupItem>["$props"];

type _TypeIsLiteral = Expect<Equal<ToggleGroupType, "single" | "multiple">>;
type _OrientationIsLiteral = Expect<Equal<ToggleGroupOrientation, "horizontal" | "vertical">>;
type _ValueAllowsSingleMultipleAndEmpty = Expect<
  Equal<ToggleGroupValue, string | readonly string[] | null>
>;
type _StateIsLiteral = Expect<Equal<ToggleGroupState, "disabled" | "empty" | "selected">>;
type _ItemStateIsLiteral = Expect<
  Equal<ToggleGroupItemState, "disabled" | "pressed" | "unpressed">
>;
type _GroupValueIsPublic = Expect<Equal<typeof group.value, ToggleGroupValue>>;
type _ItemElementIsPrimitive = Expect<Equal<typeof item.element, ToggleGroupItemExpose["element"]>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly value: ToggleGroupValue;
      readonly pressedValues: readonly string[];
      readonly disabled: boolean;
      readonly type: ToggleGroupType;
      readonly orientation: ToggleGroupOrientation;
      readonly state: ToggleGroupState;
    }
  >
>;
type _ItemSlotStateIsExact = Expect<
  Equal<
    typeof itemSlot,
    {
      readonly value: string;
      readonly pressed: boolean;
      readonly disabled: boolean;
      readonly orientation: ToggleGroupOrientation;
      readonly state: ToggleGroupItemState;
    }
  >
>;

const groupProps: GroupProps = {
  ariaDescribedby: "formatting-help",
  ariaLabel: "Formatting",
  ariaLabelledby: "formatting-label",
  as: componentTarget,
  defaultValue: ["bold"],
  disabled: false,
  loop: false,
  modelValue: null,
  orientation: "vertical",
  rovingFocus: true,
  type: "multiple",
  onChange: (value: ToggleGroupValue, previous: ToggleGroupValue, event: MouseEvent) => {
    void value;
    void previous;
    void event;
  },
  "onUpdate:modelValue": (value: ToggleGroupValue) => value,
};
const itemProps: ItemProps = {
  ariaDescribedby: "bold-help",
  ariaLabel: "Bold",
  ariaLabelledby: "bold-label",
  as: componentTarget,
  disabled: false,
  native: false,
  onPress: (value: string, pressed: boolean, event: MouseEvent) => {
    void value;
    void pressed;
    void event;
  },
  type: "button",
  value: "bold",
};

group.focus();
group.setValue("bold");
group.setValue(["bold"]);
group.setValue(null);
group.toggleValue("bold", false);
group.reset();
item.focus();

// @ts-expect-error toggle group type is a closed mode contract.
const invalidType: ToggleGroupType = "radio";

// @ts-expect-error orientation is a closed keyboard-navigation contract.
const invalidOrientation: ToggleGroupOrientation = "block";

// @ts-expect-error values are strings, arrays, or null for empty single selection.
const invalidValue: ToggleGroupValue = undefined;

// @ts-expect-error item values are always native strings.
const badItemValue: ItemProps = { value: null };

// @ts-expect-error native button type is limited to platform submit modes.
const badItemButtonType: ItemProps = { type: "menu", value: "bold" };

void badItemButtonType;
void badItemValue;
void groupProps;
void invalidOrientation;
void invalidType;
void invalidValue;
void itemProps;
