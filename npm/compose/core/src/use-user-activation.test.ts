import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useUserActivation } from "./use-user-activation.ts";
import type { UserActivationLike } from "./use-user-activation.ts";
import type { IntervalScheduler } from "./use-interval.ts";

function createScheduler(): IntervalScheduler & { tick: () => void; active: number } {
  const callbacks = new Map<number, () => void>();
  let next = 0;
  return {
    get active() {
      return callbacks.size;
    },
    setInterval: (callback) => {
      next += 1;
      callbacks.set(next, callback);
      return next;
    },
    clearInterval: (handle) => {
      if (typeof handle === "number") callbacks.delete(handle);
    },
    tick: () => {
      for (const callback of Array.from(callbacks.values())) callback();
    },
  };
}

void test("refreshes on activation-triggering events", async () => {
  const activation: UserActivationLike & { hasBeenActive: boolean; isActive: boolean } = {
    hasBeenActive: false,
    isActive: false,
  };
  const target = new EventTarget();
  const state = useUserActivation({ userActivation: activation, target, interval: 0 });
  assert.equal(state.supported.value, true);
  assert.equal(state.isActive.value, false);

  activation.hasBeenActive = true;
  activation.isActive = true;
  target.dispatchEvent(new Event("pointerdown"));
  assert.equal(state.hasBeenActive.value, true);
  assert.equal(state.isActive.value, true);

  activation.isActive = false;
  target.dispatchEvent(new Event("scroll"));
  assert.equal(state.isActive.value, true, "non-activation events are ignored");
  target.dispatchEvent(new Event("keydown"));
  assert.equal(state.isActive.value, false);
  state.stop();
});

void test("re-reads after a microtask for late activation updates", async () => {
  const activation = { hasBeenActive: false, isActive: false };
  const target = new EventTarget();
  const state = useUserActivation({ userActivation: activation, target, interval: 0 });
  target.addEventListener("keydown", () => {
    activation.isActive = true;
  });
  target.dispatchEvent(new Event("keydown"));
  assert.equal(state.isActive.value, false);
  await Promise.resolve();
  assert.equal(state.isActive.value, true);
});

void test("polls only while transient activation lasts", () => {
  const activation = { hasBeenActive: true, isActive: true };
  const scheduler = createScheduler();
  const state = useUserActivation({ userActivation: activation, target: null, scheduler });
  assert.equal(state.isActive.value, true);
  assert.equal(scheduler.active, 1);
  activation.isActive = false;
  scheduler.tick();
  assert.equal(state.isActive.value, false);
  assert.equal(scheduler.active, 0);
});

void test("custom events and scope disposal", () => {
  const activation = { hasBeenActive: false, isActive: false };
  const target = new EventTarget();
  const scheduler = createScheduler();
  const scope = effectScope();
  const state = scope.run(() =>
    useUserActivation({ userActivation: activation, target, events: ["tap"], scheduler }),
  );
  assert.ok(state);
  activation.isActive = true;
  target.dispatchEvent(new Event("tap"));
  assert.equal(state.isActive.value, true);
  assert.equal(scheduler.active, 1);

  scope.stop();
  assert.equal(scheduler.active, 0);
  activation.hasBeenActive = true;
  target.dispatchEvent(new Event("tap"));
  assert.equal(state.hasBeenActive.value, false);
});

void test("follows a reactive target", () => {
  const activation = { hasBeenActive: false, isActive: false };
  const first = new EventTarget();
  const second = new EventTarget();
  const target = shallowRef<EventTarget>(first);
  const state = useUserActivation({ userActivation: activation, target, interval: 0 });
  target.value = second;
  activation.hasBeenActive = true;
  first.dispatchEvent(new Event("keydown"));
  assert.equal(state.hasBeenActive.value, false);
  second.dispatchEvent(new Event("keydown"));
  assert.equal(state.hasBeenActive.value, true);
});

void test("reports unsupported and validates the interval", () => {
  const state = useUserActivation({ userActivation: null });
  assert.equal(state.supported.value, false);
  state.refresh();
  assert.equal(state.isActive.value, false);
  assert.throws(
    () => useUserActivation({ userActivation: null, interval: -1 }),
    /VIZE_COMPOSE_USER_ACTIVATION_INVALID_INTERVAL/u,
  );
});

void test("server rendering installs nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const activation = useUserActivation();
    return {
      supported: activation.supported,
      hasBeenActive: activation.hasBeenActive,
      isActive: activation.isActive,
    };
  });
  assert.equal(state, '{"supported":false,"hasBeenActive":false,"isActive":false}');
});
