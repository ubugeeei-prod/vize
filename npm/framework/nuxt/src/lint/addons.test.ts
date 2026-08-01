import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  setupNuxtLintConfigAddons,
  VIZE_NUXT_LINT_CONFIG_ADDONS_HOOK,
  type NuxtLintConfigAddon,
  type NuxtLintConfigAddonNuxt,
  type NuxtLintImport,
} from "./addons.ts";
import { setupNuxtLintConfigGeneration } from "./generation.ts";

type Hook = (...args: unknown[]) => unknown;

function createNuxtStub(
  extendAddons?: (addons: NuxtLintConfigAddon[]) => void | Promise<void>,
): NuxtLintConfigAddonNuxt & {
  callRegisteredHook(name: string, ...args: unknown[]): Promise<void>;
} {
  const hooks = new Map<string, Hook[]>();

  return {
    hook(name, handler) {
      const registered = hooks.get(name) ?? [];
      registered.push(handler as Hook);
      hooks.set(name, registered);
    },
    async callHook(name, addons) {
      assert.equal(name, VIZE_NUXT_LINT_CONFIG_ADDONS_HOOK);
      await extendAddons?.(addons);
    },
    async callRegisteredHook(name, ...args) {
      for (const handler of hooks.get(name) ?? []) {
        await handler(...args);
      }
    },
  };
}

function importContext(imports: NuxtLintImport[]) {
  return { getImports: async () => imports };
}

void test("auto-import addon emits the complete deterministically ordered globals list", async () => {
  const nuxt = createNuxtStub();
  const resolveAddons = setupNuxtLintConfigAddons(nuxt);

  await nuxt.callRegisteredHook(
    "imports:context",
    importContext([
      { from: "vue", name: "watch", as: "observe" },
      { from: "#app", name: "useRoute" },
      { from: "vue", name: "computed" },
      { from: "#app", name: "useFetch", as: "fetchData" },
    ]),
  );
  await nuxt.callRegisteredHook("nitro:init", {
    unimport: importContext([
      { from: "nitropack/runtime", name: "defineNitroPlugin" },
      { from: "#app", name: "useAsyncData" },
    ]),
  });

  const configs = await resolveAddons();
  assert.equal(configs.length, 1);
  assert.equal(configs[0].name, "nuxt/import-globals");
  assert.deepEqual(Object.entries(configs[0].globals ?? {}), [
    ["useAsyncData", "readonly"],
    ["fetchData", "readonly"],
    ["useRoute", "readonly"],
    ["defineNitroPlugin", "readonly"],
    ["computed", "readonly"],
    ["observe", "readonly"],
  ]);
});

void test("auto-import addon remains valid before Nuxt publishes either registry", async () => {
  const configs = await setupNuxtLintConfigAddons(createNuxtStub())();

  assert.deepEqual(configs, [{ name: "nuxt/import-globals", globals: {} }]);
});

void test("aliases use nullish fallback without losing object-prototype names", async () => {
  const nuxt = createNuxtStub();
  const resolveAddons = setupNuxtLintConfigAddons(nuxt);
  await nuxt.callRegisteredHook(
    "imports:context",
    importContext([
      { from: "source", name: "aProto", as: "__proto__" },
      { from: "source", name: "bConstructor", as: "constructor" },
      { from: "source", name: "cToString", as: "toString" },
      { from: "source", name: "dEmpty", as: "" },
    ]),
  );

  const globals = (await resolveAddons())[0].globals ?? {};
  assert.deepEqual(Object.keys(globals), ["__proto__", "constructor", "toString", ""]);
  for (const hostileName of ["__proto__", "constructor", "toString"]) {
    assert.equal(Object.hasOwn(globals, hostileName), true);
    assert.equal(globals[hostileName], "readonly");
  }
  assert.equal(globals[""], "readonly");
});

void test("addon hook contributes config items in registration order", async () => {
  const seenArrays: NuxtLintConfigAddon[][] = [];
  const nuxt = createNuxtStub(async (addons) => {
    seenArrays.push(addons);
    addons.push(
      {
        name: "first",
        async getConfigs() {
          return [{ name: "first", rules: { "vue/first": "warn" } }];
        },
      },
      {
        name: "empty",
        getConfigs() {
          return undefined;
        },
      },
      {
        name: "second",
        getConfigs() {
          return [
            { name: "second-a", globals: { contributed: "readonly" } },
            { name: "second-b", ignores: ["generated/**"] },
          ];
        },
      },
    );
  });
  const resolveAddons = setupNuxtLintConfigAddons(nuxt);

  const first = await resolveAddons();
  const second = await resolveAddons();

  assert.deepEqual(first.slice(1), [
    { name: "first", rules: { "vue/first": "warn" } },
    { name: "second-a", globals: { contributed: "readonly" } },
    { name: "second-b", ignores: ["generated/**"] },
  ]);
  assert.deepEqual(second, first);
  assert.equal(seenArrays.length, 2);
  assert.notEqual(seenArrays[0], seenArrays[1], "each generation must use a fresh addon array");
  assert.equal(seenArrays[0].length, 4);
  assert.equal(seenArrays[1].length, 4);
});

