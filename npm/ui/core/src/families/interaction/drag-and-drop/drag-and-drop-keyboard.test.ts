import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  addTarget,
  keydown,
  liveRegionText,
  mountDragAndDrop,
  rect,
  removeLiveRegions,
} from "./drag-and-drop-test-utils.ts";
import { pointer } from "../move/move-test-utils.ts";
import type { DragHarness, TargetHarness } from "./drag-and-drop-test-utils.ts";

interface KeyboardWorld {
  readonly harness: DragHarness;
  readonly alpha: TargetHarness;
  readonly bravo: TargetHarness;
  readonly unmount: () => void;
}

function mountKeyboardWorld(): KeyboardWorld {
  const harness = mountDragAndDrop({}, { label: "Task card" });
  const alpha = addTarget(harness.controller, "alpha", rect(0, 0, 100, 100), {
    label: "Backlog",
  });
  const bravo = addTarget(harness.controller, "bravo", rect(0, 200, 100, 300), {
    label: "In progress",
  });
  return {
    harness,
    alpha,
    bravo,
    unmount: () => {
      harness.unmount();
      removeLiveRegions();
    },
  };
}

test("Enter grabs, arrows cycle document-ordered targets, and Enter drops", () => {
  const world = mountKeyboardWorld();
  const { harness, alpha, bravo } = world;
  try {
    harness.host.dispatchEvent(keydown("Enter"));
    assert.equal(harness.controller.isDragging.value, true);
    assert.equal(harness.controller.targetKey.value, "alpha");
    assert.equal(alpha.events[0]?.type, "dropenter");
    assert.equal(alpha.events[0]?.pointerType, "keyboard");
    assert.equal(harness.controller.indicator.value?.edge, "inside");
    assert.match(liveRegionText() ?? "", /Picked up Task card\./);
    assert.match(liveRegionText() ?? "", /Over Backlog, drop target 1 of 2\./);

    harness.host.dispatchEvent(keydown("ArrowDown"));
    assert.equal(harness.controller.targetKey.value, "bravo");
    assert.match(liveRegionText() ?? "", /Over In progress, drop target 2 of 2\./);

    harness.host.dispatchEvent(keydown("ArrowDown"));
    assert.equal(harness.controller.targetKey.value, "alpha", "navigation must wrap");
    harness.host.dispatchEvent(keydown("ArrowUp"));
    assert.equal(harness.controller.targetKey.value, "bravo", "backwards must wrap too");
    harness.host.dispatchEvent(keydown("Home"));
    assert.equal(harness.controller.targetKey.value, "alpha");
    harness.host.dispatchEvent(keydown("End"));
    assert.equal(harness.controller.targetKey.value, "bravo");

    harness.host.dispatchEvent(keydown("Enter"));
    assert.equal(bravo.events.at(-1)?.type, "drop");
    const end = harness.events.at(-1);
    assert.equal(end?.type, "dragend");
    assert.equal(end?.targetKey, "bravo");
    assert.equal(end?.pointerType, "keyboard");
    assert.match(liveRegionText() ?? "", /Dropped Task card on In progress\./);
    assert.equal(harness.controller.isDragging.value, false);
  } finally {
    world.unmount();
  }
});

test("Escape and focus loss cancel a keyboard session with an announcement", () => {
  const world = mountKeyboardWorld();
  const { harness } = world;
  try {
    harness.host.dispatchEvent(keydown(" "));
    assert.equal(harness.controller.isDragging.value, true);
    harness.host.dispatchEvent(keydown("Escape"));
    assert.equal(harness.events.at(-1)?.isCanceled, true);
    assert.match(liveRegionText() ?? "", /Drag canceled\. Task card was not moved\./);

    harness.host.dispatchEvent(keydown(" "));
    assert.equal(harness.controller.isDragging.value, true);
    harness.host.dispatchEvent(new FocusEvent("focusout", { bubbles: true }));
    assert.equal(harness.controller.isDragging.value, false);
    assert.equal(harness.events.at(-1)?.isCanceled, true);
  } finally {
    world.unmount();
  }
});

