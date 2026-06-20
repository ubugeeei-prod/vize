import assert from "node:assert/strict";

import type { VizePluginState } from "./state.ts";
import { loadHook } from "./load.ts";

const emptyState: VizePluginState = {
  cache: new Map(),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  isProduction: true,
  root: "/project",
  clientViteBase: "/",
  serverViteBase: "/",
  server: null,
  filter: () => false,
  scanPatterns: null,
  precompileBatchSize: 128,
  ignorePatterns: [],
  mergedOptions: { exclude: [/node_modules/] },
  initialized: true,
  dynamicImportAliasRules: [],
  cssAliasRules: [],
  extractCss: true,
  componentsCssFileName: "assets/vize-components.css",
  clientViteDefine: {},
  serverViteDefine: {},
  logger: {
    log() {},
    info() {},
    warn() {},
    error() {},
  } as never,
};

const dependencyStyleLoad = loadHook(
  emptyState,
  "/project/node_modules/pkg/ProsePre.vue?vue=&type=style&index=0&lang=css.css",
  { ssr: false },
);

assert.equal(
  dependencyStyleLoad,
  null,
  "uncached dependency SFC styles should fall through to Nuxt/Vite style handling",
);

console.log("✅ vite-plugin-vize style load tests passed!");
