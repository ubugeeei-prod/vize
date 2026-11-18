import assert from "node:assert/strict";
import test from "node:test";

import { reactive } from "vue";

import { toPostMessagePayload } from "./usePostMessage.ts";

void test("postMessage payloads recursively unwrap Vue proxies", () => {
  const attrs = reactive({
    empty: {},
    nested: [{ label: "Edited 🎨" }],
  });
  const payload = toPostMessagePayload({ props: { label: "Edited 🎨" }, attrs });

  assert.deepEqual(payload, {
    props: { label: "Edited 🎨" },
    attrs: { empty: {}, nested: [{ label: "Edited 🎨" }] },
  });
  assert.doesNotThrow(() => structuredClone(payload));
});

void test("postMessage payload conversion retains circular record identity", () => {
  const raw: Record<string, unknown> = {};
  raw.self = raw;
  const payload = toPostMessagePayload(reactive(raw));

  assert.equal(payload.self, payload);
  assert.doesNotThrow(() => structuredClone(payload));
});
