import assert from "node:assert/strict";
import type { HmrContext } from "vite";

import type { VizePluginState } from "./state.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import { compileFile } from "../compiler.ts";
import { toVirtualId } from "../virtual.ts";

// #7742: one save that changes both the template and a style reports a
// non-style update type, but the style module must still be invalidated, or
// the dev server keeps serving the previous CSS until it restarts.
{
  const vueFile = "/src/TemplateAndStyle.vue";
  const previousSource = `<template><p class="box">hello</p></template><style scoped>.box { color: red; }</style>`;
  const nextSource = `<template><p class="box big">hello</p></template><style scoped>.box { color: blue; }</style>`;
  const previousCompiled = compileFile(
    vueFile,
    new Map(),
    { root: "/src", sourceMap: false, ssr: false, vapor: false },
    previousSource,
  );
  const params = new URLSearchParams();
  params.set("vue", "");
  params.set("type", "style");
  params.set("index", "0");
  params.set("scoped", `data-v-${previousCompiled.scopeId}`);
  params.set("lang", "css");
  const styleId = `${vueFile}?${params.toString()}.css`;
  const styleModule = { url: styleId };
  const componentModule = { url: toVirtualId(vueFile) };
  const invalidated: unknown[] = [];
  const state = {
    cache: new Map([[vueFile, previousCompiled]]),
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
  const ctx = {
    file: vueFile,
    server: {
      moduleGraph: {
        getModulesByFile(id: string) {
          if (id === styleId) return new Set([styleModule]);
          if (id === toVirtualId(vueFile)) return new Set([componentModule]);
          return undefined;
        },
        invalidateModule(module: unknown) {
          invalidated.push(module);
        },
      },
      ws: { send() {} },
    },
    read: async () => nextSource,
  } as unknown as HmrContext;

  const modules = await handleHotUpdateHook(state, ctx);
  assert.ok(
    modules?.includes(styleModule as never),
    "A template + style save must return the style module so Vite refreshes the CSS",
  );
  assert.ok(
    invalidated.includes(styleModule),
    "A template + style save must invalidate the style module, not only the component",
  );
  assert.ok(invalidated.includes(componentModule), "The component module is still invalidated");
}

{
  const vueFile = "/src/TemplateOnly.vue";
  const previousSource = `<template><p class="box">hello</p></template><style scoped>.box { color: red; }</style>`;
  const nextSource = `<template><p class="box big">hello</p></template><style scoped>.box { color: red; }</style>`;
  const previousCompiled = compileFile(
    vueFile,
    new Map(),
    { root: "/src", sourceMap: false, ssr: false, vapor: false },
    previousSource,
  );
  const componentModule = { url: toVirtualId(vueFile) };
  const invalidated: unknown[] = [];
  const state = {
    cache: new Map([[vueFile, previousCompiled]]),
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
  const ctx = {
    file: vueFile,
    server: {
      moduleGraph: {
        getModulesByFile(id: string) {
          if (id === toVirtualId(vueFile)) return new Set([componentModule]);
          return id.includes("type=style") ? new Set([{ url: id }]) : undefined;
        },
        invalidateModule(module: unknown) {
          invalidated.push(module);
        },
      },
      ws: { send() {} },
    },
    read: async () => nextSource,
  } as unknown as HmrContext;

  assert.deepEqual(
    await handleHotUpdateHook(state, ctx),
    [componentModule],
    "A template-only save leaves unchanged style modules alone",
  );
  assert.deepEqual(invalidated, [componentModule]);
}

console.log("✅ vite-plugin-vize template + style HMR tests passed!");
