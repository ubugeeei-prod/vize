import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { usePushSubscription } from "./use-push-subscription.ts";
import type {
  PushManagerLike,
  PushManagerSubscribeOptions,
  PushPermissionState,
  PushRegistrationLike,
  PushSubscriptionJsonLike,
  PushSubscriptionLike,
} from "./use-push-subscription.ts";

class FakeSubscription implements PushSubscriptionLike {
  readonly endpoint = "https://push.test/abc";
  readonly expirationTime = null;
  cancelled = false;

  toJSON(): PushSubscriptionJsonLike {
    return {
      endpoint: this.endpoint,
      expirationTime: null,
      keys: { p256dh: "BEl6", auth: "c2Vj" },
    };
  }

  unsubscribe(): Promise<boolean> {
    this.cancelled = true;
    return Promise.resolve(true);
  }
}

class FakePushManager implements PushManagerLike {
  current: FakeSubscription | null = null;
  readonly calls: PushManagerSubscribeOptions[] = [];
  failure: unknown = undefined;
  permission: PushPermissionState = "prompt";

  subscribe(options: PushManagerSubscribeOptions): Promise<PushSubscriptionLike> {
    this.calls.push(options);
    if (this.failure !== undefined) return Promise.reject(this.failure);
    this.current = new FakeSubscription();
    return Promise.resolve(this.current);
  }

  getSubscription(): Promise<PushSubscriptionLike | null> {
    return Promise.resolve(this.current);
  }

  permissionState(): Promise<PushPermissionState> {
    return Promise.resolve(this.permission);
  }
}

function registrationWith(pushManager: PushManagerLike): PushRegistrationLike {
  return { pushManager };
}

async function flushPromises(): Promise<void> {
  for (let index = 0; index < 6; index += 1) await Promise.resolve();
}

void test("loads the existing subscription and exposes its JSON snapshot", async () => {
  const manager = new FakePushManager();
  manager.current = new FakeSubscription();
  const push = usePushSubscription({ registration: registrationWith(manager) });
  assert.equal(push.supported.value, true);
  await flushPromises();
  assert.equal(push.subscription.value, manager.current);
  assert.deepEqual(push.json.value, {
    endpoint: "https://push.test/abc",
    expirationTime: null,
    keys: { p256dh: "BEl6", auth: "c2Vj" },
  });
});

void test("decodes a base64url VAPID key and subscribes", async () => {
  const manager = new FakePushManager();
  const push = usePushSubscription({ registration: registrationWith(manager) });
  // "+/8A" in standard base64 is "-_8A" in base64url: bytes fb ff 00.
  const subscription = await push.subscribe({ applicationServerKey: "-_8A" });
  assert.ok(subscription);
  assert.equal(push.subscription.value, subscription);
  assert.deepEqual(manager.calls[0], {
    applicationServerKey: new Uint8Array([0xfb, 0xff, 0x00]),
    userVisibleOnly: true,
  });
  await push.subscribe({ applicationServerKey: "AQI=", userVisibleOnly: false });
  assert.deepEqual(manager.calls[1]?.applicationServerKey, new Uint8Array([1, 2]));

  const raw = new Uint8Array([9, 9]);
  await push.subscribe({ applicationServerKey: raw });
  assert.equal(manager.calls[2]?.applicationServerKey, raw);
});

void test("rejects malformed key strings", async () => {
  const push = usePushSubscription({ registration: registrationWith(new FakePushManager()) });
  await assert.rejects(
    push.subscribe({ applicationServerKey: "not a key!" }),
    /VIZE_COMPOSE_PUSH_INVALID_KEY/,
  );
  await assert.rejects(push.subscribe({ applicationServerKey: "AAAAA" }), TypeError);
});

void test("unsubscribes and clears the snapshot", async () => {
  const manager = new FakePushManager();
  const push = usePushSubscription({ registration: registrationWith(manager) });
  assert.equal(await push.unsubscribe(), false);
  await push.subscribe({ applicationServerKey: "AQI" });
  const current = manager.current;
  assert.equal(await push.unsubscribe(), true);
  assert.equal(current?.cancelled, true);
  assert.equal(push.subscription.value, null);
  assert.equal(push.json.value, null);
});

void test("subscribe failures land in error", async () => {
  const manager = new FakePushManager();
  const failure = new Error("NotAllowedError");
  manager.failure = failure;
  const push = usePushSubscription({ registration: registrationWith(manager) });
  assert.equal(await push.subscribe({ applicationServerKey: "AQI" }), null);
  assert.equal(push.error.value, failure);
});

void test("reads permission state and follows a reactive registration", async () => {
  const manager = new FakePushManager();
  manager.permission = "granted";
  manager.current = new FakeSubscription();
  const registration = shallowRef<PushRegistrationLike | null>(null);
  const push = usePushSubscription({ registration });
  assert.equal(push.supported.value, false);
  assert.equal(await push.permissionState(), "unsupported");
  registration.value = registrationWith(manager);
  await nextTick();
  await flushPromises();
  assert.equal(push.subscription.value, manager.current);
  assert.equal(await push.permissionState(), "granted");
  registration.value = null;
  await nextTick();
  assert.equal(push.subscription.value, null);
});

void test("ignores late results after the scope stops", async () => {
  const manager = new FakePushManager();
  manager.current = new FakeSubscription();
  const scope = effectScope();
  const push = scope.run(() => usePushSubscription({ registration: registrationWith(manager) }));
  assert.ok(push);
  scope.stop();
  await flushPromises();
  assert.equal(push.subscription.value, null);
});

void test("server rendering reads nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const push = usePushSubscription();
    return { supported: push.supported, subscription: push.subscription, json: push.json };
  });
  assert.equal(state, '{"supported":false,"subscription":null,"json":null}');
});
