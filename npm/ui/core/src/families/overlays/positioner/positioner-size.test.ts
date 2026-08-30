import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createPositioner } from "./positioner-runtime.ts";
import { computeAvailableSize, sizeStyle } from "./positioner-size.ts";
import type { Rect, VirtualElement } from "./positioner-types.ts";

function box(x: number, y: number, width: number, height: number): Rect {
  return { height, width, x, y };
}

function virtual(rect: Rect): VirtualElement {
  return { getBoundingClientRect: () => rect };
}

const viewport = box(0, 0, 1_000, 800);
const reference = box(100, 100, 80, 40);

test("measures available space on every side", () => {
  const shared = { collisionPadding: 8, offset: 8, reference, viewport };
  assert.deepEqual(computeAvailableSize({ ...shared, placement: "top" }), {
    height: 84,
    width: 984,
  });
  assert.deepEqual(computeAvailableSize({ ...shared, placement: "bottom-start" }), {
    height: 644,
    width: 984,
  });
  assert.deepEqual(computeAvailableSize({ ...shared, placement: "left" }), {
    height: 784,
    width: 84,
  });
  assert.deepEqual(computeAvailableSize({ ...shared, placement: "right-end" }), {
    height: 784,
    width: 804,
  });
});

test("clamps available space at an empty box", () => {
  const cramped = computeAvailableSize({
    offset: 200,
    placement: "top",
    reference,
    viewport,
  });
  assert.deepEqual(cramped, { height: 0, width: 1_000 });
});

test("publishes constraints and custom properties for the available space", () => {
  assert.equal(
    sizeStyle({ height: 652, width: 1_000 }),
    ";max-width:1000px;max-height:652px" +
      ";--vize-ui-positioner-available-width:1000px" +
      ";--vize-ui-positioner-available-height:652px",
  );
});

test("constrains the host to the available space when size is enabled", () => {
  const controller = createPositioner({
    flip: false,
    offset: 8,
    placement: "bottom",
    shift: false,
    size: true,
    viewport,
  });
  controller.setReference(virtual(reference));
  controller.setFloating(virtual(box(0, 0, 200, 100)));
  assert.equal(controller.availableWidth.value, 1_000);
  assert.equal(controller.availableHeight.value, 652);
  assert.equal(
    controller.style.value,
    "position:fixed;left:0px;top:0px;transform:translate(40px, 148px)" +
      ";max-width:1000px;max-height:652px" +
      ";--vize-ui-positioner-available-width:1000px" +
      ";--vize-ui-positioner-available-height:652px",
  );
  controller.dispose();
});

test("leaves the host style untouched when size is off", () => {
  const controller = createPositioner({
    flip: false,
    offset: 8,
    placement: "bottom",
    shift: false,
    viewport,
  });
  controller.setReference(virtual(reference));
  controller.setFloating(virtual(box(0, 0, 200, 100)));
  assert.equal(
    controller.style.value,
    "position:fixed;left:0px;top:0px;transform:translate(40px, 148px)",
  );
  assert.equal(controller.availableHeight.value, 652);
  controller.dispose();
});
