import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";
import { toVirtualId } from "../virtual.ts";

const testRoot = fs.mkdtempSync(
  path.join(fs.realpathSync(os.tmpdir()), "vize-vite-plugin-peer-runtime-"),
);

function writeFixtureFile(filePath: string, content = ""): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function createTempProject(prefix: string): string {
  const root = fs.mkdtempSync(path.join(testRoot, prefix + "-"));
  writeFixtureFile(
    path.join(root, "package.json"),
    JSON.stringify({ name: "peer-runtime-fixture", private: true }, null, 2),
  );
  return root;
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

const nullResolveContext = {
  resolve: async () => null,
};

{
  const projectRoot = createTempProject("dev-source-vue-peer-runtime");
  const mainImporter = path.join(projectRoot, "src", "main.ts");
  const sfcImporter = path.join(projectRoot, "src", "PageOne.vue");
  const routerPackage = path.join(projectRoot, "node_modules", "vue-router");
  const routerEntry = path.join(routerPackage, "dist", "vue-router.js");

  writeFixtureFile(mainImporter, "import { createRouter } from 'vue-router';");
  writeFixtureFile(
    sfcImporter,
    "<script setup>import { useRouteQuery } from '@vueuse/router'</script>",
  );
  writeFixtureFile(
    path.join(routerPackage, "package.json"),
    JSON.stringify(
      {
        name: "vue-router",
        exports: {
          ".": {
            import: "./dist/vue-router.js",
            require: "./dist/vue-router.js",
          },
          "./package.json": "./package.json",
        },
        main: "dist/vue-router.js",
      },
      null,
      2,
    ),
  );
  writeFixtureFile(routerEntry, "export const createRouter = () => null;");

  const state = createState(projectRoot);

  assert.equal(
    await resolveIdHook(nullResolveContext, state, "vue-router", mainImporter, undefined),
    null,
    "Dev source imports of Vue peer runtimes should stay bare so Vite optimizes vue-router consistently with dependent packages",
  );
  assert.equal(
    await resolveIdHook(
      nullResolveContext,
      state,
      "vue-router",
      toVirtualId(sfcImporter),
      undefined,
    ),
    null,
    "Dev source SFC imports of Vue peer runtimes should not bypass Vite's dependency optimizer",
  );
}
