/** Compile-only assertions for the public Toolbar family contract. */

import type { Component } from "vue";

import { Toolbar, ToolbarItem } from "./toolbar.ts";
import type {
  ToolbarCssCustomProperty,
  ToolbarDataAttribute,
  ToolbarDataName,
  ToolbarDirection,
  ToolbarEmits,
  ToolbarExpose,
  ToolbarItemEmits,
  ToolbarItemExpose,
  ToolbarItemProps,
  ToolbarItemSlots,
  ToolbarItemSlotState,
  ToolbarItemState,
  ToolbarOrientation,
  ToolbarPart,
  ToolbarProps,
  ToolbarSlots,
  ToolbarSlotState,
  ToolbarState,
  ToolbarStyle,
} from "./toolbar.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const toolbar: ToolbarExpose;
declare const item: ToolbarItemExpose;
declare const slot: ToolbarSlotState;
declare const itemSlot: ToolbarItemSlotState;
declare const componentTarget: Component;

type ComponentToolbarProps = InstanceType<typeof Toolbar>["$props"];
type ComponentToolbarItemProps = InstanceType<typeof ToolbarItem>["$props"];

type _OrientationIsLiteral = Expect<Equal<ToolbarOrientation, "horizontal" | "vertical">>;
type _DirectionIsLiteral = Expect<Equal<ToolbarDirection, "ltr" | "rtl">>;
type _StateIsLiteral = Expect<Equal<ToolbarState, "disabled" | "idle">>;
type _ItemStateIsLiteral = Expect<Equal<ToolbarItemState, "disabled" | "idle">>;
type _PartNamesAreClosed = Expect<Equal<ToolbarPart, "item" | "root">>;
type _DataNamesAreClosed = Expect<Equal<ToolbarDataName, "toolbar" | "toolbar-item">>;
type _DataAttributesAreClosed = Expect<
  Equal<
    ToolbarDataAttribute,
    | "data-disabled"
    | "data-orientation"
    | "data-roving-focus"
    | "data-state"
    | "data-value"
    | "data-vize-ui"
  >
>;
type _CssCustomPropertiesAreClosed = Expect<
  Equal<ToolbarCssCustomProperty, "--vize-ui-toolbar-orientation">
>;
type _ToolbarPropsKeysAreClosed = Expect<
  Equal<
    keyof ToolbarProps,
    | "ariaDescribedby"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "dir"
    | "disabled"
    | "loop"
    | "orientation"
    | "rovingFocus"
  >
>;
type _ToolbarItemPropsKeysAreClosed = Expect<
  Equal<
    keyof ToolbarItemProps,
    | "ariaDescribedby"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "disabled"
    | "native"
    | "type"
    | "value"
  >
>;
type _ToolbarEmitsAreClosed = Expect<Equal<keyof ToolbarEmits, "press">>;
type _ToolbarItemEmitsAreClosed = Expect<Equal<keyof ToolbarItemEmits, "press">>;
type _ToolbarSlotReceivesState = Expect<
  Equal<Parameters<ToolbarSlots["default"]>[0], ToolbarSlotState>
>;
type _ToolbarItemSlotReceivesState = Expect<
  Equal<Parameters<ToolbarItemSlots["default"]>[0], ToolbarItemSlotState>
>;
type _ToolbarStyleIsStrict = Expect<
  Equal<
    Pick<ToolbarStyle, "--vize-ui-toolbar-orientation">,
    {
      readonly "--vize-ui-toolbar-orientation": ToolbarOrientation;
    }
  >
>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly disabled: boolean;
      readonly orientation: ToolbarOrientation;
      readonly dir: ToolbarDirection;
      readonly rovingFocus: boolean;
      readonly state: ToolbarState;
      readonly style: ToolbarStyle;
    }
  >
>;
type _ItemSlotStateIsExact = Expect<
  Equal<
    typeof itemSlot,
    {
      readonly value: string;
      readonly disabled: boolean;
      readonly orientation: ToolbarOrientation;
      readonly dir: ToolbarDirection;
      readonly state: ToolbarItemState;
    }
  >
>;

const toolbarProps = {
  ariaDescribedby: "editor-help",
  ariaLabel: "Editor actions",
  ariaLabelledby: "editor-label",
  as: componentTarget,
  dir: "rtl",
  disabled: false,
  loop: false,
  orientation: "vertical",
  rovingFocus: true,
} satisfies ToolbarProps;
const itemProps = {
  ariaDescribedby: "save-help",
  ariaLabel: "Save",
  ariaLabelledby: "save-label",
  as: componentTarget,
  disabled: false,
  native: false,
  type: "button",
  value: "save",
} satisfies ToolbarItemProps;
const toolbarComponentProps: ComponentToolbarProps = {
  ...toolbarProps,
  onPress: (value: string, event: MouseEvent) => {
    void value;
    void event;
  },
};
const itemComponentProps: ComponentToolbarItemProps = {
  ...itemProps,
  onPress: (value: string, event: MouseEvent) => {
    void value;
    void event;
  },
};

const activeValue: string | null = toolbar.activeValue;
const toolbarElement: ToolbarExpose["element"] = toolbar.element;
const itemElement: ToolbarItemExpose["element"] = item.element;
const toolbarDirection: ToolbarDirection = toolbar.dir;
const toolbarOrientation: ToolbarOrientation = toolbar.orientation;
const toolbarStyle: ToolbarStyle = toolbar.style;
const itemDirection: ToolbarDirection = item.dir;
const itemOrientation: ToolbarOrientation = item.orientation;

toolbar.focus();
toolbar.focusValue("save");
item.focus();

// @ts-expect-error Toolbar orientation is a closed keyboard-navigation contract.
const invalidOrientation: ToolbarProps = { orientation: "block" };

// @ts-expect-error Toolbar direction only accepts native inline directions.
const invalidDirection: ToolbarProps = { dir: "auto" };

// @ts-expect-error Toolbar item values are always native strings.
const badItemValue: ToolbarItemProps = { value: null };

// @ts-expect-error native button type is limited to platform submit modes.
const badItemButtonType: ToolbarItemProps = { type: "menu", value: "save" };

void Toolbar;
void ToolbarItem;
void activeValue;
void badItemButtonType;
void badItemValue;
void invalidDirection;
void invalidOrientation;
void itemComponentProps;
void itemDirection;
void itemElement;
void itemOrientation;
void itemProps;
void toolbarComponentProps;
void toolbarDirection;
void toolbarElement;
void toolbarOrientation;
void toolbarProps;
void toolbarStyle;
