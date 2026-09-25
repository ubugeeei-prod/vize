import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  bringWindowToFront,
  clampWindowRect,
  moveWindowRect,
  parseWindowLayout,
  resizeWindowRect,
  snapWindowRect,
} from "./window-manager-model.ts";

const bounds = { width: 1000, height: 600 };
const constraints = {
  minWidth: 100,
  minHeight: 80,
  maxWidth: 800,
  maxHeight: Number.POSITIVE_INFINITY,
};

test("clamps rects inside known bounds and leaves unknown bounds untouched", () => {
  assert.deepEqual(clampWindowRect({ x: 950, y: -20, width: 200, height: 100 }, bounds), {
    x: 800,
    y: 0,
    width: 200,
    height: 100,
  });
  assert.deepEqual(clampWindowRect({ x: 5, y: 5, width: 2000, height: 100 }, bounds).x, 0);
  const rect = { x: -50, y: 9999, width: 10, height: 10 };
  assert.equal(clampWindowRect(rect, null), rect);
});

test("snaps edges to bounds and neighbours within the threshold", () => {
  const other = { x: 300, y: 100, width: 200, height: 150 };
  assert.deepEqual(
    snapWindowRect({ x: 506, y: 104, width: 100, height: 100 }, [other], bounds, 8),
    {
      x: 500,
      y: 100,
      width: 100,
      height: 100,
    },
  );
  assert.equal(snapWindowRect({ x: 895, y: 300, width: 100, height: 100 }, [], bounds, 8).x, 900);
  assert.equal(
    snapWindowRect({ x: 520, y: 300, width: 100, height: 100 }, [other], bounds, 8).x,
    520,
  );
  assert.equal(snapWindowRect({ x: 3, y: 3, width: 10, height: 10 }, [], bounds, 0).x, 3);
});

test("moves by a delta, then snaps and clamps", () => {
  const moved = moveWindowRect({ x: 100, y: 100, width: 200, height: 100 }, 895, 20, [], bounds, 8);
  assert.deepEqual(moved, { x: 800, y: 120, width: 200, height: 100 });
});

test("resizes from every edge within constraints and bounds", () => {
  const rect = { x: 100, y: 100, width: 300, height: 200 };
  assert.deepEqual(resizeWindowRect(rect, "se", 50, 40, constraints, bounds), {
    x: 100,
    y: 100,
    width: 350,
    height: 240,
  });
  assert.deepEqual(resizeWindowRect(rect, "nw", 250, 150, constraints, bounds), {
    x: 300,
    y: 220,
    width: 100,
    height: 80,
  });
  assert.equal(resizeWindowRect(rect, "e", 900, 0, constraints, bounds).width, 800);
  assert.equal(
    resizeWindowRect(rect, "e", 900, 0, { ...constraints, maxWidth: 5000 }, bounds).width,
    900,
  );
  assert.deepEqual(resizeWindowRect(rect, "w", -500, 0, constraints, bounds), {
    x: 0,
    y: 100,
    width: 400,
    height: 200,
  });
  assert.equal(resizeWindowRect(rect, "n", 0, -40, constraints, null).height, 240);
});

test("raises windows to the front of the stacking order", () => {
  const order = ["a", "b", "c"];
  assert.deepEqual(bringWindowToFront(order, "a"), ["b", "c", "a"]);
  assert.equal(bringWindowToFront(order, "c"), order);
  assert.deepEqual(bringWindowToFront(order, "d"), ["a", "b", "c", "d"]);
});

test("parses stored layouts and drops malformed entries", () => {
  const layout = parseWindowLayout(
    JSON.stringify({
      windows: {
        editor: { x: 1, y: 2, width: 300, height: 200, mode: "maximized" },
        broken: { x: "1", y: 2, width: 3, height: 4, mode: "normal" },
        odd: { x: 1, y: 2, width: 3, height: 4, mode: "floating" },
      },
      order: ["broken", "editor", "editor", "ghost", 4],
    }),
  );
  assert.deepEqual(layout, {
    windows: { editor: { x: 1, y: 2, width: 300, height: 200, mode: "maximized" } },
    order: ["editor"],
  });
  assert.equal(parseWindowLayout(null), undefined);
  assert.equal(parseWindowLayout("{"), undefined);
  assert.equal(parseWindowLayout("[]"), undefined);
});
