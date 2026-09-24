import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import { useSorted } from "./use-sorted.ts";

void test("sorts naturally without mutating the source", () => {
  const source = shallowRef([10, 9, 100]);
  const sorted = useSorted(source);
  assert.deepEqual(sorted.value, [9, 10, 100]);
  assert.deepEqual(source.value, [10, 9, 100]);
  source.value = [3, 1];
  assert.deepEqual(sorted.value, [1, 3]);
  assert.deepEqual(useSorted(["b", "a"]).value, ["a", "b"]);
  assert.deepEqual(useSorted([2n, 1n]).value, [1n, 2n]);
  const early = new Date(0);
  const late = new Date(1);
  assert.deepEqual(useSorted([late, early]).value, [early, late]);
});

void test("uses a stable custom comparator", () => {
  const players = [
    { name: "a", score: 1 },
    { name: "b", score: 3 },
    { name: "c", score: 1 },
  ];
  const ranked = useSorted(players, (left, right) => right.score - left.score);
  assert.deepEqual(
    ranked.value.map((player) => player.name),
    ["b", "a", "c"],
  );
});
