import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  compactDashboardLayout,
  dashboardItemsCollide,
  dashboardLayoutRows,
  moveDashboardItem,
  normalizeDashboardLayout,
  resizeDashboardItem,
} from "./dashboard-grid-layout.ts";
import type { DashboardLayout } from "./dashboard-grid-layout.ts";

const layout: DashboardLayout = [
  { id: "a", x: 0, y: 0, w: 2, h: 2 },
  { id: "b", x: 2, y: 0, w: 2, h: 1 },
  { id: "c", x: 0, y: 2, w: 4, h: 1 },
];

function positions(items: DashboardLayout): Record<string, string> {
  return Object.fromEntries(
    items.map((item) => [item.id, `${item.x},${item.y},${item.w}x${item.h}`]),
  );
}

test("detects overlapping cells but never self-collision", () => {
  assert.equal(dashboardItemsCollide(layout[0]!, { id: "z", x: 1, y: 1, w: 1, h: 1 }), true);
  assert.equal(dashboardItemsCollide(layout[0]!, layout[1]!), false);
  assert.equal(dashboardItemsCollide(layout[0]!, layout[0]!), false);
});

test("normalizes sizes, constraints, and positions into the grid", () => {
  const normalized = normalizeDashboardLayout(
    [{ id: "x", x: 10.4, y: -2, w: 9, h: 0.2, minW: 2, maxW: 3 }],
    4,
  );
  assert.deepEqual(normalized, [{ id: "x", x: 1, y: 0, w: 3, h: 1, minW: 2, maxW: 3 }]);
});

test("compacts upward until blocked by static widgets and resolves overlaps", () => {
  const compacted = compactDashboardLayout([
    { id: "pin", x: 0, y: 1, w: 4, h: 1, static: true },
    { id: "low", x: 0, y: 5, w: 2, h: 1 },
    { id: "top", x: 2, y: 3, w: 2, h: 1 },
  ]);
  assert.deepEqual(positions(compacted), { pin: "0,1,4x1", low: "0,2,2x1", top: "2,2,2x1" });
  const pushed = compactDashboardLayout(
    [
      { id: "one", x: 0, y: 0, w: 2, h: 2 },
      { id: "two", x: 0, y: 1, w: 2, h: 1 },
    ],
    "none",
  );
  assert.deepEqual(positions(pushed), { one: "0,0,2x2", two: "0,2,2x1" });
});

test("moves push colliding widgets down and compact the rest", () => {
  const moved = moveDashboardItem(layout, "c", 0, 0, 4);
  assert.deepEqual(positions(moved), { a: "0,1,2x2", b: "2,1,2x1", c: "0,0,4x1" });
  assert.equal(moveDashboardItem(layout, "a", 0, 0, 4), layout);
  assert.equal(moveDashboardItem(layout, "missing", 1, 1, 4), layout);
  assert.deepEqual(positions(moveDashboardItem(layout, "b", 9, 0, 4)), positions(layout));
});

test("moves onto static widgets are rejected and static widgets never move", () => {
  const withStatic: DashboardLayout = [
    ...layout,
    { id: "s", x: 0, y: 4, w: 4, h: 1, static: true },
  ];
  assert.equal(moveDashboardItem(withStatic, "a", 0, 4, 4), withStatic);
  assert.equal(moveDashboardItem(withStatic, "s", 0, 0, 4), withStatic);
});

test("resizes within constraints and pushes neighbours", () => {
  const resized = resizeDashboardItem(layout, "b", 2, 3, 4);
  assert.deepEqual(positions(resized), { a: "0,0,2x2", b: "2,0,2x3", c: "0,3,4x1" });
  assert.equal(resizeDashboardItem(layout, "b", 2, 1, 4), layout);
  const limited = resizeDashboardItem([{ id: "m", x: 1, y: 0, w: 1, h: 1, maxH: 2 }], "m", 9, 9, 4);
  assert.deepEqual(positions(limited), { m: "1,0,3x2" });
  const noCompact = resizeDashboardItem(layout, "a", 2, 3, 4, "none");
  assert.deepEqual(positions(noCompact), { a: "0,0,2x3", b: "2,0,2x1", c: "0,3,4x1" });
});

test("reports the number of occupied rows", () => {
  assert.equal(dashboardLayoutRows(layout), 3);
  assert.equal(dashboardLayoutRows([]), 0);
});
