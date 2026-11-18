import assert from "node:assert/strict";
import test from "node:test";

import { reactive } from "vue";

import { toPostMessagePayload } from "./usePostMessage.ts";

void test("postMessage payloads recursively unwrap Vue proxies", () => {
  const attrs = reactive(
    JSON.parse(
      '{"empty":{},"nested":[{"label":"Edited 🎨","__proto__":{"retained":true}}],"__proto__":{"nested":true},"constructor":"Authored constructor","hasOwnProperty":"Authored method name"}',
    ),
  );
  const payload = toPostMessagePayload({ props: { label: "Edited 🎨" }, attrs });

  assert.deepEqual(
    payload,
    JSON.parse(
      '{"props":{"label":"Edited 🎨"},"attrs":{"empty":{},"nested":[{"label":"Edited 🎨","__proto__":{"retained":true}}],"__proto__":{"nested":true},"constructor":"Authored constructor","hasOwnProperty":"Authored method name"}}',
    ),
  );
  assert.doesNotThrow(() => structuredClone(payload));
});

void test("postMessage payload conversion retains circular record identity", () => {
  const raw: Record<string, unknown> = {};
  raw.self = raw;
  const payload = toPostMessagePayload(reactive(raw));

  assert.equal(payload.self, payload);
  assert.doesNotThrow(() => structuredClone(payload));
});
