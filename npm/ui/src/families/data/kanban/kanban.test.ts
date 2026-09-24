import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import type { InteractionHandle } from "../../../testing/mount.ts";
import Kanban from "./kanban.vue";
import KanbanCard from "./kanban-card.vue";
import KanbanColumnPart from "./kanban-column.vue";
import type {
  KanbanBoard,
  KanbanCardSlotProps,
  KanbanColumn,
  KanbanMoveEvent,
} from "./kanban-types.ts";

type Status = "todo" | "doing" | "done";

interface Task {
  readonly id: string;
  readonly title: string;
}

const columns: readonly KanbanColumn<Status>[] = [
  { id: "todo", title: "To do" },
  { id: "doing", title: "Doing", limit: 2 },
  { id: "done", title: "Done" },
];

const initial: KanbanBoard<Task, Status> = {
  todo: [
    { id: "t1", title: "Write spec" },
    { id: "t2", title: "Review" },
  ],
  doing: [{ id: "t3", title: "Build" }],
  done: [],
};

const handles: InteractionHandle[] = [];
afterEach(() => {
  for (const handle of handles.splice(0)) handle.unmount();
});

async function settle(): Promise<void> {
  for (let index = 0; index < 4; index++) await nextTick();
}

async function press(key: string, init: Partial<KeyboardEventInit> = {}): Promise<void> {
  const target = document.activeElement ?? document.body;
  target.dispatchEvent(
    new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init }),
  );
  await settle();
}

function card(key: string): HTMLElement {
  const element = document.querySelector(`[data-card-key="${key}"]`);
  assert.ok(element instanceof HTMLElement, `expected card ${key}`);
  return element;
}

function focusedCard(): string {
  return document.activeElement instanceof HTMLElement
    ? (document.activeElement.dataset.cardKey ?? "")
    : "";
}

function layout(): string {
  return [...document.querySelectorAll('[data-vize-ui="kanban-column"]')]
    .map((column) =>
      column instanceof HTMLElement
        ? `${column.dataset.columnId}:${[...column.querySelectorAll("[data-card-key]")]
            .map((element) => (element instanceof HTMLElement ? element.dataset.cardKey : ""))
            .join(",")}`
        : "",
    )
    .join(" ");
}

const TaskBoard = Kanban<Task, Status>;

const Harness = defineComponent({
  props: {
    boardProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    log: { type: Array as PropType<string[]>, default: () => [] },
  },
  setup(props) {
    const board = ref<KanbanBoard<Task, Status>>(initial);
    return () =>
      h("div", [
        h(
          TaskBoard,
          {
            ariaLabel: "Sprint",
            columns,
            modelValue: board.value,
            getCardKey: (task: Task) => task.id,
            getCardLabel: (task: Task) => task.title,
            ...props.boardProps,
            "onUpdate:modelValue": (value: KanbanBoard<Task, Status>) => {
              board.value = value;
            },
            onMove: (event: KanbanMoveEvent<Task, Status>) =>
              props.log.push(
                `${event.cardKey}:${event.from.columnId}${event.from.index}->${event.to.columnId}${event.to.index}`,
              ),
          },
          {
            card: (slot: KanbanCardSlotProps<Task, Status>) => slot.card.title,
            empty: () => "Nothing here",
          },
        ),
      ]);
  },
});

function mountBoard(props: Record<string, unknown> = {}, log: string[] = []): InteractionHandle {
  const handle = mountInteraction(Harness, { props: { boardProps: props, log } });
  handles.push(handle);
  return handle;
}

test("renders labelled column groups with lists of focusable cards and instructions", async () => {
  mountBoard();
  await settle();
  const board = document.querySelector('[data-vize-ui="kanban"]');
  assert.ok(board instanceof HTMLElement);
  assert.equal(board.getAttribute("aria-label"), "Sprint");
  assert.equal(board.getAttribute("aria-roledescription"), "board");
  const column = document.querySelector('[data-column-id="doing"][data-vize-ui="kanban-column"]');
  assert.ok(column instanceof HTMLElement);
  assert.equal(column.getAttribute("role"), "group");
  const heading = document.getElementById(column.getAttribute("aria-labelledby") ?? "");
  assert.equal(heading?.textContent, "Doing");
  assert.equal(column.dataset.limit, "2");
  assert.equal(card("t1").getAttribute("role"), "listitem");
  assert.equal(card("t2").getAttribute("aria-posinset"), "2");
  const instructions = document.getElementById(card("t1").getAttribute("aria-describedby") ?? "");
  assert.match(instructions?.textContent ?? "", /Space or Enter to pick up/);
  assert.equal(document.querySelectorAll('[data-vize-ui="kanban-card"][tabindex="0"]').length, 1);
  assert.equal(card("t1").getAttribute("tabindex"), "0");
  assert.equal(
    document.querySelector('[data-column-id="done"] [data-vize-ui="kanban-empty"]')?.textContent,
    "Nothing here",
  );
});

