import assert from "node:assert/strict";
import { test } from "node:test";

import { useAsyncState } from "./use-async-state.ts";

void test("executes immediately and tracks loading, readiness, and data", async () => {
  const successes: string[] = [];
  const { state, isLoading, isReady, execute } = useAsyncState(async () => "data", "initial", {
    onSuccess: (data) => successes.push(data),
  });
  assert.equal(state.value, "initial");
  assert.equal(isLoading.value, true);
  assert.equal(await execute(), "data");
  assert.equal(isLoading.value, false);
  assert.equal(isReady.value, true);
  // The explicit execution superseded the immediate one before it settled.
  assert.deepEqual(successes, ["data"]);
});

void test("passes arguments and lets only the newest execution win", async () => {
  const resolvers = new Map<string, (value: string) => void>();
  const { state, execute } = useAsyncState(
    (id: string) =>
      new Promise<string>((resolve) => {
        resolvers.set(id, resolve);
      }),
    null,
    { immediate: false },
  );
  const first = execute("a");
  const second = execute("b");
  resolvers.get("b")?.("B");
  assert.equal(await second, "B");
  resolvers.get("a")?.("A");
  assert.equal(await first, "B");
  assert.equal(state.value, "B");
});

void test("stores failures, reports them, and optionally re-throws", async () => {
  const errors: unknown[] = [];
  const failing = useAsyncState(
    async () => {
      throw new Error("boom");
    },
    0,
    { immediate: false, onError: (error) => errors.push(error), resetOnExecute: true },
  );
  assert.equal(await failing.execute(), 0);
  assert.ok(failing.error.value instanceof Error);
  assert.equal(errors.length, 1);

  const throwing = useAsyncState(
    async () => {
      throw new Error("loud");
    },
    0,
    { immediate: false, throwError: true },
  );
  await assert.rejects(throwing.execute(), /loud/);
});
