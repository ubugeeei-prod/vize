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

void test("environment-scoped SSR externalization leaves the shared client build untouched", () => {
  const sharedBuild = { rolldownOptions: { external: ["existing"] } };
  const ssrBuild = { rolldownOptions: { external: ["server-only"] } };
  const config = { build: sharedBuild, environments: { ssr: { build: ssrBuild } } };
  const clientConfig = { build: sharedBuild, environments: { client: {} } };

  externalizeVueRuntimeForNuxtSsr(config);

  assert.deepEqual(clientConfig, {
    build: { rolldownOptions: { external: ["existing"] } },
    environments: { client: {} },
  });
  assert.deepEqual(config.environments.ssr.build.rolldownOptions.external, [
    "server-only",
    "vue",
    "vue/server-renderer",
    "@vue/server-renderer",
  ]);
});

void test("SSR environments retain an inherited predicate without changing shared options", () => {
  const calls: unknown[][] = [];
  const prior = (id: string, importer: string, resolved: boolean) => {
    calls.push([id, importer, resolved]);
    return id === "server-dependency";
  };
  const sharedOptions = { external: prior };
  const config = {
    build: { rolldownOptions: sharedOptions },
    environments: { ssr: { build: { rolldownOptions: sharedOptions } } },
  };

  externalizeVueRuntimeForNuxtSsr(config);

  assert.equal(config.build.rolldownOptions.external, prior);
  assert.equal(config.environments.ssr.build.rolldownOptions.external("vue", "entry", false), true);
  assert.equal(
    config.environments.ssr.build.rolldownOptions.external("server-dependency", "entry", true),
    true,
  );
  assert.deepEqual(calls, [["server-dependency", "entry", true]]);
});

void test("a client-only environment does not acquire server external rules", () => {
  const config = {
    build: { rolldownOptions: { external: ["existing"] } },
    environments: { client: {} },
  };
  externalizeVueRuntimeForNuxtSsr(config);
  assert.deepEqual(config, {
    build: { rolldownOptions: { external: ["existing"] } },
    environments: { client: {} },
  });
});

void test("SSR owns a shared build object before inheriting external rules", () => {
  const sharedBuild = { rolldownOptions: { external: ["existing"] } };
  const config = { build: sharedBuild, environments: { ssr: { build: sharedBuild } } };
  externalizeVueRuntimeForNuxtSsr(config);
  assert.notEqual(config.environments.ssr.build, sharedBuild);
  assert.notEqual(config.environments.ssr.build.rolldownOptions, sharedBuild.rolldownOptions);
  assert.deepEqual(sharedBuild.rolldownOptions.external, ["existing"]);
  assert.deepEqual(config.environments.ssr.build.rolldownOptions.external, [
    "existing",
    "vue",
    "vue/server-renderer",
    "@vue/server-renderer",
  ]);
});

void test("an empty SSR environment inherits root options without acquiring the root build", () => {
  const config: {
    build: { rolldownOptions: { external: string[] } };
    environments: { ssr: { build?: { rolldownOptions?: { external?: unknown } } } };
  } = { build: { rolldownOptions: { external: ["existing"] } }, environments: { ssr: {} } };
  externalizeVueRuntimeForNuxtSsr(config);
  assert.deepEqual(config.build.rolldownOptions.external, ["existing"]);
  assert.deepEqual(config.environments.ssr.build?.rolldownOptions?.external, [
    "existing",
    "vue",
    "vue/server-renderer",
    "@vue/server-renderer",
  ]);
});
