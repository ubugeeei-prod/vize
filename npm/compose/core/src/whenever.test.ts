import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import { whenever } from "./whenever.ts";

void test("delivers only truthy values", () => {
  const source = shallowRef<string | null>(null);
  const values: string[] = [];
  whenever(source, (value) => values.push(value), { flush: "sync" });
  source.value = "a";
  source.value = null;
  source.value = "";
  source.value = "b";
  assert.deepEqual(values, ["a", "b"]);
});

void test("once stops after the first truthy delivery, not the first change", () => {
  const source = shallowRef(0);
  const values: number[] = [];
  whenever(source, (value) => values.push(value), { flush: "sync", once: true });
  source.value = 0;
  source.value = -1;
  source.value = 0;
  source.value = 2;
  assert.deepEqual(values, [-1]);
});

void test("immediate + once delivers an already truthy value and stops", () => {
  const source = shallowRef<{ id: number } | undefined>({ id: 1 });
  const ids: number[] = [];
  whenever(
    () => source.value,
    (value) => ids.push(value.id),
    {
      immediate: true,
      once: true,
      flush: "sync",
    },
  );
  source.value = { id: 2 };
  assert.deepEqual(ids, [1]);
});

void test("the returned handle stops the watcher", () => {
  const source = shallowRef(false);
  let calls = 0;
  const stop = whenever(source, () => (calls += 1), { flush: "sync" });
  stop();
  source.value = true;
  assert.equal(calls, 0);
});
