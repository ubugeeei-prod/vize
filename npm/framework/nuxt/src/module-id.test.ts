import assert from "node:assert/strict";
import { test } from "node:test";
import { isVizeGeneratedVueModuleId, NUXT_TRANSFORM_BRIDGE_ID_FILTER } from "./module-id.ts";

void test("Nuxt bridges canonical Vue IDs and retains legacy ownership without capturing raw/style files", () => {
  for (const id of [
    "/app/Page.vue?vue&vize",
    "/app/Page.vue.ts?vue&vize-ssr",
    "\0vize-ssr:/app/Page.vue.ts",
    "/@id/__x00__/app/Page.vue?vue&vize",
  ])
    assert.equal(isVizeGeneratedVueModuleId(id), true);
  for (const id of [
    "/app/Page.vue",
    "/app/Page.vue?vue&type=style&index=0",
    "/app/Page.vue?vue&vize&type=style",
    "/app/Page.vue?raw",
    "/app/Page.ts?vue&vize",
    "/app/Page.vue?vue",
  ])
    assert.equal(isVizeGeneratedVueModuleId(id), false);
});

void test("the transform hook admits all bridge module forms and skips unrelated modules", () => {
  for (const prefix of ["", "\0", "\0vize-ssr:", "/@id/__x00__", "__x00__"]) {
    for (const suffix of [
      ".vue?vue&vize",
      ".vue?vue&vize-ssr",
      ".vue.ts",
      ".vue.ts?vue&vize",
      ".vue.tsx?vue&vize-ssr",
      ".jsx",
      ".tsx?vue",
    ]) {
      const id = `${prefix}/app/components/Item${suffix}`;
      assert.equal(NUXT_TRANSFORM_BRIDGE_ID_FILTER.test(id), true, id);
    }
  }
  for (const suffix of [".ts", ".js", ".mjs", ".css", ".json", ".html"]) {
    for (let index = 0; index < 100; index += 1) {
      const id = `/app/utils/value${index}${suffix}`;
      assert.equal(NUXT_TRANSFORM_BRIDGE_ID_FILTER.test(id), false, id);
    }
  }
  // The hook filter is intentionally broader than the ownership predicate.
  // Raw Vue, styles and asset queries must still be rejected by the handler.
  for (const id of ["/app/Item.vue", "/app/Item.vue?vue&vize&type=style"]) {
    assert.equal(NUXT_TRANSFORM_BRIDGE_ID_FILTER.test(id), true, id);
    assert.equal(isVizeGeneratedVueModuleId(id), false, id);
  }
});
