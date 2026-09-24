import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, shallowRef } from "vue";

import { computedAsync } from "./computed-async.ts";

function deferred<Value>() {
  let resolve: (value: Value) => void = () => undefined;
  let reject: (reason: unknown) => void = () => undefined;
  const promise = new Promise<Value>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  return { promise, resolve, reject };
}

const settle = async (): Promise<void> => {
  for (let index = 0; index < 5; index += 1) await Promise.resolve();
  await nextTick();
};

void test("holds the initial value until the evaluation settles", async () => {
  const id = shallowRef(1);
  const value = computedAsync(async () => `user-${id.value}`, "loading");
  assert.equal(value.value, "loading");
  await settle();
  assert.equal(value.value, "user-1");
  id.value = 2;
  await settle();
  assert.equal(value.value, "user-2");
});

void test("stale evaluations are cancelled and never win", async () => {
  const id = shallowRef(1);
  const pending = new Map<number, ReturnType<typeof deferred<string>>>();
  const aborted: number[] = [];
  const cancelled: number[] = [];
  const evaluating = shallowRef(false);
  const value = computedAsync(
    ({ signal, onCancel }) => {
      const current = id.value;
      const request = deferred<string>();
      pending.set(current, request);
      signal.addEventListener("abort", () => aborted.push(current));
      onCancel(() => cancelled.push(current));
      return request.promise;
    },
    undefined,
    { evaluating },
  );

  assert.equal(evaluating.value, true);
  id.value = 2;
  await nextTick();
  pending.get(2)?.resolve("two");
  await settle();
  pending.get(1)?.resolve("one");
  await settle();
  assert.equal(value.value, "two");
  assert.deepEqual(aborted, [1]);
  assert.deepEqual(cancelled, [1]);
  assert.equal(evaluating.value, false);
});

void test("lazy evaluation starts on first read", async () => {
  let runs = 0;
  const value = computedAsync(
    () => {
      runs += 1;
      return 42;
    },
    0,
    { lazy: true },
  );
  await settle();
  assert.equal(runs, 0);
  assert.equal(value.value, 0);
  await settle();
  assert.equal(runs, 1);
  assert.equal(value.value, 42);
});

void test("failures reach onError and keep the previous value", async () => {
  const errors: unknown[] = [];
  const value = computedAsync(
    async () => {
      throw new Error("boom");
    },
    "kept",
    { onError: (error) => errors.push(error) },
  );
  await settle();
  assert.equal(value.value, "kept");
  assert.equal(errors.length, 1);
});

void test("stopping the scope aborts the running evaluation", async () => {
  const scope = effectScope();
  let signal: AbortSignal | undefined;
  scope.run(() =>
    computedAsync(({ signal: current }) => {
      signal = current;
      return new Promise<number>(() => undefined);
    }, 0),
  );
  scope.stop();
  assert.equal(signal?.aborted, true);
});
