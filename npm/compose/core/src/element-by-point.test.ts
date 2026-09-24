import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useElementByPoint } from "./element-by-point.ts";
import type { ElementByPointFrameHost, ElementByPointHost } from "./element-by-point.ts";
import { asElement, FakeDocument } from "./testing/fake-dom.ts";

void test("re-queries when coordinates change and while polling", async () => {
  const document = new FakeDocument();
  const left = asElement(document.createElement());
  const right = asElement(document.createElement());
  let queries = 0;
  const host: ElementByPointHost = {
    elementFromPoint: (x) => {
      queries += 1;
      return x < 50 ? left : right;
    },
    elementsFromPoint: (x) => (x < 50 ? [left] : [right, left]),
  };
  const frames: FrameRequestCallback[] = [];
  const frameHost: ElementByPointFrameHost = {
    requestAnimationFrame: (callback) => frames.push(callback),
    cancelAnimationFrame: (handle) => {
      frames[handle - 1] = () => undefined;
    },
  };
  const x = ref(10);
  const scope = effectScope();
  const point = scope.run(() => useElementByPoint({ x, y: 0, host, poll: true, frameHost }));
  assert.ok(point);
  assert.equal(point.element.value, left);

  x.value = 90;
  await nextTick();
  assert.equal(point.element.value, right);
  assert.deepEqual(point.elements.value, [right, left]);
  const before = queries;
  frames.at(-1)?.(16);
  assert.equal(queries, before + 1);

  scope.stop();
  assert.equal(frames.length, 2);
});

void test("server renders expose nothing", () => {
  const point = useElementByPoint({ x: 1, y: 1, host: () => undefined });
  assert.deepEqual(
    [point.isSupported.value, point.element.value, point.elements.value],
    [false, null, []],
  );
});
