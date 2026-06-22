import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { loadHook } from "./load.ts";
import type { VizePluginState } from "./state.ts";

const testRoot = fs.mkdtempSync(
  path.join(fs.realpathSync(os.tmpdir()), "vize-vite-plugin-dependency-sfc-"),
);

function writeFixtureFile(filePath: string, content: string): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function createState(root: string): VizePluginState {
  return {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root,
    clientViteBase: "/",
    serverViteBase: "/",
    server: {} as never,
    filter: (id) => !id.includes("node_modules") && id.endsWith(".vue"),
    scanPatterns: [],
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {
      handleNodeModulesVue: false,
      exclude: ["node_modules/**", "**/node_modules/**"],
    },
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
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
}

const projectRoot = fs.mkdtempSync(path.join(testRoot, "nuxt-runtime-"));
const nuxtRootSfc = path.join(
  projectRoot,
  "node_modules",
  "nuxt",
  "dist",
  "app",
  "components",
  "nuxt-root.vue",
);
writeFixtureFile(nuxtRootSfc, "<template><div></template>");

const state = createState(projectRoot);

assert.equal(
  loadHook(state, `${nuxtRootSfc}?nuxt_component=async&nuxt_component_name=NuxtRoot`, {
    ssr: false,
  }),
  null,
  "Dependency Vue SFC component loads should stay on Nuxt's host compiler when node_modules handling is disabled",
);
assert.equal(
  state.cache.has(nuxtRootSfc),
  false,
  "Skipped dependency SFC loads must not on-demand compile into the Vize cache",
);

console.log("vite-plugin-vize dependency SFC load tests passed!");
