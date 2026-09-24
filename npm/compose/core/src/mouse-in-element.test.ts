import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useMouseInElement } from "./mouse-in-element.ts";
import { asElement, FakeDocument, pointerEvent } from "./testing/fake-dom.ts";

void test("reports element-relative coordinates and the outside flag", () => {
  const document = new FakeDocument();
  const element = document.createElement();
  element.rect = { x: 100, y: 50, width: 200, height: 100 };
  const target = new EventTarget();
  const scope = effectScope();
  const mouse = scope.run(() => useMouseInElement(asElement(element), { target }));
  assert.ok(mouse);
  assert.equal(mouse.isOutside.value, true);

  target.dispatchEvent(pointerEvent("mousemove", { clientX: 150, clientY: 70 }));
  assert.deepEqual(
    [mouse.elementX.value, mouse.elementY.value, mouse.isOutside.value],
    [50, 20, false],
  );
  assert.deepEqual([mouse.elementWidth.value, mouse.elementPositionX.value], [200, 100]);

  target.dispatchEvent(pointerEvent("mousemove", { clientX: 10, clientY: 10 }));
  assert.deepEqual([mouse.elementX.value, mouse.isOutside.value], [-90, true]);
  scope.stop();
});

void test("freezes outside positions when handleOutside is disabled", () => {
  const document = new FakeDocument();
  const element = document.createElement();
  element.rect = { x: 0, y: 0, width: 10, height: 10 };
  const target = new EventTarget();
  const scope = effectScope();
  const mouse = scope.run(() =>
    useMouseInElement(asElement(element), { target, handleOutside: false }),
  );
  target.dispatchEvent(pointerEvent("mousemove", { clientX: 5, clientY: 5 }));
  target.dispatchEvent(pointerEvent("mousemove", { clientX: 50, clientY: 50 }));
  assert.deepEqual([mouse?.elementX.value, mouse?.isOutside.value], [5, true]);
  mouse?.stop();
  scope.stop();
});
