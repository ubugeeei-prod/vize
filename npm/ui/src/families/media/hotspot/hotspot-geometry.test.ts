import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  clampHotspotCoordinate,
  hotspotPolygonPoints,
  hotspotShapeContains,
  isPointInHotspotPolygon,
  nextHotspotInDirection,
  sanitizeHotspotHref,
} from "./hotspot-geometry.ts";

const triangle = [
  { x: 0, y: 0 },
  { x: 10, y: 0 },
  { x: 0, y: 10 },
];

test("point-in-polygon uses even-odd ray casting, including concave shapes", () => {
  assert.equal(isPointInHotspotPolygon({ x: 2, y: 2 }, triangle), true);
  assert.equal(isPointInHotspotPolygon({ x: 8, y: 8 }, triangle), false);
  assert.equal(isPointInHotspotPolygon({ x: 1, y: 1 }, triangle.slice(0, 2)), false);
  const concave = [
    { x: 0, y: 0 },
    { x: 10, y: 0 },
    { x: 10, y: 10 },
    { x: 5, y: 5 },
    { x: 0, y: 10 },
  ];
  assert.equal(isPointInHotspotPolygon({ x: 5, y: 8 }, concave), false);
  assert.equal(isPointInHotspotPolygon({ x: 5, y: 2 }, concave), true);
});

test("shapes contain points with inclusive rect and circle edges", () => {
  const rect = { type: "rect", x: 10, y: 10, width: 20, height: 10 } as const;
  assert.equal(hotspotShapeContains(rect, { x: 10, y: 20 }), true);
  assert.equal(hotspotShapeContains(rect, { x: 31, y: 15 }), false);
  const circle = { type: "circle", cx: 50, cy: 50, r: 10 } as const;
  assert.equal(hotspotShapeContains(circle, { x: 60, y: 50 }), true);
  assert.equal(hotspotShapeContains(circle, { x: 58, y: 58 }), false);
  assert.equal(hotspotShapeContains({ type: "polygon", points: triangle }, { x: 1, y: 1 }), true);
});

test("formats polygon points and clamps coordinates", () => {
  assert.equal(hotspotPolygonPoints(triangle), "0,0 10,0 0,10");
  assert.equal(clampHotspotCoordinate(-5), 0);
  assert.equal(clampHotspotCoordinate(105), 100);
  assert.equal(clampHotspotCoordinate(42.5), 42.5);
  assert.equal(clampHotspotCoordinate(Number.NaN), 0);
});

test("finds the nearest marker ahead in each direction", () => {
  const items = [
    { id: "center", x: 50, y: 50 },
    { id: "right-near-off-axis", x: 60, y: 80 },
    { id: "right-far-in-line", x: 90, y: 52 },
    { id: "up", x: 48, y: 10 },
    { id: "left", x: 10, y: 50 },
  ];
  assert.equal(nextHotspotInDirection(items, "center", "right"), "right-far-in-line");
  assert.equal(nextHotspotInDirection(items, "center", "up"), "up");
  assert.equal(nextHotspotInDirection(items, "center", "left"), "left");
  assert.equal(nextHotspotInDirection(items, "center", "down"), "right-near-off-axis");
  assert.equal(nextHotspotInDirection(items, "left", "left"), null);
  assert.equal(nextHotspotInDirection(items, "missing", "left"), null);
});

test("sanitizes region links", () => {
  assert.equal(sanitizeHotspotHref(" /rooms/porch "), "/rooms/porch");
  assert.equal(sanitizeHotspotHref("https://example.test/a"), "https://example.test/a");
  assert.equal(sanitizeHotspotHref("JavaScript:alert(1)"), undefined);
  assert.equal(sanitizeHotspotHref("data:text/html,x"), undefined);
  assert.equal(sanitizeHotspotHref("vbscript:x"), undefined);
  assert.equal(sanitizeHotspotHref("   "), undefined);
  assert.equal(sanitizeHotspotHref("/a\u0000b"), undefined);
});
