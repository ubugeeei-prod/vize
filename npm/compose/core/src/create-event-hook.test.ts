import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { createEventHook } from "./create-event-hook.ts";

void test("calls listeners in order with typed arguments", async () => {
  const hook = createEventHook<[id: string, count: number]>();
  const calls: string[] = [];
  hook.on((id, count) => calls.push(`a:${id}:${count}`));
  hook.on(async (id) => {
    calls.push(`b:${id}`);
    return id.length;
  });
  const results = await hook.trigger("x", 2);
  assert.deepEqual(calls, ["a:x:2", "b:x"]);
  assert.deepEqual(results, [1, 1]);
});

void test("off, subscription handles, clear, and size", async () => {
  const hook = createEventHook();
  let calls = 0;
  const listener = (): void => {
    calls += 1;
  };
  const subscription = hook.on(listener);
  hook.on(listener);
  assert.equal(hook.size(), 1);
  subscription.off();
  subscription.off();
  await hook.trigger();
  hook.on(listener);
  hook.off(listener);
  hook.on(() => undefined);
  hook.clear();
  assert.equal(hook.size(), 0);
  assert.equal(calls, 0);
});

void test("listeners added in a scope are removed when it stops", () => {
  const hook = createEventHook<[number]>();
  const scope = effectScope();
  scope.run(() => hook.on(() => undefined));
  assert.equal(hook.size(), 1);
  scope.stop();
  assert.equal(hook.size(), 0);
});

void test("async listener failures reject the trigger promise", async () => {
  const hook = createEventHook();
  hook.on(async () => {
    throw new Error("boom");
  });
  await assert.rejects(hook.trigger(), /boom/);
});
