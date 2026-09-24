import assert from "node:assert/strict";
import { test } from "node:test";

import { formatSchemaPath, isStandardSchema, validateStandardSchema } from "./standard-schema.ts";
import type { StandardSchemaV1 } from "./standard-schema.ts";
import { renderComposableOnServer } from "./testing/ssr-harness.ts";

const evenSchema: StandardSchemaV1<number, string> = {
  "~standard": {
    version: 1,
    vendor: "test",
    validate: (value) =>
      typeof value === "number" && value % 2 === 0
        ? { value: `even:${String(value)}` }
        : { issues: [{ message: "Odd", path: ["items", { key: 0 }, Symbol("name")] }] },
  },
};

void test("detects the Standard Schema protocol", () => {
  assert.equal(isStandardSchema(evenSchema), true);
  assert.equal(isStandardSchema({ "~standard": { version: 2, validate: () => ({}) } }), false);
  assert.equal(isStandardSchema(null), false);
  assert.equal(
    isStandardSchema(() => undefined),
    false,
  );
});

void test("formats issue paths as dotted paths", () => {
  assert.equal(formatSchemaPath(undefined), "");
  assert.equal(formatSchemaPath(["a", { key: 1 }, Symbol("b")]), "a.1.b");
});

void test("normalizes sync and async results", async () => {
  assert.deepEqual(await validateStandardSchema(evenSchema, 2), {
    status: "valid",
    value: "even:2",
  });
  assert.deepEqual(await validateStandardSchema(evenSchema, 3), {
    status: "invalid",
    issues: [{ path: "items.0.name", message: "Odd" }],
  });
  const asyncSchema: StandardSchemaV1<string> = {
    "~standard": {
      version: 1,
      vendor: "test",
      validate: async (value) => ({ issues: [{ message: `bad ${String(value)}` }] }),
    },
  };
  assert.deepEqual(await validateStandardSchema(asyncSchema, "x"), {
    status: "invalid",
    issues: [{ path: "", message: "bad x" }],
  });
});

void test("server rendering can check schemas synchronously", async () => {
  const state = await renderComposableOnServer(() => ({ schema: isStandardSchema(evenSchema) }));
  assert.equal(state, '{"schema":true}');
});
