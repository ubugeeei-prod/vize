import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useMouse } from "./mouse.ts";
import { eventWith, pointerEvent, touchEvent } from "./testing/fake-dom.ts";

void test("exposes the initial value with no source during server rendering", () => {
  const mouse = useMouse({ target: null, initialValue: { x: 4, y: 2 } });
  assert.deepEqual([mouse.x.value, mouse.y.value, mouse.sourceType.value], [4, 2, null]);
});

void test("tracks page coordinates from mouse and touch input", () => {
  const target = new EventTarget();
  const scope = effectScope();
  const mouse = scope.run(() => useMouse({ target, resetOnTouchEnds: true }));
  assert.ok(mouse);

  target.dispatchEvent(pointerEvent("mousemove", { clientX: 10, clientY: 20 }));
  assert.deepEqual([mouse.x.value, mouse.y.value, mouse.sourceType.value], [10, 20, "mouse"]);
  target.dispatchEvent(touchEvent("touchmove", [{ x: 5, y: 6 }]));
  assert.deepEqual([mouse.x.value, mouse.y.value, mouse.sourceType.value], [5, 6, "touch"]);
  target.dispatchEvent(touchEvent("touchend", [{ x: 5, y: 6 }]));
  assert.deepEqual([mouse.x.value, mouse.y.value], [0, 0]);

  scope.stop();
  target.dispatchEvent(pointerEvent("mousemove", { clientX: 1, clientY: 1 }));
  assert.equal(mouse.x.value, 0);
});

void test("supports movement, custom extractors, disabled touch, and target swaps", async () => {
  const first = new EventTarget();
  const second = new EventTarget();
  const target = ref<EventTarget>(first);
  const scope = effectScope();
  const movement = scope.run(() => useMouse({ target, type: "movement", touch: false }));
  const custom = scope.run(() =>
    useMouse({ target: first, type: (input) => (input.clientX > 5 ? { x: 1, y: 1 } : null) }),
  );

  first.dispatchEvent(
    eventWith("mousemove", { clientX: 1, button: 0, movementX: 3, movementY: -2 }),
  );
  assert.deepEqual([movement?.x.value, movement?.y.value], [3, -2]);
  assert.equal(custom?.sourceType.value, null);
  first.dispatchEvent(touchEvent("touchmove", [{ x: 9, y: 9 }]));
  assert.equal(movement?.sourceType.value, "mouse");
  assert.equal(custom?.x.value, 1);

  target.value = second;
  await nextTick();
  second.dispatchEvent(
    eventWith("mousemove", { clientX: 1, button: 0, movementX: 7, movementY: 7 }),
  );
  assert.equal(movement?.x.value, 7);
  scope.stop();
});
