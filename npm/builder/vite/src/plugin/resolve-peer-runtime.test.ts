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

function expectResolvedId(resolved: Awaited<ReturnType<typeof resolveIdHook>>): string {
  assert.ok(resolved);
  return typeof resolved === "string" ? resolved : resolved.id;
}

{
  const projectRoot = createTempProject("regular-vue-peer-nuxt-runtime");

  const appImporter = path.join(projectRoot, "composables", "content-render.ts");
  const nuxtVirtualImporter = `/@id/virtual:nuxt:${encodeURIComponent(
    path.join(projectRoot, ".nuxt", "pages.mjs"),
  )}`;
  writeFixtureFile(appImporter, "import { RouterLink } from 'vue-router';");

  const nuxtPackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "nuxt",
  );
  const nuxtPackageLink = path.join(projectRoot, "node_modules", "nuxt");
  const nuxtRouterPackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue-router@4.5.1-nuxt",
    "node_modules",
    "vue-router",
  );
  const projectHoistedRouterPackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue-router@4.6.4-hoisted",
    "node_modules",
    "vue-router",
  );
  const projectHoistedRouterLink = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "node_modules",
    "vue-router",
  );
  const nuxtRouterLink = path.join(nuxtPackage, "node_modules", "vue-router");
  const nuxtRouterEntry = path.join(nuxtRouterPackage, "dist", "vue-router.mjs");
  const projectHoistedRouterEntry = path.join(
    projectHoistedRouterPackage,
    "dist",
    "vue-router.mjs",
  );

  writeFixtureFile(
    path.join(nuxtPackage, "package.json"),
    JSON.stringify({ name: "nuxt", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(nuxtPackage, "index.js"), "module.exports = {};");
  writeFixtureFile(
    path.join(nuxtRouterPackage, "package.json"),
    JSON.stringify({ name: "vue-router", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(nuxtRouterPackage, "index.js"), "module.exports = {};");
  writeFixtureFile(nuxtRouterEntry, "export const RouterView = {};");
  writeFixtureFile(
    path.join(projectHoistedRouterPackage, "package.json"),
    JSON.stringify({ name: "vue-router", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(projectHoistedRouterPackage, "index.js"), "module.exports = {};");
  writeFixtureFile(projectHoistedRouterEntry, "export const RouterView = {};");
  fs.mkdirSync(path.dirname(nuxtPackageLink), { recursive: true });
  fs.mkdirSync(path.dirname(nuxtRouterLink), { recursive: true });
  fs.mkdirSync(path.dirname(projectHoistedRouterLink), { recursive: true });
  fs.symlinkSync(nuxtPackage, nuxtPackageLink, "dir");
  fs.symlinkSync(nuxtRouterPackage, nuxtRouterLink, "dir");
  fs.symlinkSync(projectHoistedRouterPackage, projectHoistedRouterLink, "dir");

  for (const importer of [appImporter, nuxtVirtualImporter]) {
    const resolved = await resolveIdHook(
      nullResolveContext,
      createState(projectRoot),
      "vue-router",
      importer,
      undefined,
    );

    assert.equal(
      expectResolvedId(resolved),
      nuxtRouterEntry,
      "Nuxt project Vue Router imports should share Nuxt's runtime peer package instead of a project hoist or parent workspace runtime",
    );
  }

  const appRouterLink = path.join(projectRoot, "node_modules", "vue-router");
  fs.symlinkSync(projectHoistedRouterPackage, appRouterLink, "dir");
  for (const importer of [
    appImporter,
    nuxtVirtualImporter,
    path.join(nuxtPackage, "dist", "pages", "runtime", "page.js"),
  ]) {
    const resolved = await resolveIdHook(
      nullResolveContext,
      createState(projectRoot),
      "vue-router",
      importer,
      undefined,
    );
    assert.equal(
      expectResolvedId(resolved),
      projectHoistedRouterEntry,
      "An app-owned vue-router dependency must be shared by Nuxt and generated pages imports",
    );
  }
}

{
  const workspaceRoot = createTempProject("workspace-nuxt-peer-runtime");
  const projectRoot = path.join(workspaceRoot, "packages", "app");
  const appImporter = path.join(projectRoot, "src", "app.ts");
  const workspaceImporter = path.join(workspaceRoot, "packages", "ui", "src", "Widget.vue");
  const nuxtPackage = path.join(
    workspaceRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.5.2",
    "node_modules",
    "nuxt",
  );
  const nuxtRouterPackage = path.join(
    workspaceRoot,
    "node_modules",
    ".pnpm",
    "vue-router@5.2.0_nuxt",
    "node_modules",
    "vue-router",
  );
  const workspaceRouterPackage = path.join(
    workspaceRoot,
    "node_modules",
    ".pnpm",
    "vue-router@5.2.0_workspace",
    "node_modules",
    "vue-router",
  );
  const workspaceRouterEntry = path.join(workspaceRouterPackage, "dist", "vue-router.mjs");

  writeFixtureFile(path.join(projectRoot, "package.json"), '{"name":"workspace-app"}');
  writeFixtureFile(appImporter, "import { useRoute } from 'vue-router';");
  writeFixtureFile(workspaceImporter, "<script setup>import { useRoute } from 'vue-router'</script>");
  for (const packageRoot of [nuxtPackage, nuxtRouterPackage, workspaceRouterPackage]) {
    writeFixtureFile(
      path.join(packageRoot, "package.json"),
      JSON.stringify({ name: path.basename(packageRoot), main: "index.js" }),
    );
    writeFixtureFile(path.join(packageRoot, "index.js"), "module.exports = {};");
  }
  writeFixtureFile(path.join(nuxtPackage, "dist", "app", "nuxt.js"), "");
  writeFixtureFile(path.join(nuxtRouterPackage, "dist", "vue-router.mjs"), "");
  writeFixtureFile(workspaceRouterEntry, "");
  fs.mkdirSync(path.join(projectRoot, "node_modules"), { recursive: true });
  fs.mkdirSync(path.join(nuxtPackage, "node_modules"), { recursive: true });
  fs.symlinkSync(nuxtPackage, path.join(projectRoot, "node_modules", "nuxt"), "dir");
  fs.symlinkSync(
    nuxtRouterPackage,
    path.join(nuxtPackage, "node_modules", "vue-router"),
    "dir",
  );
  fs.symlinkSync(
    workspaceRouterPackage,
    path.join(workspaceRoot, "node_modules", "vue-router"),
    "dir",
  );

  for (const importer of [
    appImporter,
    workspaceImporter,
    path.join(nuxtPackage, "dist", "app", "nuxt.js"),
  ]) {
    const state = createState(projectRoot);
    state.server = null;
    const resolved = await resolveIdHook(nullResolveContext, state, "vue-router", importer, undefined);
    assert.equal(
      expectResolvedId(resolved),
      workspaceRouterEntry,
      "A Nuxt runtime outside the app root must use the workspace's vue-router in a pnpm workspace",
    );
  }
}
