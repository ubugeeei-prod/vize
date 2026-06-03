import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";
import { toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const workspaceRoot = path.resolve(__dirname, "../../../..");
const testRoot = fs.mkdtempSync(
  path.join(fs.realpathSync(os.tmpdir()), "vize-vite-plugin-resolve-"),
);

function writeFixtureFile(filePath: string, content = ""): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function createTempRoot(prefix: string): string {
  return fs.mkdtempSync(path.join(testRoot, prefix + "-"));
}

function createTempProject(prefix: string): string {
  const root = createTempRoot(prefix);
  writeFixtureFile(
    path.join(root, "package.json"),
    JSON.stringify({ name: "resolve-fixture", private: true }, null, 2),
  );
  writeFixtureFile(path.join(root, "app", "pages", "index.vue"), "<template />\n");
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

function hasFixtureProject(projectRoot: string): boolean {
  return fs.existsSync(path.join(projectRoot, "package.json"));
}

function canResolveFixtureDependency(projectRoot: string, specifier: string): boolean {
  if (!hasFixtureProject(projectRoot)) {
    return false;
  }

  try {
    createRequire(path.join(projectRoot, "package.json")).resolve(specifier);
    return true;
  } catch {
    return false;
  }
}

function expectResolvedId(resolved: Awaited<ReturnType<typeof resolveIdHook>>): string {
  assert.notEqual(resolved, null);
  assert.notEqual(resolved, undefined);

  if (typeof resolved === "string") {
    return resolved;
  }

  assert.equal(typeof resolved, "object");
  assert.equal(typeof resolved.id, "string");
  return resolved.id;
}

{
  const tempRoot = createTempRoot("style-query");
  const source = path.join(tempRoot, "Styled.vue");
  fs.writeFileSync(source, "<template /><style scoped>.root { color: red; }</style>");
  const id = `${source}?vue=&type=style&index=0&scoped=data-v-resolve&lang=css`;

  const state = createState(tempRoot);
  state.server = null;
  const resolved = await resolveIdHook(nullResolveContext, state, id, undefined, undefined);

  assert.equal(
    expectResolvedId(resolved),
    `${id}.css`,
    "Vue style queries should resolve to CSS-visible virtual style IDs in build mode",
  );
}

{
  const tempRoot = createTempRoot("define-page");
  const source = path.join(tempRoot, "Home.vue");
  fs.writeFileSync(source, "<script setup>definePage({})</script>");

  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(tempRoot),
    `${source}?definePage`,
    undefined,
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    `\0${source}?definePage`,
    "Vue Router definePage queries should resolve to a virtual macro module",
  );
}

{
  const projectRoot = createTempProject("regular-dependency");
  let resolverCalled = false;
  const resolved = await resolveIdHook(
    {
      resolve: async () => {
        resolverCalled = true;
        return null;
      },
    },
    createState(projectRoot),
    "@sqlite.org/sqlite-wasm/sqlite-wasm/jswasm/sqlite3-bundler-friendly.mjs",
    path.join(projectRoot, "app", "content-client.ts"),
    undefined,
  );

  assert.equal(
    resolved,
    null,
    "Regular dependency imports outside Vize virtual modules should bypass Vize resolution",
  );
  assert.equal(
    resolverCalled,
    false,
    "Bypassed dependency imports should not enter Vite fallback resolution through Vize",
  );
}

{
  const parentRoot = createTempRoot("regular-vue-project-pnpm-hoist");
  const projectRoot = path.join(parentRoot, "fixture-app");
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");
  const importer = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "nuxt",
    "dist",
    "app",
    "entry.js",
  );
  writeFixtureFile(importer, "import { createApp } from 'vue';");

  const parentVue = path.join(
    parentRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0-parent",
    "node_modules",
    "vue",
    "dist",
    "vue.runtime.esm-bundler.js",
  );
  const projectVuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0-project",
    "node_modules",
    "vue",
  );
  const projectVueHoist = path.join(projectRoot, "node_modules", ".pnpm", "node_modules", "vue");
  const projectVueBundlerEntry = path.join(projectVuePackage, "dist", "vue.runtime.esm-bundler.js");

  writeFixtureFile(parentVue, "export const parent = true;");
  writeFixtureFile(
    path.join(projectVuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(projectVuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(projectVueBundlerEntry, "export const project = true;");
  fs.mkdirSync(path.dirname(projectVueHoist), { recursive: true });
  fs.symlinkSync(projectVuePackage, projectVueHoist, "dir");

  const resolved = await resolveIdHook(
    {
      resolve: async () => ({ id: parentVue }),
    },
    createState(projectRoot),
    "vue",
    importer,
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    projectVueBundlerEntry,
    "Vue runtime imports from project-local runtime modules should prefer the project pnpm hoist over a parent workspace runtime",
  );
}

{
  const parentRoot = createTempRoot("build-vue-project-pnpm-hoist");
  const projectRoot = path.join(parentRoot, "fixture-app");
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");
  const importer = path.join(projectRoot, "app", "components", "Header.vue");
  writeFixtureFile(importer, "<template />\n");

  const parentVuePackage = path.join(parentRoot, "node_modules", "vue");
  const parentVueEntry = path.join(parentVuePackage, "dist", "vue.runtime.esm-bundler.js");
  const projectVuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0-project",
    "node_modules",
    "vue",
  );
  const projectVueHoist = path.join(projectRoot, "node_modules", ".pnpm", "node_modules", "vue");
  const projectVueBundlerEntry = path.join(projectVuePackage, "dist", "vue.runtime.esm-bundler.js");

  writeFixtureFile(
    path.join(parentVuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(parentVuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(parentVueEntry, "export const parent = true;");
  writeFixtureFile(
    path.join(projectVuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(projectVuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(projectVueBundlerEntry, "export const project = true;");
  fs.mkdirSync(path.dirname(projectVueHoist), { recursive: true });
  fs.symlinkSync(projectVuePackage, projectVueHoist, "dir");

  const state = createState(projectRoot);
  state.server = null;
  const resolved = await resolveIdHook(
    {
      resolve: async () => ({ id: parentVueEntry }),
    },
    state,
    "vue",
    toVirtualId(importer),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    projectVueBundlerEntry,
    "Build Vue imports from virtual SFC modules should prefer project-local pnpm Vue over parent workspace resolution",
  );
}

{
  const parentRoot = createTempRoot("build-vue-peer-dependency-pnpm-hoist");
  const projectRoot = path.join(parentRoot, "fixture-app");
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");
  const importer = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "motion-v@2.2.1_vue@3.6.0-project",
    "node_modules",
    "motion-v",
    "dist",
    "index.mjs",
  );
  writeFixtureFile(importer, "import { ref } from 'vue';");

  const parentVuePackage = path.join(parentRoot, "node_modules", "vue");
  const parentVueEntry = path.join(parentVuePackage, "dist", "vue.runtime.esm-bundler.js");
  const projectVuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0-project",
    "node_modules",
    "vue",
  );
  const dependencyVueLink = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "motion-v@2.2.1_vue@3.6.0-project",
    "node_modules",
    "vue",
  );
  const projectVueBundlerEntry = path.join(projectVuePackage, "dist", "vue.runtime.esm-bundler.js");

  writeFixtureFile(
    path.join(parentVuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(parentVuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(parentVueEntry, "export const parent = true;");
  writeFixtureFile(
    path.join(projectVuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(projectVuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(projectVueBundlerEntry, "export const project = true;");
  fs.mkdirSync(path.dirname(dependencyVueLink), { recursive: true });
  fs.symlinkSync(projectVuePackage, dependencyVueLink, "dir");

  const state = createState(projectRoot);
  state.server = null;
  const resolved = await resolveIdHook(
    {
      resolve: async () => ({ id: parentVueEntry }),
    },
    state,
    "vue",
    importer,
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    projectVueBundlerEntry,
    "Build Vue imports from project-local peer dependencies should share the project Vue runtime",
  );
}

{
  const parentRoot = createTempRoot("regular-vue-peer-project-runtime");
  const projectRoot = path.join(parentRoot, "fixture-app");
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");
  const importer = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "nuxt",
    "dist",
    "pages",
    "runtime",
    "page.js",
  );
  writeFixtureFile(importer, "import { RouterView } from 'vue-router';");

  const parentRouter = path.join(
    parentRoot,
    "node_modules",
    ".pnpm",
    "vue-router@4.5.1-parent",
    "node_modules",
    "vue-router",
    "dist",
    "vue-router.mjs",
  );
  const projectRouterPackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue-router@5.0.3-project",
    "node_modules",
    "vue-router",
  );
  const projectRouterLink = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "vue-router",
  );
  const projectRouterEntry = path.join(projectRouterPackage, "dist", "vue-router.js");

  writeFixtureFile(parentRouter, "export const parent = true;");
  writeFixtureFile(
    path.join(projectRouterPackage, "package.json"),
    JSON.stringify(
      {
        name: "vue-router",
        exports: {
          ".": {
            import: "./dist/vue-router.js",
            require: "./index.cjs",
          },
          "./package.json": "./package.json",
        },
        main: "index.cjs",
      },
      null,
      2,
    ),
  );
  writeFixtureFile(path.join(projectRouterPackage, "index.cjs"), "module.exports = {};");
  writeFixtureFile(projectRouterEntry, "export const RouterView = {};");
  fs.mkdirSync(path.dirname(projectRouterLink), { recursive: true });
  fs.symlinkSync(projectRouterPackage, projectRouterLink, "dir");

  const resolved = await resolveIdHook(
    {
      resolve: async () => ({ id: parentRouter }),
    },
    createState(projectRoot),
    "vue-router",
    importer,
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    projectRouterEntry,
    "Vue peer runtime imports from project-local runtime modules should prefer the importer-local package over a parent workspace runtime",
  );
}

{
  const parentRoot = createTempRoot("regular-vue-peer-nuxt-runtime");
  const projectRoot = path.join(parentRoot, "fixture-app");
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");

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
}

{
  const projectRoot = createTempProject("regular-vue-outside-project");
  let resolverCalled = false;
  const resolved = await resolveIdHook(
    {
      resolve: async () => {
        resolverCalled = true;
        return null;
      },
    },
    createState(projectRoot),
    "vue",
    path.join(path.dirname(projectRoot), "outside.js"),
    undefined,
  );

  assert.equal(resolved, null, "Vue imports outside the project should stay owned by Vite");
  assert.equal(
    resolverCalled,
    false,
    "Outside-project Vue imports should not enter Vite fallback resolution through Vize",
  );
}

{
  const tempRoot = createTempRoot("js-macro");
  const importer = path.join(tempRoot, "App.vue");
  const stub = path.join(tempRoot, "component-stub.js");
  fs.writeFileSync(importer, "<template><div /></template>");
  fs.writeFileSync(stub, "export default {};");

  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(tempRoot),
    "./component-stub.js?macro=true",
    toVirtualId(importer),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    `${stub}?macro=true`,
    "non-Vue macro imports should stay regular JavaScript modules",
  );
}

{
  const tempRoot = createTempRoot("ssr-vue-runtime");
  const importer = path.join(tempRoot, "App.vue");
  fs.writeFileSync(importer, "<template><div /></template>");
  const state = createState(tempRoot);
  state.server = null;

  assert.deepEqual(
    await resolveIdHook(
      nullResolveContext,
      state,
      "@vue/server-renderer",
      toVirtualId(importer, true),
      { ssr: true },
    ),
    { id: "vue/server-renderer", external: true },
    "SSR virtual modules should externalize Vue's public server renderer entry",
  );

  assert.deepEqual(
    await resolveIdHook(
      nullResolveContext,
      state,
      "vue/dist/vue.esm-bundler.js/server-renderer",
      toVirtualId(importer, true),
      { ssr: true },
    ),
    { id: "vue/server-renderer", external: true },
    "SSR virtual modules should externalize Vue server-renderer suffixes to the public server renderer entry",
  );

  assert.deepEqual(
    await resolveIdHook(
      nullResolveContext,
      state,
      "vue/dist/vue.esm-bundler.js",
      toVirtualId(importer, true),
      { ssr: true },
    ),
    { id: "vue", external: true },
    "SSR virtual modules should not bundle Vue runtime aliases into Nuxt server output",
  );
}

{
  const projectRoot = createTempProject("dev-ssr-vue-pnpm-isolated");
  const nuxtImporter = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "nuxt",
    "dist",
    "app",
    "components",
    "nuxt-root.vue",
  );
  const vuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0-beta.1_x",
    "node_modules",
    "vue",
  );
  const rendererPackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "@vue+server-renderer@3.6.0-beta.1_x",
    "node_modules",
    "@vue",
    "server-renderer",
  );
  const vueLink = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "vue",
  );
  const rendererLink = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "@vue",
    "server-renderer",
  );
  const vueBundlerEntry = path.join(vuePackage, "dist", "vue.runtime.esm-bundler.js");
  const rendererBundlerEntry = path.join(rendererPackage, "dist", "server-renderer.esm-bundler.js");

  writeFixtureFile(nuxtImporter, "<template><Suspense /></template>");
  writeFixtureFile(
    path.join(vuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(vuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(vueBundlerEntry, "export const Suspense = Symbol();");
  writeFixtureFile(
    path.join(rendererPackage, "package.json"),
    JSON.stringify({ name: "@vue/server-renderer", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(rendererPackage, "index.js"), "module.exports = {};");
  writeFixtureFile(rendererBundlerEntry, "export const ssrRenderSuspense = () => null;");
  fs.mkdirSync(path.dirname(vueLink), { recursive: true });
  fs.mkdirSync(path.dirname(rendererLink), { recursive: true });
  fs.symlinkSync(vuePackage, vueLink, "dir");
  fs.symlinkSync(rendererPackage, rendererLink, "dir");

  const state = createState(projectRoot);
  const importer = toVirtualId(nuxtImporter, true);

  assert.equal(
    expectResolvedId(
      await resolveIdHook(nullResolveContext, state, "vue", importer, { ssr: true }),
    ),
    vueBundlerEntry,
    "Dev SSR virtual modules should resolve Vue from the importer-local package instead of externalizing a bare import",
  );

  assert.equal(
    expectResolvedId(
      await resolveIdHook(nullResolveContext, state, "vue/server-renderer", importer, {
        ssr: true,
      }),
    ),
    rendererBundlerEntry,
    "Dev SSR virtual modules should resolve public Vue server-renderer imports to the renderer ESM bundler entry",
  );

  assert.equal(
    expectResolvedId(
      await resolveIdHook(
        nullResolveContext,
        state,
        "vue/dist/vue.esm-bundler.js/server-renderer",
        importer,
        { ssr: true },
      ),
    ),
    rendererBundlerEntry,
    "Dev SSR virtual modules should resolve Vue server-renderer suffixes to the renderer ESM bundler entry",
  );
}

{
  const tempRoot = createTempRoot("alias-vue");
  const importer = path.join(tempRoot, "src", "App.vue");
  const aliased = path.join(tempRoot, "src", "views", "Aliased.vue");
  fs.mkdirSync(path.dirname(importer), { recursive: true });
  fs.mkdirSync(path.dirname(aliased), { recursive: true });
  fs.writeFileSync(importer, "<template><Aliased /></template>");
  fs.writeFileSync(aliased, "<template><div /></template>");

  const state = createState(tempRoot);
  state.filter = (id) => id === aliased;

  let resolverImporter: string | undefined;
  const resolved = await resolveIdHook(
    {
      resolve: async (id, importer) => {
        resolverImporter = importer;
        return id === "@views/Aliased.vue" ? { id: `/@fs${aliased}` } : null;
      },
    },
    state,
    "@views/Aliased.vue",
    toVirtualId(importer),
    undefined,
  );

  assert.equal(
    resolverImporter,
    importer,
    "Vite alias resolution should receive the real importer path",
  );
  assert.equal(
    expectResolvedId(resolved),
    toPluginVisibleVirtualId(aliased),
    "Aliased Vue imports should be filtered after Vite resolves the real file path",
  );
}

{
  const tempRoot = createTempRoot("bare-alias");
  const viteRoot = path.join(tempRoot, "app");
  const importer = path.join(tempRoot, "app", "pages", "index.vue");
  const packageRoot = path.join(
    tempRoot,
    "node_modules",
    ".pnpm",
    "vue-i18n@0.0.0",
    "node_modules",
    "vue-i18n",
  );
  const pnpmHoistRoot = path.join(tempRoot, "node_modules", ".pnpm", "node_modules");
  const entry = path.join(packageRoot, "dist", "vue-i18n.esm-bundler.js");

  fs.mkdirSync(path.dirname(importer), { recursive: true });
  fs.mkdirSync(path.dirname(entry), { recursive: true });
  fs.mkdirSync(pnpmHoistRoot, { recursive: true });
  fs.writeFileSync(importer, "<template><div /></template>");
  fs.writeFileSync(path.join(packageRoot, "package.json"), '{"name":"vue-i18n","version":"0.0.0"}');
  fs.writeFileSync(entry, "export const I18nInjectionKey = Symbol();");
  fs.symlinkSync(packageRoot, path.join(pnpmHoistRoot, "vue-i18n"), "dir");

  const state = createState(viteRoot);
  state.server = null;
  state.cssAliasRules = [
    {
      find: "vue-i18n",
      replacement: "vue-i18n/dist/vue-i18n.esm-bundler.js",
    },
  ];

  let resolverCalled = false;
  const resolved = await resolveIdHook(
    {
      resolve: async () => {
        resolverCalled = true;
        return null;
      },
    },
    state,
    "vue-i18n",
    toVirtualId(importer),
    undefined,
  );

  assert.equal(
    resolverCalled,
    false,
    "Bare package aliases should avoid Vite alias recursion when Node can resolve them",
  );
  assert.equal(
    expectResolvedId(resolved),
    entry,
    "Aliased package subpaths from virtual modules should resolve to loadable files",
  );
}

{
  const projectRoot = createTempProject("preserve-vite-vue");
  const importer = path.join(projectRoot, "app", "pages", "index.vue");
  const optimizedVueEntry = path.join(projectRoot, "node_modules", ".vite", "deps", "vue.js");
  writeFixtureFile(optimizedVueEntry, "export const createBlock = () => null;");

  const resolved = await resolveIdHook(
    {
      resolve: async (id) => (id === "vue" ? { id: `${optimizedVueEntry}?v=abc123` } : null),
    },
    createState(projectRoot),
    "vue",
    toVirtualId(importer),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    `${optimizedVueEntry}?v=abc123`,
    "Vite-optimized Vue entries must not be replaced by Node's CommonJS entry",
  );
}

{
  const projectRoot = createTempProject("dev-vue-node-fallback");
  const importer = path.join(projectRoot, "app", "pages", "index.vue");
  const vueRoot = path.join(projectRoot, "node_modules", "vue");
  const vueCjsEntry = path.join(vueRoot, "index.js");
  const vueBundlerEntry = path.join(vueRoot, "dist", "vue.runtime.esm-bundler.js");
  writeFixtureFile(
    path.join(vueRoot, "package.json"),
    JSON.stringify(
      {
        name: "vue",
        main: "index.js",
      },
      null,
      2,
    ),
  );
  writeFixtureFile(vueCjsEntry, "module.exports = require('./dist/vue.cjs.js');");
  writeFixtureFile(vueBundlerEntry, "export const Transition = () => null;");

  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    "vue",
    toVirtualId(importer),
    undefined,
  );

  assert.equal(
    resolved,
    null,
    "Dev virtual SFC Vue imports should stay bare so Vite can optimize and dedupe the runtime",
  );
}

{
  const projectRoot = createTempProject("build-vue-node-fallback");
  const importer = path.join(projectRoot, "app", "pages", "index.vue");
  const vueRoot = path.join(projectRoot, "node_modules", "vue");
  const vueCjsEntry = path.join(vueRoot, "index.js");
  const vueBundlerEntry = path.join(vueRoot, "dist", "vue.runtime.esm-bundler.js");
  writeFixtureFile(
    path.join(vueRoot, "package.json"),
    JSON.stringify(
      {
        name: "vue",
        main: "index.js",
      },
      null,
      2,
    ),
  );
  writeFixtureFile(vueCjsEntry, "module.exports = require('./dist/vue.cjs.js');");
  writeFixtureFile(vueBundlerEntry, "export const Transition = () => null;");

  const state = createState(projectRoot);
  state.server = null;

  const resolved = await resolveIdHook(
    nullResolveContext,
    state,
    "vue",
    toVirtualId(importer),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    vueBundlerEntry,
    "Build virtual SFC imports must resolve Vue to an ESM bundler entry, not the CommonJS package entry",
  );
}

{
  const projectRoot = createTempProject("build-vue-compiled-importer");
  const importer = path.join(projectRoot, "app", "pages", "index.vue");
  const compiledImporter = `${importer}.ts`;
  const vueRoot = path.join(projectRoot, "node_modules", "vue");
  const vueBundlerEntry = path.join(vueRoot, "dist", "vue.runtime.esm-bundler.js");
  writeFixtureFile(
    path.join(vueRoot, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(vueRoot, "index.js"), "module.exports = {};");
  writeFixtureFile(vueBundlerEntry, "export const resolveComponent = () => null;");

  const state = createState(projectRoot);
  state.server = null;

  const resolved = await resolveIdHook(
    {
      resolve: async (id) => (id === "vue" ? { id: "#entry" } : null),
    },
    state,
    "vue",
    compiledImporter,
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    vueBundlerEntry,
    "Build imports from compiled .vue.ts modules must not let Nuxt's #entry alias replace Vue runtime imports",
  );
}

{
  const projectRoot = createTempProject("build-vue-plain-sfc-importer");
  const importer = path.join(projectRoot, "app", "pages", "index.vue");
  const vueRoot = path.join(projectRoot, "node_modules", "vue");
  const vueBundlerEntry = path.join(vueRoot, "dist", "vue.runtime.esm-bundler.js");
  writeFixtureFile(
    path.join(vueRoot, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(vueRoot, "index.js"), "module.exports = {};");
  writeFixtureFile(vueBundlerEntry, "export const resolveComponent = () => null;");

  const state = createState(projectRoot);
  state.server = null;

  const resolved = await resolveIdHook(
    {
      resolve: async (id) => (id === "vue" ? { id: "#entry" } : null),
    },
    state,
    "vue",
    importer,
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    vueBundlerEntry,
    "Build imports from Vue SFC modules should bypass Nuxt's #entry alias for Vue runtime helpers",
  );
}

// pnpm-isolated dev install: the project root has no `node_modules/vue`, so
// deferring to Vite's secondary resolveId pass (which uses the \0-prefixed
// virtual ID as importer and falls back to root) cannot find Vue. The plugin
// must resolve Vue from the importer's package subtree instead.
{
  const projectRoot = createTempProject("dev-vue-pnpm-isolated-ctx");
  const nuxtImporter = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "nuxt",
    "dist",
    "app",
    "components",
    "nuxt-root.vue",
  );
  const vuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.5.30_x",
    "node_modules",
    "vue",
  );
  const vueLink = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "vue",
  );
  const vueBundlerEntry = path.join(vuePackage, "dist", "vue.runtime.esm-bundler.js");
  writeFixtureFile(nuxtImporter, "<template><div /></template>");
  writeFixtureFile(
    path.join(vuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(vuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(vueBundlerEntry, "export const Transition = () => null;");
  fs.mkdirSync(path.dirname(vueLink), { recursive: true });
  fs.symlinkSync(vuePackage, vueLink, "dir");

  const isolatedResolved = path.join(vueLink, "dist", "vue.runtime.esm-bundler.js");

  const resolved = await resolveIdHook(
    {
      resolve: async (id) => (id === "vue" ? { id: isolatedResolved } : null),
    },
    createState(projectRoot),
    "vue",
    toVirtualId(nuxtImporter),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    vueBundlerEntry,
    "Dev virtual SFC Vue imports must resolve to the importer-local Vue runtime when the project root has no hoisted node_modules/vue",
  );
}

{
  const parentRoot = createTempRoot("dev-vue-parent-with-project-pnpm-hoist");
  const projectRoot = path.join(parentRoot, "fixture-app");
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");
  const importer = path.join(projectRoot, "app", "components", "NewsSection.vue");
  writeFixtureFile(importer, "<template />\n");

  const parentVue = path.join(
    parentRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0-parent",
    "node_modules",
    "vue",
    "dist",
    "vue.runtime.esm-bundler.js",
  );
  const projectVuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0-project",
    "node_modules",
    "vue",
  );
  const projectVueHoist = path.join(projectRoot, "node_modules", ".pnpm", "node_modules", "vue");
  const projectVueBundlerEntry = path.join(projectVuePackage, "dist", "vue.runtime.esm-bundler.js");

  writeFixtureFile(parentVue, "export const parent = true;");
  writeFixtureFile(
    path.join(projectVuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(projectVuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(projectVueBundlerEntry, "export const project = true;");
  fs.mkdirSync(path.dirname(projectVueHoist), { recursive: true });
  fs.symlinkSync(projectVuePackage, projectVueHoist, "dir");

  const resolved = await resolveIdHook(
    {
      resolve: async () => ({ id: parentVue }),
    },
    createState(projectRoot),
    "vue",
    toVirtualId(importer),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    projectVueBundlerEntry,
    "Dev virtual SFC Vue imports must prefer the project pnpm hoist over a parent workspace runtime",
  );
}

// Same pnpm-isolated dev scenario, but Vite's own resolver cannot see Vue
// (e.g. when the secondary lookup uses the virtual ID as importer). The
// plugin must still find Vue via Node's resolution chain through the
// importer's package subtree.
{
  const projectRoot = createTempProject("dev-vue-pnpm-isolated-node-fallback");
  const nuxtImporter = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "nuxt",
    "dist",
    "app",
    "components",
    "nuxt-root.vue",
  );
  const vuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.5.30_x",
    "node_modules",
    "vue",
  );
  const vueLink = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "nuxt@4.4.2_x",
    "node_modules",
    "vue",
  );
  const vueBundlerEntry = path.join(vuePackage, "dist", "vue.runtime.esm-bundler.js");
  writeFixtureFile(nuxtImporter, "<template><div /></template>");
  writeFixtureFile(
    path.join(vuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(vuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(vueBundlerEntry, "export const Transition = () => null;");
  fs.mkdirSync(path.dirname(vueLink), { recursive: true });
  fs.symlinkSync(vuePackage, vueLink, "dir");

  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    "vue",
    toVirtualId(nuxtImporter),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    vueBundlerEntry,
    "Dev virtual SFC Vue imports must fall back to Node's importer-local resolution when Vite cannot see Vue from the project root",
  );
}

{
  const tempRoot = createTempRoot("regexp-bare-alias");
  const viteRoot = path.join(tempRoot, "app");
  const importer = path.join(tempRoot, "app", "src", "App.vue");
  const packageName = "vize-regexp-alias-fixture";
  const packageRoot = path.join(
    tempRoot,
    "node_modules",
    ".pnpm",
    `${packageName}@0.0.0`,
    "node_modules",
    packageName,
  );
  const pnpmHoistRoot = path.join(tempRoot, "node_modules", ".pnpm", "node_modules");
  const entry = path.join(packageRoot, "esm", "vs", "editor", "editor.main.js");

  fs.mkdirSync(path.dirname(importer), { recursive: true });
  fs.mkdirSync(path.dirname(entry), { recursive: true });
  fs.mkdirSync(pnpmHoistRoot, { recursive: true });
  fs.writeFileSync(importer, "<template><div /></template>");
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    `{"name":"${packageName}","version":"0.0.0"}`,
  );
  fs.writeFileSync(entry, "export const editor = {};");
  fs.symlinkSync(packageRoot, path.join(pnpmHoistRoot, packageName), "dir");

  const state = createState(viteRoot);
  state.server = null;
  state.cssAliasRules = [
    {
      find: /^vize-regexp-alias-fixture$/,
      replacement: "vize-regexp-alias-fixture/esm/vs/editor/editor.main.js",
    },
  ];

  const resolved = await resolveIdHook(
    nullResolveContext,
    state,
    packageName,
    toVirtualId(importer),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    entry,
    "RegExp package aliases from virtual modules should resolve to loadable files",
  );
}

{
  const projectRoot = createTempProject("vue-data-ui");
  writeFixtureFile(
    path.join(projectRoot, "node_modules", "vue-data-ui", "package.json"),
    JSON.stringify(
      {
        name: "vue-data-ui",
        exports: {
          "./style.css": "./dist/style.css",
        },
      },
      null,
      2,
    ),
  );
  writeFixtureFile(path.join(projectRoot, "node_modules", "vue-data-ui", "dist", "style.css"));

  const importer = toVirtualId(path.join(projectRoot, "app", "pages", "index.vue"));
  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    "vue-data-ui/style.css",
    importer,
    undefined,
  );

  assert.match(expectResolvedId(resolved), /node_modules[\\/]vue-data-ui[\\/]dist[\\/]style\.css$/);
}

{
  const projectRoot = createTempProject("primevue-forms");
  writeFixtureFile(
    path.join(projectRoot, "node_modules", "@primevue", "forms", "package.json"),
    JSON.stringify(
      {
        name: "@primevue/forms",
        exports: {
          "./resolvers/valibot": "./resolvers/valibot/index.mjs",
        },
      },
      null,
      2,
    ),
  );
  writeFixtureFile(
    path.join(
      projectRoot,
      "node_modules",
      "@primevue",
      "forms",
      "resolvers",
      "valibot",
      "index.mjs",
    ),
    "export default {};",
  );

  const importer = toVirtualId(path.join(projectRoot, "app", "pages", "index.vue"));
  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    "@primevue/forms/resolvers/valibot?nuxt_component=async",
    importer,
    undefined,
  );

  assert.match(
    expectResolvedId(resolved),
    /node_modules[\\/]@primevue[\\/]forms[\\/]resolvers[\\/]valibot[\\/]index\.mjs\?nuxt_component=async$/,
  );
}

{
  const projectRoot = createTempProject("nuxt-component-query");
  const source = path.join(
    projectRoot,
    "node_modules",
    "@nuxtjs",
    "mdc",
    "dist",
    "runtime",
    "components",
    "prose",
    "ProseScript.vue",
  );
  writeFixtureFile(source, "<template><script /></template>");

  const query =
    "?nuxt_component=async&nuxt_component_name=ProseScript&nuxt_component_export=default";
  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    `${source}${query}`,
    undefined,
    { ssr: true },
  );

  assert.equal(
    expectResolvedId(resolved),
    `${source}${query}`,
    "Nuxt component-loader Vue queries should stay on the real Vue path so Nuxt can rewrite the async wrapper",
  );
}

{
  const projectRoot = createTempProject("nuxt-component-query-virtual-import");
  const source = path.join(
    projectRoot,
    "node_modules",
    "@nuxtjs",
    "mdc",
    "dist",
    "runtime",
    "components",
    "prose",
    "ProseH2.vue",
  );
  writeFixtureFile(source, "<template><h2 /></template>");

  const query = "?nuxt_component=async&nuxt_component_name=ProseH2&nuxt_component_export=default";
  const virtualId = toVirtualId(source);
  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    virtualId,
    `${virtualId}${query}`,
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    virtualId,
    "Vize virtual Vue imports emitted from Nuxt component query modules should stay resolved",
  );
}

{
  const projectRoot = createTempProject("style-query");
  const source = path.join(projectRoot, "app", "components", "Styled.vue");
  writeFixtureFile(source, "<template><div /></template><style>.root{}</style>");

  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    `${source}?vue=&type=style&index=0&lang=css`,
    undefined,
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    `${source}?vue=&type=style&index=0&lang=css.css`,
    "Vue style queries should stay CSS-visible so Vite extracts them",
  );
}

{
  const projectRoot = createTempProject("vue-raw-query");
  const source = path.join(projectRoot, "app", "components", "Raw.vue");
  writeFixtureFile(source, "<template><div /></template>");

  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    `${source}?raw`,
    undefined,
    undefined,
  );

  assert.equal(resolved, null, "Vue ?raw imports should not be compiled as components");
}

{
  const projectRoot = createTempProject("plugin-visible-virtual");
  const source = path.join(projectRoot, "app", "pages", "index.vue");
  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    source,
    undefined,
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    toPluginVisibleVirtualId(source),
    "Resolved Vue SFC modules should keep a Vue-compatible query so post plugins can transform them",
  );
  assert.equal(
    expectResolvedId(resolved).startsWith("\0"),
    false,
    "Plugin-visible Vue SFC modules should not use Rollup-internal virtual IDs",
  );
}

