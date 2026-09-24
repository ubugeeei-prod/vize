import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick } from "vue";

import { useScrollLock } from "./scroll-lock.ts";
import { asElement, FakeDocument } from "./testing/fake-dom.ts";

void test("locks, restores the previous overflow, and blocks touch scrolling", async () => {
  const element = new FakeDocument().createElement();
  element.style.setProperty("overflow", "scroll");
  const scope = effectScope();
  const locked = scope.run(() => useScrollLock(asElement(element)));
  assert.ok(locked);
  assert.equal(locked.value, false);

  locked.value = true;
  await nextTick();
  assert.equal(element.styles.get("overflow"), "hidden");
  const touch = new Event("touchmove", { cancelable: true });
  element.dispatchEvent(touch);
  assert.equal(touch.defaultPrevented, true);

  locked.value = false;
  await nextTick();
  assert.equal(element.styles.get("overflow"), "scroll");
  const free = new Event("touchmove", { cancelable: true });
  element.dispatchEvent(free);
  assert.equal(free.defaultPrevented, false);

  locked.value = true;
  await nextTick();
  scope.stop();
  assert.equal(element.styles.get("overflow"), "scroll");
});

void test("initialValue locks immediately", () => {
  const element = new FakeDocument().createElement();
  const scope = effectScope();
  scope.run(() =>
    useScrollLock(asElement(element), { initialValue: true, preventTouchMove: false }),
  );
  assert.equal(element.styles.get("overflow"), "hidden");
  scope.stop();
  assert.equal(element.styles.get("overflow"), "");
});
