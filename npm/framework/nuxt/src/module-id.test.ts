import assert from "node:assert/strict";
import { test } from "node:test";
import { isVizeGeneratedVueModuleId } from "./module-id.ts";

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