test("keyboard grabs respect modifiers, disablement, opt-out, and empty target sets", () => {
  const harness = mountDragAndDrop({}, { keyboard: false });
  try {
    harness.host.dispatchEvent(keydown("Enter"));
    assert.equal(harness.controller.isDragging.value, false, "keyboard opt-out must hold");
  } finally {
    harness.unmount();
  }
  const world = mountKeyboardWorld();
  try {
    world.harness.host.dispatchEvent(keydown("Enter", { ctrlKey: true }));
    assert.equal(world.harness.controller.isDragging.value, false);
    const other = document.createElement("button");
    world.harness.host.append(other);
    other.dispatchEvent(keydown("Enter"));
    assert.equal(
      world.harness.controller.isDragging.value,
      false,
      "descendant activation must stay native",
    );
  } finally {
    world.unmount();
  }
  const empty = mountDragAndDrop();
  try {
    empty.host.dispatchEvent(keydown("Enter"));
    assert.equal(empty.controller.isDragging.value, false, "no valid target means no grab");
  } finally {
    empty.unmount();
    removeLiveRegions();
  }
});

test("keyboard sessions skip disabled targets and prefer declared edge order", () => {
  const harness = mountDragAndDrop();
  addTarget(harness.controller, "sleeping", rect(0, 0, 100, 100), { isDisabled: true });
  const ranked = addTarget(harness.controller, "ranked", rect(0, 200, 100, 300), {
    edges: ["top", "bottom"],
  });
  try {
    harness.host.dispatchEvent(keydown("Enter"));
    assert.equal(harness.controller.targetKey.value, "ranked");
    assert.equal(harness.controller.indicator.value?.edge, "top");
    assert.equal(ranked.events[0]?.edge, "top");
    harness.host.dispatchEvent(keydown("Escape"));
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("pointer announcements flow grab, move, and drop through the live region", () => {
  const world = mountKeyboardWorld();
  const { harness } = world;
  try {
    harness.host.dispatchEvent(pointer("pointerdown", 400, 400));
    document.dispatchEvent(pointer("pointermove", 420, 400));
    assert.equal(liveRegionText(), "Picked up Task card.");
    document.dispatchEvent(pointer("pointermove", 250, 50));
    assert.equal(liveRegionText(), "Over In progress, drop target 2 of 2.");
    document.dispatchEvent(pointer("pointermove", 400, 400));
    document.dispatchEvent(pointer("pointerup", 400, 400));
    assert.equal(liveRegionText(), "Task card released without a drop target.");
    const region = document.querySelector('[data-vize-ui="drag-and-drop-live"]');
    assert.equal(region?.getAttribute("role"), "status");
    assert.equal(region?.getAttribute("aria-live"), "assertive");
    assert.equal(region?.getAttribute("aria-atomic"), "true");
  } finally {
    world.unmount();
  }
});

test("custom announcement builders replace the built-in messages", () => {
  const harness = mountDragAndDrop({
    announcements: {
      grab: (context) => `lift ${context.sourceLabel}`,
      move: (context) => `over ${context.targetLabel} (${context.targetIndex})`,
      drop: () => null,
    },
  });
  addTarget(harness.controller, "zone", rect(0, 0, 100, 100), { label: "Zone" });
  try {
    harness.host.dispatchEvent(pointer("pointerdown", 400, 400));
    document.dispatchEvent(pointer("pointermove", 420, 400));
    assert.equal(liveRegionText(), "lift card");
    document.dispatchEvent(pointer("pointermove", 50, 50));
    assert.equal(liveRegionText(), "over Zone (1)");
    document.dispatchEvent(pointer("pointerup", 50, 50));
    assert.equal(liveRegionText(), "over Zone (1)", "a null builder must stay silent");
  } finally {
    harness.unmount();
    removeLiveRegions();
  }
});

test("announce() speaks manually and the live region disposes with the controller", () => {
  const harness = mountDragAndDrop();
  try {
    harness.controller.announce("Manual message");
    assert.equal(liveRegionText(), "Manual message");
    harness.controller.announce("Manual message");
    assert.equal(
      document.querySelector('[data-vize-ui="drag-and-drop-live"]')?.textContent,
      "Manual message\u00A0",
      "repeated messages must alternate so they re-announce",
    );
  } finally {
    harness.unmount();
  }
  assert.equal(liveRegionText(), null, "disposal must remove the live region");
});
