import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { computeMasonryLayout, visibleMasonryItems } from "./masonry-layout.ts";

test("appends each item to the shortest column with leftmost tie-breaking", () => {
  const layout = computeMasonryLayout([100, 50, 80, 30, 60], 3, 10);
  assert.deepEqual(layout.columns, [[0], [1, 3], [2, 4]]);
  assert.deepEqual(layout.columnHeights, [100, 90, 150]);
  assert.equal(layout.height, 150);
  assert.deepEqual(layout.placements[3], { index: 3, column: 1, top: 60, height: 30 });
});

test("treats invalid heights and gaps as zero and rejects invalid column counts", () => {
  const layout = computeMasonryLayout([Number.NaN, -5, 20], 2, -3);
  assert.deepEqual(layout.columnHeights, [20, 0]);
  assert.equal(computeMasonryLayout([], 4).height, 0);
  assert.throws(() => computeMasonryLayout([1], 0), /VIZE_UI_MASONRY_COLUMNS/);
  assert.throws(() => computeMasonryLayout([1], 1.5), /VIZE_UI_MASONRY_COLUMNS/);
});

test("selects items intersecting a window with overscan", () => {
  const layout = computeMasonryLayout([100, 100, 100, 100, 100, 100], 2, 0);
  assert.deepEqual(visibleMasonryItems(layout, 0, 50), [0, 1]);
  assert.deepEqual(visibleMasonryItems(layout, 150, 180), [2, 3]);
  assert.deepEqual(visibleMasonryItems(layout, 150, 180, 60), [0, 1, 2, 3, 4, 5]);
});
