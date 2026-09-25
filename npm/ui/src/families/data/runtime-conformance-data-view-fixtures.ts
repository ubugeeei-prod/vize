import assert from "node:assert/strict";

import { h } from "vue";
import type { VNode } from "vue";

import type { RuntimeFixture } from "../../conformance/runtime-conformance-fixtures.ts";
import DataGrid from "./data-grid/data-grid.vue";
import DataGridCell from "./data-grid/data-grid-cell.vue";
import DataGridCellEditor from "./data-grid/data-grid-cell-editor.vue";
import DataGridColumnHeader from "./data-grid/data-grid-column-header.vue";
import DataGridResizeHandle from "./data-grid/data-grid-resize-handle.vue";
import DataGridRow from "./data-grid/data-grid-row.vue";
import { createColumnHelper } from "./data-grid/data-grid-columns.ts";
import GridList from "./grid-list/grid-list.vue";
import GridListItem from "./grid-list/grid-list-item.vue";
import Kanban from "./kanban/kanban.vue";
import KanbanCard from "./kanban/kanban-card.vue";
import KanbanColumn from "./kanban/kanban-column.vue";

interface Row {
  readonly id: string;
  readonly name: string;
}

const column = createColumnHelper<Row>();
const columns = [column.accessor("name", { header: "Name" }), column.display("actions")];

function grid(): VNode {
  return h("div", [
    h(DataGrid<Row, (typeof columns)[number]>, {
      id: "runtime-grid",
      ariaLabel: "Rows",
      rows: [{ id: "a", name: "Ada" }],
      columns,
      getRowId: (row: Row) => row.id,
    }),
  ]);
}

function gridFixture(
  name: string,
  file: string,
  check: (host: HTMLElement) => void,
): RuntimeFixture {
  return {
    name,
    sourceFile: `families/data/data-grid/${file}`,
    render: grid,
    assertServerMarkup(html) {
      assert.match(html, /role="grid"/);
      assert.match(html, new RegExp(`data-vize-ui="${name}"`));
    },
    assertHydratedDom: check,
  };
}

const editorProbe: RuntimeFixture = {
  name: "data-grid-cell-editor",
  sourceFile: "families/data/data-grid/data-grid-cell-editor.vue",
  render: grid,
  assertServerMarkup(html) {
    assert.match(html, /aria-readonly="true"/);
  },
  assertHydratedDom(host) {
    assert.equal(host.querySelector('[data-vize-ui="data-grid-cell-editor"]'), null);
  },
};

export const dataViewRuntimeFixtures: readonly RuntimeFixture[] = [
  gridFixture("data-grid", "data-grid.vue", (host) => {
    assert.equal(host.querySelector('[role="grid"]')?.id, "runtime-grid");
  }),
  gridFixture("data-grid-cell", "data-grid-cell.vue", (host) => {
    assert.equal(host.querySelectorAll('[role="gridcell"]').length, 2);
  }),
  editorProbe,
  gridFixture("data-grid-column-header", "data-grid-column-header.vue", (host) => {
    assert.equal(host.querySelector('[role="columnheader"]')?.getAttribute("tabindex"), "0");
  }),
  gridFixture("data-grid-resize-handle", "data-grid-resize-handle.vue", (host) => {
    assert.equal(
      host.querySelector('[role="separator"]')?.getAttribute("aria-label"),
      "Resize Name",
    );
  }),
  gridFixture("data-grid-row", "data-grid-row.vue", (host) => {
    assert.equal(
      host.querySelector('[data-vize-ui="data-grid-row"]')?.getAttribute("aria-rowindex"),
      "2",
    );
  }),
  ...["grid-list.vue", "grid-list-item.vue"].map((file): RuntimeFixture => ({
    name: file.replace(".vue", ""),
    sourceFile: `families/data/grid-list/${file}`,
    render: () =>
      h("div", [
        h(GridList<string>, {
          id: "runtime-list",
          ariaLabel: "Items",
          items: ["one", "two"],
          getKey: (item: string) => item,
          defaultSelection: ["two"],
        }),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /role="grid"/);
      assert.match(html, /aria-selected="true"/);
    },
    assertHydratedDom(host) {
      assert.equal(host.querySelectorAll('[data-vize-ui="grid-list-item"]').length, 2);
    },
  })),
  ...["kanban.vue", "kanban-card.vue", "kanban-column.vue"].map((file): RuntimeFixture => ({
    name: file.replace(".vue", ""),
    sourceFile: `families/data/kanban/${file}`,
    render: () =>
      h("div", [
        h(Kanban<string, "todo" | "done">, {
          id: "runtime-board",
          ariaLabel: "Board",
          columns: [
            { id: "todo", title: "To do" },
            { id: "done", title: "Done" },
          ],
          modelValue: { todo: ["a"], done: [] },
          getCardKey: (card: string) => card,
        }),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /aria-roledescription="board"/);
      assert.match(html, /data-card-key="a"/);
    },
    assertHydratedDom(host) {
      assert.equal(host.querySelectorAll('[data-vize-ui="kanban-column"]').length, 2);
    },
  })),
];

// Parts are rendered by their roots above; keep them imported so fixtures fail when renamed.
void [DataGridCell, DataGridCellEditor, DataGridColumnHeader, DataGridResizeHandle, DataGridRow];
void [GridListItem, KanbanCard, KanbanColumn];
