import assert from "node:assert/strict";
import { test } from "node:test";
import { computed, shallowRef } from "vue";

import { useClamp, useProjection, useRound } from "./math.ts";

void test("clamps writable sources on read and write", () => {
  const volume = useClamp(50, 0, 100);
  volume.value = 150;
  assert.equal(volume.value, 100);
  const source = shallowRef(-5);
  const max = shallowRef(10);
  const clamped = useClamp(source, 0, max);
  assert.equal(clamped.value, 0);
  clamped.value = 7;
  assert.equal(source.value, 7);
  max.value = 5;
  assert.equal(clamped.value, 5);
});

void test("getters and computeds give readonly clamps", () => {
  const raw = shallowRef(2);
  assert.equal(useClamp(() => raw.value, 0, 1).value, 1);
  assert.equal(
    useClamp(
      computed(() => raw.value * -1),
      0,
      1,
    ).value,
    0,
  );
  assert.throws(() => useClamp(0, 2, 1).value, /VIZE_COMPOSE_MATH_INVALID_RANGE/);
});

void test("rounds with precision and methods", () => {
  const value = shallowRef(1.005);
  const precision = shallowRef(2);
  const rounded = useRound(value, { precision });
  assert.equal(rounded.value, 1.01);
  precision.value = -1;
  value.value = 1_234;
  assert.equal(rounded.value, 1_230);
  assert.equal(useRound(1.9, { method: "floor" }).value, 1);
  assert.equal(useRound(-1.1, { method: "ceil" }).value, -1);
  assert.equal(useRound(-1.9, { method: "trunc" }).value, -1);
  assert.equal(useRound(1e-7, { precision: 8 }).value, 1e-7);
  assert.equal(useRound(Number.POSITIVE_INFINITY).value, Number.POSITIVE_INFINITY);
  assert.throws(() => useRound(1, { precision: 0.5 }).value, /VIZE_COMPOSE_MATH_INVALID_PRECISION/);
});

void test("projects between domains, reversed, clamped, and degenerate", () => {
  const value = shallowRef(50);
  assert.equal(useProjection(value, [0, 100], [0, 1]).value, 0.5);
  assert.equal(useProjection(value, [0, 100], [100, 0]).value, 50);
  assert.equal(useProjection(150, [0, 100], [0, 10], { clamp: true }).value, 10);
  assert.equal(useProjection(150, [0, 100], [0, 10]).value, 15);
  assert.equal(useProjection(5, [3, 3], [7, 9]).value, 7);
});
