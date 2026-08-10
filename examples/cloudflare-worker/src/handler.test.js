import assert from "node:assert/strict";
import { test } from "node:test";

import { handleRequest } from "./handler.js";

function binding(calls) {
  return {
    compileSfc(source, options) {
      calls.push({ source, options });
      return { script: { code: "export default {}" }, errors: [], warnings: [] };
    },
  };
}

void test("GET compiles the built-in Vue SFC", async () => {
  const calls = [];
  const response = await handleRequest(new Request("https://example.test/"), async () =>
    binding(calls),
  );
  const payload = await response.json();

  assert.equal(response.status, 200);
  assert.equal(payload.ok, true);
  assert.equal(payload.package, "@vizejs/wasm");
  assert.equal(calls.length, 1);
  assert.match(calls[0].source, /<script setup lang="ts">/);
  assert.deepEqual(calls[0].options, { filename: "Counter.vue" });
});

void test("POST forwards the source and compiler options", async () => {
  const calls = [];
  const response = await handleRequest(
    new Request("https://example.test/", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        source: "<template><main /></template>",
        options: { filename: "App.vue", sourceMap: true },
      }),
    }),
    async () => binding(calls),
  );

  assert.equal(response.status, 200);
  assert.deepEqual(calls, [
    {
      source: "<template><main /></template>",
      options: { filename: "App.vue", sourceMap: true },
    },
  ]);
});

void test("invalid requests are rejected before WASM initialization", async () => {
  let initialized = false;
  const getBinding = async () => {
    initialized = true;
    return binding([]);
  };
  const response = await handleRequest(
    new Request("https://example.test/", { method: "POST", body: "not json" }),
    getBinding,
  );

  assert.equal(response.status, 400);
  assert.equal(initialized, false);
  assert.deepEqual(await response.json(), {
    ok: false,
    error: "Request body must be valid JSON.",
  });
});

void test("unsupported methods return 405 without initializing WASM", async () => {
  let initialized = false;
  const response = await handleRequest(
    new Request("https://example.test/", { method: "DELETE" }),
    async () => {
      initialized = true;
      return binding([]);
    },
  );

  assert.equal(response.status, 405);
  assert.equal(initialized, false);
});
