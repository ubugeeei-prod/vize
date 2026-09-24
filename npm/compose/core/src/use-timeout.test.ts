import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import { FakeClock } from "./testing/fake-clock.ts";
import { useTimeout, useTimeoutFn } from "./use-timeout.ts";

void test("starts immediately and fires once", () => {
  const clock = new FakeClock();
  let calls = 0;
  const { isPending } = useTimeoutFn(() => (calls += 1), 100, {
    runOnServer: true,
    scheduler: clock.timeout,
  });

  assert.equal(isPending.value, true);
  clock.advance(99);
  assert.equal(calls, 0);
  clock.advance(1);
  assert.equal(calls, 1);
  assert.equal(isPending.value, false);
  clock.advance(1_000);
  assert.equal(calls, 1);
});

void test("start forwards typed arguments and restarts a pending call", () => {
  const clock = new FakeClock();
  const received: [string, number][] = [];
  const { start, isPending } = useTimeoutFn(
    (id: string, attempt: number) => received.push([id, attempt]),
    100,
    { immediate: false, runOnServer: true, scheduler: clock.timeout },
  );

  assert.equal(isPending.value, false);
  start("a", 1);
  clock.advance(50);
  start("b", 2);
  clock.advance(50);
  assert.deepEqual(received, []);
  clock.advance(50);
  assert.deepEqual(received, [["b", 2]]);
});

void test("stop cancels and reports whether something was pending", () => {
  const clock = new FakeClock();
  let calls = 0;
  const { stop } = useTimeoutFn(() => (calls += 1), 100, {
    runOnServer: true,
    scheduler: clock.timeout,
  });

  assert.equal(stop(), true);
  assert.equal(stop(), false);
  clock.advance(200);
  assert.equal(calls, 0);
  assert.equal(clock.size, 0);
});

void test("reads a reactive delay on every start", () => {
  const clock = new FakeClock();
  const delay = shallowRef(100);
  let calls = 0;
  const { start } = useTimeoutFn(() => (calls += 1), delay, {
    immediate: false,
    runOnServer: true,
    scheduler: clock.timeout,
  });

  delay.value = 10;
  start();
  clock.advance(10);
  assert.equal(calls, 1);
});

void test("reports the pending state on the server without creating a timer", () => {
  const clock = new FakeClock();
  let calls = 0;
  const ready = useTimeout(100, { scheduler: clock.timeout, callback: () => (calls += 1) });

  assert.equal(ready.value, false);
  assert.equal(clock.size, 0);
  clock.advance(1_000);
  assert.equal(calls, 0);
  assert.equal(ready.value, false);
});

void test("the owning scope cancels the pending call", () => {
  const clock = new FakeClock();
  const scope = effectScope();
  let calls = 0;
  scope.run(() =>
    useTimeoutFn(() => (calls += 1), 100, { runOnServer: true, scheduler: clock.timeout }),
  );
  scope.stop();
  clock.advance(200);
  assert.equal(calls, 0);
  assert.equal(clock.size, 0);
});

void test("useTimeout exposes readiness with optional controls", () => {
  const clock = new FakeClock();
  let fired = 0;
  const { ready, start, stop } = useTimeout(100, {
    controls: true,
    immediate: false,
    runOnServer: true,
    scheduler: clock.timeout,
    callback: () => (fired += 1),
  });

  assert.equal(ready.value, true);
  start();
  assert.equal(ready.value, false);
  clock.advance(100);
  assert.equal(ready.value, true);
  assert.equal(fired, 1);

  start();
  assert.equal(stop(), true);
  assert.equal(ready.value, true);
});

void test("rejects delays that cannot schedule a timer", () => {
  for (const invalid of [-1, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.throws(
      () => useTimeoutFn(() => undefined, invalid),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_TIMEOUT_INVALID_DELAY]"),
    );
  }
});