test("arrow keys move the roving focus within and across columns", async () => {
  mountBoard();
  await settle();
  card("t1").focus();
  await settle();
  await press("ArrowDown");
  assert.equal(focusedCard(), "t2");
  await press("ArrowRight");
  assert.equal(focusedCard(), "t3", "the index clamps to the shorter column");
  await press("ArrowRight");
  assert.equal(focusedCard(), "t3", "empty columns are skipped at the edge");
  await press("ArrowLeft");
  assert.equal(focusedCard(), "t1");
  await press("End");
  assert.equal(focusedCard(), "t2");
  assert.equal(card("t2").getAttribute("tabindex"), "0");
});

test("keyboard drag moves a card into another column and keeps focus on it", async () => {
  const log: string[] = [];
  mountBoard({}, log);
  await settle();
  card("t1").focus();
  await settle();
  await press(" ");
  assert.equal(card("t1").getAttribute("data-dragging"), "true");
  // Targets cycle in document order: before t2, end of To do, before t3, end of Doing, ...
  for (let step = 0; step < 3; step++) await press("ArrowDown");
  await press("Enter");
  await settle();
  assert.deepEqual(log, ["t1:todo0->doing1"]);
  assert.equal(layout(), "todo:t2 doing:t3,t1 done:");
  assert.equal(focusedCard(), "t1");
});

test("Escape cancels a keyboard drag without moving", async () => {
  const log: string[] = [];
  mountBoard({}, log);
  await settle();
  card("t2").focus();
  await press("Enter");
  await press("ArrowDown");
  await press("Escape");
  assert.deepEqual(log, []);
  assert.equal(layout(), "todo:t1,t2 doing:t3 done:");
});

test("WIP limits, disabled columns, and canMove veto drop targets", async () => {
  const log: string[] = [];
  mountBoard(
    {
      columns: [
        { id: "todo", title: "To do" },
        { id: "doing", title: "Doing", limit: 1 },
        { id: "done", title: "Done", disabled: true },
      ],
      canMove: (task: Task, from: Status, to: Status) => !(task.id === "t2" && to !== from),
    },
    log,
  );
  await settle();
  card("t1").focus();
  await press(" ");
  // Doing is full and Done is disabled, so keyboard targets never enter them.
  const visited = new Set<string>();
  for (let step = 0; step < 6; step++) {
    await press("ArrowDown");
    for (const element of document.querySelectorAll("[data-drop-edge], [data-drop-target]")) {
      const owner = element.closest('[data-vize-ui="kanban-column"]');
      if (owner instanceof HTMLElement) visited.add(owner.dataset.columnId ?? "");
    }
  }
  assert.deepEqual([...visited], ["todo"]);
  await press("Escape");
  card("t2").focus();
  await press(" ");
  await press("ArrowDown");
  assert.ok(
    document.querySelector('[data-column-id="doing"] [data-drop-edge]') === null,
    "canMove vetoes cross-column moves for t2",
  );
  await press("Escape");
  assert.deepEqual(log, []);
});

test("dropping on a card's bottom edge inserts after it", async () => {
  const log: string[] = [];
  mountBoard({}, log);
  await settle();
  const t1 = card("t1");
  const t3 = card("t3");
  for (const [element, top] of [
    [t1, 0],
    [t3, 100],
  ] as const) {
    Object.defineProperty(element, "getBoundingClientRect", {
      configurable: true,
      value: () => new DOMRect(0, top, 100, 40),
    });
  }
  t1.dispatchEvent(
    new PointerEvent("pointerdown", {
      bubbles: true,
      button: 0,
      clientX: 10,
      clientY: 10,
      pointerId: 1,
      isPrimary: true,
      pointerType: "mouse",
    }),
  );
  document.dispatchEvent(
    new PointerEvent("pointermove", {
      bubbles: true,
      clientX: 10,
      clientY: 135,
      pointerId: 1,
      isPrimary: true,
      pointerType: "mouse",
    }),
  );
  document.dispatchEvent(
    new PointerEvent("pointermove", {
      bubbles: true,
      clientX: 11,
      clientY: 136,
      pointerId: 1,
      isPrimary: true,
      pointerType: "mouse",
    }),
  );
  await settle();
  assert.equal(card("t3").dataset.dropEdge, "bottom");
  document.dispatchEvent(
    new PointerEvent("pointerup", {
      bubbles: true,
      clientX: 11,
      clientY: 136,
      pointerId: 1,
      isPrimary: true,
      pointerType: "mouse",
    }),
  );
  await settle();
  assert.deepEqual(log, ["t1:todo0->doing1"]);
});

test("column and card parts are exported for custom boards", () => {
  assert.ok(KanbanCard);
  assert.ok(KanbanColumnPart);
});
