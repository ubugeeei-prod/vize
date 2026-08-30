/** Compile-only assertions for the public Stepper contract. */

import {
  Stepper,
  StepperContent,
  StepperItem,
  StepperList,
  StepperRoot,
  StepperTrigger,
  type StepperContentExpose,
  type StepperContentRole,
  type StepperContentSlotState,
  type StepperContentState,
  type StepperDirection,
  type StepperItemExpose,
  type StepperItemSlotState,
  type StepperItemState,
  type StepperListExpose,
  type StepperListSlotState,
  type StepperNavigationMode,
  type StepperOrientation,
  type StepperRootExpose,
  type StepperRootProps,
  type StepperRootState,
  type StepperSlotState,
  type StepperTriggerExpose,
  type StepperTriggerSlotState,
  type StepperValue,
} from "./stepper.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: StepperRootExpose;
declare const list: StepperListExpose;
declare const item: StepperItemExpose;
declare const trigger: StepperTriggerExpose;
declare const content: StepperContentExpose;
declare const slot: StepperSlotState;
declare const listSlot: StepperListSlotState;
declare const itemSlot: StepperItemSlotState;
declare const triggerSlot: StepperTriggerSlotState;
declare const contentSlot: StepperContentSlotState;

type _ValueAllowsEmptyCurrentStep = Expect<Equal<StepperValue, string | null>>;
type _NavigationModeIsLiteral = Expect<Equal<StepperNavigationMode, "free" | "linear">>;
type _OrientationIsLiteral = Expect<Equal<StepperOrientation, "horizontal" | "vertical">>;
type _DirectionIsLiteral = Expect<Equal<StepperDirection, "ltr" | "rtl">>;
type _RootStateIsLiteral = Expect<Equal<StepperRootState, "active" | "disabled" | "empty">>;
type _ItemStateIsLiteral = Expect<
  Equal<StepperItemState, "completed" | "current" | "disabled" | "pending">
>;
type _ContentStateIsLiteral = Expect<Equal<StepperContentState, "active" | "inactive">>;
type _ContentRoleIsLiteral = Expect<Equal<StepperContentRole, "group" | "region">>;
type _RootPropsAreExported = Expect<
  Equal<StepperRootProps["navigationMode"], StepperNavigationMode | undefined>
>;
type _RootValueIsNullable = Expect<Equal<typeof root.value, StepperValue>>;
type _ListElementIsDiv = Expect<Equal<typeof list.element, HTMLDivElement | null>>;
type _ItemElementIsDiv = Expect<Equal<typeof item.element, HTMLDivElement | null>>;
type _TriggerElementIsButton = Expect<Equal<typeof trigger.element, HTMLButtonElement | null>>;
type _ContentElementIsDiv = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly value: StepperValue;
      readonly completedValues: readonly string[];
      readonly currentIndex: number;
      readonly disabled: boolean;
      readonly navigationMode: StepperNavigationMode;
      readonly orientation: StepperOrientation;
      readonly dir: StepperDirection;
      readonly state: StepperRootState;
    }
  >
>;
type _ListSlotStateExtendsRoot = Expect<Equal<typeof listSlot, StepperListSlotState>>;
type _ItemSlotStateIsExact = Expect<
  Equal<
    typeof itemSlot,
    {
      readonly value: string;
      readonly index: number;
      readonly current: boolean;
      readonly completed: boolean;
      readonly disabled: boolean;
      readonly selectable: boolean;
      readonly locked: boolean;
      readonly orientation: StepperOrientation;
      readonly navigationMode: StepperNavigationMode;
      readonly state: StepperItemState;
    }
  >
>;
type _TriggerSlotStateMatchesItem = Expect<Equal<typeof triggerSlot, StepperTriggerSlotState>>;
type _ContentSlotStateIsExact = Expect<
  Equal<
    typeof contentSlot,
    {
      readonly value: string;
      readonly current: boolean;
      readonly active: boolean;
      readonly completed: boolean;
      readonly disabled: boolean;
      readonly orientation: StepperOrientation;
      readonly state: StepperContentState;
    }
  >
>;

const rootProps: InstanceType<typeof StepperRoot>["$props"] = {
  defaultValue: null,
  dir: "rtl",
  disabled: false,
  id: "checkout-stepper",
  loop: false,
  modelValue: "shipping",
  navigationMode: "free",
  onChange: (value: StepperValue, previous: StepperValue, event: Event | null) => {
    void value;
    void previous;
    void event;
  },
  "onUpdate:modelValue": (value: StepperValue) => value,
  orientation: "vertical",
};
const listProps: InstanceType<typeof StepperList>["$props"] = {
  ariaDescribedby: "checkout-help",
  ariaLabel: "Checkout steps",
  ariaLabelledby: "checkout-label",
};
const itemProps: InstanceType<typeof StepperItem>["$props"] = {
  ariaDescribedby: "shipping-help",
  ariaLabel: "Shipping",
  ariaLabelledby: "shipping-label",
  completed: true,
  disabled: false,
  id: "shipping-item",
  order: 1,
  textValue: "Shipping",
  value: "shipping",
};
const triggerProps: InstanceType<typeof StepperTrigger>["$props"] = {
  ariaDescribedby: "shipping-help",
  ariaLabel: "Shipping",
  ariaLabelledby: "shipping-label",
  type: "button",
};
const contentProps: InstanceType<typeof StepperContent>["$props"] = {
  ariaDescribedby: "shipping-help",
  ariaLabelledby: null,
  role: "group",
  value: "shipping",
};

root.focus();
root.setValue("billing");
root.setValue(null);
root.selectValue("shipping");
root.next();
root.previous();
root.reset();
root.isSelectable("review");
list.focus();
item.focus();
item.select();
trigger.focus();
trigger.select();
content.focusContent();

// @ts-expect-error navigation mode is a closed activation contract.
const invalidNavigationMode: StepperNavigationMode = "manual";

// @ts-expect-error orientation is a closed roving-focus contract.
const invalidOrientation: StepperOrientation = "block";

// @ts-expect-error direction is intentionally limited to logical text directions.
const invalidDirection: StepperDirection = "auto";

// @ts-expect-error values use null, not undefined, for empty current step.
const invalidValue: StepperValue = undefined;

// @ts-expect-error item value is required and string-only.
const badItemProps: InstanceType<typeof StepperItem>["$props"] = { value: null };

const badTriggerType: InstanceType<typeof StepperTrigger>["$props"] = {
  // @ts-expect-error native button type is limited to platform submit modes.
  type: "menu",
};

const badContentRole: InstanceType<typeof StepperContent>["$props"] = {
  // @ts-expect-error content role is intentionally limited to landmark grouping roles.
  role: "tabpanel",
  value: "shipping",
};

void Stepper;
void badContentRole;
void badItemProps;
void badTriggerType;
void contentProps;
void invalidDirection;
void invalidNavigationMode;
void invalidOrientation;
void invalidValue;
void itemProps;
void listProps;
void rootProps;
void triggerProps;
