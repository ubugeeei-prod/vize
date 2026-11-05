import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import type { VizePluginState } from "./state.ts";
import { transformHook } from "./load.ts";
import { toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";
import { transformCachedSfcModule } from "../test/sfc-module.ts";

const ENV_FLAGS = "/src/EnvFlags.vue";

function stateWithEnvFlags(): VizePluginState {
  return {
    cache: new Map([
      [
        ENV_FLAGS,
        {
          code: `export const flags = [import.meta.client, import.meta.server, import.meta.dev, import.meta.hot];
export default {};`,
          scopeId: "envflags",
          hasScoped: false,
          styles: [],
        },
      ],
    ]),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root: "/src",
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
    clientViteDefine: {
      "import.meta.client": "true",
      "import.meta.server": "false",
      "import.meta.dev": "true",
    },
    serverViteDefine: {},
    logger: { log() {}, info() {}, warn() {}, error() {} } as never,
  } as unknown as VizePluginState;
}

void test("the compile stage, not the pre stage, inlines a virtual SFC module's environment flags", async () => {
  const state = stateWithEnvFlags();
  // An SFC main module is still SFC text at the pre stage.
  assert.equal(
    await transformHook(state, "<template />", toVirtualId(ENV_FLAGS), { ssr: false }),
    null,
  );

  for (const id of [toVirtualId(ENV_FLAGS), toPluginVisibleVirtualId(ENV_FLAGS)]) {
    const compiled = await transformCachedSfcModule(state, id, { ssr: false });
    assert.doesNotMatch(
      compiled.code,
      /import\.meta\.(client|server|dev)/,
      `${id}: flags Vite skips for virtual ids are inlined before post plugins run`,
    );
    assert.match(compiled.code, /import\.meta\.hot/, `${id}: import.meta.hot stays for Vite HMR`);
    assert.equal(compiled.map, null, `${id}: no map is allocated for a module without one`);
  }
});

void test("compiler dependency watches do not become runtime imports", async (context) => {
  const watchedDependency = "/src/external.css";
  const watchedPath = "/src/Watched.vue";
  const watchedProjectRoot = fs.mkdtempSync(path.join(os.tmpdir(), "watch-only-project-"));
  context.after(() => fs.rmSync(watchedProjectRoot, { recursive: true, force: true }));
  const missingProjectConfig = path.join(watchedProjectRoot, "tsconfig.json");
  const watchedDeclaration = path.join(watchedProjectRoot, "Props.ts");
  fs.writeFileSync(watchedDeclaration, "export interface Props { label: string }");
  assert.equal(fs.existsSync(missingProjectConfig), false);
  const compilerDependencies = [watchedDependency, missingProjectConfig, watchedDeclaration];
  const watchedFiles: string[] = [];
  const moduleImports: string[] = [];
  const dependencyWatches: string[] = [];
  const watchedState: VizePluginState = {
    ...stateWithEnvFlags(),
    clientViteDefine: {},
    server: {
      watcher: {
        add(file: string) {
          watchedFiles.push(file);
          dependencyWatches.push(file);
        },
      },
    } as never,
    cache: new Map([
      [
        watchedPath,
        {
          code: `export function render() { return null }
const _sfc_main = {}
_sfc_main.render = render
export default _sfc_main`,
          scopeId: "watch1234",
          hasScoped: false,
          styles: [],
          dependencies: compilerDependencies,
        },
      ],
    ]),
    pendingHmrUpdateTypes: new Map(),
  };
  const watchedLoad = await transformCachedSfcModule(watchedState, toVirtualId(watchedPath), {
    ssr: false,
    addWatchFile: (id) => {
      watchedFiles.push(id);
      moduleImports.push(id);
    },
  });
  assert.ok(watchedLoad && typeof watchedLoad === "object", "Watched SFC should load");
  assert.deepEqual(
    watchedFiles,
    [watchedPath, ...compilerDependencies],
    "Virtual SFC loads should register the real source file and src dependencies with Vite's watcher",
  );
  assert.deepEqual(moduleImports, [watchedPath], "only the real SFC enters Vite's module graph");
  assert.deepEqual(dependencyWatches, compilerDependencies);
  assert.deepEqual(watchedState.cache.get(watchedPath)?.dependencies, compilerDependencies);
  assert.equal(
    fs.existsSync(missingProjectConfig),
    false,
    "watching must preserve absent discovery candidates",
  );
});
