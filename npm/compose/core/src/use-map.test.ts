import assert from "node:assert/strict";
import { test } from "node:test";
import { computed, nextTick, watch, watchEffect } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useMap } from "./use-map.ts";

void test("wraps a map with typed helpers", () => {
  const counts = useMap([["a", 1]]);
  counts.set("b", 2);
  assert.equal(counts.get("a"), 1);
  assert.equal(counts.has("b"), true);
  assert.equal(counts.size.value, 2);
  assert.deepEqual(counts.entries.value, [
    ["a", 1],
    ["b", 2],
  ]);
  assert.equal(counts.delete("a"), true);
  assert.equal(counts.delete("a"), false);
  counts.clear();
  assert.equal(counts.size.value, 0);
});

void test("update and getOrInsert derive values", () => {
  const counts = useMap<string, number>();
  assert.equal(
    counts.update("clicks", (previous = 0) => previous + 1),
    1,
  );
  assert.equal(
    counts.update("clicks", (previous = 0) => previous + 1),
    2,
  );

  let created = 0;
  const create = (key: string): number => {
    created += 1;
    return key.length;
  };
  assert.equal(counts.getOrInsert("abc", create), 3);
  assert.equal(counts.getOrInsert("abc", create), 3);
  assert.equal(created, 1);
});

void test("getOrInsert keeps a stored undefined", () => {
  const optional = useMap<string, number | undefined>([["a", undefined]]);
  assert.equal(
    optional.getOrInsert("a", () => 1),
    undefined,
  );
});

void test("tracks reads per key", async () => {
  const counts = useMap<string, number>([["a", 1]]);
  let runs = 0;
  watchEffect(() => {
    runs += 1;
    void counts.get("a");
  });
  const hasB = computed(() => counts.has("b"));

  counts.set("b", 2);
  await nextTick();
  assert.equal(runs, 1, "writing another key does not re-run the effect");
  assert.equal(hasB.value, true);

  counts.set("a", 5);
  await nextTick();
  assert.equal(runs, 2);
});

void test("reactive view and entries follow changes", async () => {
  const counts = useMap<string, number>();
  const sizes: number[] = [];
  watch(counts.size, (size) => sizes.push(size));
  const viaView = computed(() => counts.map.get("x"));

  counts.set("x", 1);
  await nextTick();
  assert.equal(viaView.value, 1);
  counts.set("y", 2);
  await nextTick();
  assert.deepEqual(sizes, [1, 2]);
  assert.deepEqual(counts.entries.value, [
    ["x", 1],
    ["y", 2],
  ]);
});

void test("reset restores the initial entries", () => {
  const counts = useMap([["a", 1]]);
  counts.set("b", 2);
  counts.delete("a");
  counts.reset();
  assert.deepEqual(counts.entries.value, [["a", 1]]);
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => {
    const counts = useMap([["a", 1]]);
    counts.set("b", 2);
    return { entries: counts.entries, size: counts.size };
  });
  assert.equal(state, '{"entries":[["a",1],["b",2]],"size":2}');
});
