import assert from "node:assert/strict";
import { test } from "node:test";
import { effect, effectScope, shallowRef } from "vue";

import { refAutoReset } from "./ref-auto-reset.ts";
import { FakeTimeouts } from "./testing/fake-timeouts.ts";

void test("resets to the default after every write", () => {
  const timers = new FakeTimeouts();
  const message = refAutoReset("", 1_000, { runOnServer: true, scheduler: timers });
  const seen: string[] = [];
  effect(() => {
    seen.push(message.value);
  });

  message.value = "Copied!";
  timers.advance(500);
  message.value = "Copied again!";
  timers.advance(999);
  assert.equal(message.value, "Copied again!");
  timers.advance(1);
  assert.equal(message.value, "");
  assert.deepEqual(seen, ["", "Copied!", "Copied again!", ""]);
});

void test("follows a reactive default and delay", () => {
  const timers = new FakeTimeouts();
  const fallback = shallowRef("idle");
  const delay = shallowRef(100);
  const status = refAutoReset(fallback, delay, { runOnServer: true, scheduler: timers });

  fallback.value = "ready";
  assert.equal(status.value, "ready");
  delay.value = 10;
  status.value = "busy";
  timers.advance(10);
  assert.equal(status.value, "ready");
});

void test("keeps written values on the server", () => {
  const timers = new FakeTimeouts();
  const status = refAutoReset("idle", 100, { scheduler: timers });
  status.value = "saved";
  timers.advance(1_000);
  assert.equal(status.value, "saved");
  assert.equal(timers.timers.size, 0);
});

void test("the owning scope cancels the pending reset", () => {
  const timers = new FakeTimeouts();
  const scope = effectScope();
  const status = scope.run(() =>
    refAutoReset("idle", 100, { runOnServer: true, scheduler: timers }),
  );
  assert.ok(status);
  status.value = "busy";
  scope.stop();
  assert.equal(timers.timers.size, 0);
});

void test("rejects invalid delays", () => {
  assert.throws(
    () => refAutoReset(0, -1),
    (error: unknown) =>
      error instanceof RangeError &&
      error.message.startsWith("[VIZE_COMPOSE_REF_AUTO_RESET_INVALID_DELAY]"),
  );
});
