import assert from "node:assert/strict";

import { effectScope, ref, type Ref } from "vue";
import { test } from "vite-plus/test";

import { createMove, useMove } from "./move.ts";
import type { MoveController, MoveEvent, MoveProps } from "./move.ts";
import {
  dispatchReportingError,
  mountMove,
  mouse,
  moveEventNames,
  pointer,
  touchEvent,
} from "./move-test-utils.ts";

test("normalizes pointer deltas after movement and restores exact selection styles", () => {
  const harness = mountMove();
  harness.host.style.setProperty("user-select", "text", "important");
  harness.host.style.setProperty("-webkit-user-select", "contain");
  harness.host.dispatchEvent(pointer("pointerdown", 10, 20));
  assert.equal(harness.controller.isMoving.value, false);
  assert.equal(harness.host.style.getPropertyValue("user-select"), "none");

  document.dispatchEvent(pointer("pointermove", 10, 20));
  document.dispatchEvent(pointer("pointermove", 13, 25));
  document.dispatchEvent(pointer("pointermove", 15, 24));
  assert.equal(harness.controller.isMoving.value, true);
  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["movestart", "move", "move"],
  );
  assert.deepEqual(
    harness.events
      .filter(({ type }) => type === "move")
      .map(({ deltaX, deltaY }) => [deltaX, deltaY]),
    [
      [3, 5],
      [2, -1],
    ],
  );
  assert.ok(harness.events.every(Object.isFrozen));

  document.dispatchEvent(pointer("pointerup", 15, 24));
  assert.equal(harness.controller.isMoving.value, false);
  assert.equal(harness.events.at(-1)?.type, "moveend");
  assert.equal(harness.events.at(-1)?.isCanceled, false);
  assert.equal(harness.host.style.getPropertyValue("user-select"), "text");
  assert.equal(harness.host.style.getPropertyPriority("user-select"), "important");
  assert.equal(harness.host.style.getPropertyValue("-webkit-user-select"), "contain");
  harness.unmount();
});

test("owns one primary pointer and cancels exotic or disabled movement safely", () => {
  const disabled = ref(false);
  const harness = mountMove({ isDisabled: disabled });
  harness.host.dispatchEvent(pointer("pointerdown", 0, 0, { pointerId: 4, pointerType: "eraser" }));
  document.dispatchEvent(pointer("pointermove", 40, 40, { pointerId: 99 }));
  assert.equal(harness.events.length, 0);
  document.dispatchEvent(pointer("pointermove", 4, 6, { pointerId: 4, pointerType: "eraser" }));
  assert.equal(harness.events[0]?.pointerType, "pointer");

  disabled.value = true;
  document.dispatchEvent(pointer("pointermove", 5, 7, { pointerId: 4, pointerType: "eraser" }));
  assert.equal(harness.events.at(-1)?.type, "moveend");
  assert.equal(harness.events.at(-1)?.isCanceled, true);

  harness.host.dispatchEvent(pointer("pointerdown", 0, 0, { button: 2 }));
  harness.host.dispatchEvent(pointer("pointerdown", 0, 0, { isPrimary: false }));
  assert.equal(harness.controller.isMoving.value, false);
  harness.unmount();
});

test("arrow keys emit atomic, repeatable movement with a reactive step", () => {
  const step = ref(3);
  const harness = mountMove({ keyboardStep: step });
  for (const [key, expected] of [
    ["ArrowLeft", [-3, 0]],
    ["Right", [3, 0]],
    ["ArrowUp", [0, -3]],
    ["Down", [0, 3]],
  ] as const) {
    const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key });
    harness.host.dispatchEvent(event);
    assert.equal(event.defaultPrevented, true);
    const move = harness.events.at(-2);
    assert.deepEqual([move?.deltaX, move?.deltaY], expected);
    assert.equal(move?.pointerType, "keyboard");
  }
  assert.deepEqual(
    harness.events.map(({ type }) => type),
    Array.from({ length: 4 }, () => ["movestart", "move", "moveend"]).flat(),
  );

  step.value = 0;
  assert.throws(
    () =>
      harness.host.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "ArrowLeft" })),
    /VIZE_UI_MOVE_OPTION.*keyboardStep/,
  );
  harness.unmount();
});

test("modified, composing, descendant, and unrelated keys preserve native behavior", () => {
  const harness = mountMove();
  const child = document.createElement("span");
  harness.host.append(child);
  for (const event of [
    new KeyboardEvent("keydown", { bubbles: true, key: "ArrowLeft", altKey: true }),
    new KeyboardEvent("keydown", { bubbles: true, key: "ArrowLeft", ctrlKey: true }),
    new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }),
  ]) {
    harness.host.dispatchEvent(event);
    assert.equal(event.defaultPrevented, false);
  }
  child.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "ArrowLeft" }));
  const composing = new KeyboardEvent("keydown", { bubbles: true, key: "ArrowLeft" });
  Object.defineProperty(composing, "isComposing", { value: true });
  harness.host.dispatchEvent(composing);
  assert.equal(harness.events.length, 0);
  harness.unmount();
});

