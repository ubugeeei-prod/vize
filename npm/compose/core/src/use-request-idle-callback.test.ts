import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { idle, useRequestIdleCallback } from "./use-request-idle-callback.ts";
import type { IdleCallbackHost, IdleDeadlineLike } from "./use-request-idle-callback.ts";

function createIdleHost(): {
  host: IdleCallbackHost;
  pending: Map<number, (deadline: IdleDeadlineLike) => void>;
  timeouts: (number | undefined)[];
  runAll: (didTimeout?: boolean) => void;
} {
  const pending = new Map<number, (deadline: IdleDeadlineLike) => void>();
  const timeouts: (number | undefined)[] = [];
  let next = 0;
  return {
    pending,
    timeouts,
    host: {
      requestIdleCallback: (callback, options) => {
        next += 1;
        pending.set(next, callback);
        timeouts.push(options?.timeout);
        return next;
      },
      cancelIdleCallback: (handle) => {
        pending.delete(handle);
      },
    },
    runAll: (didTimeout = false) => {
      const callbacks = [...pending.values()];
      pending.clear();
      for (const callback of callbacks) callback({ didTimeout, timeRemaining: () => 12 });
    },
  };
}

function createTimers(): {
  timers: TimeoutScheduler;
  queue: Map<number, { callback: () => void; delay: number }>;
  flush: () => void;
} {
  const queue = new Map<number, { callback: () => void; delay: number }>();
  let next = 0;
  return {
    queue,
    timers: {
      setTimeout: (callback, delay) => {
        next += 1;
        queue.set(next, { callback, delay });
        return next;
      },
      clearTimeout: (handle) => {
        if (typeof handle === "number") queue.delete(handle);
      },
    },
    flush: () => {
      const tasks = [...queue.values()];
      queue.clear();
      for (const task of tasks) task.callback();
    },
  };
}

void test("schedules through the native host with a timeout", () => {
  const { host, runAll, timeouts } = createIdleHost();
  const deadlines: IdleDeadlineLike[] = [];
  const idleCallback = useRequestIdleCallback((deadline) => deadlines.push(deadline), {
    host,
    timeout: 500,
  });

  assert.equal(idleCallback.supported.value, true);
  assert.equal(idleCallback.isPending.value, true);
  assert.deepEqual(timeouts, [500]);
  runAll(true);
  assert.equal(idleCallback.isPending.value, false);
  assert.equal(deadlines[0]?.didTimeout, true);
  assert.equal(deadlines[0]?.timeRemaining(), 12);
});

void test("start replaces a pending callback and cancel removes it", () => {
  const { host, pending } = createIdleHost();
  let runs = 0;
  const idleCallback = useRequestIdleCallback(() => (runs += 1), { host, immediate: false });
  assert.equal(idleCallback.isPending.value, false);

  idleCallback.start();
  idleCallback.start();
  assert.equal(pending.size, 1);
  idleCallback.cancel();
  assert.equal(pending.size, 0);
  assert.equal(idleCallback.isPending.value, false);
  assert.equal(runs, 0);
});

void test("falls back to timers with a fabricated deadline", () => {
  const { timers, queue, flush } = createTimers();
  let clock = 1000;
  const deadlines: IdleDeadlineLike[] = [];
  const idleCallback = useRequestIdleCallback((deadline) => deadlines.push(deadline), {
    host: null,
    timers,
    now: () => clock,
  });

  assert.equal(idleCallback.supported.value, false);
  assert.equal(idleCallback.isPending.value, true);
  assert.equal([...queue.values()][0]?.delay, 1);
  flush();
  assert.equal(deadlines[0]?.didTimeout, false);
  assert.equal(deadlines[0]?.timeRemaining(), 50);
  clock += 30;
  assert.equal(deadlines[0]?.timeRemaining(), 20);
  clock += 100;
  assert.equal(deadlines[0]?.timeRemaining(), 0);
});

void test("schedules nothing without a host or timers", () => {
  const idleCallback = useRequestIdleCallback(() => undefined, { host: null, timers: null });
  assert.equal(idleCallback.isPending.value, false);
});

void test("rejects an invalid timeout", () => {
  assert.throws(
    () => useRequestIdleCallback(() => undefined, { timeout: Number.NaN }),
    /VIZE_COMPOSE_IDLE_CALLBACK_INVALID_TIMEOUT/,
  );
  assert.throws(() => idle({ timeout: -1 }), /VIZE_COMPOSE_IDLE_CALLBACK_INVALID_TIMEOUT/);
});

void test("cancels the pending callback with the scope", () => {
  const { timers, queue } = createTimers();
  const scope = effectScope();
  scope.run(() => useRequestIdleCallback(() => undefined, { host: null, timers }));
  assert.equal(queue.size, 1);
  scope.stop();
  assert.equal(queue.size, 0);
});

void test("idle resolves with the deadline", async () => {
  const { host, runAll } = createIdleHost();
  const waiting = idle({ host });
  runAll();
  assert.equal((await waiting).timeRemaining(), 12);

  const fallback = await idle({ host: null, timers: null });
  assert.equal(fallback.timeRemaining(), 0);
});

void test("idle rejects and cancels when aborted", async () => {
  const { host, pending } = createIdleHost();
  const controller = new AbortController();
  const waiting = idle({ host, signal: controller.signal });
  controller.abort(new Error("stop"));
  await assert.rejects(waiting, /stop/);
  assert.equal(pending.size, 0);
  await assert.rejects(idle({ host, signal: controller.signal }), /stop/);
});

void test("server rendering schedules nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const idleCallback = useRequestIdleCallback(() => undefined, { timeout: 100 });
    return { supported: idleCallback.supported, isPending: idleCallback.isPending };
  });
  assert.equal(state, '{"supported":false,"isPending":false}');
});
