import assert from "node:assert/strict";
import { test } from "node:test";

import { useCounter } from "./use-counter.ts";

void test("starts at zero and steps by one without configuration", () => {
  const { count, increment, decrement } = useCounter();

  assert.equal(count.value, 0);
  assert.equal(increment(), 1);
  assert.equal(increment(5), 6);
  assert.equal(decrement(), 5);
  assert.equal(decrement(10), -5);
  assert.equal(count.value, -5);
});

void test("sets values directly and reports the applied result", () => {
  const { count, set } = useCounter(1, { min: 0, max: 10 });

  assert.equal(set(7), 7);
  assert.equal(set(42), 10);
  assert.equal(set(-3), 0);
  assert.equal(count.value, 0);
});

void test("clamps every transition into the inclusive bounds", () => {
  const { count, increment, decrement, atMin, atMax } = useCounter(9, { min: 0, max: 10 });

  assert.equal(atMax.value, false);
  assert.equal(increment(), 10);
  assert.equal(increment(), 10);
  assert.equal(atMax.value, true);
  assert.equal(decrement(25), 0);
  assert.equal(atMin.value, true);
  assert.equal(atMax.value, false);
  assert.equal(count.value, 0);
});

void test("clamps an out-of-range initial value", () => {
  const { count, atMax } = useCounter(99, { max: 10 });

  assert.equal(count.value, 10);
  assert.equal(atMax.value, true);
});

void test("clamps infinite deltas onto the bounds", () => {
  const { count, increment, decrement } = useCounter(5, { min: 0, max: 10 });

  assert.equal(increment(Number.POSITIVE_INFINITY), 10);
  assert.equal(decrement(Number.POSITIVE_INFINITY), 0);
  assert.equal(count.value, 0);
});

void test("never reports bound flags for infinite default bounds", () => {
  const { increment, atMin, atMax } = useCounter();

  increment(Number.MAX_SAFE_INTEGER);
  assert.equal(atMin.value, false);
  assert.equal(atMax.value, false);
});

void test("restores and replaces the reset baseline", () => {
  const { count, increment, reset } = useCounter(3, { min: 0, max: 10 });

  increment(4);
  assert.equal(reset(), 3);

  assert.equal(reset(8), 8);
  increment();
  assert.equal(reset(), 8);

  // A baseline outside the bounds is clamped before it is stored.
  assert.equal(reset(99), 10);
  assert.equal(count.value, 10);
  assert.equal(reset(), 10);
});

void test("rejects invalid bounds at creation", () => {
  const isRangeError = (error: unknown): boolean =>
    error instanceof RangeError && error.message.startsWith("[VIZE_COMPOSE_COUNTER_INVALID_RANGE]");

  assert.throws(() => useCounter(0, { min: 5, max: 4 }), isRangeError);
  assert.throws(() => useCounter(0, { min: Number.NaN }), isRangeError);
  assert.throws(() => useCounter(0, { max: Number.NaN }), isRangeError);
});

void test("rejects NaN values and leaves the count unchanged", () => {
  const isValueError = (error: unknown): boolean =>
    error instanceof RangeError && error.message.startsWith("[VIZE_COMPOSE_COUNTER_INVALID_VALUE]");

  assert.throws(() => useCounter(Number.NaN), isValueError);

  const { count, increment, set, reset } = useCounter(5);
  assert.throws(() => increment(Number.NaN), isValueError);
  assert.throws(() => set(Number.NaN), isValueError);
  assert.throws(() => reset(Number.NaN), isValueError);
  assert.equal(count.value, 5);
  assert.equal(reset(), 5);
});

void test("rejects indeterminate arithmetic instead of corrupting the count", () => {
  const { count, increment } = useCounter(Number.NEGATIVE_INFINITY);

  assert.equal(count.value, Number.NEGATIVE_INFINITY);
  assert.throws(
    () => increment(Number.POSITIVE_INFINITY),
    (error: unknown) =>
      error instanceof RangeError &&
      error.message.startsWith("[VIZE_COMPOSE_COUNTER_INVALID_VALUE]"),
  );
  assert.equal(count.value, Number.NEGATIVE_INFINITY);
});
