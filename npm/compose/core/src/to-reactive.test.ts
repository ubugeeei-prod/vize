import assert from "node:assert/strict";
import { test } from "node:test";
import { effect, isReactive, ref, shallowRef } from "vue";

import { toReactive } from "./to-reactive.ts";

void test("follows the ref's current object reactively", () => {
  const state = shallowRef<{ name: string; count?: number }>({ name: "a" });
  const view = toReactive(state);
  const seen: string[] = [];
  effect(() => {
    seen.push(view.name);
  });

  assert.ok(isReactive(view));
  state.value = { name: "b" };
  assert.equal(view.name, "b");
  assert.deepEqual(seen, ["a", "b"]);
  assert.ok("name" in view);
  assert.deepEqual(Object.keys(view), ["name"]);
});

void test("writes and deletes go to the current object and nested refs", () => {
  const nested = ref(1);
  const state = ref<{ nested: typeof nested; plain?: number }>({ nested, plain: 1 });
  const view = toReactive(state);
  assert.equal(view.nested, 1);
  view.nested = 2;
  assert.equal(nested.value, 2);
  view.plain = 3;
  assert.equal(state.value.plain, 3);
  delete view.plain;
  assert.equal("plain" in state.value, false);
});

void test("plain objects are simply made reactive", () => {
  const view = toReactive({ a: 1 });
  assert.ok(isReactive(view));
  assert.equal(view.a, 1);
});
