import assert from "node:assert/strict";
import { test } from "node:test";

import { calculateRetryDelay } from "./retry-delay.ts";

void test("uses a one-based exponential backoff with documented defaults", () => {
  assert.equal(calculateRetryDelay(1), 100);
  assert.equal(calculateRetryDelay(2), 200);
  assert.equal(calculateRetryDelay(3), 400);
  assert.equal(calculateRetryDelay(100), 30_000);
});

void test("rounds fractional backoff upward and applies the cap first", () => {
  const options = { initialDelayMs: 100, maximumDelayMs: 225, multiplier: 1.5 } as const;

  assert.equal(calculateRetryDelay(1, options), 100);
  assert.equal(calculateRetryDelay(2, options), 150);
  assert.equal(calculateRetryDelay(3, options), 225);
  assert.equal(calculateRetryDelay(4, options), 225);
  assert.equal(calculateRetryDelay(1, { initialDelayMs: 100, maximumDelayMs: 25 }), 25);
});

void test("samples the inclusive integer jitter range deterministically", () => {
  const samples = [0, 0.499, 0.999_999];
  const random = () => {
    const sample = samples.shift();
    assert.ok(sample !== undefined);
    return sample;
  };
  const options = {
    initialDelayMs: 100,
    jitterRatio: 0.5,
    maximumDelayMs: 100,
    random,
  } as const;

  assert.equal(calculateRetryDelay(1, options), 50);
  assert.equal(calculateRetryDelay(1, options), 75);
  assert.equal(calculateRetryDelay(1, options), 100);
  assert.deepEqual(samples, []);
});

void test("full jitter spans zero through the capped delay", () => {
  assert.equal(
    calculateRetryDelay(4, {
      initialDelayMs: 100,
      jitterRatio: 1,
      maximumDelayMs: 250,
      random: () => 0,
    }),
    0,
  );
  assert.equal(
    calculateRetryDelay(4, {
      initialDelayMs: 100,
      jitterRatio: 1,
      maximumDelayMs: 250,
      random: () => 0.999_999,
    }),
    250,
  );
});

void test("stays integral and bounded across extreme policy combinations", () => {
  for (const retryAttempt of [1, 2, 3, 10, Number.MAX_SAFE_INTEGER]) {
    for (const multiplier of [1, 1.1, 2, 10]) {
      for (const jitterRatio of [0, 0.25, 0.5, 1]) {
        for (const sample of [0, 0.1, 0.5, 0.999_999]) {
          const delay = calculateRetryDelay(retryAttempt, {
            initialDelayMs: 17,
            jitterRatio,
            maximumDelayMs: 1_001,
            multiplier,
            random: () => sample,
          });

          assert.ok(Number.isInteger(delay));
          assert.ok(delay >= 0);
          assert.ok(delay <= 1_001);
        }
      }
    }
  }
});

void test("does not request entropy when jitter cannot change the result", () => {
  let calls = 0;
  const random = () => {
    calls += 1;
    return 0;
  };

  assert.equal(calculateRetryDelay(2, { random }), 200);
  assert.equal(
    calculateRetryDelay(Number.MAX_SAFE_INTEGER, {
      initialDelayMs: 0,
      jitterRatio: 1,
      random,
    }),
    0,
  );
  assert.equal(
    calculateRetryDelay(Number.MAX_SAFE_INTEGER, {
      jitterRatio: 1,
      maximumDelayMs: 0,
      random,
    }),
    0,
  );
  assert.equal(calculateRetryDelay(1, { initialDelayMs: 1, jitterRatio: 0.1, random }), 1);
  assert.equal(calls, 0);
});

void test("rejects retry numbers that cannot identify an attempt", () => {
  for (const attempt of [0, -1, 1.5, Number.NaN, Number.POSITIVE_INFINITY, 2 ** 53]) {
    assert.throws(
      () => calculateRetryDelay(attempt),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_ATTEMPT]"),
    );
  }
});

void test("rejects option containers that cannot carry a delay policy", () => {
  for (const options of [null, 0, false, "", "abc", Symbol("options")]) {
    assert.throws(
      () => calculateRetryDelay(1, options as never),
      (error: unknown) =>
        error instanceof TypeError &&
        error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_OPTIONS]"),
    );
  }
});

void test("rejects non-portable backoff options before requesting entropy", () => {
  let randomCalls = 0;
  const random = () => {
    randomCalls += 1;
    return 0;
  };
  const invalidOptions = [
    { initialDelayMs: -1 },
    { initialDelayMs: null as never },
    { initialDelayMs: 0.5 },
    { initialDelayMs: 2_147_483_648 },
    { maximumDelayMs: Number.NaN },
    { maximumDelayMs: 2_147_483_648 },
    { multiplier: 0.999 },
    { multiplier: Number.POSITIVE_INFINITY },
    { jitterRatio: -0.001 },
    { jitterRatio: 1.001 },
  ] as const;

  for (const options of invalidOptions) {
    assert.throws(
      () => calculateRetryDelay(1, { jitterRatio: 1, random, ...options }),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_OPTIONS]"),
    );
  }
  assert.equal(randomCalls, 0);
});

void test("rejects invalid entropy without returning an out-of-range delay", () => {
  for (const sample of [-0.001, 1, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.throws(
      () => calculateRetryDelay(1, { jitterRatio: 1, random: () => sample }),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_RANDOM]"),
    );
  }

  assert.throws(
    () =>
      calculateRetryDelay(1, {
        jitterRatio: 1,
        random: "not callable" as never,
      }),
    (error: unknown) =>
      error instanceof TypeError && error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_RANDOM]"),
  );
  assert.throws(
    () => calculateRetryDelay(1, { jitterRatio: 1, random: null as never }),
    (error: unknown) =>
      error instanceof TypeError && error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_RANDOM]"),
  );
});
