import assert from "node:assert/strict";
import { test } from "node:test";

import { retryAsync } from "./retry-async.ts";

void test("validates execution controls before invoking user work", async () => {
  let calls = 0;
  const operation = () => {
    calls += 1;
  };

  for (const maximumRetries of [
    -1,
    0.5,
    Number.NaN,
    Number.POSITIVE_INFINITY,
    Number.MAX_SAFE_INTEGER,
  ]) {
    await assert.rejects(
      retryAsync(operation, { maximumRetries }),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_MAXIMUM_RETRIES]"),
    );
  }
  await assert.rejects(
    retryAsync(operation, { shouldRetry: true as never }),
    (error: unknown) =>
      error instanceof TypeError &&
      error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_CALLBACK]"),
  );
  await assert.rejects(
    retryAsync(null as never),
    (error: unknown) =>
      error instanceof TypeError &&
      error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_OPERATION]"),
  );
  for (const options of [null, 0, false, "", "abc", Symbol("options")]) {
    await assert.rejects(
      retryAsync(operation, options as never),
      (error: unknown) =>
        error instanceof TypeError &&
        error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_OPTIONS]"),
    );
  }
  assert.equal(calls, 0);
});

void test("rejects non-boolean asynchronous policy decisions", async () => {
  await assert.rejects(
    retryAsync(
      () => {
        throw new Error("failed");
      },
      { shouldRetry: async () => "yes" as never },
    ),
    (error: unknown) =>
      error instanceof TypeError &&
      error.message.startsWith("[VIZE_COMPOSE_RETRY_INVALID_DECISION]"),
  );
});
