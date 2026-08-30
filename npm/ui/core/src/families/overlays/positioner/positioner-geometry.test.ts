import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { computePosition } from "./positioner-geometry.ts";
import type { Rect } from "./positioner-types.ts";

function box(x: number, y: number, width: number, height: number): Rect {
  return { height, width, x, y };
}

const viewport = box(0, 0, 1_000, 800);
const floating = { height: 100, width: 200 };

test("places below a centered reference", () => {
  const result = computePosition({
    floating,
    offset: 8,
    placement: "bottom",
    reference: box(100, 100, 80, 40),
    flip: false,
    shift: false,
    hide: false,
    viewport,
  });
  assert.equal(result.x, 40);
  assert.equal(result.y, 148);
  assert.equal(result.placement, "bottom");
  assert.equal(result.hidden, false);
});

test("flips to the opposite side when it overflows less", () => {
  const result = computePosition({
    floating,
    placement: "bottom",
    reference: box(100, 720, 80, 40),
    flip: true,
    shift: false,
    hide: false,
    viewport,
  });
  assert.equal(result.placement, "top");
  assert.equal(result.y, 620);
});

test("shifts the floating box back inside the viewport", () => {
  const result = computePosition({
    collisionPadding: 8,
    floating,
    placement: "bottom",
    reference: box(900, 100, 80, 40),
    flip: false,
    shift: true,
    hide: false,
    viewport,
  });
  assert.equal(result.x, 792);
});

test("hides when the reference leaves the viewport", () => {
  const hidden = computePosition({
    floating,
    placement: "bottom",
    reference: box(100, -80, 80, 40),
    flip: false,
    shift: false,
    hide: true,
    viewport,
  });
  const visible = computePosition({
    floating,
    placement: "bottom",
    reference: box(100, 100, 80, 40),
    flip: false,
    shift: false,
    hide: true,
    viewport,
  });
  assert.equal(hidden.hidden, true);
  assert.equal(visible.hidden, false);
});

test("mirrors start alignment when the writing direction is rtl", () => {
  const ltr = computePosition({
    floating,
    placement: "top-start",
    reference: box(100, 200, 80, 40),
    rtl: false,
    flip: false,
    shift: false,
    hide: false,
    viewport,
  });
  const rtl = computePosition({
    floating,
    placement: "top-start",
    reference: box(100, 200, 80, 40),
    rtl: true,
    flip: false,
    shift: false,
    hide: false,
    viewport,
  });
  assert.equal(ltr.x, 100);
  assert.equal(rtl.x, -20);
  assert.equal(ltr.placement, "top-start");
  assert.equal(rtl.placement, "top-start");
});

test("clamps the arrow along the facing edge", () => {
  const result = computePosition({
    arrow: box(0, 0, 10, 10),
    floating,
    offset: 8,
    placement: "bottom",
    reference: box(100, 100, 80, 40),
    flip: false,
    shift: false,
    hide: false,
    viewport,
  });
  assert.equal(result.arrowX, 95);
  assert.equal(result.arrowY, -10);
});
