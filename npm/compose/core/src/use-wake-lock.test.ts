import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useWakeLock } from "./use-wake-lock.ts";
import type { WakeLockHost, WakeLockKind, WakeLockSentinelLike } from "./use-wake-lock.ts";

class FakeSentinel extends EventTarget implements WakeLockSentinelLike {
  released = false;

  release(): Promise<void> {
    if (!this.released) {
      this.released = true;
      this.dispatchEvent(new Event("release"));
    }
    return Promise.resolve();
  }
}

class FakeDocument extends EventTarget {
  visibilityState = "visible";

  setVisibility(state: string): void {
    this.visibilityState = state;
    this.dispatchEvent(new Event("visibilitychange"));
  }
}

function createHost(): {
  host: WakeLockHost;
  sentinels: FakeSentinel[];
  document: FakeDocument;
  fail: { next: unknown };
} {
  const sentinels: FakeSentinel[] = [];
  const document = new FakeDocument();
  const fail: { next: unknown } = { next: undefined };
  const host: WakeLockHost = {
    wakeLock: {
      request: (_type: WakeLockKind) => {
        if (fail.next !== undefined) {
          const error = fail.next;
          fail.next = undefined;
          return Promise.reject(error);
        }
        const sentinel = new FakeSentinel();
        sentinels.push(sentinel);
        return Promise.resolve(sentinel);
      },
    },
    document,
  };
  return { host, sentinels, document, fail };
}

async function flushPromises(): Promise<void> {
  for (let index = 0; index < 4; index += 1) await Promise.resolve();
}

void test("requests and releases a screen lock", async () => {
  const { host, sentinels } = createHost();
  const lock = useWakeLock({ host });

  assert.equal(lock.supported.value, true);
  assert.equal(await lock.request(), true);
  assert.equal(lock.active.value, true);
  assert.equal(lock.requested.value, true);
  assert.equal(await lock.request(), true, "a held lock is not requested twice");
  assert.equal(sentinels.length, 1);

  await lock.release();
  assert.equal(lock.active.value, false);
  assert.equal(lock.requested.value, false);
  assert.equal(sentinels[0]?.released, true);
});

void test("re-acquires after the page becomes visible again", async () => {
  const { host, sentinels, document } = createHost();
  const lock = useWakeLock({ host });
  await lock.request();

  document.visibilityState = "hidden";
  await sentinels[0]?.release();
  assert.equal(lock.active.value, false);
  document.setVisibility("visible");
  await flushPromises();
  assert.equal(sentinels.length, 2);
  assert.equal(lock.active.value, true);
});

void test("does not re-acquire after an explicit release or when disabled", async () => {
  const { host, sentinels, document } = createHost();
  const lock = useWakeLock({ host });
  await lock.request();
  await lock.release();
  document.setVisibility("visible");
  await flushPromises();
  assert.equal(sentinels.length, 1);

  const second = createHost();
  const passive = useWakeLock({ host: second.host, reacquireOnVisible: false });
  await passive.request();
  await second.sentinels[0]?.release();
  second.document.setVisibility("visible");
  await flushPromises();
  assert.equal(second.sentinels.length, 1);
});

void test("exposes request failures without throwing", async () => {
  const { host, fail } = createHost();
  const denial = new Error("NotAllowedError");
  fail.next = denial;
  const lock = useWakeLock({ host });

  assert.equal(await lock.request(), false);
  assert.equal(lock.error.value, denial);
  assert.equal(lock.active.value, false);
  assert.equal(await lock.request(), true);
  assert.equal(lock.error.value, undefined);
});

void test("releases the lock and listener with the scope", async () => {
  const { host, sentinels, document } = createHost();
  const scope = effectScope();
  const lock = scope.run(() => useWakeLock({ host }));
  assert.ok(lock);
  await lock.request();

  scope.stop();
  await flushPromises();
  assert.equal(sentinels[0]?.released, true);
  document.setVisibility("visible");
  await flushPromises();
  assert.equal(sentinels.length, 1);
});

void test("reports unsupported without a host", async () => {
  const lock = useWakeLock({ host: null });
  assert.equal(lock.supported.value, false);
  assert.equal(await lock.request(), false);
});

void test("server rendering requests nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const lock = useWakeLock();
    return { supported: lock.supported, active: lock.active };
  });
  assert.equal(state, '{"supported":false,"active":false}');
});
