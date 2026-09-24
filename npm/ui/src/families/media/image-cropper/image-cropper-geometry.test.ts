import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  centeredCrop,
  clampCrop,
  clampViewCenter,
  fitScale,
  fromViewport,
  moveCrop,
  normalizeRotation,
  refitCrop,
  resizeCrop,
  rotatedBounds,
  toViewport,
  zoomAroundPoint,
} from "./image-cropper-geometry.ts";
import type { CropArea } from "./image-cropper-types.ts";

const bounds = { width: 400, height: 200 };

function close(actual: CropArea, expected: CropArea): void {
  for (const key of ["x", "y", "width", "height"] as const) {
    assert.ok(
      Math.abs(actual[key] - expected[key]) < 1e-9,
      `${key}: expected ${expected[key]}, received ${actual[key]}`,
    );
  }
}

test("normalizes rotation and computes rotated bounds exactly on right angles", () => {
  assert.equal(normalizeRotation(0), 0);
  assert.equal(normalizeRotation(-90), 270);
  assert.equal(normalizeRotation(450), 90);
  assert.equal(normalizeRotation(-360), 0);
  assert.equal(normalizeRotation(Number.NaN), 0);
  assert.deepEqual(rotatedBounds(bounds, 0), bounds);
  assert.deepEqual(rotatedBounds(bounds, 180), bounds);
  assert.deepEqual(rotatedBounds(bounds, 90), { width: 200, height: 400 });
  assert.deepEqual(rotatedBounds(bounds, -90), { width: 200, height: 400 });
  const diagonal = rotatedBounds({ width: 100, height: 100 }, 45);
  assert.ok(Math.abs(diagonal.width - 100 * Math.SQRT2) < 1e-9);
  assert.ok(Math.abs(diagonal.height - 100 * Math.SQRT2) < 1e-9);
});

test("clamps crops into bounds, size limits, and aspect ratios", () => {
  close(clampCrop({ x: -10, y: 150, width: 100, height: 100 }, bounds), {
    x: 0,
    y: 100,
    width: 100,
    height: 100,
  });
  close(clampCrop({ x: 0, y: 0, width: 999, height: 999 }, bounds), {
    x: 0,
    y: 0,
    width: 400,
    height: 200,
  });
  close(clampCrop({ x: 0, y: 0, width: 0, height: 0 }, bounds, { minWidth: 50, minHeight: 20 }), {
    x: 0,
    y: 0,
    width: 50,
    height: 20,
  });
  close(clampCrop({ x: 0, y: 0, width: 300, height: 10 }, bounds, { maxWidth: 120 }), {
    x: 0,
    y: 0,
    width: 120,
    height: 10,
  });
  close(clampCrop({ x: 350, y: 0, width: 300, height: 10 }, bounds, { aspectRatio: 2 }), {
    x: 100,
    y: 0,
    width: 300,
    height: 150,
  });
  close(clampCrop({ x: 0, y: 0, width: 400, height: 10 }, bounds, { aspectRatio: 1 }), {
    x: 0,
    y: 0,
    width: 200,
    height: 200,
  });
  close(
    clampCrop({ x: 0, y: 0, width: 10, height: 10 }, bounds, { aspectRatio: 1, minHeight: 500 }),
    { x: 0, y: 0, width: 200, height: 200 },
  );
  close(clampCrop({ x: Number.NaN, y: 0, width: 10, height: 10 }, bounds), {
    x: 0,
    y: 0,
    width: 10,
    height: 10,
  });
  close(clampCrop({ x: 0, y: 0, width: 10, height: 10 }, bounds, { aspectRatio: -1 }), {
    x: 0,
    y: 0,
    width: 10,
    height: 10,
  });
});

test("centers default crops by coverage and aspect ratio", () => {
  close(centeredCrop(bounds), { x: 0, y: 0, width: 400, height: 200 });
  close(centeredCrop(bounds, {}, 0.5), { x: 100, y: 50, width: 200, height: 100 });
  close(centeredCrop(bounds, { aspectRatio: 1 }, 0.8), { x: 120, y: 20, width: 160, height: 160 });
  close(centeredCrop(bounds, { aspectRatio: 4 }, 1), { x: 0, y: 50, width: 400, height: 100 });
  close(centeredCrop(bounds, {}, 7), { x: 0, y: 0, width: 400, height: 200 });
});

test("moves crops by deltas without leaving the bounds", () => {
  const crop = { x: 10, y: 10, width: 100, height: 50 };
  close(moveCrop(crop, 20, 5, bounds), { x: 30, y: 15, width: 100, height: 50 });
  close(moveCrop(crop, -50, -50, bounds), { x: 0, y: 0, width: 100, height: 50 });
  close(moveCrop(crop, 999, 999, bounds), { x: 300, y: 150, width: 100, height: 50 });
});

