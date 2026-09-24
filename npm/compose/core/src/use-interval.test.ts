import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, shallowRef } from "vue";

import { FakeClock } from "./testing/fake-clock.ts";
import { useInterval, useIntervalFn } from "./use-interval.ts";

void test("ticks on the interval and pauses/resumes idempotently", () => {
  const clock = new FakeClock();
  let calls = 0;
  const { isActive, pause, resume } = useIntervalFn(() => (calls += 1), 100, {
    runOnServer: true,
    scheduler: clock.interval,
  });

  assert.equal(isActive.value, true);
  clock.advance(350);
  assert.equal(calls, 3);

  pause();
  pause();
  assert.equal(isActive.value, false);
  assert.equal(clock.size, 0);
  clock.advance(500);
  assert.equal(calls, 3);

  resume();
  resume();
  assert.equal(clock.size, 1);
  clock.advance(100);
  assert.equal(calls, 4);
});

void test("immediate: false waits for resume and immediateCallback fires on start", () => {
  const clock = new FakeClock();
  let calls = 0;
  const { isActive, resume } = useIntervalFn(() => (calls += 1), 100, {
    immediate: false,
    immediateCallback: true,
    runOnServer: true,
    scheduler: clock.interval,
  });

  assert.equal(isActive.value, false);
  assert.equal(clock.size, 0);
  resume();
  assert.equal(calls, 1);
  clock.advance(100);
  assert.equal(calls, 2);
});

void test("a reactive period replaces the running timer", async () => {
  const clock = new FakeClock();
  const period = shallowRef(100);
  let calls = 0;
  useIntervalFn(() => (calls += 1), period, { runOnServer: true, scheduler: clock.interval });

  period.value = 50;
  await nextTick();
  assert.equal(clock.size, 1);
  clock.advance(100);
  assert.equal(calls, 2);
});

void test("tracks the requested state on the server without creating timers", () => {
  const clock = new FakeClock();
  let calls = 0;
  const { isActive, pause, resume } = useIntervalFn(() => (calls += 1), 100, {
    immediateCallback: true,
    scheduler: clock.interval,
  });

  assert.equal(isActive.value, true);
  assert.equal(clock.size, 0);
  pause();
  resume();
  clock.advance(1_000);
  assert.equal(calls, 0);
});

void test("the owning scope clears the timer", () => {
  const clock = new FakeClock();
  const scope = effectScope();
  const controls = scope.run(() =>
    useIntervalFn(() => undefined, 100, { runOnServer: true, scheduler: clock.interval }),
  );
  assert.equal(clock.size, 1);
  scope.stop();
  assert.equal(clock.size, 0);
  assert.equal(controls?.isActive.value, false);
});

void test("rejects periods that cannot schedule a timer", () => {
  for (const invalid of [0, -1, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.throws(
      () => useIntervalFn(() => undefined, invalid),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_INTERVAL_INVALID_DELAY]"),
    );
  }
});

void test("useInterval counts ticks and exposes controls on request", () => {
  const clock = new FakeClock();
  const ticks: number[] = [];
  const counter = useInterval(10, { runOnServer: true, scheduler: clock.interval });
  const controls = useInterval(10, {
    controls: true,
    runOnServer: true,
    scheduler: clock.interval,
    callback: (count) => ticks.push(count),
  });

  clock.advance(30);
  assert.equal(counter.value, 3);
  assert.equal(controls.counter.value, 3);
  assert.deepEqual(ticks, [1, 2, 3]);

  controls.reset();
  assert.equal(controls.counter.value, 0);
  controls.pause();
  clock.advance(30);
  assert.equal(controls.counter.value, 0);
  assert.equal(counter.value, 6);
});
