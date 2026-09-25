import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h, ref } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import type { InteractionHandle } from "../../../testing/mount.ts";
import DataGrid from "./data-grid.vue";
import DataGridCell from "./data-grid-cell.vue";
import DataGridCellEditor from "./data-grid-cell-editor.vue";
import DataGridColumnHeader from "./data-grid-column-header.vue";
import DataGridResizeHandle from "./data-grid-resize-handle.vue";
import DataGridRow from "./data-grid-row.vue";
import { people, personColumns, tree } from "./data-grid-fixture.ts";
import type { Person, PersonColumn } from "./data-grid-fixture.ts";
import type { DataGridCellEditEvent, DataGridCellSlotProps } from "./data-grid-types.ts";
import {
  all,
  cell,
  click,
  columnIds,
  focused,
  header,
  one,
  press,
  rowIds,
  settle,
} from "./data-grid-test-utils.ts";

const handles: InteractionHandle[] = [];
afterEach(() => {
  for (const handle of handles.splice(0)) handle.unmount();
});

const PersonGrid = DataGrid<Person, PersonColumn>;

const Harness = defineComponent({
  props: {
    gridProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    rows: { type: Array as PropType<readonly Person[]>, default: () => people },
    log: { type: Array as PropType<string[]>, default: () => [] },
  },
  setup(props) {
    const rows = ref<readonly Person[]>(props.rows);
    return () =>
      h("div", [
        h(
          PersonGrid,
          {
            id: "people",
            ariaLabel: "People",
            rows: rows.value,
            columns: personColumns,
            getRowId: (row: Person) => row.id,
            ...props.gridProps,
            "onCell-edit": (event: DataGridCellEditEvent<Person>) => {
              props.log.push(`edit:${event.rowId}:${event.columnId}:${String(event.value)}`);
              rows.value = rows.value.map((row) =>
                row.id === event.rowId ? { ...row, [event.columnId]: event.value } : row,
              );
            },
            "onUpdate:selection": (value: readonly string[]) =>
              props.log.push(`selection:${value.join(",")}`),
            "onUpdate:sorting": (value: readonly { columnId: string; direction: string }[]) =>
              props.log.push(
                `sorting:${value.map((s) => `${s.columnId}-${s.direction}`).join(",")}`,
              ),
          },
          {
            cell: (slot: DataGridCellSlotProps<Person, PersonColumn>) =>
              slot.columnId === "actions"
                ? h("button", { type: "button", tabindex: -1 }, `Edit ${slot.row.name}`)
                : slot.columnId === "age"
                  ? `${slot.value} yrs`
                  : slot.text,
            empty: () => "No people",
          },
        ),
      ]);
  },
});

function mountGrid(props: Record<string, unknown> = {}, log: string[] = []): InteractionHandle {
  const handle = mountInteraction(Harness, { props: { gridProps: props, log } });
  handles.push(handle);
  return handle;
}

test("renders APG grid roles, counts, indices, and typed cell slots", async () => {
  mountGrid();
  await settle();
  const grid = one('[data-vize-ui="data-grid"]');
  assert.equal(grid.getAttribute("role"), "grid");
  assert.equal(grid.getAttribute("aria-label"), "People");
  assert.equal(grid.getAttribute("aria-rowcount"), "5");
  assert.equal(grid.getAttribute("aria-colcount"), "4");
  assert.deepEqual(columnIds(), ["name", "age", "team", "actions"]);
  assert.equal(header("name").textContent?.trim(), "Name");
  assert.equal(header("age").getAttribute("aria-sort"), "none");
  assert.equal(header("actions").getAttribute("aria-sort"), null, "display columns never sort");
  assert.equal(
    header("name").getAttribute("tabindex"),
    "0",
    "first header is the initial tab stop",
  );
  assert.deepEqual(rowIds(), ["ada", "grace", "alan", "linus"]);
  assert.equal(one('[data-row-id="grace"][role="row"]').getAttribute("aria-rowindex"), "3");
  assert.equal(cell("ada", "age").textContent, "36 yrs");
  assert.equal(cell("ada", "team").textContent, "Core");
  assert.equal(cell("alan", "team").textContent, "", "null path segments render empty");
  assert.equal(cell("ada", "actions").textContent, "Edit Ada");
  assert.equal(cell("ada", "age").getAttribute("aria-colindex"), "2");
  assert.equal(cell("ada", "team").getAttribute("aria-readonly"), "true");
  assert.equal(cell("ada", "name").getAttribute("aria-readonly"), null);
});

