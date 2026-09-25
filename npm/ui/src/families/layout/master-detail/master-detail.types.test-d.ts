/** Compile-only assertions for the master-detail pattern. */

import type {
  MasterDetailDetailSlotState,
  MasterDetailLayout,
  MasterDetailMasterSlotState,
} from "./master-detail.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _LayoutIsClosed = Expect<Equal<MasterDetailLayout, "split" | "stacked">>;

declare const master: MasterDetailMasterSlotState<"inbox" | "sent">;
master.select("inbox");
// @ts-expect-error select only accepts known keys.
master.select("drafts");
type _MasterSelectionIsNullable = Expect<Equal<typeof master.selected, "inbox" | "sent" | null>>;

declare const detail: MasterDetailDetailSlotState<number>;
type _DetailSelectionIsNonNull = Expect<Equal<typeof detail.selected, number>>;
detail.back() satisfies void;
