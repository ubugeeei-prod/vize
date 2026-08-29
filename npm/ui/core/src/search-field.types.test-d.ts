/** Compile-only assertions for the public search field contract. */

import { h } from "vue";

import type {
  SearchFieldAriaInvalid,
  SearchFieldClearSlotState,
  SearchFieldClearVisibility,
  SearchFieldEnterKeyHint,
  SearchFieldExpose,
  SearchFieldInputMode,
  SearchFieldSlots,
} from "./search-field.ts";
import { SearchField } from "./search-field.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const search: SearchFieldExpose;
declare const clearSlot: SearchFieldClearSlotState;

type _ValueIsString = Expect<Equal<typeof search.value, string>>;
type _CompositionIsBoolean = Expect<Equal<typeof search.composing, boolean>>;
type _ClearVisibilityIsLiteral = Expect<
  Equal<SearchFieldClearVisibility, "always" | "auto" | "never">
>;
type _ClearSlotStateIsExact = Expect<
  Equal<typeof clearSlot, { readonly disabled: boolean; readonly empty: boolean }>
>;
type _InputModeIsLiteral = Expect<
  Equal<
    SearchFieldInputMode,
    "decimal" | "email" | "none" | "numeric" | "search" | "tel" | "text" | "url"
  >
>;
type _EnterKeyHintIsLiteral = Expect<
  Equal<SearchFieldEnterKeyHint, "done" | "enter" | "go" | "next" | "previous" | "search" | "send">
>;
type _InvalidStateIsNative = Expect<
  Equal<SearchFieldAriaInvalid, boolean | "grammar" | "spelling">
>;

search.setValue("Ada");
search.clear();
search.reset();
search.focus();
search.select();
h(SearchField, { ariaLabel: "Search", defaultValue: "query", showClear: "always" });

export const slotlessSlots: SearchFieldSlots = {};
export const clearSlots: SearchFieldSlots = {
  clear: (state) => (state.empty ? "Empty" : "Clear"),
};

// @ts-expect-error search values are strings.
search.setValue(1);

// @ts-expect-error clear visibility has a closed option set.
const clearVisibility: SearchFieldClearVisibility = "visible";

void clearVisibility;
