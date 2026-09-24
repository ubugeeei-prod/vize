import assert from "node:assert/strict";
import { test } from "node:test";

import { FakeClock, FakeFrames } from "./testing/fake-clock.ts";
import { useNow, useTimestamp } from "./use-now.ts";

void test("reads the injected clock immediately and on every tick", () => {
  const clock = new FakeClock();
  const timestamp = useTimestamp({
    interval: 1_000,
    now: () => clock.now,
    offset: 5,
    runOnServer: true,
    scheduler: clock.interval,
  });

  assert.equal(timestamp.value, 5);
  clock.advance(2_000);
  assert.equal(timestamp.value, 2_005);
});

void test("initial is hydration-stable until the first tick", () => {
  const clock = new FakeClock();
  clock.advance(123_456);
  const now = useNow({
    initial: 1_000,
    now: () => clock.now,
    runOnServer: true,
    scheduler: clock.interval,
  });

  assert.equal(now.value.getTime(), 1_000);
  clock.advance(1_000);
  assert.equal(now.value.getTime(), 124_456);
});

void test("controls pause, resume, refresh, and report every update", () => {
  const clock = new FakeClock();
  const updates: number[] = [];
  const { timestamp, pause, refresh, isActive } = useTimestamp({
    controls: true,
    interval: 10,
    now: () => clock.now,
    runOnServer: true,
    scheduler: clock.interval,
    callback: (value) => updates.push(value),
  });

  clock.advance(20);
  pause();
  assert.equal(isActive.value, false);
  clock.advance(100);
  assert.equal(timestamp.value, 20);
  clock.now = 999;
  assert.equal(refresh(), 999);
  assert.deepEqual(updates, [10, 20, 999]);
});

void test("useNow controls return fresh Date instances", () => {
  let current = 50;
  const { now, refresh } = useNow({ controls: true, now: () => current, immediate: false });
  current = 75;
  const refreshed = refresh();
  assert.ok(refreshed instanceof Date);
  assert.equal(refreshed.getTime(), 75);
  assert.equal(now.value.getTime(), 75);
});

void test("the animation-frame cadence updates per frame", () => {
  const frames = new FakeFrames();
  let current = 0;
  const timestamp = useTimestamp({
    interval: "requestAnimationFrame",
    now: () => current,
    runOnServer: true,
    frameScheduler: frames,
  });
  current = 16;
  frames.frame(16);
  assert.equal(timestamp.value, 16);
});

void test("never starts a timer on the server", () => {
  const clock = new FakeClock();
  const now = useNow({ initial: 0, scheduler: clock.interval });
  assert.equal(clock.size, 0);
  assert.equal(now.value.getTime(), 0);
});
