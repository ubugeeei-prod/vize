import assert from "node:assert/strict";
import { test } from "node:test";

import { MISSKEY_HMR_TARGETS } from "../app/dev/misskey-hmr-targets.ts";
import { isVizeClientModuleUrl, NUXT_UI_HMR_MODULE_PATH } from "../app/dev/vize-client-module.ts";

const origin = "http://localhost:5173";
const suffixes = [
  ...MISSKEY_HMR_TARGETS.map((target) => target.moduleSuffix),
  NUXT_UI_HMR_MODULE_PATH,
];

test("App HMR observers consume the actual Vite-visible module producer", async () => {
  const { toPluginVisibleVirtualId } = await import("../../npm/builder/vite/src/virtual.ts");
  for (const suffix of suffixes) {
    const moduleId = toPluginVisibleVirtualId(`/authored${suffix}`);
    assert.equal(moduleId, `/authored${suffix}?vue&vize`);
    assert.equal(isVizeClientModuleUrl(`${origin}${moduleId}`, suffix, origin), true);
    const hotId = toPluginVisibleVirtualId(`/authored${suffix}`, false, "?t=123");
    assert.equal(hotId, `/authored${suffix}?vue&vize&t=123`);
    assert.equal(isVizeClientModuleUrl(`${origin}${hotId}`, suffix, origin, true), true);
  }
});

test("App HMR observers reject stale virtual TypeScript suffixes", () => {
  for (const suffix of suffixes) {
    assert.equal(isVizeClientModuleUrl(`${origin}${suffix}.ts?vue&vize`, suffix, origin), false);
  }
});

test("App HMR observers reject raw, SSR, and incomplete query identities", () => {
  for (const suffix of suffixes) {
    for (const query of ["", "?vue", "?vize", "?vue&vize-ssr", "?vue&vize&vize-ssr"]) {
      assert.equal(isVizeClientModuleUrl(`${origin}${suffix}${query}`, suffix, origin), false);
    }
  }
});

test("App HMR observers reject foreign origins and wrong authored files", () => {
  for (const suffix of suffixes) {
    assert.equal(
      isVizeClientModuleUrl(`https://foreign.invalid${suffix}?vue&vize`, suffix, origin),
      false,
    );
    assert.equal(isVizeClientModuleUrl(`${origin}${suffix}Other?vue&vize`, suffix, origin), false);
    assert.equal(isVizeClientModuleUrl(`${origin}/other.vue?vue&vize`, suffix, origin), false);
    assert.equal(isVizeClientModuleUrl("not a URL", suffix, origin), false);
  }
});

test("App HMR observers require a fresh timestamp only for hot requests", () => {
  for (const suffix of suffixes) {
    const url = `${origin}${suffix}?vue&vize`;
    assert.equal(isVizeClientModuleUrl(url, suffix, origin), true);
    assert.equal(isVizeClientModuleUrl(url, suffix, origin, true), false);
    assert.equal(isVizeClientModuleUrl(`${url}&t=123`, suffix, origin, true), true);
  }
});
