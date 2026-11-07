import assert from "node:assert/strict";
import { test } from "node:test";

import { createDataClient, defineDataResource } from "./index.ts";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => (resolve = done));
  return { promise, resolve };
}

void test("latest revalidation owns the cache even when an older request finishes last", async () => {
  const first = deferred<{ revision: number }>();
  const second = deferred<{ revision: number }>();
  let calls = 0;
  const resource = defineDataResource({
    key: "race.latest",
    source: "../fixtures/UserCard.vue",
    loader: () => (++calls === 1 ? first.promise : second.promise),
  });
  const client = createDataClient();
  const older = client.load(resource, {}, { policy: "revalidate" });
  const newer = client.load(resource, {}, { policy: "revalidate" });
  second.resolve({ revision: 2 });
  await newer;
  first.resolve({ revision: 1 });
  await older;
  const state = client.peek(resource, {});
  assert.deepEqual(state?.status === "success" ? state.data : undefined, { revision: 2 });
});

void test("replace aborts the previous request and explicit cancel reports cancellation", async () => {
  let call = 0;
  const resource = defineDataResource({
    key: "race.replace",
    source: "../fixtures/UserCard.vue",
    loader: ({ signal }) => {
      call += 1;
      if (call === 2) return Promise.resolve({ revision: 2 });
      return new Promise<{ revision: number }>((_resolve, reject) => {
        signal.addEventListener("abort", () => reject(signal.reason), { once: true });
      });
    },
  });
  const client = createDataClient();
  const previous = client.load(resource, {});
  const replacement = client.load(resource, {}, { policy: "replace" });
  assert.equal((await previous).status, "cancelled");
  assert.equal((await replacement).status, "success");

  const cancellable = client.load(resource, { next: true } as never, { policy: "replace" });
  assert.equal(client.cancel(resource, { next: true } as never, "route-left"), true);
  assert.equal((await cancellable).status, "cancelled");
});

void test("stale-while-revalidate returns old data while refreshing the cache", async () => {
  const refresh = deferred<{ revision: number }>();
  let calls = 0;
  const resource = defineDataResource({
    key: "cache.swr",
    source: "../fixtures/UserCard.vue",
    loader: () => (++calls === 1 ? { revision: 1 } : refresh.promise),
  });
  const client = createDataClient({ now: () => 50 });
  await client.load(resource, {});
  const stale = await client.load(resource, {}, { policy: "stale-while-revalidate" });
  assert.equal(stale.status, "stale");
  assert.deepEqual(stale.status === "stale" ? stale.data : undefined, { revision: 1 });
  refresh.resolve({ revision: 2 });
  await Promise.resolve();
  await Promise.resolve();
  const current = client.peek(resource, {});
  assert.deepEqual(current?.status === "success" ? current.data : undefined, { revision: 2 });
});

void test("optimistic updates roll back only while they still own the cache", async () => {
  const resource = defineDataResource({
    key: "optimistic.item",
    source: "../fixtures/UserCard.vue",
    loader: () => ({ count: 1 }),
  });
  const client = createDataClient({ now: () => 75 });
  await client.load(resource, {});
  const rollback = client.optimistic(resource, {}, (current) => ({ count: current.count + 1 }));
  const optimistic = client.peek(resource, {});
  assert.deepEqual(optimistic?.status === "stale" ? optimistic.data : undefined, { count: 2 });
  assert.equal(rollback(), true);
  const rolledBack = client.peek(resource, {});
  assert.deepEqual(rolledBack?.status === "success" ? rolledBack.data : undefined, { count: 1 });
  assert.equal(rollback(), false);
});

void test("unsupported hosts and unsubscribed listeners stay side-effect free", async () => {
  let loads = 0;
  let observations = 0;
  const resource = defineDataResource({
    key: "host.unsupported",
    source: "../fixtures/UserCard.vue",
    supports: () => "worker capability missing",
    loader: () => {
      loads += 1;
      return null;
    },
  });
  const client = createDataClient();
  const unsubscribe = client.subscribe(() => {
    observations += 1;
  });
  unsubscribe();
  const state = await client.load(resource, {});
  assert.equal(state.status, "unsupported");
  assert.equal(loads, 0);
  assert.equal(observations, 0);
});
