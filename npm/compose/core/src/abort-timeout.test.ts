import assert from "node:assert/strict";
import { test } from "node:test";

import { deadlineAbortSignal, timeoutAbortSignal } from "./abort-signal.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

interface ScheduledTimeout {
  readonly callback: () => void;
  readonly delayMs: number;
}

class TestScheduler implements TimeoutScheduler {
  readonly active = new Set<ScheduledTimeout>();
  readonly cleared: ScheduledTimeout[] = [];

  setTimeout(callback: () => void, delayMs: number): ScheduledTimeout {
    const timeout = { callback, delayMs };
    this.active.add(timeout);
    return timeout;
  }

  clearTimeout(handle: unknown): void {
    const timeout = handle as ScheduledTimeout;
    this.active.delete(timeout);
    this.cleared.push(timeout);
  }

  fire(timeout?: ScheduledTimeout): void {
    const selected = timeout ?? [...this.active][0];
    assert.ok(selected);
    this.active.delete(selected);
    selected.callback();
  }
}

function withoutNativeTimeout<Value>(callback: () => Value): Value {
  const descriptor = Object.getOwnPropertyDescriptor(AbortSignal, "timeout");
  Object.defineProperty(AbortSignal, "timeout", {
    configurable: true,
    value: undefined,
    writable: true,
  });
  try {
    return callback();
  } finally {
    if (descriptor === undefined) delete (AbortSignal as { timeout?: unknown }).timeout;
    else Object.defineProperty(AbortSignal, "timeout", descriptor);
  }
}

void test("delegates the simple case to the runtime timeout standard", () => {
  const descriptor = Object.getOwnPropertyDescriptor(AbortSignal, "timeout");
  const expected = new AbortController().signal;
  let receivedDelay: number | undefined;
  Object.defineProperty(AbortSignal, "timeout", {
    configurable: true,
    value: (delayMs: number) => {
      receivedDelay = delayMs;
      return expected;
    },
    writable: true,
  });
  try {
    assert.equal(timeoutAbortSignal(125), expected);
    assert.equal(receivedDelay, 125);
  } finally {
    if (descriptor === undefined) delete (AbortSignal as { timeout?: unknown }).timeout;
    else Object.defineProperty(AbortSignal, "timeout", descriptor);
  }
});

void test("falls back when the runtime timeout standard is unavailable", async () => {
  const signal = withoutNativeTimeout(() => timeoutAbortSignal(0));

  assert.equal(signal.aborted, false);
  await new Promise((resolve) => setTimeout(resolve, 5));
  assert.equal(signal.aborted, true);
  assert.ok(signal.reason instanceof DOMException);
  assert.equal(signal.reason.name, "TimeoutError");
});

void test("custom scheduling aborts asynchronously with a standard timeout reason", () => {
  const scheduler = new TestScheduler();
  const signal = timeoutAbortSignal(0, { scheduler });

  assert.equal(signal.aborted, false);
  assert.deepEqual(
    [...scheduler.active].map((timeout) => timeout.delayMs),
    [0],
  );
  scheduler.fire();

  assert.equal(signal.aborted, true);
  assert.ok(signal.reason instanceof DOMException);
  assert.equal(signal.reason.name, "TimeoutError");
  assert.equal(scheduler.active.size, 0);
});

void test("preserves a custom timeout reason including null", () => {
  const scheduler = new TestScheduler();
  const signal = timeoutAbortSignal(10, { scheduler, reason: null });

  scheduler.fire();
  assert.equal(signal.reason, null);
});

void test("parent cancellation wins and releases the owned timer", () => {
  const scheduler = new TestScheduler();
  const parent = new AbortController();
  const reason = { code: "navigation" };
  const signal = timeoutAbortSignal(500, { scheduler, signal: parent.signal });

  parent.abort(reason);

  assert.equal(signal.aborted, true);
  assert.equal(signal.reason, reason);
  assert.equal(scheduler.active.size, 0);
  assert.equal(scheduler.cleared.length, 1);
});

void test("an already-aborted parent prevents timer allocation", () => {
  const scheduler = new TestScheduler();
  const parent = new AbortController();
  parent.abort("already cancelled");

  const signal = timeoutAbortSignal(500, { scheduler, signal: parent.signal });

  assert.equal(signal.reason, "already cancelled");
  assert.equal(scheduler.active.size, 0);
  assert.equal(scheduler.cleared.length, 0);
});

