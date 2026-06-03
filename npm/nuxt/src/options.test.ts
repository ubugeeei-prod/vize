import assert from "node:assert/strict";

import {
  NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
  resolveNuxtBridgeOptions,
  resolveNuxtCompilerOptions,
  resolveNuxtDevOptions,
  resolveNuxtMuseaOptions,
  resolveNuxtUnoCssOptions,
} from "./options.ts";

assert.deepEqual(
  resolveNuxtCompilerOptions("/repo/app", "/docs/", "_assets", true),
  {
    devUrlBase: "/docs/_assets/",
    exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
    root: "/repo/app",
    scanPatterns: [],
  },
  "compiler true should resolve to Nuxt-aware Vize defaults",
);

assert.equal(
  resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", false),
  false,
  "compiler false should disable the Vize compiler",
);

assert.equal(
  resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", true, {
    supportsViteCompiler: false,
    vueVersion: 2,
  }),
  false,
  "Nuxt 2 projects without a Vite builder should not register Vite compiler plugins",
);

assert.deepEqual(
  resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", true, {
    supportsViteCompiler: true,
    vueVersion: 1,
  }),
  {
    devUrlBase: "/_nuxt/",
    exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
    root: "/repo/app",
    scanPatterns: [],
    vueVersion: 1,
  },
  "Vite-based legacy Vue projects should pass host-compiler compatibility mode to the Vite plugin",
);

assert.deepEqual(
  resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", {
    exclude: [/\.stories\.vue$/],
  }),
  {
    devUrlBase: "/_nuxt/",
    exclude: [NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE, /\.stories\.vue$/],
    root: "/repo/app",
    scanPatterns: [],
  },
  "Nuxt compiler defaults should skip Takumi OG image SFCs while preserving user excludes",
);

assert.deepEqual(
  resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", {
    configMode: "auto",
    customRenderer: true,
    handleNodeModulesVue: false,
    include: [/\.vue$/, /\.ce\.vue$/],
    precompileBatchSize: 32,
    sourceMap: false,
    vueVersion: 3,
  }),
  {
    configMode: "auto",
    customRenderer: true,
    devUrlBase: "/_nuxt/",
    handleNodeModulesVue: false,
    include: [/\.vue$/, /\.ce\.vue$/],
    precompileBatchSize: 32,
    root: "/repo/app",
    scanPatterns: [],
    sourceMap: false,
    vueVersion: 3,
  },
  "compiler object should expose the underlying @vizejs/vite-plugin options",
);

assert.deepEqual(
  resolveNuxtBridgeOptions({ components: false, i18n: false }),
  {
    autoImports: true,
    components: false,
    i18n: false,
    stableInjectedKeys: true,
  },
  "partial bridge options should merge with enabled defaults",
);

assert.deepEqual(
  resolveNuxtBridgeOptions(false),
  {
    autoImports: false,
    components: false,
    i18n: false,
    stableInjectedKeys: false,
  },
  "bridge false should disable every Nuxt transform bridge",
);

assert.deepEqual(
  resolveNuxtUnoCssOptions({ originalSource: { maxBytes: 4096 } }),
  {
    originalSource: { maxBytes: 4096 },
  },
  "UnoCSS original-source bridge should expose its memory limit",
);

assert.deepEqual(
  resolveNuxtUnoCssOptions({ originalSource: false }),
  {
    originalSource: false,
  },
  "UnoCSS bridge should allow disabling original SFC reads while keeping id normalization",
);

assert.deepEqual(
  resolveNuxtDevOptions({ stylesheetLinks: false }),
  {
    stylesheetLinks: false,
  },
  "dev stylesheet cleanup should be configurable",
);

assert.equal(
  resolveNuxtMuseaOptions(undefined),
  false,
  "Musea should be opt-in so normal Nuxt builds do not include the gallery plugin",
);

assert.deepEqual(
  resolveNuxtMuseaOptions(true),
  {},
  "musea true should enable the gallery with plugin defaults",
);

assert.deepEqual(
  resolveNuxtMuseaOptions({ include: ["**/*.art.vue"], inlineArt: false }),
  { include: ["**/*.art.vue"], inlineArt: false },
  "musea object should pass through explicit gallery options",
);

console.log("✅ nuxt option normalization tests passed!");
