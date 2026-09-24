import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { toPointerKind, usePointer } from "./pointer.ts";
import { pointerEvent } from "./testing/fake-dom.ts";

void test("filters device kinds, tracks the snapshot, and cleans up", () => {
  const target = new EventTarget();
  const scope = effectScope();
  const pointer = scope.run(() => usePointer({ target, pointerTypes: ["pen"] }));
  assert.ok(pointer);
  assert.equal(pointer.state.pointerType, null);

  target.dispatchEvent(pointerEvent("pointermove", { pointerType: "mouse", clientX: 3 }));
  assert.equal(pointer.state.x, 0);
  target.dispatchEvent(
    pointerEvent("pointerdown", { pointerType: "pen", clientX: 3, pointerId: 7 }),
  );
  assert.deepEqual(
    [pointer.state.x, pointer.state.pointerId, pointer.state.pointerType, pointer.isInside.value],
    [3, 7, "pen", true],
  );
  target.dispatchEvent(new Event("pointerleave"));
  assert.equal(pointer.isInside.value, false);

  scope.stop();
  target.dispatchEvent(pointerEvent("pointermove", { pointerType: "pen", clientX: 9 }));
  assert.equal(pointer.state.x, 3);
});

void test("keeps the initial snapshot without a target and narrows pointer kinds", () => {
  const pointer = usePointer({ target: null, initialValue: { x: 5, pressure: 1 } });
  assert.deepEqual([pointer.state.x, pointer.state.pressure], [5, 1]);
  assert.equal(toPointerKind("touch"), "touch");
  assert.equal(toPointerKind("eye-tracker"), null);
});
