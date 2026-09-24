import assert from "node:assert/strict";
import { beforeEach, test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useElementSize, useResizeObserver } from "./resize-observer.ts";
import {
  asElement,
  FakeDocument,
  latestObserver,
  resetObservers,
  resizeObserverHost,
  trigger,
} from "./testing/fake-dom.ts";

beforeEach(resetObservers);

void test("is unsupported without a host and never constructs an observer", () => {
  const document = new FakeDocument();
  const scope = effectScope();
  const controls = scope.run(() =>
    useResizeObserver(asElement(document.createElement()), () => undefined, {
      host: () => null,
    }),
  );

  assert.equal(controls?.isSupported.value, false);
  assert.throws(latestObserver);
  scope.stop();
});

void test("observes every resolved target with the requested box and disconnects with the scope", () => {
  const document = new FakeDocument();
  const first = asElement(document.createElement());
  const second = asElement(document.createElement());
  const scope = effectScope();
  const calls: number[] = [];
  const controls = scope.run(() =>
    useResizeObserver([first, null, second], (entries) => calls.push(entries.length), {
      host: resizeObserverHost(),
      box: "border-box",
      flush: "sync",
    }),
  );

  const observer = latestObserver();
  assert.equal(controls?.isSupported.value, true);
  assert.deepEqual([...observer.observed.keys()], [first, second]);
  assert.deepEqual(observer.observed.get(first), { box: "border-box" });
  trigger(observer, [{}, {}]);
  assert.deepEqual(calls, [2]);

  scope.stop();
  assert.equal(observer.disconnected, true);
});

void test("recreates the observer when the target ref changes", async () => {
  const document = new FakeDocument();
  const first = asElement(document.createElement());
  const second = asElement(document.createElement());
  const target = ref<Element | null>(first);
  const scope = effectScope();
  scope.run(() => useResizeObserver(target, () => undefined, { host: resizeObserverHost() }));

  const initial = latestObserver();
  target.value = second;
  await nextTick();
  const next = latestObserver();
  assert.notEqual(initial, next);
  assert.equal(initial.disconnected, true);
  assert.deepEqual([...next.observed.keys()], [second]);

  target.value = null;
  await nextTick();
  assert.equal(next.disconnected, true);
  scope.stop();
});

void test("element size sums box fragments, falls back to contentRect, and resets on unmount", async () => {
  const document = new FakeDocument();
  const element = asElement(document.createElement());
  const target = ref<Element | null>(element);
  const scope = effectScope();
  const size = scope.run(() =>
    useElementSize(target, { host: resizeObserverHost(), initialSize: { width: 5, height: 6 } }),
  );
  assert.ok(size);
  assert.equal(size.width.value, 5);

  trigger(latestObserver(), [
    {
      contentBoxSize: [
        { inlineSize: 10, blockSize: 20 },
        { inlineSize: 1, blockSize: 2 },
      ],
      contentRect: { width: 0, height: 0 },
    },
  ]);
  assert.deepEqual([size.width.value, size.height.value], [11, 22]);

  trigger(latestObserver(), [{ contentRect: { width: 7, height: 8 } }]);
  assert.deepEqual([size.width.value, size.height.value], [7, 8]);

  target.value = null;
  await nextTick();
  assert.deepEqual([size.width.value, size.height.value], [5, 6]);
  scope.stop();
});

void test("element size reads the border box when requested", () => {
  const document = new FakeDocument();
  const scope = effectScope();
  const size = scope.run(() =>
    useElementSize(asElement(document.createElement()), {
      host: resizeObserverHost(),
      box: "border-box",
    }),
  );
  trigger(latestObserver(), [
    {
      borderBoxSize: [{ inlineSize: 30, blockSize: 40 }],
      contentBoxSize: [{ inlineSize: 1, blockSize: 1 }],
    },
  ]);
  assert.deepEqual([size?.width.value, size?.height.value], [30, 40]);
  scope.stop();
});
