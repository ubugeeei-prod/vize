import assert from "node:assert/strict";
import { test } from "node:test";
import { nextTick, shallowRef } from "vue";

import { useCycleList } from "./use-cycle-list.ts";

void test("cycles forward and backward with wrap-around", () => {
  const { state, index, next, prev, go } = useCycleList(["light", "dark", "system"] as const);
  assert.equal(state.value, "light");
  assert.equal(next(), "dark");
  assert.equal(next(2), "light");
  assert.equal(prev(), "system");
  assert.equal(go(-4), "system");
  index.value = 4;
  assert.equal(state.value, "dark");
  state.value = "system";
  assert.equal(index.value, 2);
});

void test("honours initial value, fallback, and custom lookup", () => {
  const list = [{ id: 1 }, { id: 2 }, { id: 3 }];
  const { state, index } = useCycleList(list, {
    initialValue: { id: 2 },
    getIndexOf: (item, all) => all.findIndex((candidate) => candidate.id === item.id),
  });
  assert.equal(index.value, 1);
  assert.deepEqual(state.value, { id: 2 });
  state.value = { id: 99 };
  const fallback = useCycleList(["a", "b"], { initialValue: "z", fallbackIndex: 1 });
  assert.equal(fallback.index.value, 1);
});

void test("keeps the current item when the list changes, else falls back", async () => {
  const list = shallowRef(["a", "b", "c"]);
  const { state, next } = useCycleList(list);
  next();
  list.value = ["x", "b"];
  await nextTick();
  assert.equal(state.value, "b");
  list.value = ["y", "z"];
  await nextTick();
  assert.equal(state.value, "y");
});

void test("empty lists yield undefined", () => {
  const { state, next } = useCycleList<string>([]);
  assert.equal(state.value, undefined);
  assert.equal(next(), undefined);
});
