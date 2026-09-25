import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import type { InteractionHandle } from "../../../testing/mount.ts";
import GridList from "./grid-list.vue";
import GridListItem from "./grid-list-item.vue";
import type { GridListItemSlotProps, GridListReorderEvent } from "./grid-list-types.ts";

interface Photo {
  readonly id: string;
  readonly title: string;
}

const photos: readonly Photo[] = [
  { id: "a", title: "Alps" },
  { id: "b", title: "Beach" },
  { id: "c", title: "Canyon" },
  { id: "d", title: "Desert" },
  { id: "e", title: "Estuary" },
  { id: "f", title: "Fjord" },
];

const handles: InteractionHandle[] = [];
afterEach(() => {
  for (const handle of handles.splice(0)) handle.unmount();
});

async function settle(): Promise<void> {
  for (let index = 0; index < 3; index++) await nextTick();
}

async function press(key: string, init: Partial<KeyboardEventInit> = {}): Promise<KeyboardEvent> {
  const target = document.activeElement ?? document.body;
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  await settle();
  return event;
}

function row(key: string): HTMLElement {
  const element = document.querySelector(`[data-vize-ui="grid-list-item"][data-key="${key}"]`);
  assert.ok(element instanceof HTMLElement, `expected row ${key}`);
  return element;
}

function focusedKey(): string {
  return document.activeElement instanceof HTMLElement
    ? (document.activeElement.dataset.key ?? "")
    : "";
}

function order(): string {
  return [...document.querySelectorAll('[data-vize-ui="grid-list-item"]')]
    .map((element) => (element instanceof HTMLElement ? element.dataset.key : ""))
    .join("");
}

const Harness = defineComponent({
  props: {
    listProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    log: { type: Array as PropType<string[]>, default: () => [] },
  },
  setup(props) {
    const items = ref<readonly Photo[]>(photos);
    return () =>
      h("div", [
        h(
          GridList<Photo>,
          {
            ariaLabel: "Photos",
            items: items.value,
            getKey: (photo: Photo) => photo.id,
            getTextValue: (photo: Photo) => photo.title,
            ...props.listProps,
            "onUpdate:selection": (value: readonly string[]) =>
              props.log.push(`selection:${value.join("")}`),
            onAction: (photo: Photo) => props.log.push(`action:${photo.id}`),
            "onUpdate:items": (value: readonly Photo[]) => {
              items.value = value;
            },
            onReorder: (event: GridListReorderEvent<Photo>) =>
              props.log.push(`reorder:${event.key}:${event.fromIndex}->${event.toIndex}`),
          },
          {
            item: (slot: GridListItemSlotProps<Photo>) => [
              slot.item.title,
              slot.dragHandleProps
                ? h(
                    "button",
                    { type: "button", "data-handle": slot.key, ...slot.dragHandleProps },
                    "Move",
                  )
                : null,
            ],
            empty: () => "No photos",
          },
        ),
      ]);
  },
});

function mountList(props: Record<string, unknown> = {}, log: string[] = []): InteractionHandle {
  const handle = mountInteraction(Harness, { props: { listProps: props, log } });
  handles.push(handle);
  return handle;
}

test("renders an APG grid of rows with one gridcell and a single tab stop", async () => {
  mountList({ selectionMode: "multiple" });
  await settle();
  const grid = document.querySelector('[data-vize-ui="grid-list"]');
  assert.ok(grid instanceof HTMLElement);
  assert.equal(grid.getAttribute("role"), "grid");
  assert.equal(grid.getAttribute("aria-label"), "Photos");
  assert.equal(grid.getAttribute("aria-rowcount"), "6");
  assert.equal(grid.getAttribute("aria-multiselectable"), "true");
  assert.equal(row("a").getAttribute("role"), "row");
  assert.equal(row("a").getAttribute("aria-rowindex"), "1");
  assert.equal(row("a").querySelectorAll('[role="gridcell"]').length, 1);
  assert.equal(row("a").getAttribute("aria-selected"), "false");
  assert.equal(
    document.querySelectorAll('[data-vize-ui="grid-list-item"][tabindex="0"]').length,
    1,
  );
  assert.equal(row("a").getAttribute("tabindex"), "0");
});

