import assert from "node:assert/strict";
import type { HmrContext } from "vite";

import type { VizePluginState } from "./state.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import { compileFile } from "../compiler.ts";

{
  const vueFile = "/repo/design/components/Link.vue";
  const previousSource = `<template><h1>Before</h1></template>`;
  const nextSource = `<template><h1>After</h1></template>`;
  const previousCompiled = compileFile(
    vueFile,
    new Map(),
    { sourceMap: false, ssr: false, vapor: false },
    previousSource,
  );
  const fsVirtualFileModule = { url: `/@fs${vueFile}.ts?vue&vize` };
  const invalidatedModules: unknown[] = [];
  const state = {
    cache: new Map([[vueFile, previousCompiled]]),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    mergedOptions: {},
    cssAliasRules: [],
    clientViteBase: "/_nuxt/",
    root: "/repo/app",
    filter: () => true,
    logger: {
      log() {},
      error() {},
    },
  } as unknown as VizePluginState;
  const ctx = {
    file: vueFile,
    server: {
      moduleGraph: {
        getModulesByFile(id: string) {
          return id === `/@fs${vueFile}.ts` ? new Set([fsVirtualFileModule]) : undefined;
        },
        invalidateModule(receivedModule: unknown) {
          invalidatedModules.push(receivedModule);
        },
      },
      ws: {
        send() {},
      },
    },
    read: async () => nextSource,
  } as unknown as HmrContext;

  const modules = await handleHotUpdateHook(state, ctx);

  assert.deepEqual(
    modules,
    [fsVirtualFileModule],
    "External design SFC edits should return Vite's /@fs virtual module file",
  );
  assert.equal(
    state.pendingHmrUpdateTypes.get(vueFile),
    "template-only",
    "External design SFC edits should keep granular HMR when the /@fs virtual module is found",
  );
  assert.deepEqual(
    invalidatedModules,
    [fsVirtualFileModule],
    "External design SFC edits should invalidate Vite's stale /@fs transformed module",
  );
}

console.log("vite-plugin-vize external SFC HMR tests passed!");
