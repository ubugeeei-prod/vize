/** Compile-only assertions for the public RadioGroup contract. */

import {
  RadioGroup,
  RadioGroupItem,
  type RadioGroupAriaInvalid,
  type RadioGroupExpose,
  type RadioGroupItemExpose,
  type RadioGroupItemState,
  type RadioGroupOrientation,
  type RadioGroupSlotState,
  type RadioGroupState,
  type RadioGroupValue,
} from "./radio-group.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const group: RadioGroupExpose;
declare const item: RadioGroupItemExpose;
declare const slot: RadioGroupSlotState;

type _ValueAllowsEmptySelection = Expect<Equal<RadioGroupValue, string | null>>;
type _OrientationIsLiteral = Expect<Equal<RadioGroupOrientation, "horizontal" | "vertical">>;
type _StateIsLiteral = Expect<Equal<RadioGroupState, "disabled" | "empty" | "selected">>;
type _ItemStateIsLiteral = Expect<Equal<RadioGroupItemState, "checked" | "disabled" | "unchecked">>;
type _InvalidStateIsNative = Expect<Equal<RadioGroupAriaInvalid, boolean | "grammar" | "spelling">>;
type _GroupValueIsNullable = Expect<Equal<typeof group.value, RadioGroupValue>>;
type _ItemElementIsInput = Expect<Equal<typeof item.element, HTMLInputElement | null>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly value: RadioGroupValue;
      readonly disabled: boolean;
      readonly required: boolean;
      readonly invalid: boolean;
      readonly orientation: RadioGroupOrientation;
      readonly state: RadioGroupState;
    }
  >
>;

const groupProps: InstanceType<typeof RadioGroup>["$props"] = {
  ariaDescribedby: "frequency-help",
  ariaErrormessage: "frequency-error",
  ariaInvalid: "grammar",
  ariaLabel: "Email frequency",
  ariaLabelledby: "frequency-label",
  defaultValue: null,
  disabled: false,
  id: "frequency",
  modelValue: "weekly",
  name: "frequency",
  orientation: "horizontal",
  required: true,
  onChange: (value: string, previous: RadioGroupValue, event: Event) => {
    void value;
    void previous;
    void event;
  },
  "onUpdate:modelValue": (value: RadioGroupValue) => value,
};
const itemProps: InstanceType<typeof RadioGroupItem>["$props"] = {
  ariaDescribedby: "daily-help",
  ariaLabel: "Daily",
  ariaLabelledby: "daily-label",
  disabled: false,
  id: "daily",
  value: "daily",
};

group.focus();
group.setValue("daily");
group.setValue(null);
group.reset();
item.focus();

// @ts-expect-error radio group orientation is a closed styling contract.
const invalidOrientation: RadioGroupOrientation = "block";

// @ts-expect-error radio group values use null, not undefined, for an empty selection.
const invalidValue: RadioGroupValue = undefined;

// @ts-expect-error radio item values are always native strings.
const badItemProps: InstanceType<typeof RadioGroupItem>["$props"] = { value: null };

// @ts-expect-error group selected state has a closed token contract.
const invalidState: RadioGroupState = "checked";

void badItemProps;
void groupProps;
void invalidOrientation;
void invalidState;
void invalidValue;
void itemProps;
