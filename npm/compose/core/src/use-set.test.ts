import assert from "node:assert/strict";
import { test } from "node:test";
import { computed, nextTick, watch, watchEffect } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useSet } from "./use-set.ts";

void test("wraps a set with typed helpers", () => {
  const tags = useSet(["a"]);
  assert.equal(tags.add("b"), true);
  assert.equal(tags.add("b"), false);
  assert.equal(tags.has("a"), true);
  assert.equal(tags.size.value, 2);
  assert.deepEqual(tags.values.value, ["a", "b"]);
  assert.equal(tags.delete("a"), true);
  assert.equal(tags.delete("a"), false);
  tags.clear();
  assert.equal(tags.size.value, 0);
});

void test("toggles and forces membership", () => {
  const selected = useSet<number>();
  assert.equal(selected.toggle(1), true);
  assert.equal(selected.toggle(1), false);
  assert.equal(selected.toggle(1, true), true);
  assert.equal(selected.toggle(1, true), true);
  assert.equal(selected.toggle(1, undefined), false);
  assert.equal(selected.toggle(2, false), false);
  assert.deepEqual(selected.values.value, []);
});

void test("set algebra returns new plain sets", () => {
  const letters = useSet(["a", "b", "c"]);
  assert.deepEqual([...letters.union(["c", "d"])], ["a", "b", "c", "d"]);
  assert.deepEqual([...letters.intersection(["b", "c", "z"])], ["b", "c"]);
  assert.deepEqual([...letters.difference(["a"])], ["b", "c"]);
  assert.deepEqual(letters.values.value, ["a", "b", "c"], "the source is unchanged");
});

void test("tracks membership per item", async () => {
  const selected = useSet<string>();
  let runs = 0;
  watchEffect(() => {
    runs += 1;
    void selected.has("a");
  });
  const hasB = computed(() => selected.has("b"));
  const viaView = computed(() => selected.set.has("b"));

  selected.add("b");
  await nextTick();
  assert.equal(runs, 1, "adding another item does not re-run the effect");
  assert.equal(hasB.value, true);
  assert.equal(viaView.value, true);

  selected.add("a");
  await nextTick();
  assert.equal(runs, 2);
});

void test("size and values follow changes", async () => {
  const selected = useSet<number>();
  const sizes: number[] = [];
  watch(selected.size, (size) => sizes.push(size));
  selected.add(1);
  await nextTick();
  selected.toggle(2);
  await nextTick();
  assert.deepEqual(sizes, [1, 2]);
  assert.deepEqual(selected.values.value, [1, 2]);
});

void test("reset restores the initial members", () => {
  const tags = useSet(["a"]);
  tags.add("b");
  tags.delete("a");
  tags.reset();
  assert.deepEqual(tags.values.value, ["a"]);
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => {
    const tags = useSet(["b", "a"]);
    tags.toggle("c");
    return { values: tags.values, size: tags.size };
  });
  assert.equal(state, '{"values":["b","a","c"],"size":3}');
});