test("header clicks sort ascending, descending, then off; Shift adds sort keys", async () => {
  const log: string[] = [];
  mountGrid({}, log);
  await settle();
  await click(header("age"));
  assert.deepEqual(rowIds(), ["ada", "linus", "alan", "grace"]);
  assert.equal(header("age").getAttribute("aria-sort"), "ascending");
  await click(header("name"), { shiftKey: true });
  assert.deepEqual(rowIds(), ["ada", "linus", "alan", "grace"]);
  assert.equal(header("name").getAttribute("data-sort-index"), "2");
  await click(header("age"));
  assert.equal(header("age").getAttribute("aria-sort"), "descending");
  assert.deepEqual(rowIds(), ["grace", "alan", "ada", "linus"]);
  await click(header("age"));
  assert.deepEqual(rowIds(), ["ada", "grace", "alan", "linus"]);
  assert.deepEqual(log, [
    "sorting:age-ascending",
    "sorting:age-ascending,name-ascending",
    "sorting:age-descending",
    "sorting:",
  ]);
});

test("missing values sort last in both directions", async () => {
  mountGrid();
  await settle();
  await click(header("team"));
  assert.equal(rowIds().at(-1), "alan");
  await click(header("team"));
  assert.equal(rowIds().at(-1), "alan");
});

test("global, column, and custom filters narrow rows and show the empty slot", async () => {
  const handle = mountGrid({ globalFilter: "core" });
  await settle();
  assert.deepEqual(rowIds(), ["ada", "linus"]);
  await handle.wrapper.setProps({ gridProps: { globalFilter: "", columnFilters: { age: 40 } } });
  await settle();
  assert.deepEqual(rowIds(), ["grace", "alan"]);
  await handle.wrapper.setProps({
    gridProps: { columnFilters: {}, filterRow: (row: Person) => row.name.startsWith("A") },
  });
  await settle();
  assert.deepEqual(rowIds(), ["ada", "alan"]);
  await handle.wrapper.setProps({ gridProps: { globalFilter: "nobody" } });
  await settle();
  assert.deepEqual(rowIds(), []);
  assert.equal(one('[data-vize-ui="data-grid-empty"]').textContent, "No people");
  assert.equal(one('[role="grid"]').getAttribute("data-empty"), "true");
});

test("arrow, Home/End, Ctrl+Home/End, and Page keys move a single roving tab stop", async () => {
  mountGrid({ pageSize: 2 });
  await settle();
  header("name").focus();
  await settle();
  const moves: Array<[string, Partial<KeyboardEventInit>, string]> = [
    ["ArrowDown", {}, "ada:name"],
    ["ArrowRight", {}, "ada:age"],
    ["End", {}, "ada:actions"],
    ["ArrowRight", {}, "ada:actions"],
    ["Home", {}, "ada:name"],
    ["PageDown", {}, "alan:name"],
    ["End", { ctrlKey: true }, "linus:actions"],
    ["PageUp", {}, "grace:actions"],
    ["Home", { ctrlKey: true }, "header:name"],
    ["ArrowUp", {}, "header:name"],
  ];
  for (const [key, init, expected] of moves) {
    const event = await press(key, init);
    assert.equal(event.defaultPrevented, true);
    assert.equal(focused(), expected, `${key} should focus ${expected}`);
    assert.equal(all('[tabindex="0"]').length, 1);
  }
});

