import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, ref } from "vue";

import { useInfiniteScroll } from "./infinite-scroll.ts";
import { asElement, FakeDocument } from "./testing/fake-dom.ts";

const flush = async (): Promise<void> => {
  for (let index = 0; index < 5; index += 1) await Promise.resolve();
};

void test("loads until the container overflows, then on reaching the edge", async () => {
  const element = new FakeDocument().createElement();
  element.clientHeight = 100;
  element.scrollHeight = 100;
  let loads = 0;
  const scope = effectScope();
  const infinite = scope.run(() =>
    useInfiniteScroll(
      asElement(element),
      async () => {
        loads += 1;
        element.scrollHeight += 100;
      },
      { distance: 10 },
    ),
  );
  assert.ok(infinite);
  await flush();
  assert.equal(loads, 1);
  assert.equal(infinite.isLoading.value, false);

  element.scrollTop = 50;
  element.dispatchEvent(new Event("scroll"));
  await flush();
  assert.equal(loads, 1);
  element.scrollTop = 95;
  element.dispatchEvent(new Event("scroll"));
  await flush();
  assert.equal(loads, 2);
  scope.stop();
});

void test("respects canLoadMore and never runs concurrently", async () => {
  const element = new FakeDocument().createElement();
  element.clientHeight = 100;
  element.scrollHeight = 100;
  const canLoadMore = ref(false);
  let release: () => void = () => undefined;
  let loads = 0;
  const scope = effectScope();
  const infinite = scope.run(() =>
    useInfiniteScroll(
      asElement(element),
      () => {
        loads += 1;
        return new Promise<void>((resolve) => (release = resolve));
      },
      { canLoadMore },
    ),
  );
  await flush();
  assert.equal(loads, 0);
  canLoadMore.value = true;
  await flush();
  assert.equal(loads, 1);
  infinite?.reset();
  await flush();
  assert.equal(loads, 1);
  canLoadMore.value = false;
  release();
  await flush();
  assert.equal(loads, 1);
  scope.stop();
});
