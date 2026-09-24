import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, shallowRef } from "vue";

import { watchPausable } from "./watch-pausable.ts";

void test("drops changes while paused instead of replaying them", async () => {
  const source = shallowRef(0);
  const calls: [number, number][] = [];
  const { pause, resume, isActive } = watchPausable(source, (value, oldValue) =>
    calls.push([value, oldValue]),
  );

  source.value = 1;
  await nextTick();
  pause();
  assert.equal(isActive.value, false);
  source.value = 2;
  await nextTick();
  resume();
  await nextTick();
  assert.deepEqual(calls, [[1, 0]]);
  source.value = 3;
  await nextTick();
  assert.deepEqual(calls, [
    [1, 0],
    [3, 2],
  ]);
});

void test("can start paused and stop permanently", () => {
  const source = shallowRef("a");
  const values: string[] = [];
  const { resume, stop } = watchPausable(source, (value) => values.push(value), {
    initiallyPaused: true,
    flush: "sync",
  });

  source.value = "b";
  resume();
  source.value = "c";
  stop();
  source.value = "d";
  assert.deepEqual(values, ["c"]);
});

void test("stops with the owning scope", () => {
  const source = shallowRef(0);
  let calls = 0;
  const scope = effectScope();
  scope.run(() => watchPausable(source, () => (calls += 1), { flush: "sync" }));
  scope.stop();
  source.value = 1;
  assert.equal(calls, 0);
});
