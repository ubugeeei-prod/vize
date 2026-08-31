/** Compile-only assertions for the public NativeSelect contract. */

import type {
  NativeSelectAriaInvalid,
  NativeSelectDirection,
  NativeSelectEmits,
  NativeSelectExpose,
  NativeSelectMultipleValue,
  NativeSelectOption,
  NativeSelectOptionState,
  NativeSelectProps,
  NativeSelectSelectionMode,
  NativeSelectSingleValue,
  NativeSelectSlotState,
  NativeSelectSlots,
  NativeSelectState,
  NativeSelectValue,
} from "./native-select.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const select: NativeSelectExpose;
declare const slot: NativeSelectSlotState;

type _SingleValueIsString = Expect<Equal<NativeSelectSingleValue, string>>;
type _MultipleValueIsReadonlyStrings = Expect<Equal<NativeSelectMultipleValue, readonly string[]>>;
type _ValueIsStrictNativeSelectValue = Expect<Equal<NativeSelectValue, string | readonly string[]>>;
type _SelectionModeIsLiteral = Expect<Equal<NativeSelectSelectionMode, "single" | "multiple">>;
type _DirectionIsLiteral = Expect<Equal<NativeSelectDirection, "ltr" | "rtl">>;
type _InvalidStateIsNative = Expect<
  Equal<NativeSelectAriaInvalid, boolean | "grammar" | "spelling">
>;
type _StateIsLiteral = Expect<Equal<NativeSelectState, "disabled" | "empty" | "selected">>;
type _OptionStateIsLiteral = Expect<
  Equal<NativeSelectOptionState, "disabled" | "selected" | "unselected">
>;
type _SlotValueMatchesPublicValue = Expect<Equal<typeof slot.value, NativeSelectValue>>;
type _ExposeValueMatchesPublicValue = Expect<Equal<typeof select.value, NativeSelectValue>>;
type _UpdatePayloadIsPublicValue = Expect<
  Equal<NativeSelectEmits["update:modelValue"], [value: NativeSelectValue]>
>;
type _ChangePayloadIncludesPreviousAndNativeEvent = Expect<
  Equal<
    NativeSelectEmits["change"],
    [value: NativeSelectValue, previous: NativeSelectValue, nativeEvent: Event]
  >
>;
type _DefaultSlotReceivesState = Expect<
  Equal<Parameters<NonNullable<NativeSelectSlots["default"]>>[0], NativeSelectSlotState>
>;

const option = {
  disabled: false,
  label: "Todo",
  value: "todo",
} satisfies NativeSelectOption;

const props = {
  defaultValue: ["todo"],
  direction: "rtl",
  multiple: true,
  name: "status",
  options: [option],
  required: true,
} satisfies NativeSelectProps;

select.setValue("todo");
select.setValue(["todo"]);
select.clear();
select.reset();
select.focus();

// @ts-expect-error native select values are string-backed.
select.setValue(1);

// @ts-expect-error readOnly is not a native select state and is not simulated.
const readOnlyProps: NativeSelectProps = { readOnly: true };

// @ts-expect-error option values are submitted as strings.
const numberOption: NativeSelectOption = { label: "One", value: 1 };

void props;
void readOnlyProps;
void numberOption;