test("resizes free crops from every handle with anchored opposite edges", () => {
  const crop = { x: 100, y: 50, width: 100, height: 100 };
  close(resizeCrop(crop, "e", 30, 99, bounds), { x: 100, y: 50, width: 130, height: 100 });
  close(resizeCrop(crop, "w", -30, 0, bounds), { x: 70, y: 50, width: 130, height: 100 });
  close(resizeCrop(crop, "n", 0, -20, bounds), { x: 100, y: 30, width: 100, height: 120 });
  close(resizeCrop(crop, "s", 0, 20, bounds), { x: 100, y: 50, width: 100, height: 120 });
  close(resizeCrop(crop, "se", 10, 20, bounds), { x: 100, y: 50, width: 110, height: 120 });
  close(resizeCrop(crop, "nw", -10, -20, bounds), { x: 90, y: 30, width: 110, height: 120 });
  close(resizeCrop(crop, "ne", 10, -20, bounds), { x: 100, y: 30, width: 110, height: 120 });
  close(resizeCrop(crop, "sw", -10, 20, bounds), { x: 90, y: 50, width: 110, height: 120 });
  close(resizeCrop(crop, "e", 999, 0, bounds), { x: 100, y: 50, width: 300, height: 100 });
  close(resizeCrop(crop, "w", -999, 0, bounds), { x: 0, y: 50, width: 200, height: 100 });
  close(resizeCrop(crop, "e", -999, 0, bounds, { minWidth: 20 }), {
    x: 100,
    y: 50,
    width: 20,
    height: 100,
  });
  close(resizeCrop(crop, "n", 0, 999, bounds), { x: 100, y: 149, width: 100, height: 1 });
  close(resizeCrop(crop, "s", 0, 999, bounds, { maxHeight: 120 }), {
    x: 100,
    y: 50,
    width: 100,
    height: 120,
  });
});

test("resizes aspect-locked crops along the dominant axis and within bounds", () => {
  const crop = { x: 100, y: 50, width: 100, height: 50 };
  const locked = { aspectRatio: 2 };
  close(resizeCrop(crop, "se", 40, 5, bounds, locked), { x: 100, y: 50, width: 140, height: 70 });
  close(resizeCrop(crop, "se", 0, 40, bounds, locked), { x: 100, y: 50, width: 180, height: 90 });
  close(resizeCrop(crop, "nw", -20, 0, bounds, locked), { x: 80, y: 40, width: 120, height: 60 });
  close(resizeCrop(crop, "se", 999, 999, bounds, locked), {
    x: 100,
    y: 50,
    width: 300,
    height: 150,
  });
  close(resizeCrop(crop, "e", 60, 0, bounds, locked), { x: 100, y: 35, width: 160, height: 80 });
  close(resizeCrop(crop, "w", -999, 0, bounds, locked), { x: 0, y: 25, width: 200, height: 100 });
  close(resizeCrop(crop, "s", 0, 10, bounds, locked), { x: 90, y: 50, width: 120, height: 60 });
  close(resizeCrop(crop, "n", 0, -999, bounds, locked), { x: 50, y: 0, width: 200, height: 100 });
  close(resizeCrop(crop, "se", -999, -999, bounds, { aspectRatio: 2, minWidth: 40 }), {
    x: 100,
    y: 50,
    width: 40,
    height: 20,
  });
});

test("re-fits crops after bounds change by keeping the relative center", () => {
  close(refitCrop({ x: 0, y: 0, width: 100, height: 100 }, bounds, { width: 200, height: 400 }), {
    x: 0,
    y: 50,
    width: 100,
    height: 100,
  });
  close(
    refitCrop({ x: 150, y: 50, width: 100, height: 100 }, bounds, { width: 200, height: 400 }),
    { x: 50, y: 150, width: 100, height: 100 },
  );
  close(refitCrop({ x: 0, y: 0, width: 400, height: 200 }, bounds, { width: 200, height: 400 }), {
    x: 0,
    y: 100,
    width: 200,
    height: 200,
  });
  close(refitCrop({ x: 0, y: 0, width: 5, height: 5 }, { width: 0, height: 0 }, bounds), {
    x: 0,
    y: 0,
    width: 400,
    height: 200,
  });
});

test("maps the view with contain scale, clamped centers, and zoom anchors", () => {
  const viewport = { width: 200, height: 200 };
  assert.equal(fitScale(viewport, bounds), 0.5);
  assert.equal(fitScale({ width: 0, height: 10 }, bounds), 0);
  assert.equal(fitScale(viewport, { width: 0, height: 1 }), 0);

  assert.deepEqual(clampViewCenter({ x: 0, y: 0 }, bounds, viewport, 0.5), { x: 200, y: 100 });
  assert.deepEqual(clampViewCenter({ x: 0, y: 0 }, bounds, viewport, 2), { x: 50, y: 50 });
  assert.deepEqual(clampViewCenter({ x: 999, y: 999 }, bounds, viewport, 2), { x: 350, y: 150 });
  assert.deepEqual(clampViewCenter({ x: 5, y: 5 }, bounds, viewport, 0), { x: 200, y: 100 });
  assert.deepEqual(clampViewCenter({ x: Number.NaN, y: 0 }, bounds, viewport, 2), {
    x: 200,
    y: 50,
  });

  const center = { x: 200, y: 100 };
  assert.deepEqual(toViewport({ x: 0, y: 0 }, center, viewport, 0.5), { x: 0, y: 50 });
  assert.deepEqual(fromViewport({ x: 0, y: 50 }, center, viewport, 0.5), { x: 0, y: 0 });
  assert.deepEqual(fromViewport({ x: 1, y: 1 }, center, viewport, 0), center);

  const anchor = { x: 100, y: 50 };
  const zoomed = zoomAroundPoint(center, anchor, 0.5, 1);
  assert.deepEqual(zoomed, { x: 150, y: 75 });
  assert.deepEqual(
    toViewport(anchor, zoomed, viewport, 1),
    toViewport(anchor, center, viewport, 0.5),
    "the anchor stays under the same viewport position",
  );
  assert.deepEqual(zoomAroundPoint(center, anchor, 0, 1), center);
});
