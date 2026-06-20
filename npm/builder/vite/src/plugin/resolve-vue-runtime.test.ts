import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";

const testRoot = fs.mkdtempSync(
  path.join(fs.realpathSync(os.tmpdir()), "vize-vue-runtime-resolve-"),
);

function writeFixtureFile(filePath: string, content = ""): void {
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
    filter: () => true,
    scanPatterns: ["**/*.vue"],
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {},
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

{
  const projectRoot = fs.mkdtempSync(path.join(testRoot, "vuetify-runtime-"));
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");

  const importer = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vuetify@3.11.0_vue@3.6.0",
    "node_modules",
    "vuetify",
    "lib",
    "components",
    "VCard",
    "VCard.mjs",
  );
  writeFixtureFile(importer, "import { withDirectives } from 'vue';");

  const vuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0",
    "node_modules",
    "vue",
  );
  const projectVueHoist = path.join(projectRoot, "node_modules", ".pnpm", "node_modules", "vue");
  const vueBundlerEntry = path.join(vuePackage, "dist", "vue.runtime.esm-bundler.js");
  writeFixtureFile(
    path.join(vuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(vuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(vueBundlerEntry, "export const withDirectives = () => null;");
  fs.mkdirSync(path.dirname(projectVueHoist), { recursive: true });
  fs.symlinkSync(vuePackage, projectVueHoist, "dir");

  const optimizedVueEntry = path.join(projectRoot, "node_modules", ".vite", "deps", "vue.js");
  writeFixtureFile(optimizedVueEntry, "export const withDirectives = () => null;");

  const resolved = await resolveIdHook(
    {
      resolve: async (id) => (id === "vue" ? { id: `${optimizedVueEntry}?v=abc123` } : null),
    },
    createState(projectRoot),
    "vue",
    importer,
    undefined,
  );

  assert.equal(
    resolved,
    null,
    "Dev Vue imports from dependencies should stay with Vite's optimized runtime to avoid duplicate Vue instances",
  );
}