test("rtl flips horizontal cell movement", async () => {
  mountGrid({ dir: "rtl" });
  await settle();
  cell("ada", "age").focus();
  await settle();
  await press("ArrowLeft");
  assert.equal(focused(), "ada:team");
  await press("ArrowRight");
  assert.equal(focused(), "ada:age");
});

test("header keys sort, reorder with Ctrl+Shift+arrows, and resize with Alt+arrows", async () => {
  mountGrid();
  await settle();
  header("age").focus();
  await settle();
  await press("Enter");
  assert.equal(header("age").getAttribute("aria-sort"), "ascending");
  await press(" ");
  assert.equal(header("age").getAttribute("aria-sort"), "descending");
  await press("ArrowRight", { ctrlKey: true, shiftKey: true });
  assert.deepEqual(columnIds(), ["name", "team", "age", "actions"]);
  assert.equal(focused(), "header:age", "focus follows the moved column");
  const before = header("age").style.getPropertyValue("--vize-ui-data-grid-column-width");
  await press("ArrowRight", { altKey: true });
  assert.equal(before, "150px");
  assert.equal(header("age").style.getPropertyValue("--vize-ui-data-grid-column-width"), "160px");
});

test("resize handles are labelled separators driven by pointer and arrows within limits", async () => {
  mountGrid();
  await settle();
  const handle = one('[data-vize-ui="data-grid-resize-handle"]', header("name"));
  assert.equal(handle.getAttribute("role"), "separator");
  assert.equal(handle.getAttribute("aria-label"), "Resize Name");
  assert.equal(handle.getAttribute("aria-valuenow"), "150");
  assert.equal(handle.getAttribute("aria-valuemin"), "40");
  assert.equal(
    one('[data-vize-ui="data-grid-resize-handle"]', header("team")).getAttribute("aria-label"),
    "Resize Team",
  );
  assert.equal(header("actions").querySelector('[data-vize-ui="data-grid-resize-handle"]'), null);
  handle.dispatchEvent(
    new PointerEvent("pointerdown", { bubbles: true, button: 0, clientX: 100, pointerId: 1 }),
  );
  handle.dispatchEvent(
    new PointerEvent("pointermove", { bubbles: true, clientX: 160, pointerId: 1 }),
  );
  handle.dispatchEvent(
    new PointerEvent("pointerup", { bubbles: true, clientX: 160, pointerId: 1 }),
  );
  await settle();
  const resized = one('[data-vize-ui="data-grid-resize-handle"]', header("name"));
  assert.equal(resized.getAttribute("aria-valuenow"), "210");
  assert.equal(
    cell("ada", "name").style.getPropertyValue("--vize-ui-data-grid-column-width"),
    "210px",
  );
  resized.dispatchEvent(
    new PointerEvent("pointerdown", { bubbles: true, button: 0, clientX: 300, pointerId: 1 }),
  );
  resized.dispatchEvent(
    new PointerEvent("pointermove", { bubbles: true, clientX: -500, pointerId: 1 }),
  );
  await settle();
  assert.equal(resized.getAttribute("aria-valuenow"), "40", "widths clamp to minWidth");
  resized.dispatchEvent(
    new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true }),
  );
  await settle();
  assert.equal(resized.getAttribute("aria-valuenow"), "50");
});

test("visibility, order, and pinning props lay out columns with sticky offsets", async () => {
  mountGrid({
    columnVisibility: { team: false },
    columnOrder: ["age", "name", "team", "actions"],
    columnPinning: { start: ["name"], end: ["actions"] },
    columnSizing: { name: 120 },
  });
  await settle();
  assert.deepEqual(columnIds(), ["name", "age", "actions"]);
  assert.equal(header("name").getAttribute("data-pin"), "start");
  assert.equal(header("actions").getAttribute("data-pin"), "end");
  assert.equal(header("age").getAttribute("data-pin"), null);
  assert.equal(header("name").style.getPropertyValue("--vize-ui-data-grid-pin-offset"), "0px");
  assert.equal(
    one('[role="grid"]').style.getPropertyValue("--vize-ui-data-grid-total-width"),
    "420px",
  );
  assert.equal(one('[role="grid"]').getAttribute("aria-colcount"), "3");
});

