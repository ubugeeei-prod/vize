import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type { IntervalScheduler } from "./use-interval.ts";
import { usePersistentStorage, useStorageEstimate } from "./use-storage-estimate.ts";
import type { StorageEstimateLike, StorageManagerLike } from "./use-storage-estimate.ts";

class FakeStorage implements StorageManagerLike {
  estimates = 0;
  next: StorageEstimateLike = { usage: 25, quota: 100 };
  failure: unknown = undefined;
  isPersisted = false;
  grant = true;

  estimate(): Promise<StorageEstimateLike> {
    this.estimates += 1;
    return this.failure === undefined ? Promise.resolve(this.next) : Promise.reject(this.failure);
  }

  persisted(): Promise<boolean> {
    return Promise.resolve(this.isPersisted);
  }

  persist(): Promise<boolean> {
    if (this.failure !== undefined) return Promise.reject(this.failure);
    this.isPersisted = this.grant;
    return Promise.resolve(this.grant);
  }
}

class FakeScheduler implements IntervalScheduler {
  readonly timers = new Map<number, { callback: () => void; intervalMs: number }>();
  private nextId = 0;

  setInterval = (callback: () => void, intervalMs: number): unknown => {
    this.nextId += 1;
    this.timers.set(this.nextId, { callback, intervalMs });
    return this.nextId;
  };

  clearInterval = (handle: unknown): void => {
    if (typeof handle === "number") this.timers.delete(handle);
  };

  tick(): void {
    for (const timer of this.timers.values()) timer.callback();
  }
}

async function flushPromises(): Promise<void> {
  for (let index = 0; index < 4; index += 1) await Promise.resolve();
}

void test("reads usage, quota and details on creation", async () => {
  const storage = new FakeStorage();
  storage.next = { usage: 30, quota: 120, usageDetails: { indexedDB: 30 } };
  const estimate = useStorageEstimate({ storage });
  assert.equal(estimate.supported.value, true);
  await flushPromises();
  assert.equal(estimate.usage.value, 30);
  assert.equal(estimate.quota.value, 120);
  assert.deepEqual(estimate.usageDetails.value, { indexedDB: 30 });
  assert.equal(estimate.percentUsed.value, 25);
});

void test("refresh re-reads and missing details stay null", async () => {
  const storage = new FakeStorage();
  const estimate = useStorageEstimate({ storage, immediate: false });
  assert.equal(estimate.usage.value, null);
  assert.equal(estimate.percentUsed.value, null);
  assert.equal(await estimate.refresh(), true);
  assert.equal(estimate.percentUsed.value, 25);
  assert.equal(estimate.usageDetails.value, null);
});

void test("polls through the injected scheduler and stops with the scope", async () => {
  const storage = new FakeStorage();
  const scheduler = new FakeScheduler();
  const scope = effectScope();
  const estimate = scope.run(() => useStorageEstimate({ storage, interval: 1000, scheduler }));
  assert.ok(estimate);
  await flushPromises();
  assert.equal(storage.estimates, 1);
  assert.equal([...scheduler.timers.values()][0]?.intervalMs, 1000);
  storage.next = { usage: 50, quota: 100 };
  scheduler.tick();
  await flushPromises();
  assert.equal(storage.estimates, 2);
  assert.equal(estimate.percentUsed.value, 50);
  scope.stop();
  assert.equal(scheduler.timers.size, 0);
});

void test("re-estimates when a reactive host changes", async () => {
  const first = new FakeStorage();
  const second = new FakeStorage();
  second.next = { usage: 1, quota: 4 };
  const host = shallowRef<StorageManagerLike | null>(first);
  const estimate = useStorageEstimate({ storage: host });
  await flushPromises();
  host.value = second;
  await nextTick();
  await flushPromises();
  assert.equal(estimate.percentUsed.value, 25);
  host.value = null;
  await nextTick();
  assert.equal(estimate.supported.value, false);
});

void test("an estimate from a removed host cannot restore stale state", async () => {
  let resolveFirst: (estimate: StorageEstimateLike) => void = () => {};
  const first: StorageManagerLike = {
    estimate: () =>
      new Promise((resolve) => {
        resolveFirst = resolve;
      }),
  };
  const host = shallowRef<StorageManagerLike | null>(first);
  const estimate = useStorageEstimate({ storage: host });
  host.value = null;
  await nextTick();
  resolveFirst({ usage: 90, quota: 100 });
  await flushPromises();

  assert.equal(estimate.supported.value, false);
  assert.equal(estimate.usage.value, null);
  assert.equal(estimate.quota.value, null);
  assert.equal(estimate.percentUsed.value, null);
});

void test("exposes estimate failures and validates the interval", async () => {
  const storage = new FakeStorage();
  const failure = new Error("denied");
  storage.failure = failure;
  const estimate = useStorageEstimate({ storage, immediate: false });
  assert.equal(await estimate.refresh(), false);
  assert.equal(estimate.error.value, failure);
  storage.failure = undefined;
  assert.equal(await estimate.refresh(), true);
  assert.equal(estimate.error.value, undefined);
  assert.throws(
    () => useStorageEstimate({ storage, interval: -1 }),
    /VIZE_COMPOSE_STORAGE_ESTIMATE_INVALID_INTERVAL/,
  );
});

void test("reports unsupported without a storage manager", async () => {
  const estimate = useStorageEstimate({ storage: null });
  assert.equal(estimate.supported.value, false);
  assert.equal(await estimate.refresh(), false);
  const persistent = usePersistentStorage({ storage: {} });
  assert.equal(persistent.supported.value, false);
  assert.equal(await persistent.persist(), false);
});

void test("reads and requests persistent storage", async () => {
  const storage = new FakeStorage();
  storage.isPersisted = true;
  const persistent = usePersistentStorage({ storage });
  assert.equal(persistent.supported.value, true);
  await flushPromises();
  assert.equal(persistent.persisted.value, true);

  const fresh = new FakeStorage();
  fresh.grant = false;
  const denied = usePersistentStorage({ storage: fresh, immediate: false });
  assert.equal(await denied.persist(), false);
  assert.equal(denied.persisted.value, false);
  fresh.grant = true;
  assert.equal(await denied.persist(), true);
  assert.equal(await denied.check(), true);
  assert.equal(denied.persisted.value, true);
});

void test("persist failures land in error", async () => {
  const storage = new FakeStorage();
  const failure = new Error("blocked");
  storage.failure = failure;
  const persistent = usePersistentStorage({ storage, immediate: false });
  assert.equal(await persistent.persist(), false);
  assert.equal(persistent.error.value, failure);
});

void test("server rendering reads nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const estimate = useStorageEstimate({ interval: 1000 });
    const persistent = usePersistentStorage();
    return {
      supported: estimate.supported,
      usage: estimate.usage,
      quota: estimate.quota,
      usageDetails: estimate.usageDetails,
      percentUsed: estimate.percentUsed,
      persistSupported: persistent.supported,
      persisted: persistent.persisted,
    };
  });
  assert.equal(
    state,
    '{"supported":false,"usage":null,"quota":null,"usageDetails":null,"percentUsed":null,"persistSupported":false,"persisted":false}',
  );
});
