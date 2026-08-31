/** Compile-only assertions for the public history contract. */

import type { ShallowRef } from "vue";

import { createEditableTransaction, createHistory } from "./history.ts";
import type { HistoryTransaction } from "./history.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const history = createHistory({ limit: 100, coalesceWindow: 500, now: () => 0 });

history.pushSnapshot({
  before: { title: "a" },
  after: { title: "b" },
  apply(value: { title: string }) {
    void value;
  },
  isEqual: (left, right) => left.title === right.title,
  label: "Rename",
  coalesceKey: "rename",
});

export const frame: HistoryTransaction = history.beginTransaction("Batch");
export const result: number = history.transaction("Batch", () => 42);

export const session = createEditableTransaction({
  history,
  read: () => "value",
  write(value: string) {
    void value;
  },
});

type _CanUndoIsReadonly = Expect<Equal<typeof history.canUndo, Readonly<ShallowRef<boolean>>>>;
type _DepthIsReadonly = Expect<Equal<typeof history.undoDepth, Readonly<ShallowRef<number>>>>;
type _LabelIsNullable = Expect<
  Equal<typeof history.undoLabel, Readonly<ShallowRef<string | null>>>
>;
type _UndoReports = Expect<Equal<ReturnType<typeof history.undo>, boolean>>;
type _CommitReports = Expect<Equal<ReturnType<typeof session.commit>, boolean>>;
type _EditingIsReadonly = Expect<Equal<typeof session.isEditing, Readonly<ShallowRef<boolean>>>>;

// @ts-expect-error consumers cannot mutate readonly reactive state.
history.canUndo.value = true;
// @ts-expect-error entries need both undo and redo.
history.push({ undo: () => undefined });
// @ts-expect-error snapshot values must agree with the apply parameter.
history.pushSnapshot({ before: 1, after: "2", apply: (value: number) => void value });
// @ts-expect-error limit must be a number.
createHistory({ limit: "100" });
// @ts-expect-error updates must match the session value type.
session.update(42);
// @ts-expect-error editable sessions require a history controller.
createEditableTransaction({ read: () => "", write: () => undefined });
