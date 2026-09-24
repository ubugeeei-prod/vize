/** Compile-only assertions for the public NumberField contract. */

import {
  NumberField,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
  createNumberFieldParser,
  stepNumberFieldValue,
  type NumberFieldBounds,
  type NumberFieldChangeSource,
  type NumberFieldEmits,
  type NumberFieldExpose,
  type NumberFieldParser,
  type NumberFieldProps,
  type NumberFieldSlotState,
  type NumberFieldSlots,
  type NumberFieldState,
  type NumberFieldStepDirection,
  type NumberFieldTriggerSlotState,
  type NumberFieldValue,
} from "./number-field.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const control: NumberFieldExpose;
declare const slot: NumberFieldSlotState;
declare const trigger: NumberFieldTriggerSlotState;
declare const bounds: NumberFieldBounds;

type _ValueIsNullableNumber = Expect<Equal<NumberFieldValue, number | null>>;
type _StateIsClosed = Expect<
  Equal<
    NumberFieldState,
    "disabled" | "empty" | "in-range" | "invalid" | "max" | "min" | "readonly"
  >
>;
type _DirectionIsClosed = Expect<Equal<NumberFieldStepDirection, "increment" | "decrement">>;
type _SourceIsClosed = Expect<
  Equal<
    NumberFieldChangeSource,
    "api" | "blur" | "enter" | "keyboard" | "reset" | "trigger" | "wheel"
  >
>;
type _ModelValueAcceptsNull = Expect<
  Equal<NumberFieldProps["modelValue"], NumberFieldValue | undefined>
>;
type _FormatOptionsAreIntl = Expect<
  Equal<NumberFieldProps["formatOptions"], Intl.NumberFormatOptions | undefined>
>;
type _UpdatePayload = Expect<
  Equal<NumberFieldEmits["update:modelValue"], [value: NumberFieldValue]>
>;
type _ChangePayload = Expect<
  Equal<
    NumberFieldEmits["change"],
    [value: NumberFieldValue, previous: NumberFieldValue, source: NumberFieldChangeSource]
  >
>;
type _SlotUsesState = Expect<
  Equal<Parameters<NumberFieldSlots["default"]>[0], NumberFieldSlotState>
>;
type _ExposeValue = Expect<Equal<typeof control.value, NumberFieldValue>>;
type _ExposeInput = Expect<Equal<typeof control.input, HTMLInputElement | null>>;
type _SlotFormatted = Expect<Equal<typeof slot.formattedValue, string>>;
type _TriggerDirection = Expect<Equal<typeof trigger.direction, NumberFieldStepDirection>>;
type _ParserParse = Expect<Equal<ReturnType<NumberFieldParser["parse"]>, number | null>>;
type _Stepping = Expect<Equal<ReturnType<typeof stepNumberFieldValue>, number>>;

const props = {
  allowWheel: true,
  clampOnCommit: false,
  defaultValue: null,
  formatOptions: { style: "currency", currency: "EUR" },
  largeStep: 100,
  locale: "de-DE",
  max: 1000,
  min: 0,
  snapOnCommit: true,
  step: 5,
} satisfies NumberFieldProps;
const componentProps: InstanceType<typeof NumberField>["$props"] = {
  ariaLabel: "Price",
  modelValue: 12,
  name: "price",
  onChange: (
    value: NumberFieldValue,
    previous: NumberFieldValue,
    source: NumberFieldChangeSource,
  ) => {
    void value;
    void previous;
    void source;
  },
  "onUpdate:modelValue": (value: NumberFieldValue) => value,
};
const inputProps: InstanceType<typeof NumberFieldInput>["$props"] = { placeholder: "0" };
const incrementProps: InstanceType<typeof NumberFieldIncrement>["$props"] = { ariaLabel: "Plus" };
const decrementProps: InstanceType<typeof NumberFieldDecrement>["$props"] = { ariaLabel: "Minus" };
const parser: NumberFieldParser = createNumberFieldParser("en-US", { style: "percent" });

control.increment(2);
control.decrement();
control.setValue(null);
control.commit();
stepNumberFieldValue(1, 1, bounds);
stepNumberFieldValue(null, -1, bounds, 10);

// @ts-expect-error values are numbers or null, never formatted strings.
control.setValue("12");

// @ts-expect-error step directions are signed units.
stepNumberFieldValue(1, 2, bounds);

// @ts-expect-error the state token is closed.
const state: NumberFieldState = "loading";

// @ts-expect-error controlled values reject strings.
const invalidProps = { modelValue: "12" } satisfies NumberFieldProps;

// @ts-expect-error format options must be Intl.NumberFormat options.
const invalidFormat = { formatOptions: { style: "money" } } satisfies NumberFieldProps;

void componentProps;
void decrementProps;
void incrementProps;
void inputProps;
void invalidFormat;
void invalidProps;
void parser;
void props;
void state;
