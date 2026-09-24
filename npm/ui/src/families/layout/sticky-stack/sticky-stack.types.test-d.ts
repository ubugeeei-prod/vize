/** Compile-only assertions for sticky stacking. */

import { computeStickyOffsets, sortByDocumentOrder } from "./sticky-stack.ts";
import type { StickyOffsets, StickyStackItemSlotState } from "./sticky-stack.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _OffsetsAreTyped = Expect<Equal<ReturnType<typeof computeStickyOffsets>, StickyOffsets>>;
computeStickyOffsets([10, 20], 4, [true, false]);
// @ts-expect-error enabled flags are booleans.
computeStickyOffsets([10], 0, ["yes"]);

const sorted = sortByDocumentOrder([{ element: null, name: "a" as const }]);
type _SortPreservesEntryType = Expect<Equal<typeof sorted, { element: null; name: "a" }[]>>;
// @ts-expect-error entries need an element field.
sortByDocumentOrder([{ name: "a" }]);

const _slot: StickyStackItemSlotState = { top: 0, stuck: false };
// @ts-expect-error slot state is readonly.
_slot.top = 1;
