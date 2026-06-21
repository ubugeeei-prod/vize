import assert from "node:assert/strict";
import type { HmrContext } from "vite";

import type { VizePluginState } from "./state.ts";
import {
  handleHotUpdateHook,
  handleGenerateBundleHook,
  resolveComponentsCssFileName,
  VIZE_COMPONENTS_CSS_BASENAME,
  VIZE_COMPONENTS_CSS_FILE,
} from "./hmr.ts";
import { compileFile } from "../compiler.ts";
import { toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";

function createState(): VizePluginState {
  return {
    extractCss: true,
    componentsCssFileName: VIZE_COMPONENTS_CSS_FILE,
    collectedCss: new Map([
      ["/src/App.vue", ".app { color: red; }"],
      ["/src/Page.vue", ".page { color: blue; }"],
    ]),
    logger: {
      log() {},
    },
  } as VizePluginState;
}

assert.equal(
  resolveComponentsCssFileName(undefined),
  VIZE_COMPONENTS_CSS_FILE,
  "default Vite builds should keep the existing assets/ output path",
);
assert.equal(
  resolveComponentsCssFileName("_nuxt"),
  `_nuxt/${VIZE_COMPONENTS_CSS_BASENAME}`,
  "Nuxt client builds should emit component CSS under build.assetsDir",
);
assert.equal(
  resolveComponentsCssFileName("."),
  VIZE_COMPONENTS_CSS_BASENAME,
  "assetless builds should place component CSS beside chunks",
);

{
  const vueFile = "/src/App.vue";
  const dependencyFile = "/src/imported.css";
  const module = { url: toVirtualId(vueFile) };
  let invalidatedModule: unknown;
  const state = {
    cache: new Map([
      [
        vueFile,
        {
          code: "export default {}",
          scopeId: "app12345",
          hasScoped: false,
          dependencies: [dependencyFile],
        },
      ],
    ]),
    ssrCache: new Map(),
    collectedCss: new Map([[vueFile, ".app { color: red; }"]]),
    precompileMetadata: new Map([[vueFile, { mtimeMs: 1, size: 1 }]]),
    pendingHmrUpdateTypes: new Map(),
    root: "/src",
    logger: {
      log() {},
    },
  } as unknown as VizePluginState;
  const ctx = {
    file: dependencyFile,
    server: {
      moduleGraph: {
        getModulesByFile(id: string) {
          return id === toVirtualId(vueFile) ? new Set([module]) : undefined;
        },
        invalidateModule(receivedModule: unknown) {
          invalidatedModule = receivedModule;
        },
      },
    },
    read: async () => "",
  } as unknown as HmrContext;

  const modules = await handleHotUpdateHook(state, ctx);

  assert.deepEqual(modules, [module], "SFC src dependency updates should reload the owner module");
  assert.equal(state.cache.has(vueFile), false, "Dependency updates should clear the client cache");
  assert.equal(state.ssrCache.has(vueFile), false, "Dependency updates should clear the SSR cache");
  assert.equal(
    state.collectedCss.has(vueFile),
    false,
    "Dependency updates should clear collected CSS",
  );
  assert.equal(
    state.precompileMetadata.has(vueFile),
    false,
    "Dependency updates should clear precompile metadata",
  );
  assert.equal(
    state.pendingHmrUpdateTypes.get(vueFile),
    "full-reload",
    "Dependency updates should force full reload HMR for the owner SFC",
  );
  assert.equal(invalidatedModule, module, "Dependency updates should invalidate the owner module");
}

{
  const vueFile = "/src/App.vue";
  const dependencyFile = "/src/imported.css";
  const nullVirtualModule = { url: toVirtualId(vueFile) };
  const visibleVirtualModule = { url: toPluginVisibleVirtualId(vueFile) };
  const visibleVirtualFileModule = { url: `${vueFile}.ts?vue&vize` };
  const rawVueModule = { url: vueFile };
  const invalidatedModules: unknown[] = [];
  const state = {
    cache: new Map([
      [
        vueFile,
        {
          code: "export default {}",
          scopeId: "app12345",
          hasScoped: false,
          dependencies: [dependencyFile],
        },
      ],
    ]),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    root: "/src",
    logger: {
      log() {},
    },
  } as unknown as VizePluginState;
  const modulesByFile = new Map<string, Set<unknown>>([
    [toVirtualId(vueFile), new Set([nullVirtualModule])],
    [toPluginVisibleVirtualId(vueFile), new Set([visibleVirtualModule])],
    [`${vueFile}.ts`, new Set([visibleVirtualFileModule])],
    [vueFile, new Set([rawVueModule])],
  ]);
  const ctx = {
    file: dependencyFile,
    server: {
      moduleGraph: {
        getModulesByFile(id: string) {
          return modulesByFile.get(id);
        },
        invalidateModule(receivedModule: unknown) {
          invalidatedModules.push(receivedModule);
        },
      },
    },
    read: async () => "",
  } as unknown as HmrContext;

  const modules = await handleHotUpdateHook(state, ctx);

  assert.deepEqual(
    new Set(modules),
    new Set([nullVirtualModule, visibleVirtualModule, visibleVirtualFileModule, rawVueModule]),
    "Dependency updates should return every module graph representation of the owner SFC",
  );
  assert.deepEqual(
    new Set(invalidatedModules),
    new Set([nullVirtualModule, visibleVirtualModule, visibleVirtualFileModule, rawVueModule]),
    "Dependency updates should invalidate every module graph representation of the owner SFC",
  );
}

{
  const vueFile = "/src/App.vue";
  const previousSource = `<template><h1>You did it!</h1></template>`;
  const nextSource = `<template><h1>You did not do it!</h1></template>`;
  const previousCompiled = compileFile(
    vueFile,
    new Map(),
    { sourceMap: false, ssr: false, vapor: false },
    previousSource,
  );
  const visibleVirtualFileModule = { url: `${vueFile}.ts?vue&vize` };
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
          return id === `${vueFile}.ts` ? new Set([visibleVirtualFileModule]) : undefined;
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
    [visibleVirtualFileModule],
    "Vue SFC edits should return Vite's query-stripped plugin-visible virtual module file",
  );
  assert.equal(
    state.pendingHmrUpdateTypes.get(vueFile),
    "template-only",
    "Template edits should keep granular HMR when the virtual module is found",
  );
}

