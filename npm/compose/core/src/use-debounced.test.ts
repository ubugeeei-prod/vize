import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { useDebounced } from "./use-debounced.ts";

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
  const source = shallowRef("initial");
  const { debounced, pending, cancel, flush } = useDebounced(source, 200, { scheduler });

  source.value = "changed";
  assert.equal(debounced.value, "changed");
  assert.equal(pending.value, false);
  assert.equal(scheduler.active.size, 0);
  assert.equal(cancel(), false);
  assert.equal(flush(), false);
});

void test("defers updates until the wait elapses", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef("a");
  const { debounced, pending } = useDebounced(source, 300, { runOnServer: true, scheduler });

  assert.equal(debounced.value, "a");
  source.value = "b";
  assert.equal(debounced.value, "a");
  assert.equal(pending.value, true);
  assert.deepEqual(
    [...scheduler.active].map((timer) => timer.delayMs),
    [300],
  );

  scheduler.fire();
  assert.equal(debounced.value, "b");
  assert.equal(pending.value, false);
  assert.equal(scheduler.active.size, 0);
});

void test("restarts the wait on every synchronous write and applies the newest value", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef(0);
  const { debounced, pending } = useDebounced(source, 100, { runOnServer: true, scheduler });

  source.value = 1;
  source.value = 2;
  source.value = 3;
  assert.equal(scheduler.active.size, 1);
  assert.equal(scheduler.cleared.length, 2);
  assert.equal(pending.value, true);

  scheduler.fire();
  assert.equal(debounced.value, 3);
});

void test("cancel discards the trailing update and keeps debouncing afterwards", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef("kept");
  const { debounced, pending, cancel } = useDebounced(source, 100, {
    runOnServer: true,
    scheduler,
  });

  source.value = "discarded";
  assert.equal(cancel(), true);
  assert.equal(cancel(), false);
  assert.equal(debounced.value, "kept");
  assert.equal(pending.value, false);
  assert.equal(scheduler.active.size, 0);

  source.value = "next";
  assert.equal(pending.value, true);
  scheduler.fire();
  assert.equal(debounced.value, "next");
});

void test("flush applies the current source value immediately", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef("a");
  const { debounced, pending, flush } = useDebounced(source, 100, {
    runOnServer: true,
    scheduler,
  });

  assert.equal(flush(), false);
  source.value = "b";
  source.value = "c";
  assert.equal(flush(), true);
  assert.equal(debounced.value, "c");
  assert.equal(pending.value, false);
  assert.equal(scheduler.active.size, 0);
  assert.equal(flush(), false);
});

void test("reads the reactive wait when scheduling without restarting pending timers", () => {
  const scheduler = new TestTimeoutScheduler();
  const waitMs = shallowRef(100);
  const source = shallowRef(0);
  const { debounced } = useDebounced(source, waitMs, { runOnServer: true, scheduler });

  source.value = 1;
  waitMs.value = 25;
  assert.deepEqual(
    [...scheduler.active].map((timer) => timer.delayMs),
    [100],
  );
  assert.equal(scheduler.cleared.length, 0);

  scheduler.fire();
  assert.equal(debounced.value, 1);
  source.value = 2;
  assert.deepEqual(
    [...scheduler.active].map((timer) => timer.delayMs),
    [25],
  );
});

void test("rejects waits that cannot schedule a timer, even in mirror mode", () => {
  const isWaitError = (error: unknown): boolean =>
    error instanceof RangeError && error.message.startsWith("[VIZE_COMPOSE_DEBOUNCE_INVALID_WAIT]");

  for (const waitMs of [-1, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.throws(() => useDebounced(shallowRef(0), waitMs), isWaitError);
  }

  // A zero wait is valid: it defers to the next scheduler tick.
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef(0);
  const { debounced } = useDebounced(source, 0, { runOnServer: true, scheduler });
  source.value = 1;
  assert.equal(debounced.value, 0);
  scheduler.fire();
  assert.equal(debounced.value, 1);

  // A wait that turns invalid reactively fails at the scheduling write.
  const reactiveWait = shallowRef(100);
  const reactiveSource = shallowRef(0);
  useDebounced(reactiveSource, reactiveWait, { runOnServer: true, scheduler });
  reactiveWait.value = Number.NaN;
  assert.throws(() => {
    reactiveSource.value = 1;
  }, isWaitError);
});

void test("clears the pending timer when the owning scope stops", () => {
  const scheduler = new TestTimeoutScheduler();
  const source = shallowRef("a");
  const scope = effectScope();
  const controls = scope.run(() => useDebounced(source, 100, { runOnServer: true, scheduler }));
  assert.ok(controls);

  source.value = "b";
  assert.equal(controls.pending.value, true);
  assert.equal(scheduler.active.size, 1);

  scope.stop();
  assert.equal(scheduler.active.size, 0);
  assert.equal(scheduler.cleared.length, 1);
  assert.equal(controls.pending.value, false);
  assert.equal(controls.debounced.value, "a");
});
