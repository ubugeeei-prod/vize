import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, ref } from "vue";

import { createColumnHelper, readPath } from "./data-grid-columns.ts";
import { people, personColumns, tree } from "./data-grid-fixture.ts";
import type { Person, PersonColumn } from "./data-grid-fixture.ts";
import { useDataGrid } from "./data-grid-model.ts";
import type { DataGridOptions, DataGridStateKey } from "./data-grid-model.ts";
import { buildRowModels, toggleSorting } from "./data-grid-rows.ts";
import type { DataGridSort } from "./data-grid-types.ts";

function withGrid(
  options: Partial<DataGridOptions<Person, PersonColumn>>,
  run: (grid: ReturnType<typeof useDataGrid<Person, PersonColumn>>) => void,
): void {
  const scope = effectScope();
  scope.run(() =>
    run(
      useDataGrid<Person, PersonColumn>({
        rows: people,
        columns: personColumns,
        getRowId: (row) => row.id,
        ...options,
      }),
    ),
  );
  scope.stop();
}

test("toggleSorting cycles ascending, descending, off and keeps multi-key priority", () => {
  let sorting: DataGridSort[] = toggleSorting([], "age", false);
  assert.deepEqual(sorting, [{ columnId: "age", direction: "ascending" }]);
  sorting = toggleSorting(sorting, "name", true);
  assert.deepEqual(
    sorting.map((sort) => sort.columnId),
    ["age", "name"],
  );
  sorting = toggleSorting(sorting, "age", true);
  assert.deepEqual(sorting[0], { columnId: "age", direction: "descending" });
  sorting = toggleSorting(sorting, "age", true);
  assert.deepEqual(sorting, [{ columnId: "name", direction: "ascending" }]);
  assert.deepEqual(toggleSorting(sorting, "age", false), [
    { columnId: "age", direction: "ascending" },
  ]);
});

test("readPath short-circuits nullish segments and ignores non-objects", () => {
  assert.equal(readPath({ a: { b: 1 } }, "a.b"), 1);
  assert.equal(readPath({ a: null }, "a.b"), undefined);
  assert.equal(readPath("text", "length"), undefined);
});

test("buildRowModels flattens expanded trees with depth, parents, and stable sort", () => {
  const rows = buildRowModels<Person>({
    rows: tree,
    columns: personColumns,
    getRowId: (row) => row.id,
    getSubRows: (row) => row.reports,
    sorting: [{ columnId: "age", direction: "descending" }],
    globalFilter: "",
    columnFilters: {},
    expanded: new Set(["ceo"]),
    selected: new Set(["cfo"]),
  });
  assert.deepEqual(
    rows.map((row) => [row.id, row.depth, row.parentId, row.index, row.selected]),
    [
      ["ceo", 0, null, 0, false],
      ["cfo", 1, "ceo", 1, true],
      ["cto", 1, "ceo", 2, false],
      ["solo", 0, null, 3, false],
    ],
  );
  assert.equal(rows[0]?.expandable, true);
  assert.equal(rows[0]?.expanded, true);
});

test("custom compare and default string comparison are locale and numeric aware", () => {
  const column = createColumnHelper<{ readonly label: string }>();
  const rows = buildRowModels({
    rows: [{ label: "item 10" }, { label: "item 2" }, { label: "Item 1" }],
    columns: [column.accessor("label")],
    getRowId: (row) => row.label,
    sorting: [{ columnId: "label", direction: "ascending" }],
    globalFilter: "",
    columnFilters: {},
    expanded: new Set(),
    selected: new Set(),
  });
  assert.deepEqual(
    rows.map((row) => row.id),
    ["Item 1", "item 2", "item 10"],
  );
});

test("uncontrolled slices update state and report every change", () => {
  const changes: string[] = [];
  withGrid(
    {
      selectionMode: "multiple",
      onStateChange: (key: DataGridStateKey, value: unknown) =>
        changes.push(`${key}:${JSON.stringify(value)}`),
    },
    (grid) => {
      assert.equal(grid.toggleSort("actions"), false, "display columns never sort");
      assert.equal(grid.toggleSort("age"), true);
      assert.equal(grid.setColumnVisible("actions", false), false, "hideable: false is respected");
      assert.equal(grid.setColumnVisible("team", false), true);
      assert.deepEqual(
        grid.columnModels.value.map((column) => column.id),
        ["name", "age", "actions"],
      );
      assert.equal(grid.moveColumn("actions", 0), true);
      assert.equal(grid.setColumnVisible("team", true), true);
      assert.deepEqual(
        grid.columnModels.value.map((column) => column.id),
        ["actions", "name", "age", "team"],
        "hidden columns keep their slot while moved columns reorder",
      );
      assert.equal(grid.pinColumn("team", "start"), true);
      assert.equal(grid.columnModels.value[0]?.id, "team");
      assert.equal(grid.columnModels.value[1]?.pinOffset, 0);
      assert.equal(grid.resizeColumn("actions", 500), false, "resizable: false is respected");
      assert.equal(grid.resizeColumn("age", 500), true);
      assert.equal(grid.resizeColumn("name", Number.NaN), false);
      assert.equal(grid.select("ada"), true);
      assert.equal(grid.select("alan", "range"), true);
      assert.deepEqual(
        grid.state.value.selection,
        ["ada", "linus", "alan"],
        "ranges follow sorted order",
      );
      assert.equal(grid.someSelected.value, true);
      assert.equal(grid.toggleAllSelected(), true);
      assert.equal(grid.allSelected.value, true);
      assert.equal(grid.toggleAllSelected(), true);
      assert.deepEqual(grid.state.value.selection, []);
    },
  );
  assert.ok(changes.includes('sorting:[{"columnId":"age","direction":"ascending"}]'));
  assert.ok(changes.some((change) => change.startsWith("columnPinning:")));
});

test("controlled slices only change through the owner", () => {
  const sorting = ref<readonly DataGridSort[]>([]);
  withGrid({ state: { sorting } }, (grid) => {
    grid.toggleSort("age");
    assert.deepEqual(grid.state.value.sorting, []);
    sorting.value = [{ columnId: "name", direction: "descending" }];
    assert.equal(grid.rowModels.value[0]?.id, "linus");
  });
});

test("edits parse, validate, report parse errors, and never mutate rows", () => {
  const edits: unknown[] = [];
  const column = createColumnHelper<Person>();
  const columns = [
    column.accessor("age", {
      editable: true,
      parse: (input) => {
        const value = Number(input);
        if (!Number.isFinite(value)) throw new Error("Not a number");
        return value;
      },
    }),
  ];
  const scope = effectScope();
  scope.run(() => {
    const grid = useDataGrid({
      rows: people,
      columns,
      getRowId: (row) => row.id,
      onCellEdit: (event) => edits.push(event.value),
    });
    assert.equal(grid.startEdit({ rowId: null, columnId: "age" }), false);
    assert.equal(grid.startEdit({ rowId: "ada", columnId: "age" }), true);
    assert.equal(grid.editing.value?.draft, "36");
    grid.setDraft("x");
    assert.equal(grid.commitEdit(), false);
    assert.equal(grid.editing.value?.error, "Not a number");
    grid.setDraft("40");
    assert.equal(grid.commitEdit(), true);
    assert.deepEqual(edits, [40]);
    assert.equal(people[0]?.age, 36);
    assert.equal(grid.cancelEdit(), false);
  });
  scope.stop();
});
