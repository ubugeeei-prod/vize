import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useIdle } from "./idle.ts";
import type { IdleHost } from "./idle.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

class IdleWindow extends EventTarget implements IdleHost {
  readonly document = Object.assign(new EventTarget(), { hidden: false });
}

function manualScheduler(): TimeoutScheduler & {
  fire: () => void;
  readonly active: () => boolean;
} {
  let queued: (() => void) | undefined;
  return {
    setTimeout: (callback) => {
      queued = callback;
      return 1;
    },
    clearTimeout: () => {
      queued = undefined;
    },
    fire: () => {
      const callback = queued;
      queued = undefined;
      callback?.();
    },
    active: () => queued !== undefined,
  };
}

void test("does not start timers on the server", () => {
  const idle = useIdle(1000, { host: () => undefined, initialState: true });
  assert.deepEqual([idle.idle.value, idle.lastActive.value], [true, null]);
  idle.reset();
  assert.equal(idle.lastActive.value, null);
});

void test("becomes idle after the timeout and wakes on activity or visibility", () => {
  const host = new IdleWindow();
  const scheduler = manualScheduler();
  let clock = 100;
  const scope = effectScope();
  const idle = scope.run(() => useIdle(5000, { host, scheduler, now: () => clock }));
  assert.ok(idle);
  assert.deepEqual([idle.idle.value, idle.lastActive.value], [false, 100]);

  scheduler.fire();
  assert.equal(idle.idle.value, true);
  clock = 200;
  host.dispatchEvent(new Event("keydown"));
  assert.deepEqual([idle.idle.value, idle.lastActive.value], [false, 200]);

  scheduler.fire();
  host.document.hidden = true;
  host.document.dispatchEvent(new Event("visibilitychange"));
  assert.equal(idle.idle.value, true);
  host.document.hidden = false;
  host.document.dispatchEvent(new Event("visibilitychange"));
  assert.equal(idle.idle.value, false);

  scope.stop();
  assert.equal(scheduler.active(), false);
  host.dispatchEvent(new Event("mousemove"));
  assert.equal(scheduler.active(), false);
});

void test("honors a custom event list", () => {
  const host = new IdleWindow();
  const scheduler = manualScheduler();
  const idle = useIdle(10, {
    host,
    scheduler,
    events: ["scroll"],
    listenForVisibilityChange: false,
  });
  scheduler.fire();
  host.dispatchEvent(new Event("keydown"));
  assert.equal(idle.idle.value, true);
  host.dispatchEvent(new Event("scroll"));
  assert.equal(idle.idle.value, false);
});
