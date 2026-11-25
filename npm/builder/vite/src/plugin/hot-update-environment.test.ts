import assert from "node:assert/strict";
import type { DevEnvironment, EnvironmentModuleNode, HotUpdateOptions } from "vite";

import { compileFile } from "../compiler.ts";
import { toPluginVisibleVirtualId } from "../virtual.ts";
import { CompiledModuleCache } from "./compiled-module-cache.ts";
import { getVueModuleFileCandidates } from "./hmr-module-graph.ts";
import { handleHotUpdateEnvironmentHook } from "./hot-update-environment.ts";
import type { VizePluginState } from "./state.ts";

const vueFile = "/src/App.vue";
const previousSource = `<template><h1>before</h1></template>`;
const nextSource = `<template><h1>after</h1></template>`;

function createState(): VizePluginState {
  return {
    cache: new Map([
      [
        vueFile,
        compileFile(
          vueFile,
          new Map(),
          { sourceMap: false, ssr: false, vapor: false },
          previousSource,
        ),
      ],
    ]),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    mergedOptions: {},
    cssAliasRules: [],
    clientViteBase: "/",
    root: "/src",
    filter: () => true,
    logger: { log() {}, error() {} },
  } as unknown as VizePluginState;
}

function createOptions(modules: EnvironmentModuleNode[], file = vueFile): HotUpdateOptions {
  return {
    type: "update",
    file,
    timestamp: 1,
    modules,
    read: async () => nextSource,
    server: {} as HotUpdateOptions["server"],
  };
}

{
  const state = createState();
  const dependencyFile = "/src/template.html";
  state.cache.get(vueFile)!.dependencies = [dependencyFile];
  const rawModule = { url: vueFile } as EnvironmentModuleNode;
  const clientModule = {
    id: toPluginVisibleVirtualId(vueFile),
    url: toPluginVisibleVirtualId(vueFile),
  } as EnvironmentModuleNode;

  const modules = await handleHotUpdateEnvironmentHook(
    state,
    createEnvironment("client", clientModule),
    createOptions([rawModule], dependencyFile),
  );

  assert.deepEqual(
    modules,
    [clientModule],
    "client dependency HMR should exclude the raw owner module",
  );
}

{
  const state = createState();
  const dependencyFile = "/src/template.html";
  const compiled = state.cache.get(vueFile)!;
  compiled.dependencies = [dependencyFile];
  const indexedCache = new CompiledModuleCache();
  indexedCache.set(vueFile, compiled);
  state.cache = indexedCache;

  const rawModule = { url: vueFile } as EnvironmentModuleNode;
  const firstClientModule = {
    id: toPluginVisibleVirtualId(vueFile),
    url: toPluginVisibleVirtualId(vueFile),
  } as EnvironmentModuleNode;
  const restoredClientModule = {
    id: toPluginVisibleVirtualId(vueFile),
    url: toPluginVisibleVirtualId(vueFile),
  } as EnvironmentModuleNode;
  let graphModule: EnvironmentModuleNode | undefined = firstClientModule;
  let ensureCount = 0;
  const environment = {
    name: "client",
    moduleGraph: {
      getModuleById(id: string) {
        return id === toPluginVisibleVirtualId(vueFile) ? graphModule : undefined;
      },
      async getModuleByUrl() {
        throw new Error("unresolved speculative candidate");
      },
      getModulesByFile() {},
      async ensureEntryFromUrl(url: string, setIsSelfAccepting: boolean) {
        assert.equal(url, toPluginVisibleVirtualId(vueFile));
        assert.equal(setIsSelfAccepting, false);
        ensureCount += 1;
        graphModule = restoredClientModule;
        return restoredClientModule;
      },
      invalidateModule() {},
    },
    hot: { send() {} },
  } as unknown as DevEnvironment;

  assert.deepEqual(
    await handleHotUpdateEnvironmentHook(
      state,
      environment,
      createOptions([rawModule], dependencyFile),
    ),
    [firstClientModule],
  );

  graphModule = undefined;
  assert.deepEqual(
    await handleHotUpdateEnvironmentHook(
      state,
      environment,
      createOptions([rawModule], dependencyFile),
    ),
    [restoredClientModule],
    "a repeated dependency update should restore the owner removed from Vite's graph",
  );
  assert.equal(ensureCount, 1, "only the missing second owner should be restored");

  environment.moduleGraph.ensureEntryFromUrl = async () => {
    throw new Error("client module restoration failed");
  };
  graphModule = undefined;
  assert.equal(
    await handleHotUpdateEnvironmentHook(
      state,
      environment,
      createOptions([rawModule], dependencyFile),
    ),
    undefined,
    "a failed owner restoration should keep Vite's default HMR handling",
  );
}

