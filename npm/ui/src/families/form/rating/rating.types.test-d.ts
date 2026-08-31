/** Compile-only assertions for the public Rating contract. */

import {
  Rating,
  type RatingAriaInvalid,
  type RatingDirection,
  type RatingEmits,
  type RatingExpose,
  type RatingItemSlotState,
  type RatingItemState,
  type RatingProps,
  type RatingSlotState,
  type RatingSlots,
  type RatingState,
  type RatingValue,
} from "./rating.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const control: RatingExpose;
declare const slot: RatingSlotState;
declare const item: RatingItemSlotState;

type _ValueAllowsEmptySelection = Expect<Equal<RatingValue, number | null>>;
type _DirectionIsClosed = Expect<Equal<RatingDirection, "ltr" | "rtl">>;
type _StateIsClosed = Expect<
  Equal<RatingState, "disabled" | "empty" | "invalid" | "readonly" | "selected">
>;
type _ItemStateIsClosed = Expect<
  Equal<RatingItemState, "checked" | "disabled" | "readonly" | "unchecked">
>;
type _InvalidStateIsNative = Expect<Equal<RatingAriaInvalid, boolean | "grammar" | "spelling">>;
type _ExposeValueIsNullableNumber = Expect<Equal<typeof control.value, RatingValue>>;
type _ExposeElementsAreInputs = Expect<Equal<typeof control.elements, readonly HTMLInputElement[]>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly value: RatingValue;
      readonly min: number;
      readonly max: number;
      readonly count: number;
      readonly items: readonly number[];
      readonly percent: number;
      readonly direction: RatingDirection;
      readonly disabled: boolean;
      readonly readOnly: boolean;
      readonly required: boolean;
      readonly invalid: boolean;
      readonly clearable: boolean;
      readonly state: RatingState;
    }
  >
>;
type _ItemSlotCarriesItemAndGroupState = Expect<
  Equal<
    typeof item,
    {
      readonly value: number;
      readonly index: number;
      readonly currentValue: RatingValue;
      readonly checked: boolean;
      readonly active: boolean;
      readonly percent: number;
      readonly min: number;
      readonly max: number;
      readonly count: number;
      readonly direction: RatingDirection;
      readonly disabled: boolean;
      readonly readOnly: boolean;
      readonly required: boolean;
      readonly invalid: boolean;
      readonly clearable: boolean;
      readonly state: RatingItemState;
    }
  >
>;
type _PropsModelValueIsNullableNumber = Expect<
  Equal<RatingProps["modelValue"], RatingValue | undefined>
>;
type _UpdatePayloadIsNullableNumber = Expect<
  Equal<RatingEmits["update:modelValue"], [value: RatingValue]>
>;
type _ChangePayloadIncludesPreviousAndNativeEvent = Expect<
  Equal<RatingEmits["change"], [value: RatingValue, previous: RatingValue, nativeEvent: Event]>
>;
type _DefaultSlotUsesState = Expect<Equal<Parameters<RatingSlots["default"]>[0], RatingSlotState>>;
type _ItemSlotUsesState = Expect<Equal<Parameters<RatingSlots["item"]>[0], RatingItemSlotState>>;

const props = {
  ariaInvalid: "grammar",
  clearable: true,
  count: 5,
  defaultValue: null,
  dir: "rtl",
  max: 5,
  min: 1,
  modelValue: 3,
  readOnly: true,
} satisfies RatingProps;
const componentProps: InstanceType<typeof Rating>["$props"] = {
  ariaLabel: "Movie score",
  defaultValue: 2,
  name: "score",
  onChange: (value: RatingValue, previous: RatingValue, event: Event) => {
    void value;
    void previous;
    void event;
  },
  onClear: (previous: number, event: Event) => {
    void previous;
    void event;
  },
  "onUpdate:modelValue": (value: RatingValue) => value,
};

control.focus();
control.setValue(3);
control.setValue(null);
control.clear();
control.reset();

// @ts-expect-error rating values are numbers or null.
control.setValue("3");

// @ts-expect-error rating direction is closed to native LTR and RTL.
const direction: RatingDirection = "auto";

// @ts-expect-error rating props reject string controlled values.
const invalidProps = { modelValue: "3" } satisfies RatingProps;

// @ts-expect-error rating state is closed to the public data contract.
const state: RatingState = "loading";

void componentProps;
void direction;
void invalidProps;
void props;
void state;