void test("an already-aborted parent bypasses the native timeout", () => {
  const descriptor = Object.getOwnPropertyDescriptor(AbortSignal, "timeout");
  const parent = new AbortController();
  const reason = { code: "already-cancelled" };
  let allocations = 0;
  parent.abort(reason);
  Object.defineProperty(AbortSignal, "timeout", {
    configurable: true,
    value: () => {
      allocations += 1;
      return new AbortController().signal;
    },
    writable: true,
  });
  try {
    const signal = timeoutAbortSignal(500, { signal: parent.signal });

    assert.equal(signal.reason, reason);
    assert.equal(allocations, 0);
  } finally {
    if (descriptor === undefined) delete (AbortSignal as { timeout?: unknown }).timeout;
    else Object.defineProperty(AbortSignal, "timeout", descriptor);
  }
});

void test("a scheduler failure cannot attach a parent listener", () => {
  const parent = new AbortController();
  let additions = 0;
  const originalAdd = parent.signal.addEventListener.bind(parent.signal);
  parent.signal.addEventListener = (
    type: string,
    callback: EventListenerOrEventListenerObject,
    options?: AddEventListenerOptions | boolean,
  ) => {
    additions += 1;
    return originalAdd(type, callback, options);
  };
  const scheduler: TimeoutScheduler = {
    setTimeout: () => {
      throw new Error("scheduler unavailable");
    },
    clearTimeout: () => undefined,
  };

  assert.throws(
    () => timeoutAbortSignal(1, { scheduler, signal: parent.signal }),
    /scheduler unavailable/,
  );
  assert.equal(additions, 0);
});

void test("a partial parent registration is released before rethrowing", () => {
  const scheduler = new TestScheduler();
  const parent = new AbortController();
  let removals = 0;
  const originalAdd = parent.signal.addEventListener.bind(parent.signal);
  const originalRemove = parent.signal.removeEventListener.bind(parent.signal);
  parent.signal.addEventListener = (
    type: string,
    callback: EventListenerOrEventListenerObject,
    options?: AddEventListenerOptions | boolean,
  ) => {
    originalAdd(type, callback, options);
    throw new Error("registration failed after attaching");
  };
  parent.signal.removeEventListener = (
    type: string,
    callback: EventListenerOrEventListenerObject,
    options?: EventListenerOptions | boolean,
  ) => {
    removals += 1;
    return originalRemove(type, callback, options);
  };

  assert.throws(
    () => timeoutAbortSignal(500, { scheduler, signal: parent.signal }),
    /registration failed after attaching/,
  );
  assert.equal(removals, 1);
  assert.equal(scheduler.active.size, 0);
  assert.equal(scheduler.cleared.length, 1);
});

void test("rejects delays that hosts would clamp or reinterpret", () => {
  for (const delayMs of [-1, 0.5, Number.NaN, Number.POSITIVE_INFINITY, 2_147_483_648]) {
    assert.throws(
      () => timeoutAbortSignal(delayMs),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_ABORT_TIMEOUT_INVALID_DELAY]"),
    );
  }
});

void test("converts future, fractional, and past deadlines predictably", () => {
  const futureScheduler = new TestScheduler();
  deadlineAbortSignal(1_500.25, {
    now: () => 1_000,
    scheduler: futureScheduler,
  });
  assert.deepEqual(
    [...futureScheduler.active].map((timeout) => timeout.delayMs),
    [501],
  );

  const pastScheduler = new TestScheduler();
  const past = deadlineAbortSignal(900, { now: () => 1_000, scheduler: pastScheduler });
  assert.equal(past.aborted, false);
  assert.deepEqual(
    [...pastScheduler.active].map((timeout) => timeout.delayMs),
    [0],
  );
});

void test("rejects invalid deadlines and clock values before allocating timers", () => {
  const scheduler = new TestScheduler();
  for (const [deadline, now] of [
    [Number.NaN, 0],
    [0, Number.NaN],
    [Number.POSITIVE_INFINITY, 0],
    [2_147_483_648, 0],
  ] as const) {
    assert.throws(
      () => deadlineAbortSignal(deadline, { now: () => now, scheduler }),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_ABORT_DEADLINE_INVALID]"),
    );
  }
  assert.equal(scheduler.active.size, 0);
});
