import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { swipeDirection, usePointerSwipe, useSwipe } from "./swipe.ts";
import type { SwipeDirection } from "./swipe.ts";
import { asElement, FakeDocument, pointerEvent, touchEvent } from "./testing/fake-dom.ts";

void test("classifies displacement by dominant axis and threshold", () => {
  assert.equal(swipeDirection(10, 5, 50), "none");
  assert.equal(swipeDirection(60, 5, 50), "left");
  assert.equal(swipeDirection(-60, 5, 50), "right");
  assert.equal(swipeDirection(5, 60, 50), "up");
  assert.equal(swipeDirection(5, -60, 50), "down");
});

void test("touch swipes report direction and lifecycle callbacks", () => {
  const document = new FakeDocument();
  const element = document.createElement();
  const events: string[] = [];
  const scope = effectScope();
  const swipe = scope.run(() =>
    useSwipe(asElement(element), {
      threshold: 20,
      passive: false,
      onSwipeStart: () => events.push("start"),
      onSwipe: (event) => events.push(`move:${String(event.defaultPrevented)}`),
      onSwipeEnd: (_event, direction: SwipeDirection) => events.push(`end:${direction}`),
    }),
  );
  assert.ok(swipe);
  assert.equal(swipe.direction.value, "none");

  element.dispatchEvent(touchEvent("touchstart", [{ x: 100, y: 100 }]));
  element.dispatchEvent(touchEvent("touchmove", [{ x: 95, y: 100 }]));
  assert.equal(swipe.isSwiping.value, false);
  element.dispatchEvent(touchEvent("touchmove", [{ x: 40, y: 110 }]));
  assert.equal(swipe.isSwiping.value, true);
  assert.equal(swipe.lengthX.value, 60);
  assert.equal(swipe.direction.value, "left");
  element.dispatchEvent(touchEvent("touchend", [{ x: 40, y: 110 }]));
  assert.equal(swipe.isSwiping.value, false);
  assert.deepEqual(events, ["start", "move:true", "end:left"]);

  scope.stop();
  element.dispatchEvent(touchEvent("touchstart", [{ x: 0, y: 0 }]));
  assert.deepEqual(events.length, 3);
});

void test("pointer swipes capture the pointer and ignore secondary pointers", () => {
  const document = new FakeDocument();
  const element = document.createElement();
  const ends: SwipeDirection[] = [];
  const scope = effectScope();
  const swipe = scope.run(() =>
    usePointerSwipe(asElement(element), {
      threshold: 10,
      disableTextSelect: true,
      onSwipeEnd: (_event, direction) => ends.push(direction),
    }),
  );
  assert.ok(swipe);

  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 1, clientX: 0, clientY: 0 }));
  assert.ok(element.captured.has(1));
  assert.equal(element.styles.get("user-select"), "none");
  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 2 }));
  element.dispatchEvent(pointerEvent("pointermove", { pointerId: 2, clientX: 0, clientY: -99 }));
  element.dispatchEvent(pointerEvent("pointermove", { pointerId: 1, clientX: 0, clientY: 30 }));
  assert.equal(swipe.direction.value, "down");
  element.dispatchEvent(pointerEvent("pointerup", { pointerId: 1, clientX: 0, clientY: 30 }));
  assert.deepEqual(ends, ["down"]);
  assert.equal(element.captured.size, 0);
  assert.equal(element.styles.get("user-select"), "");

  element.dispatchEvent(pointerEvent("pointerdown", { pointerId: 3, button: 2 }));
  assert.equal(element.captured.size, 0);
  scope.stop();
});

void test("pointer swipes honor the device filter", () => {
  const document = new FakeDocument();
  const element = document.createElement();
  const swipe = usePointerSwipe(asElement(element), { pointerTypes: ["touch"] });
  element.dispatchEvent(pointerEvent("pointerdown", { pointerType: "mouse" }));
  assert.equal(element.captured.size, 0);
  swipe.stop();
});
