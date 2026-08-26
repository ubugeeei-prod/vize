import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope } from "vue";

import { createGridVirtualizer, useGridVirtualizer } from "./virtualizer.ts";

const options = {
  rowCount: 100,
  columnCount: 50,
  rowSize: 20,
  columnSize: 50,
  overscan: 0,
  initialRect: { width: 200, height: 100 },
} as const;

test("windows rows and columns over one shared viewport", () => {
  const grid = createGridVirtualizer(options);

  assert.deepEqual(grid.rows.range.value, { startIndex: 0, endIndex: 4 });
  assert.deepEqual(grid.columns.range.value, { startIndex: 0, endIndex: 3 });
  assert.equal(grid.rows.totalSize.value, 2000);
  assert.equal(grid.columns.totalSize.value, 2500);

  assert.equal(grid.virtualCells.value.length, 5 * 4);
  assert.deepEqual(grid.virtualCells.value[0], {
    rowIndex: 0,
    columnIndex: 0,
    key: "0:0",
    top: 0,
    left: 0,
    width: 50,
    height: 20,
  });
  const last = grid.virtualCells.value.at(-1);
  assert.deepEqual(last, {
    rowIndex: 4,
    columnIndex: 3,
    key: "4:3",
    top: 80,
    left: 150,
    width: 50,
    height: 20,
  });
  grid.dispose();
});

test("cells follow scrolling on both axes", () => {
  const grid = createGridVirtualizer(options);

  grid.rows.scrollToOffset(400);
  grid.columns.scrollToOffset(500);
  assert.deepEqual(grid.rows.range.value, { startIndex: 20, endIndex: 24 });
  assert.deepEqual(grid.columns.range.value, { startIndex: 10, endIndex: 13 });
  assert.deepEqual(
    grid.virtualCells.value.map((cell) => cell.key).slice(0, 4),
    ["20:10", "20:11", "20:12", "20:13"],
    "cells stay in row-major order",
  );
  grid.dispose();
});

test("scrolls to a cell on both axes", () => {
  const grid = createGridVirtualizer(options);

  grid.scrollToCell(50, 25, "start");
  assert.equal(grid.rows.scrollOffset.value, 1000);
  assert.equal(grid.columns.scrollOffset.value, 1250);

  grid.scrollToCell(50, 25);
  assert.equal(grid.rows.scrollOffset.value, 1000, "auto keeps visible cells in place");
  grid.dispose();
});

test("estimates and measures rows independently of columns", () => {
  const grid = createGridVirtualizer({
    ...options,
    rowSize: undefined,
    estimateRowSize: 20,
  });

  grid.rows.resizeItem(0, 60);
  assert.equal(grid.rows.totalSize.value, 2040);
  assert.equal(grid.columns.totalSize.value, 2500, "columns are untouched");
  assert.equal(grid.virtualCells.value[0]?.height, 60);
  grid.dispose();
});

test("validates sizing per axis and disposes both axes", () => {
  assert.throws(
    () => createGridVirtualizer({ ...options, columnSize: undefined }),
    /one of itemSize or estimateItemSize/,
  );

  const grid = createGridVirtualizer(options);
  grid.dispose();
  assert.throws(() => grid.rows.scrollToOffset(0), /VIZE_UI_VIRTUALIZER_DISPOSED/);
  assert.throws(() => grid.columns.scrollToOffset(0), /VIZE_UI_VIRTUALIZER_DISPOSED/);

  assert.throws(() => useGridVirtualizer(options), /VIZE_UI_VIRTUALIZER_SETUP/);
  const scope = effectScope();
  const scoped = scope.run(() => useGridVirtualizer(options));
  assert.ok(scoped);
  scope.stop();
  assert.throws(() => scoped.rows.scrollToOffset(0), /VIZE_UI_VIRTUALIZER_DISPOSED/);
});
