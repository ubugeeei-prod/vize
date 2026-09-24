import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { onLongPress } from "./on-long-press.ts";
import type { LongPressRelease } from "./on-long-press.ts";
import { asElement, FakeDocument, pointerEvent } from "./testing/fake-dom.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

function manualScheduler(): TimeoutScheduler & { fire: () => void } {
  let queued: (() => void) | undefined;
  return {
    setTimeout: (callback) => {
      queued = callback;
      return 1;
    },
    clearTimeout: () => {
      queued = undefined;
    },
    fire: () => {
      const callback = queued;
      queued = undefined;
      callback?.();
    },
  };
}

void test("fires after the hold and reports releases", () => {
  const element = new FakeDocument().createElement();
  const scheduler = manualScheduler();
  let clock = 0;
  let presses = 0;
  const releases: LongPressRelease[] = [];
  const scope = effectScope();
  scope.run(() =>
    onLongPress(asElement(element), () => (presses += 1), {
      scheduler,
      now: () => clock,
      onRelease: (release) => releases.push(release),
    }),
  );

  element.dispatchEvent(pointerEvent("pointerdown"));
  clock = 600;
  scheduler.fire();
  element.dispatchEvent(pointerEvent("pointerup"));
  assert.equal(presses, 1);
  assert.deepEqual(releases.at(-1), { duration: 600, distance: 0, isLongPress: true });

  element.dispatchEvent(pointerEvent("pointerdown"));
  element.dispatchEvent(pointerEvent("pointerup"));
  scheduler.fire();
  assert.equal(presses, 1);
  assert.equal(releases.at(-1)?.isLongPress, false);

  element.dispatchEvent(pointerEvent("pointerdown"));
  element.dispatchEvent(pointerEvent("pointermove", { clientX: 30 }));
  scheduler.fire();
  assert.equal(presses, 1);

  scope.stop();
  element.dispatchEvent(pointerEvent("pointerdown"));
  scheduler.fire();
  assert.equal(presses, 1);
});

void test("once, self, and prevent modifiers", () => {
  const document = new FakeDocument();
  const element = document.createElement();
  const scheduler = manualScheduler();
  let presses = 0;
  const stop = onLongPress(asElement(element), () => (presses += 1), {
    scheduler,
    modifiers: { once: true, self: true, prevent: true },
  });
  const fromChild = pointerEvent("pointerdown");
  Object.defineProperty(fromChild, "target", { value: document.createElement() });
  element.dispatchEvent(fromChild);
  scheduler.fire();
  assert.equal(presses, 0);

  const own = pointerEvent("pointerdown");
  Object.defineProperty(own, "target", { value: element });
  element.dispatchEvent(own);
  assert.equal(own.defaultPrevented, true);
  scheduler.fire();
  element.dispatchEvent(own);
  scheduler.fire();
  assert.equal(presses, 1);
  stop();
});
