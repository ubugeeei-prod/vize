import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { effectScope, nextTick, ref } from "vue";
import type { EffectScope } from "vue";
import {
  Temporal,
  useTemporalNow,
  useTemporalZonedDateTime,
  type TemporalScheduler,
} from "./index.ts";

interface ScheduledTimer {
  readonly callback: () => void;
  readonly intervalMs: number;
}

class TestScheduler implements TemporalScheduler {
  readonly active = new Set<ScheduledTimer>();
  readonly cleared: ScheduledTimer[] = [];

  setInterval(callback: () => void, intervalMs: number): ScheduledTimer {
    const timer = { callback, intervalMs };
    this.active.add(timer);
    return timer;
  }

  clearInterval(handle: unknown): void {
    const timer = handle as ScheduledTimer;
    this.active.delete(timer);
    this.cleared.push(timer);
  }

  tick(): void {
    for (const timer of this.active) timer.callback();
  }
}

function runInScope<T>(callback: () => T): [EffectScope, T] {
  const scope = effectScope();
  const value = scope.run(callback);
  assert.notEqual(value, undefined);
  return [scope, value as T];
}

void test("provides a deterministic clock without starting a server timer", () => {
  const scheduler = new TestScheduler();
  const expected = Temporal.Instant.from("2026-07-19T00:00:00Z");
  const [scope, clock] = runInScope(() =>
    useTemporalNow({
      now: () => expected,
      scheduler,
    }),
  );

  assert.equal(clock.instant.value.toString(), "2026-07-19T00:00:00Z");
  assert.equal(clock.refresh(), expected);
  assert.equal(scheduler.active.size, 0);
  scope.stop();
});

void test("replaces, pauses, resumes, and disposes the active timer", async () => {
  const scheduler = new TestScheduler();
  const intervalMs = ref(250);
  const paused = ref(false);
  let epochNanoseconds = 0n;
  const [scope, clock] = runInScope(() =>
    useTemporalNow({
      intervalMs,
      paused,
      runOnServer: true,
      scheduler,
      now: () => Temporal.Instant.fromEpochNanoseconds(epochNanoseconds++),
    }),
  );

  assert.deepEqual(
    [...scheduler.active].map((timer) => timer.intervalMs),
    [250],
  );
  scheduler.tick();
  assert.equal(clock.instant.value.epochNanoseconds, 1n);

  intervalMs.value = 400.9;
  await nextTick();
  assert.deepEqual(
    [...scheduler.active].map((timer) => timer.intervalMs),
    [400],
  );
  assert.equal(scheduler.cleared.length, 1);

  paused.value = true;
  await nextTick();
  assert.equal(scheduler.active.size, 0);
  assert.equal(scheduler.cleared.length, 2);

  paused.value = false;
  await nextTick();
  assert.equal(scheduler.active.size, 1);

  scope.stop();
  assert.equal(scheduler.active.size, 0);
  assert.equal(scheduler.cleared.length, 3);
});

void test("rejects intervals that cannot create a predictable timer", () => {
  for (const intervalMs of [0, -1, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.throws(
      () =>
        runInScope(() =>
          useTemporalNow({
            intervalMs,
            runOnServer: true,
            scheduler: new TestScheduler(),
          }),
        ),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_TEMPORAL_INVALID_INTERVAL]"),
    );
  }
});

void test("reacts to time-zone changes", () => {
  const timeZone = ref<Temporal.TimeZoneLike>("UTC");
  const instant = Temporal.Instant.from("2026-07-19T00:00:00Z");
  const [scope, zoned] = runInScope(() =>
    useTemporalZonedDateTime({
      now: () => instant,
      timeZone,
    }),
  );

  assert.equal(zoned.value.hour, 0);
  timeZone.value = "Asia/Tokyo";
  assert.equal(zoned.value.hour, 9);
  scope.stop();
});

void test("publishes an importable ESM distribution with declarations", async () => {
  const packageJson = JSON.parse(
    await readFile(new URL("../package.json", import.meta.url), "utf8"),
  ) as {
    readonly exports: {
      readonly ".": { readonly import: string; readonly types: string };
    };
  };

  assert.equal(packageJson.exports["."].import, "./dist/index.mjs");
  assert.equal(packageJson.exports["."].types, "./dist/index.d.mts");

  const distribution = await import("../dist/index.mjs");
  assert.equal(typeof distribution.useTemporalNow, "function");
  assert.equal(typeof distribution.Temporal.Instant.from, "function");
});
