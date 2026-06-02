import assert from "node:assert/strict";

import {
  fromPluginVisibleVirtualId,
  fromVirtualId,
  isPluginVisibleSsrVirtualId,
  isVizeVirtual,
  normalizeVizeVirtualVueModuleId,
  toPluginVisibleVirtualId,
} from "./virtual.ts";

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

const visibleVirtualId = toPluginVisibleVirtualId("/repo/app/components/Foo.vue");
const visibleSsrVirtualId = toPluginVisibleVirtualId(
  "/repo/app/components/Foo.vue",
  true,
  "?vue&used=true",
);

assert.equal(
  visibleVirtualId,
  "/repo/app/components/Foo.vue.ts?vue&vize",
  "Plugin-visible virtual IDs should keep the Vue query without using a null-byte prefix",
);
assert.equal(
  visibleSsrVirtualId,
  "/repo/app/components/Foo.vue.ts?vue&vize-ssr&used=true",
  "SSR plugin-visible virtual IDs should preserve non-Vize query parameters",
);
assert.equal(
  fromPluginVisibleVirtualId(visibleVirtualId),
  "/repo/app/components/Foo.vue",
  "Plugin-visible virtual IDs should resolve back to the real .vue path",
);
assert.equal(
  isPluginVisibleSsrVirtualId(visibleSsrVirtualId),
  true,
  "SSR plugin-visible virtual IDs should keep an explicit SSR marker",
);

console.log("✅ vite-plugin-vize virtual module tests passed!");