test("multiple selection: click replaces, Ctrl toggles, Shift extends, Space and Ctrl+A", async () => {
  const log: string[] = [];
  mountGrid({ selectionMode: "multiple" }, log);
  await settle();
  const grid = one('[role="grid"]');
  assert.equal(grid.getAttribute("aria-multiselectable"), "true");
  assert.equal(one('[data-row-id="ada"][role="row"]').getAttribute("aria-selected"), "false");
  await click(cell("ada", "name"));
  await click(cell("alan", "name"), { ctrlKey: true });
  await click(cell("linus", "name"), { shiftKey: true });
  assert.deepEqual(log.at(-1), "selection:ada,alan,linus");
  assert.equal(one('[data-row-id="linus"][role="row"]').getAttribute("aria-selected"), "true");
  await click(cell("grace", "name"));
  assert.equal(log.at(-1), "selection:grace");
  await press(" ");
  assert.equal(log.at(-1), "selection:");
  await press("ArrowDown", { shiftKey: true });
  assert.equal(log.at(-1), "selection:grace,alan");
  await press("a", { ctrlKey: true });
  assert.equal(log.at(-1), "selection:ada,grace,alan,linus");
});

test("single selection keeps one row and none mode publishes no aria-selected", async () => {
  const log: string[] = [];
  mountGrid({ selectionMode: "single" }, log);
  await settle();
  await click(cell("ada", "name"));
  await click(cell("grace", "name"), { ctrlKey: true });
  assert.equal(log.at(-1), "selection:grace");
  assert.equal(one('[role="grid"]').getAttribute("aria-multiselectable"), null);
  handles.splice(0).forEach((handle) => handle.unmount());
  mountGrid();
  await settle();
  assert.equal(one('[data-row-id="ada"][role="row"]').getAttribute("aria-selected"), null);
});

test("Enter or F2 edits, Enter commits through parse and validate, Escape cancels", async () => {
  const log: string[] = [];
  mountGrid({}, log);
  await settle();
  cell("ada", "name").focus();
  await settle();
  await press("Enter");
  const input = one('[data-vize-ui="data-grid-cell-editor"]');
  assert.ok(input instanceof HTMLInputElement);
  assert.ok(document.activeElement === input, "the editor takes focus");
  assert.equal(input.value, "Ada");
  assert.equal(input.getAttribute("aria-label"), "Name");
  assert.equal(cell("ada", "name").getAttribute("data-editing"), "true");
  input.value = "  ";
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await press("Enter");
  assert.equal(input.getAttribute("aria-invalid"), "true", "validation keeps the editor open");
  assert.equal(input.dataset.error, "Name is required");
  input.value = "Ada L.";
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await press("Enter");
  assert.deepEqual(log, ["edit:ada:name:Ada L."]);
  assert.equal(cell("ada", "name").textContent, "Ada L.");
  assert.equal(focused(), "ada:name", "focus returns to the cell");

  cell("ada", "age").focus();
  await settle();
  await press("F2");
  const ageInput = one('[data-vize-ui="data-grid-cell-editor"]');
  assert.ok(ageInput instanceof HTMLInputElement);
  ageInput.value = "37";
  ageInput.dispatchEvent(new Event("input", { bubbles: true }));
  await press("Escape");
  assert.equal(document.querySelector('[data-vize-ui="data-grid-cell-editor"]'), null);
  assert.equal(focused(), "ada:age");
  cell("ada", "age").dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
  await settle();
  const again = one('[data-vize-ui="data-grid-cell-editor"]');
  assert.ok(again instanceof HTMLInputElement);
  again.value = "37";
  again.dispatchEvent(new Event("input", { bubbles: true }));
  await press("Tab");
  assert.equal(log.at(-1), "edit:ada:age:37", "parse converts editor text");
});

