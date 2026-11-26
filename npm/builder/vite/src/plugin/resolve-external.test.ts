import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";
import { toVirtualId } from "../virtual.ts";

const testRoot = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), "vize-external-resolve-"));

function writeFixtureFile(filePath: string, content = ""): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

/** A project whose page imports from packages installed in `node_modules`. */
function createProject(prefix: string): { root: string; importer: string } {
  const root = fs.mkdtempSync(path.join(testRoot, `${prefix}-`));
  writeFixtureFile(path.join(root, "package.json"), '{"name":"external-fixture"}');
  const page = path.join(root, "app", "pages", "index.vue");
  writeFixtureFile(page, "<template><div /></template>");
  return { root, importer: toVirtualId(page, true) };
}

/** A package with separate CommonJS and ES module entries, as `vue-router` ships. */
function installDualPackage(root: string, name: string): { cjs: string; esm: string } {
  const packageRoot = path.join(root, "node_modules", name);
  const cjs = path.join(packageRoot, "index.cjs");
  const esm = path.join(packageRoot, "index.mjs");
  writeFixtureFile(
    path.join(packageRoot, "package.json"),
    JSON.stringify({
      name,
      main: "index.cjs",
      exports: { ".": { import: "./index.mjs", require: "./index.cjs" } },
    }),
  );
  writeFixtureFile(cjs, 'module.exports = require("vue");');
  writeFixtureFile(esm, 'export * from "vue";');
  return { cjs, esm };
}

function createBuildState(root: string): VizePluginState {
  return {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: true,
    root,
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
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
  // Nuxt auto-imports `onBeforeRouteLeave` from `vue-router`, and its server
  // build externalizes the package. Bundling the file Node's `require`
  // condition selects instead puts `require("vue")` into an ES module chunk.
  const { root, importer } = createProject("ssr-external");
  installDualPackage(root, "vue-router");

  const resolved = await resolveIdHook(
    { resolve: async (id) => (id === "vue-router" ? { id, external: true } : null) },
    createBuildState(root),
    "vue-router",
    importer,
    { ssr: true },
  );

  assert.deepEqual(
    resolved,
    { id: "vue-router", external: true },
    "a package the server build externalizes stays external for imports from an SFC",
  );
}

{
  // A bare answer that is not external is not loadable, so Node still resolves
  // it from the page that imports it.
  const { root, importer } = createProject("ssr-bare-answer");
  const { cjs } = installDualPackage(root, "bare-answer-fixture");

  const resolved = await resolveIdHook(
    { resolve: async (id) => (id === "bare-answer-fixture" ? { id } : null) },
    createBuildState(root),
    "bare-answer-fixture",
    importer,
    { ssr: true },
  );

  assert.equal(
    resolved,
    cjs,
    "a bare answer without an external decision resolves to a loadable file",
  );
}

{
  // An alias can point a bare request at a path that the host then
  // externalizes under another package name.
  const { root, importer } = createProject("ssr-aliased-external");
  installDualPackage(root, "aliased-target-fixture");
  const state = createBuildState(root);
  state.cssAliasRules = [{ find: "aliased-fixture", replacement: "/virtual/aliased-fixture" }];

  const resolved = await resolveIdHook(
    {
      resolve: async (id) =>
        id === "/virtual/aliased-fixture" ? { id: "aliased-target-fixture", external: true } : null,
    },
    state,
    "aliased-fixture",
    importer,
    { ssr: true },
  );

  assert.deepEqual(
    resolved,
    { id: "aliased-target-fixture", external: true },
    "an aliased request the server build externalizes stays external",
  );
}

fs.rmSync(testRoot, { recursive: true, force: true });
console.log("vite-plugin-vize external resolve tests passed!");
