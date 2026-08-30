/** Compile-only assertions for the public Tabs contract. */

import {
  Tabs,
  TabsContent,
  TabsList,
  TabsRoot,
  TabsTrigger,
  type TabsActivationMode,
  type TabsContentExpose,
  type TabsContentSlotState,
  type TabsContentState,
  type TabsDirection,
  type TabsListExpose,
  type TabsListSlotState,
  type TabsOrientation,
  type TabsRootExpose,
  type TabsSlotState,
  type TabsState,
  type TabsTriggerExpose,
  type TabsTriggerSlotState,
  type TabsTriggerState,
  type TabsValue,
} from "./tabs.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: TabsRootExpose;
declare const list: TabsListExpose;
declare const trigger: TabsTriggerExpose;
declare const content: TabsContentExpose;
declare const slot: TabsSlotState;
declare const listSlot: TabsListSlotState;
declare const triggerSlot: TabsTriggerSlotState;
declare const contentSlot: TabsContentSlotState;

type _ValueAllowsEmptySelection = Expect<Equal<TabsValue, string | null>>;
type _ActivationModeIsLiteral = Expect<Equal<TabsActivationMode, "automatic" | "manual">>;
type _OrientationIsLiteral = Expect<Equal<TabsOrientation, "horizontal" | "vertical">>;
type _DirectionIsLiteral = Expect<Equal<TabsDirection, "ltr" | "rtl">>;
type _StateIsLiteral = Expect<Equal<TabsState, "disabled" | "empty" | "selected">>;
type _TriggerStateIsLiteral = Expect<Equal<TabsTriggerState, "active" | "disabled" | "inactive">>;
type _ContentStateIsLiteral = Expect<Equal<TabsContentState, "active" | "inactive">>;
type _RootValueIsNullable = Expect<Equal<typeof root.value, TabsValue>>;
type _ListElementIsDiv = Expect<Equal<typeof list.element, HTMLDivElement | null>>;
type _TriggerElementIsButton = Expect<Equal<typeof trigger.element, HTMLButtonElement | null>>;
type _ContentElementIsDiv = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly value: TabsValue;
      readonly disabled: boolean;
      readonly activationMode: TabsActivationMode;
      readonly orientation: TabsOrientation;
      readonly dir: TabsDirection;
      readonly state: TabsState;
    }
  >
>;
type _ListSlotStateExtendsRoot = Expect<Equal<typeof listSlot, TabsListSlotState>>;
type _TriggerSlotStateIsExact = Expect<
  Equal<
    typeof triggerSlot,
    {
      readonly value: string;
      readonly selected: boolean;
      readonly disabled: boolean;
      readonly activationMode: TabsActivationMode;
      readonly orientation: TabsOrientation;
      readonly state: TabsTriggerState;
    }
  >
>;
type _ContentSlotStateIsExact = Expect<
  Equal<
    typeof contentSlot,
    {
      readonly value: string;
      readonly selected: boolean;
      readonly disabled: boolean;
      readonly orientation: TabsOrientation;
      readonly state: TabsContentState;
    }
  >
>;

const rootProps: InstanceType<typeof TabsRoot>["$props"] = {
  activationMode: "manual",
  defaultValue: null,
  dir: "rtl",
  disabled: false,
  id: "settings-tabs",
  loop: false,
  modelValue: "profile",
  orientation: "vertical",
  onChange: (value: TabsValue, previous: TabsValue, event: Event | null) => {
    void value;
    void previous;
    void event;
  },
  "onUpdate:modelValue": (value: TabsValue) => value,
};
const listProps: InstanceType<typeof TabsList>["$props"] = {
  ariaDescribedby: "settings-help",
  ariaLabel: "Settings sections",
  ariaLabelledby: "settings-label",
};
const triggerProps: InstanceType<typeof TabsTrigger>["$props"] = {
  ariaDescribedby: "profile-help",
  ariaLabel: "Profile",
  ariaLabelledby: "profile-label",
  disabled: false,
  order: 1,
  textValue: "Profile",
  type: "button",
  value: "profile",
};
const contentProps: InstanceType<typeof TabsContent>["$props"] = {
  ariaDescribedby: "profile-help",
  ariaLabelledby: null,
  value: "profile",
};

root.focus();
root.setValue("profile");
root.setValue(null);
root.reset();
list.focus();
trigger.focus();
content.focusContent();

// @ts-expect-error activation mode is a closed keyboard contract.
const invalidActivationMode: TabsActivationMode = "hover";

// @ts-expect-error orientation is a closed roving-focus contract.
const invalidOrientation: TabsOrientation = "block";

// @ts-expect-error direction is intentionally limited to logical text directions.
const invalidDirection: TabsDirection = "auto";

// @ts-expect-error values use null, not undefined, for empty selection.
const invalidValue: TabsValue = undefined;

// @ts-expect-error trigger value is required and string-only.
const badTriggerProps: InstanceType<typeof TabsTrigger>["$props"] = { value: null };

const badTriggerType: InstanceType<typeof TabsTrigger>["$props"] = {
  // @ts-expect-error native button type is limited to platform submit modes.
  type: "menu",
  value: "profile",
};

void Tabs;
void badTriggerProps;
void badTriggerType;
void contentProps;
void invalidActivationMode;
void invalidDirection;
void invalidOrientation;
void invalidValue;
void listProps;
void rootProps;
void triggerProps;
