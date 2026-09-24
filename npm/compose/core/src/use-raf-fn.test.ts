import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import { FakeFrames } from "./testing/fake-clock.ts";
import { type RafFrame, useRafFn } from "./use-raf-fn.ts";

void test("delivers every frame with deltas", () => {
  const frames = new FakeFrames();
  const seen: RafFrame[] = [];
  useRafFn((frame) => seen.push(frame), { runOnServer: true, scheduler: frames });

  frames.frame(1_000);
  frames.frame(1_016);
  frames.frame(1_033);
  assert.deepEqual(seen, [
    { delta: 0, timestamp: 1_000 },
    { delta: 16, timestamp: 1_016 },
    { delta: 17, timestamp: 1_033 },
  ]);
});

void test("fpsLimit skips frames that arrive too early", () => {
  const frames = new FakeFrames();
  const limit = shallowRef<number | undefined>(10);
  const seen: number[] = [];
  useRafFn(({ timestamp }) => seen.push(timestamp), {
    runOnServer: true,
    scheduler: frames,
    fpsLimit: limit,
  });

  for (let timestamp = 0; timestamp <= 250; timestamp += 50) frames.frame(timestamp);
  assert.deepEqual(seen, [0, 100, 200]);

  limit.value = undefined;
  frames.frame(260);
  assert.deepEqual(seen, [0, 100, 200, 260]);
});

void test("pause cancels the pending frame and resume restarts deltas", () => {
  const frames = new FakeFrames();
  const seen: RafFrame[] = [];
  const { pause, resume, isActive } = useRafFn((frame) => seen.push(frame), {
    runOnServer: true,
    scheduler: frames,
    immediate: false,
  });

  assert.equal(isActive.value, false);
  assert.equal(frames.pending.size, 0);
  resume();
  frames.frame(10);
  pause();
  assert.equal(frames.pending.size, 0);
  resume();
  frames.frame(500);
  assert.deepEqual(seen, [
    { delta: 0, timestamp: 10 },
    { delta: 0, timestamp: 500 },
  ]);
});

void test("once delivers a single frame", () => {
  const frames = new FakeFrames();
  let calls = 0;
  const { isActive } = useRafFn(() => (calls += 1), {
    runOnServer: true,
    scheduler: frames,
    once: true,
  });
  frames.frame(1);
  frames.frame(2);
  assert.equal(calls, 1);
  assert.equal(isActive.value, false);
});

void test("without a frame host it only tracks the requested state", () => {
  assert.equal(typeof globalThis.requestAnimationFrame, "undefined");
  let calls = 0;
  const { isActive, pause } = useRafFn(() => (calls += 1), { runOnServer: true });
  assert.equal(isActive.value, true);
  pause();
  assert.equal(isActive.value, false);
  assert.equal(calls, 0);
});

void test("the owning scope cancels the frame loop", () => {
  const frames = new FakeFrames();
  const scope = effectScope();
  scope.run(() => useRafFn(() => undefined, { runOnServer: true, scheduler: frames }));
  assert.equal(frames.pending.size, 1);
  scope.stop();
  assert.equal(frames.pending.size, 0);
});

void test("rejects invalid frame-rate limits", () => {
  for (const invalid of [0, -5, Number.NaN]) {
    assert.throws(
      () => useRafFn(() => undefined, { fpsLimit: invalid }),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_RAF_INVALID_FPS_LIMIT]"),
    );
  }
});

void test("requests no frames on the server unless asked to", () => {
  const frames = new FakeFrames();
  const { isActive } = useRafFn(() => undefined, { scheduler: frames });
  assert.equal(isActive.value, true);
  assert.equal(frames.pending.size, 0);
});
