import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, ref } from "vue";

import { useEventListener } from "./event-listener.ts";

void test("moves a listener between reactive targets and disposes it with the scope", () => {
  const first = new EventTarget();
  const second = new EventTarget();
  const target = ref<EventTarget | null>(first);
  const scope = effectScope();
  let calls = 0;
  const controls = scope.run(() =>
    useEventListener(target, "change", () => (calls += 1), { flush: "sync" }),
  );

  assert.ok(controls);
  assert.equal(controls.isListening.value, true);
  first.dispatchEvent(new Event("change"));
  target.value = second;
  first.dispatchEvent(new Event("change"));
  second.dispatchEvent(new Event("change"));
  assert.equal(calls, 2);

  scope.stop();
  second.dispatchEvent(new Event("change"));
  assert.equal(controls.isListening.value, false);
  assert.equal(calls, 2);
});

void test("supports idempotent manual and one-shot listener controls", () => {
  const target = new EventTarget();
  const scope = effectScope();
  let calls = 0;
  const controls = scope.run(() =>
    useEventListener(target, "submit", () => (calls += 1), {
      flush: "sync",
      immediate: false,
      once: true,
    }),
  );

  assert.ok(controls);
  target.dispatchEvent(new Event("submit"));
  assert.equal(calls, 0);
  assert.equal(controls.start(), true);
  assert.equal(controls.start(), false);
  target.dispatchEvent(new Event("submit"));
  assert.equal(calls, 1);
  assert.equal(controls.isListening.value, false);
  assert.equal(controls.start(), true);
  target.dispatchEvent(new Event("submit"));
  assert.equal(calls, 2);

  controls.stop();
  controls.stop();
  scope.stop();
});

void test("stops when the supplied signal aborts", () => {
  const target = new EventTarget();
  const signal = new AbortController();
  let calls = 0;
  const controls = useEventListener(target, "update", () => (calls += 1), {
    flush: "sync",
    signal: signal.signal,
  });

  target.dispatchEvent(new Event("update"));
  signal.abort();
  target.dispatchEvent(new Event("update"));

  assert.equal(calls, 1);
  assert.equal(controls.isListening.value, false);
  assert.equal(controls.start(), false);
});
