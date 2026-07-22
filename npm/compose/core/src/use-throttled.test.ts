import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { useThrottled } from "./use-throttled.ts";

interface ScheduledTimeout {
  readonly callback: () => void;
  readonly delayMs: number;
}

class TestTimeoutScheduler implements TimeoutScheduler {
  readonly active = new Set<ScheduledTimeout>();
  readonly cleared: ScheduledTimeout[] = [];

  setTimeout(callback: () => void, delayMs: number): ScheduledTimeout {
    const timer = { callback, delayMs };
    this.active.add(timer);
    return timer;
  }

  clearTimeout(handle: unknown): void {
    const timer = handle as ScheduledTimeout;
    if (this.active.delete(timer)) this.cleared.push(timer);
  }

  fire(): void {
    const timers = [...this.active];
    this.active.clear();
    for (const timer of timers) timer.callback();
  }
}

void test("mirrors the source synchronously during server rendering", () => {
  // Node has no `window`, so without `runOnServer` this is the server path
  // even though a scheduler is provided.
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef(0);
  const { throttled, pending, cancel, flush } = useThrottled(source, 100, { scheduler });

  source.value = 1;
  source.value = 2;
  assert.equal(throttled.value, 2);
  assert.equal(pending.value, false);
  assert.equal(scheduler.active.size, 0);
  assert.equal(cancel(), false);
  assert.equal(flush(), false);
});

void test("applies the leading edge and chains trailing updates", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef(0);
  const { throttled, pending } = useThrottled(source, 100, { runOnServer: true, scheduler });

  source.value = 1;
  assert.equal(throttled.value, 1);
  assert.equal(pending.value, false);
  assert.equal(scheduler.active.size, 1);

  source.value = 2;
  source.value = 3;
  assert.equal(throttled.value, 1);
  assert.equal(pending.value, true);
  assert.equal(scheduler.active.size, 1);

  // The window ends: the newest value applies and the next window opens
  // back to back, keeping applications spaced.
  scheduler.fire();
  assert.equal(throttled.value, 3);
  assert.equal(pending.value, false);
  assert.equal(scheduler.active.size, 1);

  // A quiet window closes silently and the next change leads again.
  scheduler.fire();
  assert.equal(scheduler.active.size, 0);
  source.value = 4;
  assert.equal(throttled.value, 4);
});

void test("waits for the window end when leading is disabled", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef("start");
  const { throttled, pending } = useThrottled(source, 100, {
    leading: false,
    runOnServer: true,
    scheduler,
  });

  source.value = "queued";
  assert.equal(throttled.value, "start");
  assert.equal(pending.value, true);

  scheduler.fire();
  assert.equal(throttled.value, "queued");
  assert.equal(pending.value, false);
});

void test("drops window changes when trailing is disabled", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef(0);
  const { throttled, pending } = useThrottled(source, 100, {
    trailing: false,
    runOnServer: true,
    scheduler,
  });

  source.value = 1;
  assert.equal(throttled.value, 1);
  source.value = 2;
  assert.equal(pending.value, false);

  scheduler.fire();
  assert.equal(throttled.value, 1);
  assert.equal(scheduler.active.size, 0);

  source.value = 3;
  assert.equal(throttled.value, 3);
});

void test("rejects disabling both edges", () => {
  assert.throws(
    () => useThrottled(shallowRef(0), 100, { leading: false, trailing: false }),
    (error: unknown) =>
      error instanceof TypeError &&
      error.message.startsWith("[VIZE_COMPOSE_THROTTLE_INVALID_EDGES]"),
  );
});

void test("cancel closes the window so the next change leads again", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef(0);
  const { throttled, pending, cancel } = useThrottled(source, 100, {
    runOnServer: true,
    scheduler,
  });

  source.value = 1;
  source.value = 2;
  assert.equal(cancel(), true);
  assert.equal(pending.value, false);
  assert.equal(throttled.value, 1);
  assert.equal(scheduler.active.size, 0);

  // An open window without a trailing update reports false but still closes.
  source.value = 3;
  assert.equal(throttled.value, 3);
  assert.equal(cancel(), false);
  assert.equal(scheduler.active.size, 0);
  source.value = 4;
  assert.equal(throttled.value, 4);
});

void test("flush applies the trailing update immediately and closes the window", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef("a");
  const { throttled, pending, flush } = useThrottled(source, 100, {
    runOnServer: true,
    scheduler,
  });

  assert.equal(flush(), false);
  source.value = "b";
  source.value = "c";
  assert.equal(flush(), true);
  assert.equal(throttled.value, "c");
  assert.equal(pending.value, false);
  assert.equal(scheduler.active.size, 0);

  source.value = "d";
  assert.equal(throttled.value, "d");
});

void test("reads the reactive wait each time a window opens", () => {
  const scheduler = new TestTimeoutScheduler();
  const waitMs = shallowRef(100);
  const source = shallowRef(0);
  useThrottled(source, waitMs, { runOnServer: true, scheduler });

  source.value = 1;
  assert.deepEqual(
    [...scheduler.active].map((timer) => timer.delayMs),
    [100],
  );

  // Changing the wait never disturbs the already-open window; the chained
  // window opened by a trailing update uses the new wait.
  waitMs.value = 40;
  source.value = 2;
  assert.deepEqual(
    [...scheduler.active].map((timer) => timer.delayMs),
    [100],
  );
  scheduler.fire();
  assert.deepEqual(
    [...scheduler.active].map((timer) => timer.delayMs),
    [40],
  );
});

void test("rejects waits that cannot schedule a timer, even in mirror mode", () => {
  const isWaitError = (error: unknown): boolean =>
    error instanceof RangeError && error.message.startsWith("[VIZE_COMPOSE_THROTTLE_INVALID_WAIT]");

  for (const waitMs of [-1, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.throws(() => useThrottled(shallowRef(0), waitMs), isWaitError);
  }

  const reactiveWait = shallowRef(100);
  const reactiveSource = shallowRef(0);
  useThrottled(reactiveSource, reactiveWait, {
    runOnServer: true,
    scheduler: new TestTimeoutScheduler(),
  });
  reactiveWait.value = Number.NaN;
  assert.throws(() => {
    reactiveSource.value = 1;
  }, isWaitError);
});

void test("clears the window timer when the owning scope stops", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef(0);
  const scope = effectScope();
  const controls = scope.run(() => useThrottled(source, 100, { runOnServer: true, scheduler }));
  assert.ok(controls);

  source.value = 1;
  source.value = 2;
  assert.equal(controls.pending.value, true);
  assert.equal(scheduler.active.size, 1);

  scope.stop();
  assert.equal(scheduler.active.size, 0);
  assert.equal(scheduler.cleared.length, 1);
  assert.equal(controls.pending.value, false);
  assert.equal(controls.throttled.value, 1);
});
