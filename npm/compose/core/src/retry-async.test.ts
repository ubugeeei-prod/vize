import assert from "node:assert/strict";
import { test } from "node:test";

import { retryAsync } from "./retry-async.ts";
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

  fire(): void {
    const timeout = [...this.active][0];
    assert.ok(timeout);
    this.active.delete(timeout);
    timeout.callback();
  }
}

interface Deferred<Value> {
  readonly promise: Promise<Value>;
  readonly resolve: (value: Value) => void;
  readonly reject: (reason: unknown) => void;
}

function deferred<Value>(): Deferred<Value> {
  let resolve!: (value: Value) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<Value>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, reject, resolve };
}

async function drainMicrotasks(): Promise<void> {
  for (let index = 0; index < 8; index += 1) await Promise.resolve();
}

void test("returns the first successful value without scheduling", async () => {
  const scheduler = new TestScheduler();
  const attempts: number[] = [];

  const value = await retryAsync(
    ({ attempt, signal }) => {
      attempts.push(attempt);
      assert.equal(signal.aborted, false);
      return { ok: true } as const;
    },
    { scheduler },
  );

  assert.deepEqual(value, { ok: true });
  assert.deepEqual(attempts, [1]);
  assert.equal(scheduler.active.size, 0);
});

void test("retries synchronous and asynchronous failures with increasing delays", async () => {
  const scheduler = new TestScheduler();
  const errors = [new Error("first"), new Error("second")];
  const attempts: number[] = [];
  const execution = retryAsync(
    async ({ attempt }) => {
      attempts.push(attempt);
      const error = errors.shift();
      if (error !== undefined) throw error;
      return "complete" as const;
    },
    { scheduler },
  );

  await drainMicrotasks();
  assert.deepEqual(
    [...scheduler.active].map(({ delayMs }) => delayMs),
    [100],
  );
  scheduler.fire();
  await drainMicrotasks();
  assert.deepEqual(
    [...scheduler.active].map(({ delayMs }) => delayMs),
    [200],
  );
  scheduler.fire();

  assert.equal(await execution, "complete");
  assert.deepEqual(attempts, [1, 2, 3]);
});

void test("propagates the final operation error exactly after exhaustion", async () => {
  const scheduler = new TestScheduler();
  const failures = [new Error("one"), new Error("two")];
  let attempts = 0;
  const execution = retryAsync(
    () => {
      const error = failures[attempts];
      attempts += 1;
      throw error;
    },
    { maximumRetries: 1, scheduler },
  );

  await drainMicrotasks();
  scheduler.fire();

  await assert.rejects(execution, (error: unknown) => error === failures[1]);
  assert.equal(attempts, 2);
  assert.equal(scheduler.active.size, 0);
});

void test("a negative policy preserves the evaluated operation error", async () => {
  const scheduler = new TestScheduler();
  const failure = { code: "not-retryable" };
  const contexts: unknown[] = [];

  await assert.rejects(
    retryAsync(
      () => {
        throw failure;
      },
      {
        jitterRatio: 1,
        random: () => {
          throw new Error("entropy must stay lazy");
        },
        scheduler,
        shouldRetry: async (context) => {
          contexts.push(context);
          return false;
        },
      },
    ),
    (error: unknown) => error === failure,
  );
  assert.deepEqual(
    contexts.map((context) => {
      const { attempt, error, retryAttempt } = context as {
        attempt: number;
        error: unknown;
        retryAttempt: number;
      };
      return { attempt, error, retryAttempt };
    }),
    [{ attempt: 1, error: failure, retryAttempt: 1 }],
  );
  assert.equal(scheduler.active.size, 0);
});

void test("publishes the exact scheduled retry context before waiting", async () => {
  const scheduler = new TestScheduler();
  const failure = new Error("transient");
  const notifications: unknown[] = [];
  const execution = retryAsync(
    ({ attempt }) => {
      if (attempt === 1) throw failure;
      return attempt;
    },
    {
      initialDelayMs: 25,
      onRetry: async (context) => {
        notifications.push(context);
      },
      scheduler,
    },
  );

  await drainMicrotasks();
  const notification = notifications[0] as
    | {
        attempt: number;
        delayMs: number;
        error: unknown;
        nextAttempt: number;
        retryAttempt: number;
        signal: AbortSignal;
      }
    | undefined;
  assert.ok(notification);
  const { attempt, delayMs, error, nextAttempt, retryAttempt, signal } = notification;
  assert.deepEqual(
    { attempt, delayMs, error, nextAttempt, retryAttempt },
    {
      attempt: 1,
      delayMs: 25,
      error: failure,
      nextAttempt: 2,
      retryAttempt: 1,
    },
  );
  assert.equal(signal.aborted, false);
  scheduler.fire();
  assert.equal(await execution, 2);
});

void test("propagates a retry observer failure before allocating a timer", async () => {
  const scheduler = new TestScheduler();
  const observerFailure = { code: "observer-failed" };

  await assert.rejects(
    retryAsync(
      () => {
        throw new Error("transient");
      },
      {
        onRetry: async () => {
          throw observerFailure;
        },
        scheduler,
      },
    ),
    (error: unknown) => error === observerFailure,
  );
  assert.equal(scheduler.active.size, 0);
});

void test("pre-abort prevents every callback and timer allocation", async () => {
  const scheduler = new TestScheduler();
  const controller = new AbortController();
  const reason = { code: "cancelled-before-start" };
  let calls = 0;
  controller.abort(reason);

  await assert.rejects(
    retryAsync(
      () => {
        calls += 1;
      },
      { scheduler, signal: controller.signal },
    ),
    (error: unknown) => error === reason,
  );
  assert.equal(calls, 0);
  assert.equal(scheduler.active.size, 0);
});

void test("abort wins an operation race and observes its late rejection", async () => {
  const controller = new AbortController();
  const operation = deferred<never>();
  const reason = { code: "navigation" };
  const execution = retryAsync(() => operation.promise, { signal: controller.signal });

  await drainMicrotasks();
  controller.abort(reason);
  await assert.rejects(execution, (error: unknown) => error === reason);
  operation.reject(new Error("late operation failure"));
  await drainMicrotasks();
});

void test("abort during backoff releases the owned timer", async () => {
  const scheduler = new TestScheduler();
  const controller = new AbortController();
  const reason = new Error("stopped");
  let attempts = 0;
  const execution = retryAsync(
    () => {
      attempts += 1;
      throw new Error("retry me");
    },
    { scheduler, signal: controller.signal },
  );

  await drainMicrotasks();
  assert.equal(scheduler.active.size, 1);
  controller.abort(reason);

  await assert.rejects(execution, (error: unknown) => error === reason);
  assert.equal(attempts, 1);
  assert.equal(scheduler.active.size, 0);
  assert.equal(scheduler.cleared.length, 1);
});

void test("abort wins an asynchronous retry-policy decision", async () => {
  const scheduler = new TestScheduler();
  const controller = new AbortController();
  const decision = deferred<boolean>();
  const reason = { code: "policy-cancelled" };
  let randomCalls = 0;
  const execution = retryAsync(
    () => {
      throw new Error("transient");
    },
    {
      jitterRatio: 1,
      random: () => {
        randomCalls += 1;
        return 0;
      },
      scheduler,
      shouldRetry: () => decision.promise,
      signal: controller.signal,
    },
  );

  await drainMicrotasks();
  controller.abort(reason);
  await assert.rejects(execution, (error: unknown) => error === reason);
  decision.resolve(true);
  await drainMicrotasks();
  assert.equal(randomCalls, 0);
  assert.equal(scheduler.active.size, 0);
});
