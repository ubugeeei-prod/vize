/** Compile-only assertions for the public Kanban contract. */

import type {
  KanbanBoard,
  KanbanCardSlotProps,
  KanbanColumnDefinition,
  KanbanMoveEvent,
} from "./kanban.ts";
import { Kanban } from "./kanban.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type Status = "todo" | "done";

interface Task {
  readonly id: string;
}

declare const move: KanbanMoveEvent<Task, Status>;
declare const slot: KanbanCardSlotProps<Task, Status>;

type _Board = Expect<
  Equal<
    KanbanBoard<Task, Status>,
    { readonly todo: readonly Task[]; readonly done: readonly Task[] }
  >
>;
type _FromColumn = Expect<Equal<typeof move.from.columnId, Status>>;
type _SlotColumnId = Expect<Equal<typeof slot.column.id, Status>>;
type _Card = Expect<Equal<typeof slot.card, Task>>;

type Props<Card, ColumnId extends string> = Parameters<typeof Kanban<Card, ColumnId>>[0];
const columns: readonly KanbanColumnDefinition<Status>[] = [
  { id: "todo", title: "To do" },
  { id: "done", title: "Done", limit: 5 },
];
const props: Props<Task, Status> = {
  columns,
  modelValue: { todo: [], done: [] },
  getCardKey: (task) => task.id,
  canMove: (task, from, to) => task.id.length > 0 && from !== to,
  "onUpdate:modelValue": (board) => board.done,
};

const missingColumn: Props<Task, Status> = {
  columns,
  // @ts-expect-error every column needs a card list.
  modelValue: { todo: [] },
  getCardKey: (task) => task.id,
};
// @ts-expect-error column ids are checked against the board keys.
const badColumn: KanbanColumnDefinition<Status> = { id: "archived", title: "Archived" };

void [badColumn, missingColumn, props];
