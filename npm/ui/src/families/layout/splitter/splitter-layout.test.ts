import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  clampSplitterLayout,
  isValidSplitterLayout,
  parseSplitterLayout,
  resizeSplitterLayout,
  resolveDefaultSplitterLayout,
  splitterLayoutsEqual,
} from "./splitter.ts";
import type { SplitterPanelConstraints } from "./splitter.ts";

const free: SplitterPanelConstraints = {
  collapsedSize: 0,
  collapsible: false,
  maxSize: 100,
  minSize: 0,
};
const bounded = (minSize: number, maxSize = 100): SplitterPanelConstraints => ({
  ...free,
  maxSize,
  minSize,
});
const collapsible = (minSize: number, collapsedSize = 0): SplitterPanelConstraints => ({
  ...free,
  collapsedSize,
  collapsible: true,
  minSize,
});

test("validates, compares, and parses layouts", () => {
  assert.equal(isValidSplitterLayout([30, 70], 2), true);
  assert.equal(isValidSplitterLayout([30, 60], 2), false);
  assert.equal(isValidSplitterLayout([30, 70], 3), false);
  assert.equal(isValidSplitterLayout([-10, 110], 2), false);
  assert.equal(isValidSplitterLayout("30,70", 2), false);
  assert.equal(splitterLayoutsEqual([30, 70], [30.0000001, 69.9999999]), true);
  assert.equal(splitterLayoutsEqual([30, 70], undefined), false);
  assert.deepEqual(parseSplitterLayout("[25,75]"), [25, 75]);
  assert.equal(parseSplitterLayout("[25,70]"), undefined);
  assert.equal(parseSplitterLayout("{"), undefined);
  assert.equal(parseSplitterLayout(null), undefined);
});

test("resolves defaults by sharing the remainder among unsized panels", () => {
  assert.deepEqual(resolveDefaultSplitterLayout([20, undefined, undefined]), [20, 40, 40]);
  assert.deepEqual(
    resolveDefaultSplitterLayout([undefined, undefined, undefined]),
    [33.333, 33.333, 33.333],
  );
  assert.deepEqual(resolveDefaultSplitterLayout([60, 60, undefined]), [60, 60, 0]);
});

test("clamps layouts into constraints and rebalances the remainder", () => {
  assert.deepEqual(clampSplitterLayout([10, 90], [bounded(20), bounded(0, 100)]), [20, 80]);
  assert.deepEqual(clampSplitterLayout([0, 100], [collapsible(20), free]), [0, 100]);
  assert.deepEqual(clampSplitterLayout([70, 30], [bounded(0, 50), bounded(0, 60)]), [50, 50]);
});

test("resizes adjacent panels and cascades past a minimum", () => {
  const constraints = [bounded(10), bounded(10), bounded(10)];
  assert.deepEqual(resizeSplitterLayout([30, 30, 40], constraints, 0, 10), [40, 20, 40]);
  assert.deepEqual(resizeSplitterLayout([30, 30, 40], constraints, 0, 40), [70, 10, 20]);
  assert.deepEqual(resizeSplitterLayout([30, 30, 40], constraints, 1, -35), [15, 10, 75]);
  assert.deepEqual(resizeSplitterLayout([30, 70], [bounded(0, 50), free], 0, 40), [50, 50]);
  const unchanged = [50, 50] as const;
  assert.equal(resizeSplitterLayout(unchanged, [free, free], 1, 10), unchanged);
  assert.equal(resizeSplitterLayout(unchanged, [free, free], 0, 0), unchanged);
});

test("snaps collapsible panels closed and open at the midpoint", () => {
  const constraints = [collapsible(20, 4), free];
  assert.deepEqual(resizeSplitterLayout([30, 70], constraints, 0, -15), [20, 80], "above midpoint");
  assert.deepEqual(resizeSplitterLayout([30, 70], constraints, 0, -20), [4, 96], "past midpoint");
  const collapsed = [4, 96] as const;
  assert.equal(resizeSplitterLayout(collapsed, constraints, 0, 5), collapsed, "short drag stays");
  assert.deepEqual(resizeSplitterLayout(collapsed, constraints, 0, 9), [20, 80], "long drag opens");
});
