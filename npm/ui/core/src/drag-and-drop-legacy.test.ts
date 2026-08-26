import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  addTarget,
  mountDragAndDrop,
  rect,
  removeLiveRegions,
} from "./drag-and-drop-test-utils.ts";
import { mouse, touchEvent } from "./move-test-utils.ts";

test("legacy touch contacts drag, hit test, and drop through document listeners", () => {
  const harness = mountDragAndDrop();
  const zone = addTarget(harness.controller, "zone", rect(0, 0, 100, 100));
  try {
    harness.host.dispatchEvent(
      touchEvent("touchstart", [{ clientX: 200, clientY: 200, identifier: 9 }]),
    );
    document.dispatchEvent(
      touchEvent("touchmove", [{ clientX: 150, clientY: 150, identifier: 9 }]),
    );
    assert.equal(harness.events[0]?.type, "dragstart");
    assert.equal(harness.events[0]?.pointerType, "touch");
    document.dispatchEvent(touchEvent("touchmove", [{ clientX: 50, clientY: 50, identifier: 9 }]));
    assert.equal(zone.registration.isOver.value, true);
    document.dispatchEvent(touchEvent("touchmove", [{ clientX: 60, clientY: 60, identifier: 4 }]));
    assert.equal(
      harness.events.filter((event) => event.type === "dragmove").length,
      2,
      "unrelated touch identifiers must be ignored",
    );
    document.dispatchEvent(touchEvent("touchend", [{ clientX: 50, clientY: 50, identifier: 9 }]));
    assert.equal(zone.events.at(-1)?.type, "drop");
    assert.equal(harness.events.at(-1)?.type, "dragend");
    assert.equal(harness.events.at(-1)?.isCanceled, false);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("legacy touch cancel settles the session as canceled", () => {
  const harness = mountDragAndDrop();
  try {
    harness.host.dispatchEvent(
      touchEvent("touchstart", [{ clientX: 10, clientY: 10, identifier: 3 }]),
    );
    document.dispatchEvent(touchEvent("touchmove", [{ clientX: 60, clientY: 10, identifier: 3 }]));
    document.dispatchEvent(
      touchEvent("touchcancel", [{ clientX: 60, clientY: 10, identifier: 3 }]),
    );
    assert.equal(harness.events.at(-1)?.type, "dragend");
    assert.equal(harness.events.at(-1)?.isCanceled, true);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("legacy mouse environments drag through document mouse listeners", () => {
  const harness = mountDragAndDrop();
  const zone = addTarget(harness.controller, "zone", rect(0, 0, 100, 100));
  try {
    harness.host.dispatchEvent(mouse("mousedown", 200, 200));
    document.dispatchEvent(mouse("mousemove", 150, 150));
    assert.equal(harness.events[0]?.type, "dragstart");
    assert.equal(harness.events[0]?.pointerType, "mouse");
    document.dispatchEvent(mouse("mousemove", 50, 50));
    assert.equal(zone.registration.isOver.value, true);
    document.dispatchEvent(mouse("mouseup", 50, 50));
    assert.equal(zone.events.at(-1)?.type, "drop");
    assert.equal(harness.events.at(-1)?.type, "dragend");
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("compatibility mouse events after a recent touch are suppressed", () => {
  const harness = mountDragAndDrop();
  try {
    harness.host.dispatchEvent(
      touchEvent("touchstart", [{ clientX: 10, clientY: 10, identifier: 2 }], 1_000),
    );
    document.dispatchEvent(touchEvent("touchend", [{ clientX: 10, clientY: 10, identifier: 2 }]));
    const emulated = mouse("mousedown", 10, 10);
    Object.defineProperty(emulated, "timeStamp", { value: 1_200 });
    harness.host.dispatchEvent(emulated);
    document.dispatchEvent(mouse("mousemove", 80, 10));
    assert.deepEqual(harness.events, [], "the emulated mouse press must be ignored");
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("a native dragstart on the handle cancels the armed contact", () => {
  const harness = mountDragAndDrop();
  try {
    harness.host.dispatchEvent(mouse("mousedown", 10, 10));
    document.dispatchEvent(mouse("mousemove", 80, 10));
    assert.equal(harness.controller.isDragging.value, true);
    harness.host.dispatchEvent(new Event("dragstart", { bubbles: true, cancelable: true }));
    assert.equal(harness.controller.isDragging.value, false);
    assert.equal(harness.events.at(-1)?.isCanceled, true);
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});
