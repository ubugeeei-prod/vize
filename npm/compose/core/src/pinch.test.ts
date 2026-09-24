import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { usePinch } from "./pinch.ts";
import type { PinchSnapshot } from "./pinch.ts";
import { asElement, FakeDocument, pointerEvent } from "./testing/fake-dom.ts";

void test("derives scale, rotation, and origin from two pointers", () => {
  const document = new FakeDocument();
  const element = document.createElement();
  const phases: string[] = [];
  let last: PinchSnapshot | undefined;
  const scope = effectScope();
  const pinch = scope.run(() =>
    usePinch(asElement(element), {
      onPinchStart: () => phases.push("start"),
      onPinch: (snapshot) => {
        last = snapshot;
        phases.push("pinch");
      },
      onPinchEnd: () => phases.push("end"),
    }),
  );
  assert.ok(pinch);
  assert.deepEqual([pinch.scale.value, pinch.rotation.value], [1, 0]);

  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 1, clientX: 0, clientY: 0 }));
  assert.equal(pinch.isPinching.value, false);
  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 2, clientX: 100, clientY: 0 }));
  assert.equal(pinch.isPinching.value, true);
  assert.equal(pinch.initialDistance.value, 100);
  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 3, clientX: 9, clientY: 9 }));

  element.dispatchEvent(pointerEvent("pointermove", { pointerId: 2, clientX: 0, clientY: 200 }));
  assert.equal(pinch.scale.value, 2);
  assert.equal(pinch.rotation.value, 90);
  assert.deepEqual({ ...pinch.origin }, { x: 0, y: 100 });
  assert.deepEqual(last, { scale: 2, rotation: 90, origin: { x: 0, y: 100 }, distance: 200 });

  element.dispatchEvent(pointerEvent("pointerup", { pointerId: 1 }));
  assert.equal(pinch.isPinching.value, false);
  assert.equal(pinch.scale.value, 2);
  assert.deepEqual(phases, ["start", "pinch", "end"]);
  pinch.reset();
  assert.equal(pinch.scale.value, 1);

  scope.stop();
  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 4 }));
  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 5 }));
  assert.equal(pinch.isPinching.value, false);
});

void test("normalizes rotation across the ±180° seam", () => {
  const document = new FakeDocument();
  const element = document.createElement();
  const pinch = usePinch(asElement(element));
  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 1, clientX: 0, clientY: 0 }));
  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 2, clientX: -100, clientY: 1 }));
  element.dispatchEvent(pointerEvent("pointermove", { pointerId: 2, clientX: -100, clientY: -1 }));
  assert.ok(Math.abs(pinch.rotation.value) < 5);
  pinch.stop();
});