{
  const projectRoot = createTempProject("dependency-scan-virtual");
  const source = path.join(projectRoot, "app", "pages", "index.vue");
  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    source,
    undefined,
    { scan: true },
  );

  assert.equal(
    expectResolvedId(resolved),
    toVirtualId(source),
    "Dependency scans should use load-hook virtual IDs instead of plugin-visible file-like IDs",
  );
}

{
  const projectRoot = createTempProject("ssr-entry");
  const source = path.join(projectRoot, "app", "pages", "index.vue");
  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    source,
    undefined,
    { isEntry: true, ssr: true },
  );

  assert.equal(
    expectResolvedId(resolved),
    toPluginVisibleVirtualId(source, true),
    "SSR resolves should use a dedicated virtual module ID",
  );
}

{
  const projectRoot = createTempProject("ssr-upgrade");
  const source = path.join(projectRoot, "app", "pages", "index.vue");
  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    toVirtualId(source),
    undefined,
    { isEntry: false, ssr: true },
  );

  assert.equal(
    expectResolvedId(resolved),
    toPluginVisibleVirtualId(source, true),
    "SSR resolution should upgrade client virtual IDs to SSR-specific virtual IDs",
  );
}

{
  const projectRoot = createTempProject("ssr-upgrade-visible");
  const source = path.join(projectRoot, "app", "pages", "index.vue");
  const resolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    toPluginVisibleVirtualId(source),
    undefined,
    { isEntry: false, ssr: true },
  );

  assert.equal(
    expectResolvedId(resolved),
    toPluginVisibleVirtualId(source, true),
    "SSR resolution should not duplicate the Vue-compatible query",
  );
}

