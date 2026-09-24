import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, ref } from "vue";

import { useDraggable } from "./draggable.ts";
import type { DragAxis } from "./draggable.ts";
import { asElement, FakeDocument, pointerEvent } from "./testing/fake-dom.ts";

function setup() {
  const document = new FakeDocument();
  const element = document.createElement();
  element.rect = { x: 10, y: 20, width: 50, height: 30 };
  return { document, element };
}

void test("drags relative to the grab point and reports lifecycle callbacks", () => {
  const { document, element } = setup();
  const events: string[] = [];
  const scope = effectScope();
  const drag = scope.run(() =>
    useDraggable(asElement(element), {
      initialValue: { x: 10, y: 20 },
      onStart: () => void events.push("start"),
      onMove: (position) => events.push(`move:${position.x},${position.y}`),
      onEnd: () => events.push("end"),
    }),
  );
  assert.ok(drag);
  assert.equal(drag.style.value, "left:10px;top:20px;");

  element.dispatchEvent(pointerEvent("pointerdown", { clientX: 15, clientY: 25 }));
  assert.equal(drag.isDragging.value, true);
  document.dispatchEvent(pointerEvent("pointermove", { clientX: 115, clientY: 75 }));
  assert.deepEqual(drag.position.value, { x: 110, y: 70 });
  document.dispatchEvent(pointerEvent("pointerup", { clientX: 115, clientY: 75 }));
  assert.equal(drag.isDragging.value, false);
  document.dispatchEvent(pointerEvent("pointermove", { clientX: 500, clientY: 500 }));
  assert.equal(drag.x.value, 110);
  assert.deepEqual(events, ["start", "move:110,70", "end"]);

  scope.stop();
  element.dispatchEvent(pointerEvent("pointerdown"));
  assert.equal(drag.isDragging.value, false);
});

void test("honors axis, bounds, handle, exact, disabled, and onStart cancellation", () => {
  const { document, element } = setup();
  const handle = document.createElement();
  const axis = ref<DragAxis>("x");
  const disabled = ref(false);
  let allow = true;
  const scope = effectScope();
  const drag = scope.run(() =>
    useDraggable(asElement(element), {
      handle: asElement(handle),
      axis,
      disabled,
      exact: true,
      bounds: { left: 0, top: 0, right: 100, bottom: 100 },
      onStart: () => allow,
    }),
  );
  assert.ok(drag);

  element.dispatchEvent(pointerEvent("pointerdown"));
  assert.equal(drag.isDragging.value, false);
  const down = pointerEvent("pointerdown", { clientX: 10, clientY: 20 });
  Object.defineProperty(down, "target", { value: handle });
  handle.dispatchEvent(down);
  document.dispatchEvent(pointerEvent("pointermove", { clientX: 500, clientY: 500 }));
  assert.deepEqual(drag.position.value, { x: 50, y: 0 });
  document.dispatchEvent(pointerEvent("pointerup"));

  disabled.value = true;
  handle.dispatchEvent(down);
  assert.equal(drag.isDragging.value, false);
  disabled.value = false;
  allow = false;
  handle.dispatchEvent(down);
  assert.equal(drag.isDragging.value, false);
  scope.stop();
});

void test("bounds may be a container element and secondary buttons are ignored", () => {
  const { document, element } = setup();
  const container = document.createElement();
  container.rect = { x: 0, y: 0, width: 80, height: 60 };
  const drag = useDraggable(asElement(element), { bounds: asElement(container) });
  element.dispatchEvent(pointerEvent("pointerdown", { button: 2 }));
  assert.equal(drag.isDragging.value, false);
  element.dispatchEvent(pointerEvent("pointerdown", { clientX: 10, clientY: 20 }));
  document.dispatchEvent(pointerEvent("pointermove", { clientX: -50, clientY: 900 }));
  assert.deepEqual(drag.position.value, { x: 0, y: 30 });
  drag.stop();
});
