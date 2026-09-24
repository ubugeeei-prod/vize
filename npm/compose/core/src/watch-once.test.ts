import assert from "node:assert/strict";
import { test } from "node:test";
import { nextTick, shallowRef } from "vue";

import { watchOnce } from "./watch-once.ts";

void test("delivers exactly one change", async () => {
  const source = shallowRef(0);
  const calls: [number, number][] = [];
  watchOnce(source, (value, oldValue) => calls.push([value, oldValue]));
  source.value = 1;
  await nextTick();
  source.value = 2;
  await nextTick();
  assert.deepEqual(calls, [[1, 0]]);
});

void test("with immediate the single call is the immediate one", () => {
  const source = shallowRef("a");
  const calls: [string, string | undefined][] = [];
  watchOnce(source, (value, oldValue) => calls.push([value, oldValue]), {
    immediate: true,
    flush: "sync",
  });
  source.value = "b";
  assert.deepEqual(calls, [["a", undefined]]);
});

void test("the handle stops the watcher before it fires", () => {
  const source = shallowRef(0);
  let calls = 0;
  const stop = watchOnce(source, () => (calls += 1), { flush: "sync" });
  stop();
  source.value = 1;
  assert.equal(calls, 0);
});
