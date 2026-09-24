import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import { FakeClock } from "./testing/fake-clock.ts";
import { watchThrottled } from "./watch-throttled.ts";

function setup(options: { leading?: boolean; trailing?: boolean } = {}) {
  const clock = new FakeClock();
  const source = shallowRef(0);
  const calls: [number, number][] = [];
  const handle = watchThrottled(source, (value, oldValue) => calls.push([value, oldValue]), {
    throttle: 100,
    flush: "sync",
    runOnServer: true,
    scheduler: clock.timeout,
    ...options,
  });
  return { clock, source, calls, handle };
}

void test("leading and trailing calls chain old values without gaps", () => {
  const { clock, source, calls } = setup();
  source.value = 1;
  source.value = 2;
  source.value = 3;
  assert.deepEqual(calls, [[1, 0]]);
  clock.advance(100);
  assert.deepEqual(calls, [
    [1, 0],
    [3, 1],
  ]);
  clock.advance(100);
  source.value = 4;
  assert.deepEqual(calls.at(-1), [4, 3]);
});

void test("leading: false delivers only at the window end", () => {
  const { clock, source, calls } = setup({ leading: false });
  source.value = 1;
  source.value = 2;
  assert.deepEqual(calls, []);
  clock.advance(100);
  assert.deepEqual(calls, [[2, 0]]);
});

void test("trailing: false drops changes inside the window", () => {
  const { clock, source, calls } = setup({ trailing: false });
  source.value = 1;
  source.value = 2;
  clock.advance(100);
  source.value = 3;
  assert.deepEqual(calls, [
    [1, 0],
    [3, 1],
  ]);
});

void test("flush, cancel, and stop control the trailing call", () => {
  const { clock, source, calls, handle } = setup();
  source.value = 1;
  source.value = 2;
  assert.equal(handle.flush(), true);
  assert.equal(handle.flush(), false);
  source.value = 3;
  assert.equal(handle.cancel(), true);
  clock.advance(100);
  assert.deepEqual(calls, [
    [1, 0],
    [2, 1],
  ]);
  handle.stop();
  source.value = 9;
  clock.advance(500);
  assert.equal(calls.length, 2);
  assert.equal(clock.size, 0);
});

void test("runs synchronously on the server and stops with the scope", () => {
  const clock = new FakeClock();
  const source = shallowRef(0);
  const values: number[] = [];
  watchThrottled(source, (value) => values.push(value), {
    throttle: 100,
    flush: "sync",
    scheduler: clock.timeout,
  });
  source.value = 1;
  source.value = 2;
  assert.deepEqual(values, [1, 2]);

  const scope = effectScope();
  const scoped: number[] = [];
  scope.run(() =>
    watchThrottled(source, (value) => scoped.push(value), {
      throttle: 100,
      flush: "sync",
      runOnServer: true,
      scheduler: clock.timeout,
    }),
  );
  source.value = 3;
  source.value = 4;
  scope.stop();
  clock.advance(200);
  assert.deepEqual(scoped, [3]);
  assert.throws(
    () => watchThrottled(source, () => undefined, { throttle: -1 }),
    (error: unknown) =>
      error instanceof RangeError &&
      error.message.startsWith("[VIZE_COMPOSE_WATCH_THROTTLED_INVALID_DELAY]"),
  );
});
