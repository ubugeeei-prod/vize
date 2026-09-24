import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import { reactify } from "./reactify.ts";

void test("lifts a function over refs, getters, and plain values", () => {
  const add = reactify((left: number, right: number) => left + right);
  const left = shallowRef(1);
  const right = shallowRef(2);
  const sum = add(left, () => right.value * 10);
  assert.equal(sum.value, 21);
  left.value = 5;
  assert.equal(sum.value, 25);
  assert.equal(add(1, 2).value, 3);
});
