import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  PAN_ZOOM_IDENTITY,
  clampScale,
  clampToBounds,
  coverScale,
  createTransform,
  fitTransform,
  normalizeWheelDelta,
  panBy,
  pinchTransform,
  transformEquals,
  transformToCss,
  wheelZoomFactor,
  zoomAt,
} from "./pan-zoom-transform.ts";

const limits = { minScale: 0.5, maxScale: 4 } as const;
const viewport = { width: 400, height: 300 } as const;

test("creates frozen transforms and replaces non-finite members", () => {
  const transform = createTransform(Number.NaN, 5, 0);
  assert.deepEqual(transform, { x: 0, y: 5, scale: 1 });
  assert.ok(Object.isFrozen(transform));
  assert.deepEqual(createTransform(1, Number.POSITIVE_INFINITY, -2), { x: 1, y: 0, scale: 1 });
  assert.ok(Object.isFrozen(PAN_ZOOM_IDENTITY));
  assert.equal(transformEquals({ x: 1, y: 2, scale: 1 }, { x: 1 + 1e-9, y: 2, scale: 1 }), true);
  assert.equal(transformEquals({ x: 1, y: 2, scale: 1 }, { x: 1.01, y: 2, scale: 1 }), false);
});

test("clamps scale into ordered limits", () => {
  assert.equal(clampScale(10, limits), 4);
  assert.equal(clampScale(0.1, limits), 0.5);
  assert.equal(clampScale(2, limits), 2);
  assert.equal(clampScale(Number.NaN, limits), 1);
  assert.equal(clampScale(3, { minScale: 5, maxScale: 2 }), 2, "inverted limits collapse to max");
});

test("zooms around a point keeping the content under it fixed", () => {
  const start = createTransform(10, 20, 1);
  const zoomed = zoomAt(start, 2, { x: 110, y: 120 });
  assert.deepEqual(zoomed, { x: -90, y: -80, scale: 2 });
  // The content point (100, 100) stays at viewport (110, 120).
  assert.equal(100 * zoomed.scale + zoomed.x, 110);
  assert.equal(100 * zoomed.scale + zoomed.y, 120);
  assert.deepEqual(panBy(start, 5, -5), { x: 15, y: 15, scale: 1 });
});

test("contain centers smaller axes and prevents gaps on larger axes", () => {
  const content = { width: 200, height: 600 };
  const clamped = clampToBounds(createTransform(50, 40, 1), viewport, content, "contain");
  assert.equal(clamped.x, 100, "narrow content is centered horizontally");
  assert.equal(clamped.y, 0, "tall content cannot reveal space above it");
  const bottom = clampToBounds(createTransform(0, -900, 1), viewport, content, "contain");
  assert.equal(bottom.y, -300, "tall content cannot reveal space below it");
  assert.deepEqual(
    clampToBounds(createTransform(5, 5, 1), viewport, { width: 0, height: 0 }, "contain"),
    { x: 5, y: 5, scale: 1 },
    "unmeasured content is left unchanged",
  );
  const none = createTransform(-1000, 1000, 3);
  assert.ok(clampToBounds(none, viewport, content, "none") === none);
});

test("cover raises the scale to cover the viewport without gaps", () => {
  const content = { width: 200, height: 100 };
  assert.equal(coverScale(viewport, content), 3);
  assert.equal(coverScale(viewport, { width: 0, height: 10 }), 0);
  const covered = clampToBounds(createTransform(0, 0, 1), viewport, content, "cover");
  assert.equal(covered.scale, 3);
  assert.equal(covered.x, -200);
  assert.equal(covered.y, 0);
  const larger = clampToBounds(createTransform(50, 50, 4), viewport, content, "cover");
  assert.deepEqual(larger, { x: 0, y: 0, scale: 4 });
});

test("custom bounds keep the viewport center inside a content region", () => {
  const region = { x: 0, y: 0, width: 100, height: 100 };
  const inside = clampToBounds(createTransform(150, 100, 1), viewport, viewport, region);
  assert.deepEqual(inside, { x: 150, y: 100, scale: 1 }, "center (50, 50) is inside");
  const outside = clampToBounds(createTransform(-500, -500, 2), viewport, viewport, region);
  // Center must clamp to content (100, 100): x = 200 - 100 * 2.
  assert.deepEqual(outside, { x: 0, y: -50, scale: 2 });
  const unmeasured = createTransform(-500, -500, 2);
  assert.ok(clampToBounds(unmeasured, { width: 0, height: 0 }, viewport, region) === unmeasured);
});

test("fits and centers content within padding and scale limits", () => {
  assert.deepEqual(fitTransform(viewport, { width: 800, height: 300 }, limits), {
    x: 0,
    y: 75,
    scale: 0.5,
  });
  assert.deepEqual(fitTransform(viewport, { width: 100, height: 100 }, limits, 50), {
    x: 100,
    y: 50,
    scale: 2,
  });
  assert.deepEqual(fitTransform(viewport, { width: 10, height: 10 }, limits), {
    x: 180,
    y: 130,
    scale: 4,
  });
  assert.ok(fitTransform(viewport, { width: 0, height: 10 }, limits) === PAN_ZOOM_IDENTITY);
});

test("normalizes wheel deltas and converts them to zoom factors", () => {
  assert.deepEqual(normalizeWheelDelta({ deltaX: 3, deltaY: -2, deltaMode: 0 }, viewport), {
    x: 3,
    y: -2,
  });
  assert.deepEqual(normalizeWheelDelta({ deltaX: 1, deltaY: 2, deltaMode: 1 }, viewport), {
    x: 16,
    y: 32,
  });
  assert.deepEqual(normalizeWheelDelta({ deltaX: 1, deltaY: -1, deltaMode: 2 }, viewport), {
    x: 400,
    y: -300,
  });
  assert.ok(wheelZoomFactor(-100, false) > 1);
  assert.ok(wheelZoomFactor(100, false) < 1);
  assert.equal(wheelZoomFactor(0, true), 1);
  assert.ok(wheelZoomFactor(-10, true) > wheelZoomFactor(-10, false), "pinches are more sensitive");
  assert.equal(wheelZoomFactor(Number.NaN, false), 1);
});

test("pinch scales by the distance ratio around the moving midpoint", () => {
  const start = createTransform(0, 0, 1);
  const from = { first: { x: 100, y: 100 }, second: { x: 200, y: 100 } };
  const spread = { first: { x: 50, y: 100 }, second: { x: 250, y: 100 } };
  assert.deepEqual(pinchTransform(start, from, spread, limits), { x: -150, y: -100, scale: 2 });
  const moved = { first: { x: 110, y: 120 }, second: { x: 210, y: 120 } };
  assert.deepEqual(pinchTransform(start, from, moved, limits), { x: 10, y: 20, scale: 1 });
  const huge = { first: { x: 0, y: 100 }, second: { x: 1000, y: 100 } };
  assert.equal(pinchTransform(start, from, huge, limits).scale, 4);
  const collapsed = { first: { x: 100, y: 100 }, second: { x: 100, y: 100 } };
  assert.equal(pinchTransform(start, collapsed, spread, limits).scale, 1);
});

test("serializes CSS transforms for origin 0 0", () => {
  assert.equal(transformToCss(createTransform(10, -5, 1.5)), "translate(10px, -5px) scale(1.5)");
});
