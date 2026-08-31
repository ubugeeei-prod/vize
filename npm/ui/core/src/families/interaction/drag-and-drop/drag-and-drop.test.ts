import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, ref } from "vue";

import { createDragAndDrop, useDragAndDrop } from "./drag-and-drop.ts";
import {
  addTarget,
  mountDragAndDrop,
  rect,
  removeLiveRegions,
} from "./drag-and-drop-test-utils.ts";
import { pointer } from "../move/move-test-utils.ts";

function drag(host: Element, x: number, y: number): void {
  host.dispatchEvent(pointer("pointerdown", x, y));
}

function move(x: number, y: number): void {
  document.dispatchEvent(pointer("pointermove", x, y));
}

function release(x: number, y: number): void {
  document.dispatchEvent(pointer("pointerup", x, y));
}

test("starts after the start distance and drops on the target under the pointer", () => {
  const harness = mountDragAndDrop();
  const alpha = addTarget(harness.controller, "alpha", rect(0, 0, 100, 100));
  const bravo = addTarget(harness.controller, "bravo", rect(0, 200, 100, 300));
  try {
    drag(harness.host, 10, 10);
    move(12, 11);
    assert.equal(harness.events.length, 0, "sub-threshold movement must stay silent");
    move(20, 10);
    assert.equal(harness.events[0]?.type, "dragstart");
    assert.deepEqual(harness.events[0]?.payload, { kind: "card", data: { id: 1 } });
    assert.equal(harness.controller.isDragging.value, true);
    assert.equal(harness.controller.sourceKey.value, "card");
    assert.equal(harness.source.isDragging.value, true);

    move(50, 50);
    assert.equal(alpha.events[0]?.type, "dropenter");
    assert.equal(alpha.registration.isOver.value, true);
    assert.equal(harness.controller.targetKey.value, "alpha");
    assert.equal(harness.controller.indicator.value?.edge, "inside");

    move(250, 50);
    assert.equal(alpha.events.at(-1)?.type, "dropleave");
    assert.equal(alpha.registration.isOver.value, false);
    assert.equal(bravo.events[0]?.type, "dropenter");

    release(250, 50);
    assert.equal(bravo.events.at(-1)?.type, "drop");
    const end = harness.events.at(-1);
    assert.equal(end?.type, "dragend");
    assert.equal(end?.targetKey, "bravo");
    assert.equal(end?.isCanceled, false);
    assert.equal(harness.controller.isDragging.value, false);
    assert.equal(harness.controller.targetKey.value, null);
    assert.equal(harness.controller.indicator.value, null);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("settles a sub-threshold press silently and can re-arm afterwards", () => {
  const harness = mountDragAndDrop();
  addTarget(harness.controller, "alpha", rect(0, 0, 100, 100));
  try {
    drag(harness.host, 10, 10);
    move(12, 12);
    release(12, 12);
    assert.deepEqual(harness.events, []);
    drag(harness.host, 10, 10);
    move(40, 10);
    assert.equal(harness.events[0]?.type, "dragstart");
    release(40, 10);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("prefers the innermost containing target and skips rejecting targets", () => {
  const harness = mountDragAndDrop();
  const outer = addTarget(harness.controller, "outer", rect(0, 0, 300, 300));
  const inner = addTarget(harness.controller, "inner", rect(50, 50, 150, 150), {}, outer.element);
  addTarget(harness.controller, "greedy", rect(0, 0, 300, 300), {
    accepts: () => false,
  });
  addTarget(harness.controller, "sleeping", rect(0, 0, 300, 300), { isDisabled: true });
  try {
    drag(harness.host, 400, 400);
    move(100, 100);
    assert.equal(harness.controller.targetKey.value, "inner");
    assert.equal(inner.registration.isOver.value, true);
    move(250, 250);
    assert.equal(harness.controller.targetKey.value, "outer");
    assert.equal(outer.events.at(-1)?.type, "dropenter");
    release(250, 250);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("resolves directional edges with indicator line and placeholder geometry", () => {
  const harness = mountDragAndDrop();
  const zone = addTarget(harness.controller, "zone", rect(0, 0, 100, 100), {
    edges: ["top", "bottom"],
  });
  try {
    drag(harness.host, 400, 400);
    move(50, 20);
    assert.equal(harness.controller.indicator.value?.edge, "top");
    assert.deepEqual(harness.controller.indicator.value?.line, rect(0, 0, 0, 100));
    assert.deepEqual(harness.controller.indicator.value?.rect, rect(0, 0, 100, 100));
    move(50, 80);
    assert.equal(harness.controller.indicator.value?.edge, "bottom");
    assert.deepEqual(harness.controller.indicator.value?.line, rect(100, 0, 100, 100));
    const kinds = zone.events.map((event) => event.type);
    assert.deepEqual(kinds.slice(0, 2), ["dropenter", "dropmove"]);
    release(50, 80);
    assert.equal(zone.events.at(-1)?.edge, "bottom");
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("cancels on Escape with a canceled dragend and a leave for the target", () => {
  const harness = mountDragAndDrop();
  const zone = addTarget(harness.controller, "zone", rect(0, 0, 100, 100));
  try {
    drag(harness.host, 200, 200);
    move(50, 50);
    assert.equal(zone.registration.isOver.value, true);
    document.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
    );
    const end = harness.events.at(-1);
    assert.equal(end?.type, "dragend");
    assert.equal(end?.isCanceled, true);
    assert.equal(end?.targetKey, null);
    assert.equal(zone.events.at(-1)?.type, "dropleave");
    assert.equal(zone.registration.isOver.value, false);
    assert.equal(harness.controller.isDragging.value, false);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("reports a targetless drop without cancellation", () => {
  const harness = mountDragAndDrop();
  addTarget(harness.controller, "zone", rect(0, 0, 100, 100));
  try {
    drag(harness.host, 200, 200);
    move(400, 400);
    release(400, 400);
    const end = harness.events.at(-1);
    assert.equal(end?.type, "dragend");
    assert.equal(end?.targetKey, null);
    assert.equal(end?.isCanceled, false);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("reactive disablement cancels the active session before new movement", () => {
  const disabled = ref(false);
  const harness = mountDragAndDrop({ isDisabled: disabled });
  try {
    drag(harness.host, 10, 10);
    move(60, 10);
    assert.equal(harness.controller.isDragging.value, true);
    disabled.value = true;
    move(80, 10);
    const end = harness.events.at(-1);
    assert.equal(end?.type, "dragend");
    assert.equal(end?.isCanceled, true);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("auto-scrolls a container while the pointer holds its edge band", () => {
  const container = { scrollLeft: 0, scrollTop: 0 } as unknown as Element;
  const harness = mountDragAndDrop({
    autoScroll: {
      container: () => container,
      getRect: () => rect(0, 0, 100, 100),
      threshold: 20,
      speed: 10,
    },
  });
  try {
    drag(harness.host, 50, 50);
    move(50, 40);
    assert.equal(container.scrollTop, 0, "the center must not scroll");
    move(50, 95);
    assert.ok(container.scrollTop > 0, "the bottom band must scroll down");
    const scrolled = container.scrollTop;
    move(5, 50);
    assert.ok(container.scrollLeft < 0, "the left band must scroll backwards");
    assert.equal(container.scrollTop, scrolled, "the vertical center must rest");
    release(5, 50);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("source disposal cancels its own session and frees the key", () => {
  const harness = mountDragAndDrop();
  try {
    drag(harness.host, 10, 10);
    move(60, 10);
    harness.source.dispose();
    const end = harness.events.at(-1);
    assert.equal(end?.type, "dragend");
    assert.equal(end?.isCanceled, true);
    const again = harness.controller.registerSource({ key: "card" });
    again.dispose();
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("target disposal while hovered clears over state without callbacks", () => {
  const harness = mountDragAndDrop();
  const zone = addTarget(harness.controller, "zone", rect(0, 0, 100, 100));
  try {
    drag(harness.host, 200, 200);
    move(50, 50);
    assert.equal(harness.controller.targetKey.value, "zone");
    const before = zone.events.length;
    zone.registration.dispose();
    assert.equal(zone.events.length, before, "disposal must not fire target callbacks");
    assert.equal(harness.controller.targetKey.value, null);
    assert.equal(harness.controller.indicator.value, null);
    release(50, 50);
    assert.equal(harness.events.at(-1)?.targetKey, null);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("validates options, rejects duplicate keys, and is terminal after disposal", () => {
  assert.throws(
    () => createDragAndDrop({ onDragStart: 1 as never }),
    /VIZE_UI_DRAG_AND_DROP_OPTION/,
  );
  assert.throws(() => createDragAndDrop({ startDistance: -1 }), /VIZE_UI_DRAG_AND_DROP_OPTION/);
  const controller = createDragAndDrop();
  controller.registerSource({ key: "card" });
  assert.throws(() => controller.registerSource({ key: "card" }), /duplicate key/);
  controller.registerTarget({ key: "zone", element: () => null });
  assert.throws(() => controller.registerTarget({ key: "zone", element: () => null }), /duplicate/);
  assert.throws(
    () => controller.registerTarget({ key: "bad", element: () => null, edges: [] }),
    /VIZE_UI_DRAG_AND_DROP_OPTION/,
  );
  assert.equal(controller.cancel(), false);
  controller.dispose();
  controller.dispose();
  assert.throws(() => controller.cancel(), /VIZE_UI_DRAG_AND_DROP_DISPOSED/);
  assert.throws(() => controller.registerSource({ key: "late" }), /DISPOSED/);
});

test("useDragAndDrop requires a scope and disposes with it", () => {
  assert.throws(() => useDragAndDrop(), /VIZE_UI_DRAG_AND_DROP_SETUP/);
  const scope = effectScope();
  const controller = scope.run(() => useDragAndDrop());
  assert.ok(controller);
  scope.stop();
  assert.throws(() => controller.cancel(), /VIZE_UI_DRAG_AND_DROP_DISPOSED/);
});