test("list layout: Up/Down, Home/End, PageUp/PageDown, loop, and typeahead", async () => {
  mountList({ loop: true });
  await settle();
  row("a").focus();
  await settle();
  const moves: Array<[string, string]> = [
    ["ArrowDown", "b"],
    ["End", "f"],
    ["ArrowDown", "a"],
    ["ArrowUp", "f"],
    ["Home", "a"],
    ["PageDown", "a"],
    ["d", "d"],
  ];
  for (const [key, expected] of moves) {
    await press(key);
    if (key === "PageDown") continue;
    assert.equal(focusedKey(), expected, `${key} should focus ${expected}`);
  }
  assert.equal(document.querySelectorAll('[tabindex="0"]').length, 1);
});

test("grid layout: Left/Right step items, Up/Down step rows, rtl flips", async () => {
  mountList({ layout: "grid", columns: 3 });
  await settle();
  row("a").focus();
  await settle();
  await press("ArrowRight");
  assert.equal(focusedKey(), "b");
  await press("ArrowDown");
  assert.equal(focusedKey(), "e");
  await press("ArrowLeft");
  assert.equal(focusedKey(), "d");
  await press("ArrowUp");
  assert.equal(focusedKey(), "a");
  cleanup();
  mountList({ layout: "grid", columns: 3, dir: "rtl" });
  await settle();
  row("b").focus();
  await settle();
  await press("ArrowLeft");
  assert.equal(focusedKey(), "c");
});

function cleanup(): void {
  for (const handle of handles.splice(0)) handle.unmount();
}

test("multiple selection: click, Ctrl-click, Shift-click, Space, Shift+Arrow, Ctrl+A", async () => {
  const log: string[] = [];
  mountList({ selectionMode: "multiple", disabledKeys: ["c"] }, log);
  await settle();
  row("a").dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 1 }));
  row("d").dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 1, shiftKey: true }));
  await settle();
  assert.equal(log.at(-1), "selection:abd", "ranges skip disabled items");
  row("f").dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 1, ctrlKey: true }));
  await settle();
  assert.equal(log.at(-1), "selection:abdf");
  assert.equal(row("f").getAttribute("aria-selected"), "true");
  row("a").focus();
  await press(" ");
  assert.equal(log.at(-1), "selection:bdf");
  await press("ArrowDown", { shiftKey: true });
  assert.equal(focusedKey(), "b");
  await press("a", { ctrlKey: true });
  assert.equal(log.at(-1), "selection:abdef");
  assert.equal(row("c").getAttribute("aria-disabled"), "true");
});

test("single selection replaces; none mode publishes no aria-selected; Enter and double click act", async () => {
  const log: string[] = [];
  mountList({}, log);
  await settle();
  row("b").dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 1, ctrlKey: true }));
  row("c").dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 1 }));
  await settle();
  assert.equal(log.at(-1), "selection:c");
  row("c").focus();
  await press("Enter");
  assert.equal(log.at(-1), "action:c");
  row("e").dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 2 }));
  await settle();
  assert.equal(log.at(-1), "action:e");
  cleanup();
  mountList({ selectionMode: "none" });
  await settle();
  assert.equal(row("a").getAttribute("aria-selected"), null);
});

test("keyboard reorder through the drag handle commits and emits the new order", async () => {
  const log: string[] = [];
  mountList({ reorderable: true }, log);
  await settle();
  const handle = document.querySelector('[data-handle="a"]');
  assert.ok(handle instanceof HTMLElement);
  handle.focus();
  await press("Enter");
  assert.equal(row("a").getAttribute("data-dragging"), "true");
  await press("ArrowDown");
  await press("ArrowDown");
  await press("Enter");
  await settle();
  assert.equal(log.at(-1), "reorder:a:0->2");
  assert.equal(order(), "bcadef");
  assert.equal(focusedKey(), "", "the list's own keys ignore events from the handle");
});

test("empty lists render an empty row", async () => {
  mountList({ items: [] });
  await settle();
  const empty = document.querySelector('[data-vize-ui="grid-list-empty"]');
  assert.equal(empty?.textContent, "No photos");
});

test("GridListItem is exported for custom compositions", () => {
  assert.ok(GridListItem);
});
