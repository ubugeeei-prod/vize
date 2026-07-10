import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { loadHook } from "./load.ts";
import { shouldLoadCompiledVueSfcPath } from "./load-sfc.ts";
import type { VizePluginState } from "./state.ts";

const testRoot = fs.mkdtempSync(
  path.join(fs.realpathSync(os.tmpdir()), "vize-vite-plugin-dependency-sfc-"),
);

function writeFixtureFile(filePath: string, content: string): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

type StateOptions = {
  filter?: VizePluginState["filter"];
  handleNodeModulesVue?: boolean;
};

function createState(root: string, options: StateOptions = {}): VizePluginState {
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
    filter: options.filter ?? ((id) => !id.includes("node_modules") && id.endsWith(".vue")),
    scanPatterns: [],
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {
      handleNodeModulesVue: options.handleNodeModulesVue ?? false,
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

function captureWarnings(callback: () => void): string[] {
  const warnings: string[] = [];
  const originalWarn = console.warn;
  console.warn = (...args: unknown[]) => warnings.push(args.map(String).join(" "));
  try {
    callback();
  } finally {
    console.warn = originalWarn;
  }
  return warnings;
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
writeFixtureFile(nuxtRootSfc, "<template><div /></template>");

const state = createState(projectRoot);

const warnings = captureWarnings(() => {
  const componentLoad = loadHook(state, nuxtRootSfc, { ssr: false });
  assert.ok(
    componentLoad && typeof componentLoad === "object",
    "Plain dependency Vue SFC loads should compile before raw SFCs reach Vite define transforms",
  );
});

assert.equal(
  state.cache.has(nuxtRootSfc),
  true,
  "Plain dependency Vue SFC loads should on-demand compile into the Vize cache",
);
assert.deepEqual(
  warnings,
  [],
  "Dependency SFC rewrite warnings should be suppressed when node_modules handling is disabled",
);

const pnpmNuxtRootSfc = path.join(
  projectRoot,
  "node_modules",
  ".pnpm",
  "nuxt@3.19.3_vue@3.5.13",
  "node_modules",
  "nuxt",
  "dist",
  "app",
  "components",
  "nuxt-root.vue",
);
writeFixtureFile(pnpmNuxtRootSfc, "<template><span /></template>");

const filteredDependencyState = createState(projectRoot, { handleNodeModulesVue: true });
const filteredDependencyWarnings = captureWarnings(() => {
  const componentLoad = loadHook(filteredDependencyState, pnpmNuxtRootSfc, { ssr: false });
  assert.ok(
    componentLoad && typeof componentLoad === "object",
    "Filtered dependency Vue SFC loads should still compile for the Vite transform pipeline",
  );
});

assert.equal(
  filteredDependencyState.cache.has(pnpmNuxtRootSfc),
  true,
  "Filtered dependency Vue SFC loads should still populate the Vize cache",
);
assert.deepEqual(
  filteredDependencyWarnings,
  [],
  "Dependency SFC rewrite warnings should be suppressed when node_modules is excluded by filter",
);

const includedDependencyState = createState(projectRoot, {
  filter: (id) => id.endsWith(".vue"),
  handleNodeModulesVue: true,
});
const includedDependencyWarnings = captureWarnings(() => {
  const componentLoad = loadHook(includedDependencyState, pnpmNuxtRootSfc, { ssr: false });
  assert.ok(
    componentLoad && typeof componentLoad === "object",
    "Included dependency Vue SFC loads should compile normally",
  );
});

assert.ok(
  includedDependencyWarnings.length > 0,
  "Dependency SFC rewrite warnings should still log when node_modules SFCs are explicitly included",
);

const componentState = createState(projectRoot);
assert.equal(
  loadHook(componentState, `${nuxtRootSfc}?nuxt_component=async&nuxt_component_name=NuxtRoot`, {
    ssr: false,
  }),
  null,
  "Dependency Vue SFC component queries should stay on Nuxt's runtime route when node_modules handling is disabled",
);
assert.equal(
  componentState.cache.has(nuxtRootSfc),
  false,
  "Skipped dependency component queries must not on-demand compile into the Vize cache",
);

// With no ?nuxt_component query (plain build-time import), the file must NOT
// be skipped even when handleNodeModulesVue is false. Without this, Vite
// transform plugins (e.g. vite:define) receive raw .vue source and fail.
assert.equal(
  shouldLoadCompiledVueSfcPath(state, nuxtRootSfc, false),
  true,
  "Plain .vue build-time imports from node_modules must not be skipped when handleNodeModulesVue is false",
);

// Sanity-check: the ?nuxt_component runtime case is still skipped.
assert.equal(
  shouldLoadCompiledVueSfcPath(state, nuxtRootSfc, true),
  false,
  "?nuxt_component runtime loads from node_modules must be skipped when handleNodeModulesVue is false",
);

console.log("vite-plugin-vize dependency SFC load tests passed!");
