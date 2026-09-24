import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useWebLocks } from "./use-web-locks.ts";
import type {
  LockManagerLike,
  WebLockInfo,
  WebLockLike,
  WebLockManagerRequestOptions,
} from "./use-web-locks.ts";

interface Waiter {
  readonly name: string;
  readonly grant: () => void;
}

/** Exclusive-only fake lock manager with FIFO queues per name. */
class FakeLockManager implements LockManagerLike {
  readonly busy = new Set<string>();
  readonly waiters: Waiter[] = [];
  readonly calls: WebLockManagerRequestOptions[] = [];

  async request(
    name: string,
    options: WebLockManagerRequestOptions,
    callback: (lock: WebLockLike | null) => unknown,
  ): Promise<unknown> {
    this.calls.push(options);
    if (options.signal?.aborted) throw options.signal.reason;
    if (this.busy.has(name)) {
      if (options.ifAvailable) return callback(null);
      await new Promise<void>((resolve, reject) => {
        const waiter = { name, grant: resolve };
        this.waiters.push(waiter);
        options.signal?.addEventListener("abort", () => {
          this.waiters.splice(this.waiters.indexOf(waiter), 1);
          reject(options.signal?.reason);
        });
      });
    }
    this.busy.add(name);
    try {
      return await callback({ name, mode: options.mode });
    } finally {
      this.busy.delete(name);
      const next = this.waiters.findIndex((waiter) => waiter.name === name);
      if (next !== -1) this.waiters.splice(next, 1)[0]?.grant();
    }
  }

  query(): Promise<{ held: WebLockInfo[]; pending: WebLockInfo[] }> {
    return Promise.resolve({
      held: [...this.busy].map((name) => ({ name, mode: "exclusive" as const })),
      pending: this.waiters.map(({ name }) => ({ name, mode: "exclusive" as const })),
    });
  }
}

function deferred(): { promise: Promise<void>; resolve: () => void } {
  let resolve = (): void => {};
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

async function flushPromises(): Promise<void> {
  for (let index = 0; index < 6; index += 1) await Promise.resolve();
}

void test("runs the callback while the lock is held and returns its result", async () => {
  const locks = new FakeLockManager();
  const controls = useWebLocks({ locks });
  assert.equal(controls.supported.value, true);

  const gate = deferred();
  const result = controls.request("sync", async (lock) => {
    assert.deepEqual(lock, { name: "sync", mode: "exclusive" });
    await gate.promise;
    return 42;
  });
  await flushPromises();
  assert.deepEqual(controls.held.value, ["sync"]);
  assert.deepEqual(controls.pending.value, []);
  gate.resolve();
  assert.equal(await result, 42);
  assert.deepEqual(controls.held.value, []);
});

void test("tracks pending requests queued behind a held lock", async () => {
  const locks = new FakeLockManager();
  const controls = useWebLocks({ locks });
  const gate = deferred();
  const first = controls.request("job", () => gate.promise);
  const second = controls.request("job", () => "second", { mode: "shared" });
  await flushPromises();
  assert.deepEqual(controls.held.value, ["job"]);
  assert.deepEqual(controls.pending.value, ["job"]);
  assert.deepEqual(await controls.query(), {
    held: [{ name: "job", mode: "exclusive" }],
    pending: [{ name: "job", mode: "exclusive" }],
  });
  gate.resolve();
  await first;
  assert.equal(await second, "second");
  assert.deepEqual(controls.pending.value, []);
});

void test("ifAvailable resolves to a discriminated outcome", async () => {
  const locks = new FakeLockManager();
  const controls = useWebLocks({ locks });
  assert.deepEqual(await controls.request("leader", () => "led", { ifAvailable: true }), {
    acquired: true,
    value: "led",
  });

  const gate = deferred();
  const holder = controls.request("leader", () => gate.promise);
  await flushPromises();
  let ran = false;
  const attempt = await controls.request(
    "leader",
    () => {
      ran = true;
    },
    { ifAvailable: true },
  );
  assert.deepEqual(attempt, { acquired: false });
  assert.equal(ran, false);
  assert.deepEqual(locks.calls.at(-1), { mode: "exclusive", ifAvailable: true, steal: false });
  gate.resolve();
  await holder;
});

void test("forwards caller aborts and records the failure", async () => {
  const locks = new FakeLockManager();
  const controls = useWebLocks({ locks });
  const gate = deferred();
  const holder = controls.request("job", () => gate.promise);
  const abort = new AbortController();
  const waiting = controls.request("job", () => "never", { signal: abort.signal });
  await flushPromises();
  const reason = new Error("stop");
  abort.abort(reason);
  await assert.rejects(waiting, (cause) => cause === reason);
  assert.equal(controls.error.value, reason);
  assert.deepEqual(controls.pending.value, []);
  gate.resolve();
  await holder;
});

void test("aborts pending requests when the scope stops", async () => {
  const locks = new FakeLockManager();
  const outer = useWebLocks({ locks });
  const gate = deferred();
  const holder = outer.request("job", () => gate.promise);
  const scope = effectScope();
  const controls = scope.run(() => useWebLocks({ locks }));
  assert.ok(controls);
  const waiting = controls.request("job", () => "never");
  await flushPromises();
  scope.stop();
  await assert.rejects(waiting, { name: "AbortError" });
  assert.equal(locks.waiters.length, 0);
  gate.resolve();
  await holder;
});

void test("rejects callback failures and invalid option combinations", async () => {
  const controls = useWebLocks({ locks: new FakeLockManager() });
  const failure = new Error("boom");
  await assert.rejects(
    controls.request("job", () => Promise.reject(failure)),
    (cause) => cause === failure,
  );
  assert.deepEqual(controls.held.value, []);
  await assert.rejects(
    controls.request("job", () => 1, { steal: true, mode: "shared" }),
    /VIZE_COMPOSE_WEB_LOCKS_INVALID_OPTIONS/,
  );
  await assert.rejects(
    controls.request("job", () => 1, { ifAvailable: true, signal: new AbortController().signal }),
    TypeError,
  );
});

void test("reports unsupported without a lock manager", async () => {
  const controls = useWebLocks({ locks: null });
  assert.equal(controls.supported.value, false);
  await assert.rejects(
    controls.request("job", () => 1),
    /VIZE_COMPOSE_WEB_LOCKS_UNSUPPORTED/,
  );
  assert.deepEqual(await controls.query(), { held: [], pending: [] });
});

void test("server rendering requests nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const controls = useWebLocks();
    return { supported: controls.supported, held: controls.held, pending: controls.pending };
  });
  assert.equal(state, '{"supported":false,"held":[],"pending":[]}');
});