void test("the most recently published unimport contexts drive regeneration", async () => {
  const nuxt = createNuxtStub();
  const resolveAddons = setupNuxtLintConfigAddons(nuxt);

  await nuxt.callRegisteredHook(
    "imports:context",
    importContext([{ from: "old", name: "oldImport" }]),
  );
  assert.deepEqual((await resolveAddons())[0].globals, { oldImport: "readonly" });

  await nuxt.callRegisteredHook(
    "imports:context",
    importContext([{ from: "new", name: "newImport" }]),
  );
  await nuxt.callRegisteredHook("nitro:init", {
    unimport: importContext([{ from: "nitro", name: "serverImport" }]),
  });
  assert.deepEqual((await resolveAddons())[0].globals, {
    newImport: "readonly",
    serverImport: "readonly",
  });
});

void test("generation writes the recorded initial and regenerated artifacts byte for byte", async (t) => {
  const rootDir = await mkdtemp(path.join(os.tmpdir(), "vize-nuxt-lint-imports-"));
  t.after(() => rm(rootDir, { force: true, recursive: true }));
  const hooks = new Map<string, Hook[]>();
  const nuxt = {
    options: {
      rootDir,
      buildDir: path.join(rootDir, ".nuxt"),
      srcDir: path.join(rootDir, "app"),
      dir: {},
      _layers: [{ config: { srcDir: path.join(rootDir, "app") } }],
    },
    hook(name: string, handler: Hook) {
      hooks.set(name, [...(hooks.get(name) ?? []), handler]);
    },
    async callHook(name: string, ...args: unknown[]) {
      for (const handler of hooks.get(name) ?? []) await handler(...args);
    },
  };
  const corpus = JSON.parse(
    await readFile(
      new URL(
        "../../../nuxt-lint-config/test/nuxt-eslint-compat/fixtures/corpus.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as {
    importGlobals: { nuxt: NuxtLintImport[]; nitro: NuxtLintImport[] };
  };
  const recording = JSON.parse(
    await readFile(
      new URL(
        "../../../nuxt-lint-config/test/nuxt-eslint-compat/fixtures/nuxt-eslint-output.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as {
    typeScriptDetected: boolean;
    importGlobals: { artifacts: { initial: string; regenerated: string } };
  };

  const resolveAddons = setupNuxtLintConfigAddons(nuxt);
  const generation = await setupNuxtLintConfigGeneration({ autoInit: false }, nuxt, {
    resolveAddons,
    hasTypeScript: () => recording.typeScriptDetected,
    resolvePluginSpecifier: () => "../node_modules/oxlint-plugin-vize/dist/index.mjs",
  });
  assert.equal(
    await readFile(generation?.configFile ?? "", "utf8"),
    recording.importGlobals.artifacts.initial,
  );

  await nuxt.callHook("imports:context", importContext(corpus.importGlobals.nuxt));
  await nuxt.callHook("nitro:init", {
    unimport: importContext(corpus.importGlobals.nitro),
  });
  await nuxt.callHook("builder:generateApp");
  assert.equal(
    await readFile(generation?.configFile ?? "", "utf8"),
    recording.importGlobals.artifacts.regenerated,
  );
});

void test("registry and third-party addon failures remain visible", async () => {
  const registryNuxt = createNuxtStub();
  const resolveRegistry = setupNuxtLintConfigAddons(registryNuxt);
  await registryNuxt.callRegisteredHook("imports:context", {
    async getImports() {
      throw new Error("registry failed");
    },
  });
  await assert.rejects(resolveRegistry, /registry failed/);

  const addonNuxt = createNuxtStub((addons) => {
    addons.push({
      name: "broken",
      getConfigs() {
        throw new Error("addon failed");
      },
    });
  });
  await assert.rejects(setupNuxtLintConfigAddons(addonNuxt), /addon failed/);
});