test("legacy touch owns one contact and suppresses its compatibility mouse event", () => {
  const isolated = document.implementation.createHTMLDocument("legacy move");
  const host = isolated.createElement("div");
  isolated.body.append(host);
  const events: MoveEvent[] = [];
  const controller = createMove({
    onMoveStart: (event) => events.push(event),
    onMove: (event) => events.push(event),
    onMoveEnd: (event) => events.push(event),
  });
  for (const [property, type] of Object.entries(moveEventNames) as Array<
    [keyof MoveProps, string]
  >) {
    host.addEventListener(type, controller.moveProps[property] as EventListener);
  }

  host.dispatchEvent(touchEvent("touchstart", [{ identifier: 31, clientX: 4, clientY: 8 }], 100));
  isolated.dispatchEvent(touchEvent("touchmove", [{ identifier: 99, clientX: 50, clientY: 60 }]));
  isolated.dispatchEvent(touchEvent("touchmove", [{ identifier: 31, clientX: 9, clientY: 12 }]));
  isolated.dispatchEvent(
    touchEvent("touchend", [{ identifier: 31, clientX: 9, clientY: 12 }], 200),
  );
  assert.deepEqual(
    events.map(({ type }) => type),
    ["movestart", "move", "moveend"],
  );
  assert.deepEqual([events[1]?.deltaX, events[1]?.deltaY], [5, 4]);
  assert.ok(events.every(({ pointerType }) => pointerType === "touch"));

  const emulated = new MouseEvent("mousedown", { bubbles: true, button: 0 });
  Object.defineProperty(emulated, "timeStamp", { value: 201 });
  host.dispatchEvent(emulated);
  isolated.dispatchEvent(
    new MouseEvent("mousemove", { bubbles: true, clientX: 100, clientY: 100 }),
  );
  assert.equal(events.length, 3);
  controller.dispose();
});

test("legacy mouse fallback emits deltas when Pointer Events are unavailable", () => {
  const isolated = document.implementation.createHTMLDocument("legacy mouse move");
  const host = isolated.createElement("div");
  isolated.body.append(host);
  const deltas: Array<readonly [number, number]> = [];
  const controller = createMove({
    onMove: ({ deltaX, deltaY }) => deltas.push([deltaX, deltaY]),
  });
  host.addEventListener("mousedown", controller.moveProps.onMousedown);
  host.dispatchEvent(mouse("mousedown", 1, 2));
  isolated.dispatchEvent(mouse("mousemove", 5, 9));
  isolated.dispatchEvent(mouse("mouseup", 5, 9));
  assert.deepEqual(deltas, [[4, 7]]);
  controller.dispose();
});

test("manual, drag, visibility, blur, and scope cancellation settle ownership", () => {
  for (const finish of ["manual", "drag", "hidden", "blur"] as const) {
    const harness = mountMove();
    harness.host.dispatchEvent(pointer("pointerdown", 0, 0));
    document.dispatchEvent(pointer("pointermove", 2, 3));
    if (finish === "manual") {
      assert.equal(harness.controller.cancel(), true);
    } else if (finish === "drag") {
      harness.host.dispatchEvent(new DragEvent("dragstart", { bubbles: true }));
    } else if (finish === "hidden") {
      const descriptor = Object.getOwnPropertyDescriptor(document, "visibilityState");
      Object.defineProperty(document, "visibilityState", { configurable: true, value: "hidden" });
      try {
        document.dispatchEvent(new Event("visibilitychange"));
      } finally {
        if (descriptor) Object.defineProperty(document, "visibilityState", descriptor);
        else delete (document as { visibilityState?: DocumentVisibilityState }).visibilityState;
      }
    } else {
      window.dispatchEvent(new Event("blur"));
    }
    assert.equal(harness.controller.isMoving.value, false);
    assert.equal(harness.events.at(-1)?.isCanceled, true);
    harness.unmount();
  }

  assert.throws(() => useMove(), /VIZE_UI_MOVE_SETUP/);
  const scope = effectScope();
  const scoped = scope.run(() => useMove())!;
  scope.stop();
  assert.throws(() => scoped.cancel(), /VIZE_UI_MOVE_DISPOSED/);
});

test("a stationary pointer attempt ends silently and remains manually cancelable", () => {
  const harness = mountMove();
  harness.host.dispatchEvent(pointer("pointerdown", 1, 2));
  assert.equal(harness.controller.cancel(), true);
  assert.equal(harness.controller.cancel(), false);
  assert.deepEqual(harness.events, []);
  harness.unmount();
});