{
  const vueFile = "/src/Delegated.vue";
  const previousSource = `<template><div :class="$style.root">red</div></template><style module>.root { color: red; }</style>`;
  const nextSource = `<template><div :class="$style.root">red</div></template><style module>.root { color: blue; }</style>`;
  const cache = new Map();
  const previousCompiled = compileFile(
    vueFile,
    cache,
    { sourceMap: false, ssr: false, vapor: false },
    previousSource,
  );
  const styleModule = { url: `${vueFile}?vue&type=style&index=0&lang=css&module=.css` };
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
    logger: {
      log() {},
      error() {},
    },
  } as unknown as VizePluginState;
  const params = new URLSearchParams();
  params.set("vue", "");
  params.set("type", "style");
  params.set("index", "0");
  params.set("lang", "css");
  params.set("module", "");
  const styleId = `${vueFile}?${params.toString()}`;
  const ctx = {
    file: vueFile,
    server: {
      moduleGraph: {
        getModulesByFile(id: string) {
          return id === `${styleId}.css` ? new Set([styleModule]) : undefined;
        },
        invalidateModule() {},
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
    [styleModule],
    "Delegated style-only updates should match Vite's resolved .css style module id",
  );
}

{
  const emitted: Array<{ type: "asset"; fileName: string; source: string }> = [];
  const existingCss = new Set(["assets/app.css"]);
  const dynamicCss = new Set<string>();
  const bundle = {
    "assets/index.js": {
      type: "chunk",
      isEntry: true,
      isDynamicEntry: false,
      viteMetadata: {
        importedCss: existingCss,
      },
    },
    "assets/lazy.js": {
      type: "chunk",
      isEntry: false,
      isDynamicEntry: true,
      viteMetadata: {
        importedCss: dynamicCss,
      },
    },
    "assets/shared.js": {
      type: "chunk",
      isEntry: false,
      isDynamicEntry: false,
    },
    "assets/logo.svg": {
      type: "asset",
    },
  } satisfies Parameters<typeof handleGenerateBundleHook>[2];

  handleGenerateBundleHook(createState(), (file) => emitted.push(file), bundle);

  assert.deepEqual(emitted, [
    {
      type: "asset",
      fileName: VIZE_COMPONENTS_CSS_FILE,
      source: ".app { color: red; }\n\n.page { color: blue; }",
    },
  ]);
  assert.equal(existingCss.has("assets/app.css"), true);
  assert.equal(existingCss.has(VIZE_COMPONENTS_CSS_FILE), true);
  assert.equal(dynamicCss.has(VIZE_COMPONENTS_CSS_FILE), true);
  assert.equal("viteMetadata" in bundle["assets/shared.js"], false);
}

{
  const emitted: Array<{ type: "asset"; fileName: string; source: string }> = [];
  const cssFileName = "_nuxt/vize-components.css";
  const bundle = {
    "_nuxt/index.js": {
      type: "chunk",
      isEntry: true,
      isDynamicEntry: false,
    },
  } satisfies Parameters<typeof handleGenerateBundleHook>[2];
  const state = {
    ...createState(),
    componentsCssFileName: cssFileName,
  };

  handleGenerateBundleHook(state, (file) => emitted.push(file), bundle);

  const entryChunk = bundle["_nuxt/index.js"] as {
    viteMetadata?: { importedCss?: Set<string> };
  };
  assert.equal(emitted[0]?.fileName, cssFileName);
  assert.deepEqual(entryChunk.viteMetadata?.importedCss, new Set([cssFileName]));
}

{
  const emitted: Array<{ type: "asset"; fileName: string; source: string }> = [];
  const bundle = {
    "assets/index.js": {
      type: "chunk",
      isEntry: true,
      isDynamicEntry: false,
    },
  } satisfies Parameters<typeof handleGenerateBundleHook>[2];

  handleGenerateBundleHook(createState(), (file) => emitted.push(file), bundle);

  const entryChunk = bundle["assets/index.js"] as {
    viteMetadata?: { importedCss?: Set<string> };
  };
  assert.equal(emitted.length, 1);
  assert.deepEqual(entryChunk.viteMetadata?.importedCss, new Set([VIZE_COMPONENTS_CSS_FILE]));
}

{
  const emitted: Array<{ type: "asset"; fileName: string; source: string }> = [];
  const bundle = {
    "assets/index.js": {
      type: "chunk",
      isEntry: true,
      isDynamicEntry: false,
    },
  } satisfies Parameters<typeof handleGenerateBundleHook>[2];
  const state = {
    ...createState(),
    extractCss: false,
  };

  handleGenerateBundleHook(state, (file) => emitted.push(file), bundle);

  assert.equal(emitted.length, 0, "SSR builds should not emit or attach extracted CSS");
  assert.equal(
    "viteMetadata" in bundle["assets/index.js"],
    false,
    "SSR chunks should not reference client-only extracted CSS",
  );
}
console.log("✅ vite-plugin-vize plugin hmr tests passed!");
