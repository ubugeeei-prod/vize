import assert from "node:assert/strict";

import { fromVirtualId, isVizeVirtual, normalizeVizeVirtualVueModuleId } from "./virtual.ts";

const clientVirtualId = "\0/repo/app/components/Foo.vue.ts?macro=true";
const ssrVirtualId = "\0vize-ssr:/repo/app/components/Foo.vue.ts?vue&type=template";

assert.equal(
  isVizeVirtual(clientVirtualId),
  true,
  "Client virtual IDs should remain detectable when Vite appends query parameters",
);
assert.equal(
  isVizeVirtual(ssrVirtualId),
  true,
  "SSR virtual IDs should remain detectable when Vite appends query parameters",
);

assert.equal(
  fromVirtualId(clientVirtualId),
  "/repo/app/components/Foo.vue",
  "Client virtual IDs should resolve back to the real .vue path without the synthetic suffix",
);
assert.equal(
  fromVirtualId(ssrVirtualId),
  "/repo/app/components/Foo.vue",
  "SSR virtual IDs should resolve back to the real .vue path without preserving request queries",
);

assert.equal(
  normalizeVizeVirtualVueModuleId(clientVirtualId),
  "/repo/app/components/Foo.vue?macro=true",
  "Normalized client virtual IDs should keep the original query string for downstream plugins",
);
assert.equal(
  normalizeVizeVirtualVueModuleId(ssrVirtualId),
  "/repo/app/components/Foo.vue?vue&type=template",
  "Normalized SSR virtual IDs should keep the original query string for downstream plugins",
);

console.log("✅ vite-plugin-vize virtual module tests passed!");