{
  const state = createState();
  const dependencyFile = "/src/server-template.html";
  const compiled = state.cache.get(vueFile)!;
  compiled.dependencies = [dependencyFile];
  const ssrCache = new CompiledModuleCache();
  ssrCache.set(vueFile, compiled);
  state.cache.get(vueFile)!.dependencies = [];
  state.ssrCache = ssrCache;
  let ensureCount = 0;
  const environment = createEnvironment("client") as DevEnvironment & {
    moduleGraph: DevEnvironment["moduleGraph"] & {
      ensureEntryFromUrl: DevEnvironment["moduleGraph"]["ensureEntryFromUrl"];
    };
  };
  environment.moduleGraph.ensureEntryFromUrl = async () => {
    ensureCount += 1;
    throw new Error("SSR-only owners must not create a client module");
  };

  assert.deepEqual(
    await handleHotUpdateEnvironmentHook(
      state,
      environment,
      createOptions([{ url: vueFile } as EnvironmentModuleNode], dependencyFile),
    ),
    [],
    "an SSR-only dependency owner should remain absent from the client graph",
  );
  assert.equal(ensureCount, 0);
}

function createEnvironment(name: string, clientModule?: EnvironmentModuleNode): DevEnvironment {
  return {
    name,
    moduleGraph: {
      getModuleById(id: string) {
        return id === toPluginVisibleVirtualId(vueFile) ? clientModule : undefined;
      },
      async getModuleByUrl() {
        throw new Error("unresolved speculative candidate");
      },
      getModulesByFile() {},
      invalidateModule() {},
    },
    hot: { send() {} },
  } as unknown as DevEnvironment;
}

{
  const state = createState();
  const rawModule = { url: vueFile } as EnvironmentModuleNode;
  const clientModule = {
    id: toPluginVisibleVirtualId(vueFile),
    url: toPluginVisibleVirtualId(vueFile),
  } as EnvironmentModuleNode;

  const modules = await handleHotUpdateEnvironmentHook(
    state,
    createEnvironment("client", clientModule),
    createOptions([rawModule]),
  );

  assert.deepEqual(modules, [clientModule], "client HMR should return the real virtual module");
  assert.equal(
    state.pendingHmrUpdateTypes.get(vueFile),
    "template-only",
    "the client hook should classify against the browser cache baseline",
  );

  const noOpModules = await handleHotUpdateEnvironmentHook(
    state,
    createEnvironment("client", clientModule),
    createOptions([rawModule]),
  );
  assert.deepEqual(noOpModules, [], "a repeated source update should be a no-op");
  assert.equal(state.pendingHmrUpdateTypes.has(vueFile), false);
}

{
  const state = createState();
  const previous = state.cache.get(vueFile);
  const rawModule = { url: vueFile } as EnvironmentModuleNode;
  let read = false;
  const options = createOptions([rawModule]);
  options.read = async () => {
    read = true;
    return nextSource;
  };

  const modules = await handleHotUpdateEnvironmentHook(state, createEnvironment("client"), options);

  assert.deepEqual(modules, [], "an unloaded client SFC should not fall back to raw reload");
  assert.equal(state.cache.get(vueFile), previous, "an empty client graph should preserve cache");
  assert.equal(read, false, "an empty client graph should not read or compile the source");
  assert.equal(
    state.pendingHmrUpdateTypes.has(vueFile),
    false,
    "an unloaded client SFC should not retain an HMR classification",
  );

  const clientModule = {
    id: toPluginVisibleVirtualId(vueFile),
    url: toPluginVisibleVirtualId(vueFile),
  } as EnvironmentModuleNode;
  const loadedModules = await handleHotUpdateEnvironmentHook(
    state,
    createEnvironment("client", clientModule),
    createOptions([rawModule]),
  );
  assert.deepEqual(
    loadedModules,
    [clientModule],
    "the loaded client graph should still classify after an empty graph ran first",
  );
  assert.equal(state.pendingHmrUpdateTypes.get(vueFile), "template-only");
}

{
  const state = createState();
  const rawModule = { url: vueFile } as EnvironmentModuleNode;
  const clientModule = {
    id: toPluginVisibleVirtualId(vueFile),
    url: toPluginVisibleVirtualId(vueFile),
  } as EnvironmentModuleNode;
  const options = createOptions([rawModule]);
  options.read = async () => {
    throw new Error("re-compilation failed");
  };

  const modules = await handleHotUpdateEnvironmentHook(
    state,
    createEnvironment("client", clientModule),
    options,
  );

  assert.equal(
    modules,
    undefined,
    "a failed re-compilation should keep Vite's default handling instead of reporting no update",
  );
}

