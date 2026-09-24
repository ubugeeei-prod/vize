import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { useVibrate } from "./use-vibrate.ts";
import type { VibrationHost, VibrationPattern } from "./use-vibrate.ts";

class FakeVibration implements VibrationHost {
  readonly calls: VibrationPattern[] = [];
  accept = true;

  vibrate(pattern: VibrationPattern): boolean {
    this.calls.push(pattern);
    return this.accept;
  }
}

class ManualScheduler implements TimeoutScheduler {
  readonly timers = new Map<number, { callback: () => void; delayMs: number }>();
  next = 0;

  setTimeout(callback: () => void, delayMs: number): unknown {
    this.next += 1;
    this.timers.set(this.next, { callback, delayMs });
    return this.next;
  }

  clearTimeout(handle: unknown): void {
    if (typeof handle === "number") this.timers.delete(handle);
  }

  tick(): void {
    const timers = [...this.timers.values()];
    this.timers.clear();
    for (const timer of timers) timer.callback();
  }
}

void test("vibrates with the default or explicit pattern", () => {
  const host = new FakeVibration();
  const vibration = useVibrate({ host, pattern: [100, 50, 100] });

  assert.equal(vibration.supported.value, true);
  assert.equal(vibration.vibrate(), true);
  assert.equal(vibration.vibrating.value, true);
  assert.equal(vibration.vibrate(30), true);
  assert.deepEqual(host.calls, [[100, 50, 100], [30]]);
});

void test("repeats until stopped", () => {
  const host = new FakeVibration();
  const scheduler = new ManualScheduler();
  const vibration = useVibrate({ host, pattern: 100, interval: 500, scheduler });

  vibration.vibrate();
  assert.equal(scheduler.timers.size, 1);
  assert.equal([...scheduler.timers.values()][0]?.delayMs, 500);
  scheduler.tick();
  assert.equal(host.calls.length, 2);

  vibration.stop();
  assert.equal(scheduler.timers.size, 0);
  assert.equal(vibration.vibrating.value, false);
  assert.deepEqual(host.calls.at(-1), 0);
});

void test("stops repeating when the platform rejects the pattern", () => {
  const host = new FakeVibration();
  const scheduler = new ManualScheduler();
  const vibration = useVibrate({ host, interval: 100, scheduler });
  vibration.vibrate([10]);
  host.accept = false;
  scheduler.tick();
  assert.equal(vibration.vibrating.value, false);
  assert.equal(scheduler.timers.size, 0);
});

void test("an empty pattern is accepted but not considered vibrating", () => {
  const host = new FakeVibration();
  const vibration = useVibrate({ host });
  assert.equal(vibration.vibrate(), true);
  assert.equal(vibration.vibrating.value, false);
});

void test("validates patterns and intervals", () => {
  const vibration = useVibrate({ host: new FakeVibration() });
  assert.throws(() => vibration.vibrate(-1), /VIZE_COMPOSE_VIBRATE_INVALID_PATTERN/);
  assert.throws(() => vibration.vibrate([1.5]), /VIZE_COMPOSE_VIBRATE_INVALID_PATTERN/);
  assert.throws(
    () => useVibrate({ interval: Number.NaN }),
    /VIZE_COMPOSE_VIBRATE_INVALID_INTERVAL/,
  );
});

void test("cancels vibration and repetition with the scope", () => {
  const host = new FakeVibration();
  const scheduler = new ManualScheduler();
  const scope = effectScope();
  const vibration = scope.run(() => useVibrate({ host, interval: 100, scheduler }));
  assert.ok(vibration);
  vibration.vibrate(50);
  scope.stop();
  assert.equal(scheduler.timers.size, 0);
  assert.deepEqual(host.calls.at(-1), 0);
});

void test("reports unsupported without a host", () => {
  const vibration = useVibrate({ host: null });
  assert.equal(vibration.supported.value, false);
  assert.equal(vibration.vibrate(10), false);
});

void test("server rendering vibrates nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const vibration = useVibrate({ pattern: 10, interval: 100 });
    return { supported: vibration.supported, vibrating: vibration.vibrating };
  });
  assert.equal(state, '{"supported":false,"vibrating":false}');
});
