import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import { FakeClock } from "./testing/fake-clock.ts";
import { useCountdown } from "./use-countdown.ts";

void test("counts down to zero, reporting ticks and completion once", () => {
  const clock = new FakeClock();
  const ticks: number[] = [];
  let completed = 0;
  const { remaining, isComplete, isActive, start } = useCountdown(3, {
    runOnServer: true,
    scheduler: clock.interval,
    onTick: (value) => ticks.push(value),
    onComplete: () => (completed += 1),
  });

  assert.equal(isActive.value, false);
  start();
  clock.advance(5_000);
  assert.deepEqual(ticks, [2, 1, 0]);
  assert.equal(remaining.value, 0);
  assert.equal(isComplete.value, true);
  assert.equal(completed, 1);
  assert.equal(isActive.value, false);
  assert.equal(clock.size, 0);
});

void test("pause keeps the remaining count; reset and start restore it", () => {
  const clock = new FakeClock();
  const initial = shallowRef(5);
  const { remaining, pause, resume, reset, start } = useCountdown(initial, {
    immediate: true,
    intervalMs: 100,
    runOnServer: true,
    scheduler: clock.interval,
  });

  clock.advance(200);
  pause();
  clock.advance(500);
  assert.equal(remaining.value, 3);
  resume();
  clock.advance(100);
  assert.equal(remaining.value, 2);

  initial.value = 10;
  reset();
  assert.equal(remaining.value, 10);
  clock.advance(500);
  assert.equal(remaining.value, 10);

  start(2);
  clock.advance(100);
  assert.equal(remaining.value, 1);
});

void test("stop jumps to zero without completing and resume at zero stays idle", () => {
  const clock = new FakeClock();
  let completed = 0;
  const { remaining, stop, resume, isActive } = useCountdown(4, {
    immediate: true,
    runOnServer: true,
    scheduler: clock.interval,
    onComplete: () => (completed += 1),
  });

  stop();
  assert.equal(remaining.value, 0);
  resume();
  assert.equal(isActive.value, false);
  assert.equal(completed, 0);
});

void test("keeps the initial count on the server", () => {
  const clock = new FakeClock();
  const { remaining } = useCountdown(30, { immediate: true, scheduler: clock.interval });
  assert.equal(clock.size, 0);
  assert.equal(remaining.value, 30);
});

void test("rejects invalid counts", () => {
  for (const invalid of [-1, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.throws(
      () => useCountdown(invalid),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_COUNTDOWN_INVALID_COUNT]"),
    );
  }
  const { start } = useCountdown(1);
  assert.throws(() => start(-2), RangeError);
});
