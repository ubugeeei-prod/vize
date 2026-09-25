/** Compile-only assertions for slot and prop merging helpers. */

import type { Slot, StyleValue } from "vue";

import { hasSlotContent, isHandlerKey, mergeProps, presentSlotNames } from "./slot-utils.ts";
import type { MergedProps } from "./slot-utils.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const onClickA = (_event: MouseEvent): void => undefined;
const onClickB = (_event: MouseEvent): number => 1;

const merged = mergeProps(
  { id: "a", class: ["x"], onClick: onClickA, "aria-label": "first" },
  { class: { y: true }, style: { color: "red" }, onClick: onClickB, disabled: true },
  { id: undefined as string | undefined, tabindex: 0 },
);
type _ClassBecomesString = Expect<Equal<typeof merged.class, string>>;
type _StyleBecomesStyleValue = Expect<Equal<typeof merged.style, StyleValue>>;
type _HandlersAreUnioned = Expect<Equal<typeof merged.onClick, typeof onClickA | typeof onClickB>>;
const mergedHandlerArray = mergeProps({ onClick: [onClickA, onClickB] });
type _HandlerArraysBecomeCallable = Expect<
  Equal<typeof mergedHandlerArray.onClick, typeof onClickA | typeof onClickB>
>;
type _OptionalLaterValuesKeepEarlierTypes = Expect<Equal<typeof merged.id, string>>;
type _OtherKeysArePreserved = Expect<Equal<typeof merged.disabled, true>>;
type _LabelSurvives = Expect<Equal<(typeof merged)["aria-label"], "first">>;
type _TabindexSurvives = Expect<Equal<typeof merged.tabindex, 0>>;
type _EmptyMergeIsEmpty = Expect<Equal<MergedProps<[]>, {}>>;
// @ts-expect-error keys absent from every source do not exist.
void merged.title;

declare const slot: Slot<{ readonly item: number }> | undefined;
hasSlotContent(slot, { item: 1 }) satisfies boolean;
// @ts-expect-error slot props are checked.
hasSlotContent(slot, { item: "1" });

declare const slots: { readonly header?: Slot; readonly footer?: Slot };
const names = presentSlotNames(slots);
type _SlotNamesAreTyped = Expect<Equal<typeof names, ("header" | "footer")[]>>;

declare const key: string;
if (isHandlerKey(key)) {
  key satisfies `on${Capitalize<string>}`;
}
