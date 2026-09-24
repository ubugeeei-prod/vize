import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useElementHover } from "./element-hover.ts";
import { asElement, FakeDocument } from "./testing/fake-dom.ts";
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

void test("tracks enter and leave immediately without delays", () => {
  const element = new FakeDocument().createElement();
  const scope = effectScope();
  const hovered = scope.run(() => useElementHover(asElement(element)));
  element.dispatchEvent(new Event("pointerenter"));
  assert.equal(hovered?.value, true);
  element.dispatchEvent(new Event("pointerleave"));
  assert.equal(hovered?.value, false);
  scope.stop();
  element.dispatchEvent(new Event("pointerenter"));
  assert.equal(hovered?.value, false);
});

void test("delays cancel each other", () => {
  const element = new FakeDocument().createElement();
  const scheduler = manualScheduler();
  const scope = effectScope();
  const hovered = scope.run(() =>
    useElementHover(asElement(element), { delayEnter: 100, delayLeave: 100, scheduler }),
  );
  element.dispatchEvent(new Event("pointerenter"));
  assert.equal(hovered?.value, false);
  element.dispatchEvent(new Event("pointerleave"));
  scheduler.fire();
  assert.equal(hovered?.value, false);
  element.dispatchEvent(new Event("pointerenter"));
  scheduler.fire();
  assert.equal(hovered?.value, true);
  scope.stop();
});