{
  const parentRoot = createTempRoot("vue-parent-runtime");
  const projectRoot = path.join(parentRoot, "nested-project");
  writeFixtureFile(path.join(parentRoot, "node_modules", "vue", "package.json"), "{}");
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");
  const source = path.join(projectRoot, "app", "pages", "index.vue");
  writeFixtureFile(source, "<template />\n");
  const resolvedVue = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0",
    "node_modules",
    "vue",
    "dist",
    "vue.runtime.esm-bundler.js",
  );

  const resolved = await resolveIdHook(
    {
      resolve: async () => ({ id: resolvedVue }),
    },
    createState(projectRoot),
    "vue",
    toVirtualId(source),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    resolvedVue,
    "Vue runtime imports from virtual modules should prefer project-local pnpm Vue over parent root resolution",
  );
}

{
  const projectRoot = path.join(workspaceRoot, "tests", "_fixtures", "_git", "npmx.dev");
  if (canResolveFixtureDependency(projectRoot, "vue-data-ui/style.css")) {
    const importer = toVirtualId(path.join(projectRoot, "app", "pages", "index.vue"));
    const resolved = await resolveIdHook(
      nullResolveContext,
      createState(projectRoot),
      "vue-data-ui/style.css",
      importer,
      undefined,
    );

    assert.match(expectResolvedId(resolved), /vue-data-ui\/dist\/style\.css$/);
  }
}

