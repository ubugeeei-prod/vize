import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, ref } from "vue";

import {
  createInfiniteLoader,
  type InfiniteLoadContext,
  type VirtualRange,
} from "./virtualizer.ts";

interface Deferred {
  readonly promise: Promise<void>;
  readonly resolve: () => void;
  readonly reject: (error: unknown) => void;
}

function deferred(): Deferred {
  let resolve!: () => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<void>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });
  return { promise, resolve, reject };
}

const flush = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

test("loads forward when the range nears the end", async () => {
  const count = ref(20);
  const range = ref<VirtualRange | null>({ startIndex: 0, endIndex: 5 });
  const contexts: InfiniteLoadContext[] = [];
  let gate = deferred();
  const loader = createInfiniteLoader({
    range,
    count,
    threshold: 2,
    canLoadForward: true,
    onLoadForward(context) {
      contexts.push(context);
      return gate.promise;
    },
  });

  assert.equal(loader.forwardStatus.value, "idle", "a distant range does not load");
  range.value = { startIndex: 12, endIndex: 17 };
  assert.equal(loader.forwardStatus.value, "loading");
  assert.equal(contexts.length, 1);
  assert.deepEqual(contexts[0]?.direction, "forward");
  assert.equal(contexts[0]?.signal.aborted, false);

  range.value = { startIndex: 13, endIndex: 18 };
  assert.equal(contexts.length, 1, "one in-flight load per direction");

  count.value = 40;
  gate.resolve();
  await flush();
  assert.equal(loader.forwardStatus.value, "idle");
  assert.equal(contexts.length, 1, "the grown collection is beyond the threshold again");

  gate = deferred();
  range.value = { startIndex: 34, endIndex: 39 };
  assert.equal(contexts.length, 2);
  loader.dispose();
});

test("loads backward at the leading edge", () => {
  const directions: string[] = [];
  const loader = createInfiniteLoader({
    range: () => ({ startIndex: 3, endIndex: 9 }),
    count: () => 100,
    threshold: 3,
    canLoadBackward: true,
    onLoadBackward: (context) => {
      directions.push(context.direction);
    },
  });

  assert.deepEqual(directions, ["backward"]);
  loader.dispose();
});

test("starts an initial forward load for an empty collection", () => {
  let calls = 0;
  const loader = createInfiniteLoader({
    range: () => null,
    count: () => 0,
    canLoadForward: true,
    onLoadForward: () => {
      calls += 1;
    },
  });
  assert.equal(calls, 1);
  loader.dispose();
});

test("ignores stale results after cancellation", async () => {
  const range = ref<VirtualRange | null>({ startIndex: 0, endIndex: 19 });
  const gate = deferred();
  const contexts: InfiniteLoadContext[] = [];
  const errors: unknown[] = [];
  const loader = createInfiniteLoader({
    range,
    count: () => 20,
    threshold: 1,
    canLoadForward: true,
    onLoadForward(context) {
      contexts.push(context);
      return gate.promise;
    },
    onLoadError: (direction, error) => errors.push([direction, error]),
  });

  assert.equal(loader.forwardStatus.value, "loading");
  loader.cancel("forward");
  assert.equal(loader.forwardStatus.value, "idle");
  assert.equal(contexts[0]?.signal.aborted, true, "cancellation aborts the load signal");

  gate.resolve();
  await flush();
  assert.equal(contexts.length, 1, "a stale settlement never re-triggers loading");
  assert.deepEqual(errors, []);
  loader.dispose();
});

test("reports errors for live loads and swallows canceled ones", async () => {
  const errors: unknown[][] = [];
  let gate = deferred();
  const loader = createInfiniteLoader({
    range: () => ({ startIndex: 0, endIndex: 19 }),
    count: () => 20,
    canLoadForward: true,
    onLoadForward: () => gate.promise,
    onLoadError: (direction, error) => errors.push([direction, error]),
  });

  const failure = new Error("network down");
  gate.reject(failure);
  await flush();
  assert.deepEqual(errors, [["forward", failure]]);
  assert.equal(loader.forwardStatus.value, "idle");

  gate = deferred();
  loader.check();
  assert.equal(loader.forwardStatus.value, "loading");
  loader.cancel();
  gate.reject(new Error("ignored"));
  await flush();
  assert.equal(errors.length, 1, "canceled rejections are swallowed");
  loader.dispose();
});

test("respects direction gates and validates options", () => {
  let calls = 0;
  const loader = createInfiniteLoader({
    range: () => ({ startIndex: 0, endIndex: 19 }),
    count: () => 20,
    onLoadForward: () => {
      calls += 1;
    },
  });
  assert.equal(calls, 0, "loading stays off until canLoadForward is true");
  loader.dispose();
  assert.throws(() => loader.check(), /VIZE_UI_VIRTUALIZER_DISPOSED/);

  assert.throws(
    () =>
      createInfiniteLoader({
        range: () => null,
        count: () => 0,
        onLoadForward: 5 as unknown as () => void,
      }),
    /onLoadForward must be a function/,
  );
  assert.throws(
    () => createInfiniteLoader({ range: () => null, count: () => 0, threshold: -1 }),
    /threshold must resolve to a non-negative integer/,
  );
});

test("binds disposal to the owning effect scope", async () => {
  const { useInfiniteLoader } = await import("./virtualizer.ts");
  assert.throws(
    () => useInfiniteLoader({ range: () => null, count: () => 0 }),
    /VIZE_UI_VIRTUALIZER_SETUP/,
  );
  const scope = effectScope();
  const loader = scope.run(() => useInfiniteLoader({ range: () => null, count: () => 0 }));
  assert.ok(loader);
  scope.stop();
  assert.throws(() => loader.check(), /VIZE_UI_VIRTUALIZER_DISPOSED/);
});