test("reentrant cancellation suppresses a stale delta and callback failures aggregate", () => {
  let controller!: MoveController;
  const events: string[] = [];
  const host = document.createElement("div");
  document.body.append(host);
  controller = createMove({
    onMoveStart: () => {
      events.push("start");
      controller.cancel();
    },
    onMove: () => events.push("move"),
    onMoveEnd: () => events.push("end"),
  });
  host.addEventListener("pointerdown", controller.moveProps.onPointerdown);
  host.dispatchEvent(pointer("pointerdown", 0, 0));
  document.dispatchEvent(pointer("pointermove", 1, 1));
  assert.deepEqual(events, ["start", "end"]);
  assert.equal(controller.isMoving.value, false);
  controller.dispose();
  host.remove();

  const keyboardEvents: MoveEvent[] = [];
  let keyboardController!: MoveController;
  const keyboardHost = document.createElement("div");
  document.body.append(keyboardHost);
  keyboardController = createMove({
    onMoveStart: (event) => {
      keyboardEvents.push(event);
      keyboardController.cancel();
    },
    onMove: (event) => keyboardEvents.push(event),
    onMoveEnd: (event) => keyboardEvents.push(event),
  });
  keyboardHost.addEventListener("keydown", keyboardController.moveProps.onKeydown);
  keyboardHost.dispatchEvent(
    new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "ArrowRight" }),
  );
  assert.deepEqual(
    keyboardEvents.map(({ type }) => type),
    ["movestart", "moveend"],
  );
  assert.equal(keyboardEvents[1]?.isCanceled, true);
  assert.equal(keyboardEvents[1]?.originalEvent, null);
  keyboardController.dispose();
  keyboardHost.remove();

  const failing = mountMove({
    onMoveStart: () => {
      throw new Error("start failed");
    },
    onMove: () => {
      throw new Error("move failed");
    },
  });
  failing.host.dispatchEvent(pointer("pointerdown", 0, 0));
  assert.throws(() => document.dispatchEvent(pointer("pointermove", 1, 1)), AggregateError);
  assert.equal(failing.controller.isMoving.value, true);
  failing.controller.dispose();
  failing.host.remove();
});

test("nested move hosts give ownership to the nearest bound target", () => {
  const outer = mountMove();
  const inner = mountMove();
  outer.host.append(inner.host);
  inner.host.dispatchEvent(pointer("pointerdown", 0, 0));
  document.dispatchEvent(pointer("pointermove", 2, 1));
  document.dispatchEvent(pointer("pointerup", 2, 1));
  assert.deepEqual(
    inner.events.map(({ type }) => type),
    ["movestart", "move", "moveend"],
  );
  assert.deepEqual(outer.events, []);
  inner.unmount();
  outer.unmount();
});

test("rejects invalid runtime options with stable diagnostics", () => {
  assert.throws(() => createMove({ keyboardStep: -1 }), /VIZE_UI_MOVE_OPTION.*keyboardStep/);
  assert.throws(
    () => createMove({ onMoveStart: "callback" as never }),
    /VIZE_UI_MOVE_OPTION.*onMoveStart/,
  );
});

test("invalid reactive disablement settles movement before surfacing its diagnostic", () => {
  const disabled = ref<unknown>(false);
  const harness = mountMove({ isDisabled: disabled as Ref<boolean> });
  harness.host.style.setProperty("user-select", "text", "important");
  harness.host.dispatchEvent(pointer("pointerdown", 0, 0));
  document.dispatchEvent(pointer("pointermove", 2, 3));
  disabled.value = "invalid";

  const reportedError = dispatchReportingError(() =>
    document.dispatchEvent(pointer("pointermove", 4, 5)),
  );
  assert.match(String(reportedError), /VIZE_UI_MOVE_OPTION.*isDisabled/);
  assert.equal(harness.controller.isMoving.value, false);
  assert.equal(harness.events.at(-1)?.type, "moveend");
  assert.equal(harness.events.at(-1)?.isCanceled, true);
  assert.equal(harness.host.style.getPropertyValue("user-select"), "text");
  assert.equal(harness.host.style.getPropertyPriority("user-select"), "important");
  harness.controller.dispose();
  harness.host.remove();
});

test("disposal is terminal and exhaustive after a selection cleanup failure", () => {
  const harness = mountMove();
  harness.host.style.setProperty("user-select", "text", "important");
  harness.host.dispatchEvent(pointer("pointerdown", 0, 0));
  document.dispatchEvent(pointer("pointermove", 2, 3));

  const style = harness.host.style;
  const originalRemoveProperty = style.removeProperty.bind(style);
  style.removeProperty = function removeProperty(property: string): string {
    if (property === "-webkit-user-select") throw new Error("restore failed");
    return originalRemoveProperty(property);
  };
  try {
    assert.throws(() => harness.controller.dispose(), /restore failed/);
  } finally {
    style.removeProperty = originalRemoveProperty;
  }

  assert.equal(harness.controller.isMoving.value, false);
  assert.equal(harness.host.style.getPropertyValue("user-select"), "text");
  assert.equal(harness.host.style.getPropertyPriority("user-select"), "important");
  assert.throws(() => harness.controller.cancel(), /VIZE_UI_MOVE_DISPOSED/);
  harness.host.remove();
});
