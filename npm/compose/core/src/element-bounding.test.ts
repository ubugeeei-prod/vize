import assert from "node:assert/strict";
import { beforeEach, test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useElementBounding } from "./element-bounding.ts";
import type { ElementBoundingHost } from "./element-bounding.ts";
import {
  asElement,
  FakeDocument,
  FakeObserver,
  latestObserver,
  resetObservers,
  trigger,
} from "./testing/fake-dom.ts";

beforeEach(resetObservers);

class BoundingHost extends EventTarget implements ElementBoundingHost {
  readonly ResizeObserver = FakeObserver as unknown as typeof ResizeObserver;
}

void test("measures on resolve, resize observation, window resize, and nested scroll", () => {
  const document = new FakeDocument();
  const fake = document.createElement();
  fake.rect = { x: 10, y: 20, width: 100, height: 50 };
  const host = new BoundingHost();
  const scope = effectScope();
  const bounds = scope.run(() => useElementBounding(asElement(fake), { host, flush: "sync" }));
  assert.ok(bounds);
  assert.deepEqual(
    [bounds.x.value, bounds.top.value, bounds.right.value, bounds.bottom.value],
    [10, 20, 110, 70],
  );

  fake.rect = { x: 0, y: 0, width: 5, height: 5 };
  trigger(latestObserver(), [{}]);
  assert.equal(bounds.width.value, 5);

  fake.rect = { x: 1, y: 2, width: 5, height: 5 };
  host.dispatchEvent(new Event("resize"));
  assert.equal(bounds.left.value, 1);

  fake.rect = { x: 1, y: -40, width: 5, height: 5 };
  host.dispatchEvent(new Event("scroll"));
  assert.equal(bounds.y.value, -40);

  scope.stop();
  fake.rect = { x: 99, y: 99, width: 5, height: 5 };
  host.dispatchEvent(new Event("scroll"));
  assert.equal(bounds.x.value, 1);
});

void test("resets to zero on unmount unless disabled", async () => {
  const document = new FakeDocument();
  const fake = document.createElement();
  fake.rect = { x: 3, y: 4, width: 5, height: 6 };
  const target = ref<Element | null>(asElement(fake));
  const keep = ref<Element | null>(asElement(fake));
  const scope = effectScope();
  const reset = scope.run(() => useElementBounding(target, { host: () => null }));
  const kept = scope.run(() => useElementBounding(keep, { host: () => null, reset: false }));

  target.value = null;
  keep.value = null;
  await nextTick();
  assert.equal(reset?.height.value, 0);
  assert.equal(kept?.height.value, 6);
  scope.stop();
});

void test("disabled window listeners ignore scroll and resize", () => {
  const document = new FakeDocument();
  const fake = document.createElement();
  const host = new BoundingHost();
  const scope = effectScope();
  const bounds = scope.run(() =>
    useElementBounding(asElement(fake), {
      host,
      windowResize: false,
      windowScroll: false,
      flush: "sync",
    }),
  );
  fake.rect = { x: 9, y: 9, width: 9, height: 9 };
  host.dispatchEvent(new Event("resize"));
  host.dispatchEvent(new Event("scroll"));
  assert.equal(bounds?.x.value, 0);
  bounds?.update();
  assert.equal(bounds?.x.value, 9);
  scope.stop();
});
