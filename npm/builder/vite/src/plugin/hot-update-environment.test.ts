import assert from "node:assert/strict";
import type { DevEnvironment, EnvironmentModuleNode, HotUpdateOptions } from "vite";

import { compileFile } from "../compiler.ts";
import { toPluginVisibleVirtualId } from "../virtual.ts";
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
