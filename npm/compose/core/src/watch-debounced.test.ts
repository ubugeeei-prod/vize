import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, reactive, shallowRef } from "vue";

import { FakeClock } from "./testing/fake-clock.ts";
import { watchDebounced } from "./watch-debounced.ts";

void test("delivers the settled value with the old value from before the burst", () => {
  const clock = new FakeClock();
  const source = shallowRef(0);
  const calls: [number, number][] = [];
  watchDebounced(source, (value, oldValue) => calls.push([value, oldValue]), {
    debounce: 100,
    flush: "sync",
    runOnServer: true,
    scheduler: clock.timeout,
  });

  source.value = 1;
  clock.advance(50);
  source.value = 2;
  clock.advance(50);
  source.value = 3;
  assert.deepEqual(calls, []);
  clock.advance(100);
  assert.deepEqual(calls, [[3, 0]]);

  source.value = 4;
  clock.advance(100);
  assert.deepEqual(calls, [
    [3, 0],
    [4, 3],
  ]);
});

void test("maxWait bounds starvation under continuous changes", () => {
  const clock = new FakeClock();
  const source = shallowRef(0);
  const values: number[] = [];
  watchDebounced(source, (value) => values.push(value), {
    debounce: 100,
    maxWait: 250,
    flush: "sync",
    runOnServer: true,
    scheduler: clock.timeout,
  });

  for (let step = 1; step <= 6; step += 1) {
    source.value = step;
    clock.advance(60);
  }
  assert.deepEqual(values, [5]);
});

void test("flush, cancel, and stop control the pending call", () => {
  const clock = new FakeClock();
  const source = shallowRef("a");
  const values: string[] = [];
  const { flush, cancel, stop } = watchDebounced(source, (value) => values.push(value), {
    debounce: 100,
    flush: "sync",
    runOnServer: true,
    scheduler: clock.timeout,
  });

  assert.equal(flush(), false);
  source.value = "b";
  assert.equal(flush(), true);
  assert.deepEqual(values, ["b"]);

  source.value = "c";
  assert.equal(cancel(), true);
  assert.equal(cancel(), false);
  clock.advance(200);
  assert.deepEqual(values, ["b"]);

  source.value = "d";
  stop();
  clock.advance(200);
  source.value = "e";
  clock.advance(200);
  assert.deepEqual(values, ["b"]);
  assert.equal(clock.size, 0);
});

void test("runs cleanup registered by the previous call before the next one", () => {
  const clock = new FakeClock();
  const source = shallowRef(0);
  const log: string[] = [];
  const { stop } = watchDebounced(
    source,
    (value, _oldValue, onCleanup) => {
      log.push(`run ${value}`);
      onCleanup(() => log.push(`cleanup ${value}`));
    },
    { debounce: 10, flush: "sync", runOnServer: true, scheduler: clock.timeout },
  );

  source.value = 1;
  clock.advance(10);
  source.value = 2;
  clock.advance(10);
  stop();
  assert.deepEqual(log, ["run 1", "cleanup 1", "run 2", "cleanup 2"]);
});

void test("supports tuple and reactive sources with the default flush", async () => {
  const clock = new FakeClock();
  const left = shallowRef(1);
  const state = reactive({ count: 0 });
  const seen: unknown[] = [];
  watchDebounced([left, () => state.count], (value) => seen.push(value), {
    debounce: 10,
    runOnServer: true,
    scheduler: clock.timeout,
  });
  watchDebounced(state, (value) => seen.push(value.count), {
    debounce: 10,
    runOnServer: true,
    scheduler: clock.timeout,
  });

  left.value = 2;
  state.count = 5;
  await nextTick();
  clock.advance(10);
  assert.deepEqual(seen, [[2, 5], 5]);
});

void test("runs synchronously on the server", () => {
  const clock = new FakeClock();
  const source = shallowRef(0);
  const values: number[] = [];
  watchDebounced(source, (value) => values.push(value), {
    debounce: 500,
    flush: "sync",
    scheduler: clock.timeout,
  });
  source.value = 1;
  assert.deepEqual(values, [1]);
  assert.equal(clock.size, 0);
});

void test("stops with the owning scope and validates delays", () => {
  const clock = new FakeClock();
  const source = shallowRef(0);
  let calls = 0;
  const scope = effectScope();
  scope.run(() =>
    watchDebounced(source, () => (calls += 1), {
      debounce: 10,
      flush: "sync",
      runOnServer: true,
      scheduler: clock.timeout,
    }),
  );
  source.value = 1;
  scope.stop();
  clock.advance(100);
  assert.equal(calls, 0);

  assert.throws(
    () => watchDebounced(source, () => undefined, { debounce: -1 }),
    (error: unknown) =>
      error instanceof RangeError &&
      error.message.startsWith("[VIZE_COMPOSE_WATCH_DEBOUNCED_INVALID_DELAY]"),
  );
  assert.throws(() => watchDebounced(source, () => undefined, { maxWait: Number.NaN }), RangeError);
});
