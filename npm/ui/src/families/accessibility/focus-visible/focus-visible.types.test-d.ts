/** Compile-only assertions for the public FocusVisible contract. */

import type { ComputedRef, ShallowRef } from "vue";

import type {
  FocusVisibleModality,
  FocusVisibleProviderExpose,
  FocusVisibleSlotState,
  FocusVisibleState,
} from "./focus-visible.ts";
import {
  FocusVisibleProvider,
  isTextEntryElement,
  shouldShowFocusRing,
  useFocusVisible,
} from "./focus-visible.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const state: FocusVisibleState;
declare const slot: FocusVisibleSlotState;
declare const exposed: FocusVisibleProviderExpose;

type _Modality = Expect<Equal<FocusVisibleModality, "keyboard" | "pointer" | "touch" | "virtual">>;
type _Visible = Expect<Equal<typeof state.isFocusVisible, ComputedRef<boolean>>>;
type _StateModality = Expect<
  Equal<typeof state.modality, Readonly<ShallowRef<FocusVisibleModality | null>>>
>;
type _SlotModality = Expect<Equal<typeof slot.modality, FocusVisibleModality | null>>;
type _Element = Expect<Equal<typeof exposed.element, HTMLDivElement | null>>;
type _Use = Expect<Equal<ReturnType<typeof useFocusVisible>, FocusVisibleState>>;
type _Heuristic = Expect<Equal<ReturnType<typeof shouldShowFocusRing>, boolean>>;
type _TextEntry = Expect<Equal<ReturnType<typeof isTextEntryElement>, boolean>>;

const props: InstanceType<typeof FocusVisibleProvider>["$props"] = {
  attribute: "data-ring",
  disabled: false,
};

// @ts-expect-error attribute names are strings.
const badAttribute: InstanceType<typeof FocusVisibleProvider>["$props"] = { attribute: true };

// @ts-expect-error modalities are a closed union.
const badModality: FocusVisibleModality = "mouse";

void badAttribute;
void badModality;
void props;
