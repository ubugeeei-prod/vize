import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import path from "node:path";

type MuseaNuxtRuntime = typeof import("./index.ts");

async function loadRuntime(): Promise<MuseaNuxtRuntime> {
  return (await import(new URL("../dist/index.mjs", import.meta.url).href)) as MuseaNuxtRuntime;
}

void test("Nuxt Musea plugin resolves Nuxt virtual imports to mock modules", async () => {
  const { nuxtMusea } = await loadRuntime();
  const plugin = nuxtMusea({
    route: { path: "/mock" },
    runtimeConfig: { public: { apiBase: "/api" } },
    stateMocks: { ready: true },
  });
  assert.equal(plugin.name, "vite-plugin-musea-nuxt");
  assert.equal(typeof plugin.resolveId, "function");
  assert.equal(typeof plugin.load, "function");

  const importsId = await plugin.resolveId.call(null, "#imports", undefined, {
    assertions: {},
    custom: {},
    isEntry: false,
    ssr: false,
  });
  assert.equal(importsId, "\0musea-nuxt:imports");

  const importsCode = await plugin.load.call(null, importsId, { ssr: false });
  assert.equal(typeof importsCode, "string");
  assert.match(importsCode, /export \* from '.+auto-imports\.js';/);
  assert.match(importsCode, /configureNuxtMuseaMocks\(_config\);/);
  assert.match(importsCode, /"path":"\/mock"/);
  assert.match(importsCode, /"ready":true/);

  const appSubpathId = await plugin.resolveId.call(null, "#app/composables/router", undefined, {
    assertions: {},
    custom: {},
    isEntry: false,
    ssr: false,
  });
  assert.equal(appSubpathId, "\0musea-nuxt:imports");
});

void test("Nuxt Musea plugin exposes built-in component mocks through #components", async () => {
  const { nuxtMusea } = await loadRuntime();
  const plugin = nuxtMusea();
  assert.equal(typeof plugin.resolveId, "function");
  assert.equal(typeof plugin.load, "function");

  const componentsId = await plugin.resolveId.call(null, "#components", undefined, {
    assertions: {},
    custom: {},
    isEntry: false,
    ssr: false,
  });
  assert.equal(componentsId, "\0musea-nuxt:components");

  const buildComponentsId = await plugin.resolveId.call(null, "#build/components", undefined, {
    assertions: {},
    custom: {},
    isEntry: false,
    ssr: false,
  });
  assert.equal(buildComponentsId, "\0musea-nuxt:components");

  const componentsCode = await plugin.load.call(null, componentsId, { ssr: false });
  assert.equal(typeof componentsCode, "string");
  for (const exportName of [
    "NuxtLink",
    "NuxtPage",
    "ClientOnly",
    "NuxtRouteAnnouncer",
    "NuxtImg",
  ]) {
    assert.match(componentsCode, new RegExp(`\\b${exportName}\\b`));
  }
});

void test("Nuxt component mocks stay ESM-only", () => {
  const source = fs.readFileSync(path.join(import.meta.dirname, "mocks/components.ts"), "utf-8");

  assert.doesNotMatch(source, /\brequire\s*\(/);
});