{
  const state = createState();
  const previous = state.cache.get(vueFile);
  const rawModule = { url: vueFile } as EnvironmentModuleNode;
  const originalModules = [rawModule];
  let read = false;
  const options = createOptions(originalModules);
  options.read = async () => {
    read = true;
    return nextSource;
  };

  const modules = await handleHotUpdateEnvironmentHook(state, createEnvironment("ssr"), options);

  assert.equal(modules, originalModules, "non-client environments should preserve their modules");
  assert.equal(state.cache.get(vueFile), previous, "non-client hooks should not compile twice");
  assert.equal(read, false, "non-client hooks should leave source reads to the client hook");
}

// Eviction retains dependency routing until the owner is requested again, so
// each environment must invalidate its own graph regardless of hook order.
for (const order of [["client", "ssr"], ["ssr", "client"], ["ssr"], ["worker"]]) {
  const state = createState();
  const dependencyFile = "/src/types.ts";
  const compiled = { ...state.cache.get(vueFile)!, dependencies: [dependencyFile] };
  state.cache = new CompiledModuleCache();
  state.ssrCache = new CompiledModuleCache();
  if (order.includes("client")) state.cache.set(vueFile, compiled);
  state.ssrCache.set(vueFile, compiled);
  const invalidated: string[] = [];

  for (const name of order) {
    const module = {
      id: toPluginVisibleVirtualId(vueFile),
      url: toPluginVisibleVirtualId(vueFile),
    } as EnvironmentModuleNode;
    const environment = createEnvironment(name, module);
    environment.moduleGraph.invalidateModule = (affected) => {
      assert.equal(affected, module);
      invalidated.push(name);
    };
    environment.moduleGraph.ensureEntryFromUrl = async () => {
      throw new Error("loaded owners must already exist in their own environment");
    };
    assert.deepEqual(
      await handleHotUpdateEnvironmentHook(state, environment, createOptions([], dependencyFile)),
      [module],
      `${name} must invalidate its dependency owner in ${order.join(" then ")} order`,
    );
  }
  assert.deepEqual(invalidated, order);
  assert.equal(state.cache.has(vueFile), false);
  assert.equal(state.ssrCache.has(vueFile), false);
}

// A type edit reaches every environment's hook. Each must find its owners'
// modules through the graph's indexes: resolving a candidate runs every
// plugin's `resolveId`, and an environment without the owners has nothing to do.
{
  const state = createState();
  const dependencyFile = "/src/types.ts";
  const compiled = { ...state.cache.get(vueFile)!, dependencies: [dependencyFile] };
  const owners = Array.from({ length: 1000 }, (_, index) => `/src/Item${index}.vue`);
  state.cache = new CompiledModuleCache();
  state.ssrCache = new CompiledModuleCache();
  for (const owner of owners) {
    state.cache.set(owner, compiled);
    state.ssrCache.set(owner, compiled);
  }

  for (const name of ["client", "ssr", "worker"]) {
    const graph = new Map(
      name === "worker"
        ? []
        : owners.map((owner) => {
            // Requested by its plain URL, as a direct `transformRequest` does.
            const id = toPluginVisibleVirtualId(owner);
            return [id, { id, url: owner } as EnvironmentModuleNode];
          }),
    );
    let lookups = 0;
    const invalidated: EnvironmentModuleNode[] = [];
    const environment = {
      name,
      moduleGraph: {
        getModuleById(id: string) {
          lookups += 1;
          return graph.get(id);
        },
        urlToModuleMap: {
          get(url: string) {
            lookups += 1;
            return graph.get(url);
          },
        },
        getModulesByFile() {
          lookups += 1;
          return undefined;
        },
        async getModuleByUrl() {
          assert.fail(`${name} must not resolve an owner candidate`);
        },
        async ensureEntryFromUrl() {
          assert.fail(`${name} must not create an owner module`);
        },
        invalidateModule(module: EnvironmentModuleNode) {
          invalidated.push(module);
        },
      },
      pluginContainer: {
        async resolveId() {
          assert.fail(`${name} must not run the plugin pipeline`);
        },
      },
      hot: { send() {} },
    } as unknown as DevEnvironment;

    const options = createOptions([], dependencyFile);
    options.read = async () => assert.fail("a dependency update reads no SFC source");
    const modules = await handleHotUpdateEnvironmentHook(state, environment, options);
    assert.deepEqual(modules, [...graph.values()], `${name} returns exactly its owner modules`);
    assert.deepEqual(invalidated, [...graph.values()]);
    // Three index reads per spelling of each owner: O(dependents), no resolution.
    assert.equal(lookups, owners.length * 3 * getVueModuleFileCandidates(vueFile).length);
  }
  for (const owner of owners) {
    assert.equal(state.cache.has(owner), false, "the next batched request recompiles the owner");
    assert.equal(state.ssrCache.has(owner), false);
  }
}