{
  const projectRoot = path.join(workspaceRoot, "tests", "_fixtures", "_git", "vuefes-2025");
  if (canResolveFixtureDependency(projectRoot, "@primevue/forms/resolvers/valibot")) {
    const importer = toVirtualId(path.join(projectRoot, "app", "pages", "index.vue"));
    const resolved = await resolveIdHook(
      nullResolveContext,
      createState(projectRoot),
      "@primevue/forms/resolvers/valibot?nuxt_component=async",
      importer,
      undefined,
    );

    assert.match(
      expectResolvedId(resolved),
      /@primevue\/forms\/resolvers\/valibot\/index\.mjs\?nuxt_component=async$/,
    );
  }
}

{
  const projectRoot = path.join(workspaceRoot, "tests", "_fixtures", "_git", "npmx.dev");
  if (hasFixtureProject(projectRoot)) {
    const source = path.join(projectRoot, "app", "pages", "index.vue");
    const resolved = await resolveIdHook(
      nullResolveContext,
      createState(projectRoot),
      source,
      undefined,
      { isEntry: true, ssr: true },
    );

    assert.equal(
      expectResolvedId(resolved),
      toPluginVisibleVirtualId(source, true),
      "SSR resolves should use a dedicated virtual module ID",
    );
  }
}

{
  const projectRoot = path.join(workspaceRoot, "tests", "_fixtures", "_git", "npmx.dev");
  if (hasFixtureProject(projectRoot)) {
    const source = path.join(projectRoot, "app", "pages", "index.vue");
    const resolved = await resolveIdHook(
      nullResolveContext,
      createState(projectRoot),
      toVirtualId(source),
      undefined,
      { isEntry: false, ssr: true },
    );

    assert.equal(
      expectResolvedId(resolved),
      toPluginVisibleVirtualId(source, true),
      "SSR resolution should upgrade client virtual IDs to SSR-specific virtual IDs",
    );
  }
}

console.log("vite-plugin-vize resolve tests passed!");