test("read-only cells never enter edit mode", async () => {
  mountGrid();
  await settle();
  cell("ada", "team").focus();
  await settle();
  const event = await press("Enter");
  assert.equal(event.defaultPrevented, false);
  assert.equal(document.querySelector('[data-vize-ui="data-grid-cell-editor"]'), null);
});

test("tree rows render a treegrid with levels; arrows expand, collapse, and climb", async () => {
  const handle = mountInteraction(Harness, {
    props: { rows: tree, gridProps: { getSubRows: (row: Person) => row.reports } },
  });
  handles.push(handle);
  await settle();
  const grid = one('[data-vize-ui="data-grid"]');
  assert.equal(grid.getAttribute("role"), "treegrid");
  const ceo = one('[data-row-id="ceo"][role="row"]');
  assert.equal(ceo.getAttribute("aria-level"), "1");
  assert.equal(ceo.getAttribute("aria-expanded"), "false");
  assert.equal(one('[data-row-id="solo"][role="row"]').getAttribute("aria-expanded"), null);
  cell("ceo", "name").focus();
  await settle();
  await press("ArrowRight");
  assert.deepEqual(rowIds(), ["ceo", "cto", "cfo", "solo"]);
  assert.equal(one('[data-row-id="cto"][role="row"]').getAttribute("aria-level"), "2");
  assert.equal(focused(), "ceo:name", "expanding keeps focus");
  await press("ArrowDown");
  await press("ArrowLeft");
  assert.equal(focused(), "ceo:name", "ArrowLeft on a child moves to its parent");
  await press("ArrowLeft");
  assert.deepEqual(rowIds(), ["ceo", "solo"]);
  await handle.wrapper.setProps({
    gridProps: {
      getSubRows: (row: Person) => row.reports,
      globalFilter: "money",
      expanded: ["ceo"],
    },
  });
  await settle();
  assert.deepEqual(rowIds(), ["ceo", "cfo"], "filters keep ancestors of matching rows");
});

test("virtualization renders a window with absolute row indices and scrolls to focus", async () => {
  const many: Person[] = Array.from({ length: 200 }, (_, index) => ({
    id: `p${index}`,
    name: `Person ${index}`,
    age: index,
    team: null,
  }));
  const handle = mountInteraction(Harness, {
    props: {
      rows: many,
      gridProps: { virtualize: true, rowHeight: 20, initialViewportHeight: 100, overscan: 1 },
    },
  });
  handles.push(handle);
  await settle();
  const grid = one('[data-vize-ui="data-grid"]');
  assert.equal(grid.getAttribute("aria-rowcount"), "201");
  assert.equal(grid.getAttribute("data-virtualized"), "true");
  const rendered = rowIds();
  assert.ok(rendered.length < 20, `renders a window, got ${rendered.length}`);
  assert.equal(
    one('[data-vize-ui="data-grid-body"]').style.getPropertyValue(
      "--vize-ui-data-grid-body-height",
    ),
    "4000px",
  );
  const second = one('[data-row-id="p1"][role="row"]');
  assert.equal(second.getAttribute("aria-rowindex"), "3");
  assert.equal(second.style.getPropertyValue("--vize-ui-data-grid-row-start"), "20px");
  cell("p0", "name").focus();
  await settle();
  await press("End", { ctrlKey: true });
  await settle();
  assert.equal(focused(), "p199:actions");
});

test("exported parts stay importable for custom layouts", () => {
  for (const part of [
    DataGridCell,
    DataGridCellEditor,
    DataGridColumnHeader,
    DataGridResizeHandle,
    DataGridRow,
  ]) {
    assert.ok(part);
  }
});
