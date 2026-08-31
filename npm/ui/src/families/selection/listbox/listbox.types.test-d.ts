/** Compile-only assertions for the public Listbox contract. */

import { h } from "vue";

import {
  Listbox,
  ListboxItem,
  listboxSelectedValues,
  normalizeListboxValue,
  type ListboxAriaInvalid,
  type ListboxDirection,
  type ListboxExpose,
  type ListboxItemExpose,
  type ListboxItemSlotState,
  type ListboxItemState,
  type ListboxMultipleValue,
  type ListboxOrientation,
  type ListboxSelectionMode,
  type ListboxSingleValue,
  type ListboxSlotState,
  type ListboxState,
  type ListboxValue,
} from "./listbox.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const listbox: ListboxExpose;
declare const item: ListboxItemExpose;
declare const slot: ListboxSlotState;
declare const itemSlot: ListboxItemSlotState;

type _SingleValueAllowsEmptySelection = Expect<Equal<ListboxSingleValue, string | null>>;
type _MultipleValueIsReadonlyArray = Expect<Equal<ListboxMultipleValue, readonly string[]>>;
type _ValueAllowsSingleOrMultiple = Expect<Equal<ListboxValue, string | null | readonly string[]>>;
type _ModeIsLiteral = Expect<Equal<ListboxSelectionMode, "single" | "multiple">>;
type _OrientationIsLiteral = Expect<Equal<ListboxOrientation, "horizontal" | "vertical">>;
type _DirectionIsLiteral = Expect<Equal<ListboxDirection, "ltr" | "rtl">>;
type _StateIsLiteral = Expect<Equal<ListboxState, "disabled" | "empty" | "selected">>;
type _ItemStateIsLiteral = Expect<Equal<ListboxItemState, "disabled" | "selected" | "unselected">>;
type _InvalidStateIsNative = Expect<Equal<ListboxAriaInvalid, boolean | "grammar" | "spelling">>;
type _ExposeValueIsPublicValue = Expect<Equal<typeof listbox.value, ListboxValue>>;
type _ExposeSelectedValuesAreReadonly = Expect<
  Equal<typeof listbox.selectedValues, readonly string[]>
>;
type _ItemElementIsDiv = Expect<Equal<typeof item.element, HTMLDivElement | null>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly value: ListboxValue;
      readonly selectedValues: readonly string[];
      readonly activeValue: string | null;
      readonly disabled: boolean;
      readonly required: boolean;
      readonly invalid: boolean;
      readonly selectionMode: ListboxSelectionMode;
      readonly orientation: ListboxOrientation;
      readonly direction: ListboxDirection;
      readonly state: ListboxState;
    }
  >
>;
type _ItemSlotStateIsExact = Expect<
  Equal<
    typeof itemSlot,
    {
      readonly value: string;
      readonly active: boolean;
      readonly selected: boolean;
      readonly disabled: boolean;
      readonly selectionMode: ListboxSelectionMode;
      readonly state: ListboxItemState;
    }
  >
>;

const listboxProps: InstanceType<typeof Listbox>["$props"] = {
  ariaDescribedby: "letters-help",
  ariaErrormessage: "letters-error",
  ariaInvalid: "spelling",
  ariaLabel: "Letters",
  ariaLabelledby: "letters-label",
  defaultValue: ["alpha"],
  direction: "rtl",
  disabled: false,
  id: "letters",
  loop: true,
  modelValue: "bravo",
  orientation: "horizontal",
  required: true,
  selectionMode: "multiple",
  typeahead: true,
  typeaheadTimeout: 700,
  onChange: (value: ListboxValue, previous: ListboxValue, event: Event) => {
    void value;
    void previous;
    void event;
  },
  "onUpdate:modelValue": (value: ListboxValue) => value,
};
const itemProps: InstanceType<typeof ListboxItem>["$props"] = {
  ariaDescribedby: "alpha-help",
  ariaLabel: "Alpha",
  ariaLabelledby: "alpha-label",
  disabled: false,
  id: "alpha",
  order: 1,
  textValue: "Alpha",
  value: "alpha",
};

h(Listbox, listboxProps, {
  default: (state: ListboxSlotState) => state.selectedValues,
  empty: (state: ListboxSlotState) => state.state,
});
h(ListboxItem, itemProps, {
  default: (state: ListboxItemSlotState) => state.value,
  indicator: (state: ListboxItemSlotState) => state.selected,
});

listbox.focus();
listbox.navigate("next");
listbox.setActiveValue("alpha");
listbox.setValue(["alpha"]);
listbox.selectValue("bravo", new Event("select"));
listbox.toggleValue("alpha");
listbox.clear();
listbox.reset();
item.focus();
item.select();
normalizeListboxValue("alpha", "single");
listboxSelectedValues(["alpha", "bravo"]);

// @ts-expect-error selection mode is a closed contract.
const invalidMode: ListboxSelectionMode = "range";

// @ts-expect-error listbox values use null, not undefined, for an empty selection.
const invalidValue: ListboxValue = undefined;

// @ts-expect-error listbox item values are always strings.
const badItemProps: InstanceType<typeof ListboxItem>["$props"] = { value: null };

// @ts-expect-error navigation exposes movement commands only.
listbox.navigate("focus");

void badItemProps;
void invalidMode;
void invalidValue;
void itemProps;
void listboxProps;
