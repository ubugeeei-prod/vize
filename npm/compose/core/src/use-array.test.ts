import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import {
  useArrayEvery,
  useArrayFilter,
  useArrayFind,
  useArrayMap,
  useArrayReduce,
  useArraySome,
  useArrayUnique,
} from "./use-array.ts";

void test("filter, map, find, some, and every follow the source", () => {
  const list = shallowRef([1, 2, 3, 4]);
  const even = useArrayFilter(list, (value) => value % 2 === 0);
  const doubled = useArrayMap(list, (value, index) => value * 2 + index);
  const firstBig = useArrayFind(list, (value) => value > 2);
  const hasBig = useArraySome(list, (value) => value > 3);
  const allPositive = useArrayEvery(list, (value) => value > 0);

  assert.deepEqual(even.value, [2, 4]);
  assert.deepEqual(doubled.value, [2, 5, 8, 11]);
  assert.equal(firstBig.value, 3);
  assert.equal(hasBig.value, true);
  assert.equal(allPositive.value, true);

  list.value = [-1];
  assert.deepEqual(even.value, []);
  assert.equal(firstBig.value, undefined);
  assert.equal(hasBig.value, false);
  assert.equal(allPositive.value, false);
});

void test("reduce with and without an initial value", () => {
  const list = shallowRef([1, 2, 3]);
  const initial = shallowRef(10);
  const seeded = useArrayReduce(list, (sum, value) => sum + value, initial);
  const unseeded = useArrayReduce(list, (sum, value) => sum + value);
  const indices: number[] = [];
  const traced = useArrayReduce(list, (sum, value, index) => {
    indices.push(index);
    return sum + value;
  });

  assert.equal(seeded.value, 16);
  assert.equal(unseeded.value, 6);
  assert.equal(traced.value, 6);
  assert.deepEqual(indices, [1, 2]);
  initial.value = 0;
  assert.equal(seeded.value, 6);
  list.value = [];
  assert.equal(unseeded.value, undefined);
  assert.equal(seeded.value, 0);
});

void test("unique keeps first occurrences", () => {
  assert.deepEqual(useArrayUnique([1, 2, 1, Number.NaN, Number.NaN]).value, [1, 2, Number.NaN]);
  const rows = [
    { id: 1, v: "a" },
    { id: 1, v: "b" },
    { id: 2, v: "c" },
  ];
  assert.deepEqual(
    useArrayUnique(rows, (left, right) => left.id === right.id).value.map((row) => row.v),
    ["a", "c"],
  );
});
