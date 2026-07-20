import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useAsyncResource } from "./async-resource.ts";

void test("commits only the newest execution result", async () => {
  const pending = new Map<number, (value: number) => void>();
  const signals: AbortSignal[] = [];
  const resource = useAsyncResource<number, readonly [number]>((context, value) => {
    signals.push(context.signal);
    return new Promise((resolve) => pending.set(value, resolve));
  });

  const first = resource.execute(1);
  const second = resource.execute(2);
  assert.equal(signals[0]?.aborted, true);
  pending.get(2)?.(20);
  assert.deepEqual(await second, { status: "success", data: 20 });
  pending.get(1)?.(10);
  assert.deepEqual(await first, { status: "superseded" });
  assert.equal(resource.data.value, 20);
});

void test("leaves superseded work running when cancellation is disabled", async () => {
  const pending = new Map<number, (value: number) => void>();
  const signals: AbortSignal[] = [];
  const resource = useAsyncResource<number, readonly [number]>(
    (context, value) => {
      signals.push(context.signal);
      return new Promise((resolve) => pending.set(value, resolve));
    },
    { cancelPrevious: false },
  );

  const first = resource.execute(1);
  const second = resource.execute(2);
  assert.equal(signals[0]?.aborted, false);
  pending.get(1)?.(10);
  assert.deepEqual(await first, { status: "superseded" });
  pending.get(2)?.(20);
  assert.deepEqual(await second, { status: "success", data: 20 });
});

void test("reports idempotent manual cancellation without an error state", async () => {
  const resource = useAsyncResource<never, readonly []>(
    ({ signal }) =>
      new Promise((_, reject) =>
        signal.addEventListener("abort", () => reject(signal.reason), { once: true }),
      ),
  );
  const execution = resource.execute();

  assert.equal(resource.cancel("manual"), true);
  assert.equal(resource.cancel("again"), false);
  assert.deepEqual(await execution, { status: "cancelled", reason: "manual" });
  assert.equal(resource.status.value, "cancelled");
  assert.equal(resource.pending.value, false);
  assert.equal(resource.error.value, undefined);
});

void test("returns typed loader failures as explicit results", async () => {
  const failure = { code: "unavailable" } as const;
  const resource = useAsyncResource<never, readonly [], typeof failure>(async () => {
    throw failure;
  });

  assert.deepEqual(await resource.execute(), { status: "error", error: failure });
  assert.equal(resource.error.value, failure);
  assert.equal(resource.status.value, "error");
});

void test("clears refresh data and restores the documented initial value", async () => {
  let resolve: ((value: number) => void) | undefined;
  const resource = useAsyncResource<number, readonly []>(
    () => new Promise((complete) => (resolve = complete)),
    { initialData: 1, keepData: false },
  );
  const execution = resource.execute();

  assert.equal(resource.data.value, undefined);
  resolve?.(2);
  assert.equal((await execution).status, "success");
  resource.reset();
  assert.equal(resource.data.value, 1);
  assert.equal(resource.status.value, "idle");
});

void test("cancels active work when its reactive scope stops", async () => {
  const scope = effectScope();
  const resource = scope.run(() =>
    useAsyncResource<never, readonly []>(
      ({ signal }) =>
        new Promise((_, reject) =>
          signal.addEventListener("abort", () => reject(signal.reason), { once: true }),
        ),
    ),
  );
  assert.ok(resource);
  const execution = resource.execute();

  scope.stop();
  assert.equal((await execution).status, "cancelled");
  assert.equal(resource.status.value, "cancelled");
});
