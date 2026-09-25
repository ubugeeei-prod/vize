import assert from "node:assert/strict";
import test from "node:test";

import { externalizeVueRuntimeForNuxtSsr } from "./ssr-runtime.ts";

void test("Nuxt SSR build shares external Vue with Vize SFC chunks", () => {
  const nitroRuntime = /^nitro\/runtime$/;
  const config = {
    build: { rolldownOptions: { external: [nitroRuntime, "#internal/nitro", "vue"] } },
  };

  externalizeVueRuntimeForNuxtSsr(config);
  externalizeVueRuntimeForNuxtSsr(config);

  assert.deepEqual(config.build.rolldownOptions.external, [
    nitroRuntime,
    "#internal/nitro",
    "vue",
    "vue/server-renderer",
    "@vue/server-renderer",
  ]);
});

void test("Nuxt SSR build preserves an external predicate", () => {
  const prior = (id: string) => id === "nitro/runtime";
  const config = { build: { rolldownOptions: { external: prior } } };
  externalizeVueRuntimeForNuxtSsr(config);

  assert.equal(config.build.rolldownOptions.external("vue"), true);
  assert.equal(config.build.rolldownOptions.external("vue/server-renderer"), true);
  assert.equal(config.build.rolldownOptions.external("@vue/server-renderer"), true);
  assert.equal(config.build.rolldownOptions.external("nitro/runtime"), true);
  assert.equal(config.build.rolldownOptions.external("other"), false);
});

void test("Vite builds without Rolldown options are left alone", () => {
  const config = { build: { rollupOptions: { external: ["nitro/runtime"] } } };
  externalizeVueRuntimeForNuxtSsr(config);
  assert.deepEqual(config, { build: { rollupOptions: { external: ["nitro/runtime"] } } });
});
