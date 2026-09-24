import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  imageComparePositionForKey,
  imageComparePositionFromPoint,
  normalizeImageComparePosition,
} from "./image-compare-value.ts";

test("normalizes positions by clamping and snapping", () => {
  assert.equal(normalizeImageComparePosition(42.4), 42.4);
  assert.equal(normalizeImageComparePosition(42.4, 1), 42);
  assert.equal(normalizeImageComparePosition(42.6, 5), 45);
  assert.equal(normalizeImageComparePosition(0.36, 0.1), 0.4);
  assert.equal(normalizeImageComparePosition(-4, 1), 0);
  assert.equal(normalizeImageComparePosition(104, 1), 100);
  assert.equal(normalizeImageComparePosition(100, 7), 98, "snapping never exceeds the range");
  assert.equal(normalizeImageComparePosition(Number.NaN, 1), 0);
  assert.equal(normalizeImageComparePosition(55, Number.POSITIVE_INFINITY), 55);
  assert.equal(normalizeImageComparePosition(55, -2), 55);
});

test("maps pointer coordinates per axis and reading direction", () => {
  const rect = { left: 100, top: 50, width: 200, height: 400 };
  const ltr = { orientation: "horizontal", dir: "ltr" } as const;
  assert.equal(imageComparePositionFromPoint(rect, 150, 0, ltr), 25);
  assert.equal(imageComparePositionFromPoint(rect, 150, 0, { ...ltr, dir: "rtl" }), 75);
  assert.equal(
    imageComparePositionFromPoint(rect, 0, 150, { orientation: "vertical", dir: "rtl" }),
    25,
  );
  assert.equal(imageComparePositionFromPoint({ ...rect, width: 0 }, 150, 0, ltr), null);
  assert.equal(
    imageComparePositionFromPoint({ ...rect, height: 0 }, 0, 150, {
      orientation: "vertical",
      dir: "ltr",
    }),
    null,
  );
});

test("maps slider keys to the next position", () => {
  const options = { orientation: "horizontal", dir: "ltr", step: 2, pageStep: 10 } as const;
  assert.equal(imageComparePositionForKey("ArrowRight", 50, options), 52);
  assert.equal(imageComparePositionForKey("ArrowLeft", 50, options), 48);
  assert.equal(imageComparePositionForKey("ArrowRight", 50, { ...options, dir: "rtl" }), 48);
  assert.equal(imageComparePositionForKey("ArrowLeft", 50, { ...options, dir: "rtl" }), 52);
  assert.equal(imageComparePositionForKey("ArrowUp", 50, options), 48);
  assert.equal(imageComparePositionForKey("ArrowDown", 50, options), 52);
  assert.equal(
    imageComparePositionForKey("ArrowRight", 50, {
      ...options,
      orientation: "vertical",
      dir: "rtl",
    }),
    52,
    "vertical dividers ignore the reading direction",
  );
  assert.equal(imageComparePositionForKey("PageUp", 50, options), 60);
  assert.equal(imageComparePositionForKey("PageDown", 50, options), 40);
  assert.equal(imageComparePositionForKey("Home", 50, options), 0);
  assert.equal(imageComparePositionForKey("End", 50, options), 100);
  assert.equal(imageComparePositionForKey("Enter", 50, options), null);
});
