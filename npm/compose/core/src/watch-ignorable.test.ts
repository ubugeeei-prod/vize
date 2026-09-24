import assert from "node:assert/strict";
import { test } from "node:test";
import { nextTick, shallowRef } from "vue";

import { watchIgnorable } from "./watch-ignorable.ts";

void test("sync watchers skip writes made inside ignoreUpdates", () => {
  const source = shallowRef(0);
  const values: number[] = [];
  const { ignoreUpdates, ignorePrevAsyncUpdates, stop } = watchIgnorable(
    source,
    (value) => values.push(value),
    { flush: "sync" },
  );

  source.value = 1;
  ignoreUpdates(() => {
    source.value = 2;
  });
  ignorePrevAsyncUpdates();
  source.value = 3;
  stop();
  source.value = 4;
  assert.deepEqual(values, [1, 3]);
});

void test("ignoreUpdates restores delivery even when the updater throws", () => {
  const source = shallowRef(0);
  const values: number[] = [];
  const { ignoreUpdates } = watchIgnorable(source, (value) => values.push(value), {
    flush: "sync",
  });
  assert.throws(() =>
    ignoreUpdates(() => {
      source.value = 1;
      throw new Error("boom");
    }),
  );
  source.value = 2;
  assert.deepEqual(values, [2]);
});

void test("batched watchers skip batches made only of ignored writes", async () => {
  const source = shallowRef(0);
  const values: number[] = [];
  const { ignoreUpdates } = watchIgnorable(source, (value) => values.push(value));

  ignoreUpdates(() => {
    source.value = 1;
  });
  await nextTick();
  assert.deepEqual(values, []);

  ignoreUpdates(() => {
    source.value = 2;
  });
  source.value = 3;
  await nextTick();
  assert.deepEqual(values, [3]);
});

void test("ignorePrevAsyncUpdates swallows the queued batch", async () => {
  const source = shallowRef(0);
  const values: number[] = [];
  const { ignorePrevAsyncUpdates, stop } = watchIgnorable(source, (value) => values.push(value));

  source.value = 1;
  ignorePrevAsyncUpdates();
  await nextTick();
  source.value = 2;
  await nextTick();
  stop();
  source.value = 3;
  await nextTick();
  assert.deepEqual(values, [2]);
});

void test("once stops both internal watchers after the first delivery", async () => {
  const source = shallowRef(0);
  const values: number[] = [];
  watchIgnorable(source, (value) => values.push(value), { once: true });
  source.value = 1;
  await nextTick();
  source.value = 2;
  await nextTick();
  assert.deepEqual(values, [1]);
});
