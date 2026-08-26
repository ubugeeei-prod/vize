import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import { createPositioner, usePositioner } from "./positioner.ts";
import PositionerArrow from "./positioner-arrow.vue";
import Positioner from "./positioner.vue";
import type { Rect, VirtualElement } from "./positioner-types.ts";

function box(x: number, y: number, width: number, height: number): Rect {
  return { height, width, x, y };
}

function virtual(rect: Rect): VirtualElement {
  return { getBoundingClientRect: () => rect };
}

const viewport = box(0, 0, 1_000, 800);

test("renders a fixed host before the first measure", () => {
  const handle = mountInteraction(Positioner, { slots: { default: "Menu" } });
  try {
    assert.equal(handle.root().getAttribute("data-vize-ui"), "positioner");
    assert.equal(handle.root().getAttribute("data-vize-positioner-ready"), "false");
    assert.equal(handle.root().getAttribute("data-vize-placement"), "bottom");
    assert.match(handle.root().getAttribute("style") ?? "", /translate\(0px, 0px\)/);
    assert.equal(handle.root().textContent, "Menu");
  } finally {
    handle.unmount();
  }
});

test("places below a virtual reference", async () => {
  const controller = createPositioner({
    flip: false,
    offset: 8,
    placement: "bottom",
    shift: false,
    viewport,
  });
  controller.setReference(virtual(box(100, 100, 80, 40)));
  controller.setFloating(virtual(box(0, 0, 200, 100)));
  assert.equal(controller.x.value, 40);
  assert.equal(controller.y.value, 148);
  assert.equal(controller.ready.value, true);
  assert.equal(
    controller.style.value,
    "position:fixed;left:0px;top:0px;transform:translate(40px, 148px)",
  );
  controller.dispose();
});

test("clamps the arrow along the facing edge", () => {
  const controller = createPositioner({
    flip: false,
    offset: 8,
    placement: "bottom",
    shift: false,
    viewport,
  });
  controller.setReference(virtual(box(100, 100, 80, 40)));
  controller.setFloating(virtual(box(0, 0, 200, 100)));
  controller.setArrow(virtual(box(0, 0, 10, 10)));
  assert.equal(controller.arrowX.value, 95);
  assert.equal(controller.arrowY.value, -10);
  controller.dispose();
});

test("rejects an arrow outside Positioner", () => {
  assert.throws(() => mountInteraction(PositionerArrow), /VIZE_UI_CONTEXT_MISSING/);
});

test("exposes the rendered element for composition", () => {
  const handle = mountInteraction(Positioner, { slots: { default: "Visible" } });
  try {
    const exposed = handle.exposes<{ element: HTMLElement | null }>();
    assert.ok(exposed.element === handle.root());
  } finally {
    handle.unmount();
  }
});

test("renders an arrow inside the positioner", async () => {
  const Root = defineComponent({
    name: "PositionerArrowProbe",
    setup() {
      return () =>
        h(
          Positioner,
          { reference: virtual(box(20, 20, 40, 20)), viewport },
          {
            default: () => [h(PositionerArrow), "Menu"],
          },
        );
    },
  });
  const handle = mountInteraction(Root);
  try {
    await nextTick();
    const arrow = handle.root().querySelector('[data-vize-ui="positioner-arrow"]');
    assert.ok(arrow instanceof HTMLElement);
    assert.match(arrow.getAttribute("style") ?? "", /position:\s*absolute/);
  } finally {
    handle.unmount();
  }
});

test("rejects composable use outside an effect scope", () => {
  assert.throws(() => usePositioner(), /VIZE_UI_POSITIONER_SETUP/);
});

test("disposes with the current effect scope", () => {
  const scope = effectScope();
  const controller = scope.run(() => usePositioner());
  assert.equal(controller?.ready.value, false);
  scope.stop();
  assert.throws(() => controller?.update(), /VIZE_UI_POSITIONER_DISPOSED/);
});
