import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { surfaceErrors } from "./move-internal.ts";

test("multiple move cleanup errors remain inspectable without native AggregateError", () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, "AggregateError");
  Object.defineProperty(globalThis, "AggregateError", { configurable: true, value: undefined });
  try {
    assert.throws(
      () => surfaceErrors([new Error("listeners"), new Error("selection")], "move failed"),
      (error: unknown) => {
        const aggregate = error as Error & { errors?: unknown[] };
        assert.equal(aggregate.name, "AggregateError");
        assert.equal(aggregate.message, "move failed");
        assert.deepEqual(
          aggregate.errors?.map((reason) => (reason as Error).message),
          ["listeners", "selection"],
        );
        return true;
      },
    );
  } finally {
    if (descriptor) Object.defineProperty(globalThis, "AggregateError", descriptor);
  }
});
