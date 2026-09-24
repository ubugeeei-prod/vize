import assert from "node:assert/strict";
import { beforeEach, test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useElementVisibility, useIntersectionObserver } from "./intersection-observer.ts";
import {
  asDocument,
  asElement,
  FakeDocument,
  intersectionObserverHost,
  latestObserver,
  resetObservers,
  trigger,
} from "./testing/fake-dom.ts";

beforeEach(resetObservers);

void test("passes root, margin, and thresholds and rebuilds when they change", async () => {
  const document = new FakeDocument();
  const element = asElement(document.createElement());
  const root = asElement(document.createElement());
  const margin = ref("10px");
  const scope = effectScope();
  scope.run(() =>
    useIntersectionObserver(element, () => undefined, {
      host: intersectionObserverHost(),
      root,
      rootMargin: margin,
      threshold: [0, 0.5],
    }),
  );
  const first = latestObserver();
  assert.deepEqual(first.init, { root, rootMargin: "10px", threshold: [0, 0.5] });

  margin.value = "20px";
  await nextTick();
  assert.equal(first.disconnected, true);
  assert.deepEqual(latestObserver().init, { root, rootMargin: "20px", threshold: [0, 0.5] });
  scope.stop();
  assert.equal(latestObserver().disconnected, true);
});

void test("accepts a document root and supports pause, resume, and permanent stop", () => {
  const document = new FakeDocument();
  const element = asElement(document.createElement());
  const scope = effectScope();
  const controls = scope.run(() =>
    useIntersectionObserver(element, () => undefined, {
      host: intersectionObserverHost(),
      root: asDocument(document),
      immediate: false,
    }),
  );
  assert.ok(controls);
  assert.equal(controls.isActive.value, false);
  assert.throws(latestObserver);

  controls.resume();
  assert.equal(controls.isActive.value, true);
  assert.equal(
    latestObserver().init && (latestObserver().init as { root: unknown }).root,
    document,
  );
  controls.pause();
  assert.equal(latestObserver().disconnected, true);
  controls.stop();
  controls.resume();
  assert.equal(controls.isActive.value, false);
  scope.stop();
});

void test("element visibility follows the newest entry and honors once", () => {
  const document = new FakeDocument();
  const element = asElement(document.createElement());
  const scope = effectScope();
  const visibility = scope.run(() =>
    useElementVisibility(element, { host: intersectionObserverHost(), once: true }),
  );
  assert.ok(visibility);
  assert.equal(visibility.isVisible.value, false);

  trigger(latestObserver(), [
    { time: 2, isIntersecting: false },
    { time: 1, isIntersecting: true },
  ]);
  assert.equal(visibility.isVisible.value, false);
  trigger(latestObserver(), [{ time: 3, isIntersecting: true }]);
  assert.equal(visibility.isVisible.value, true);
  assert.equal(latestObserver().disconnected, true);
  scope.stop();
});

void test("visibility keeps the server value without a capability", () => {
  const document = new FakeDocument();
  const visibility = useElementVisibility(asElement(document.createElement()), {
    host: () => undefined,
    initialValue: true,
  });
  assert.equal(visibility.isSupported.value, false);
  assert.equal(visibility.isVisible.value, true);
  visibility.stop();
});
