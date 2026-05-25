import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";
import { toVirtualId } from "../virtual.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const workspaceRoot = path.resolve(__dirname, "../../../..");
const agentTestRoot = path.join(workspaceRoot, "__agent_only", "tests", "vite-plugin-vize");
fs.mkdirSync(agentTestRoot, { recursive: true });

function writeFixtureFile(filePath: string, content = ""): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function createTempRoot(prefix: string): string {
  return fs.mkdtempSync(path.join(agentTestRoot, prefix + "-"));
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

  assert.deepEqual(
    await resolveIdHook(
      nullResolveContext,
      createState(tempRoot),
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
      createState(tempRoot),
      "vue/dist/vue.esm-bundler.js",
      toVirtualId(importer, true),
      { ssr: true },
    ),
    { id: "vue", external: true },
    "SSR virtual modules should not bundle Vue runtime aliases into Nuxt server output",
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
    toVirtualId(aliased),
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
    `${toVirtualId(source, true)}${query}`,
    "Nuxt component-loader Vue queries should resolve to Vize virtual modules with the query preserved",
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
    toVirtualId(source, true),
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
    toVirtualId(source, true),
    "SSR resolution should upgrade client virtual IDs to SSR-specific virtual IDs",
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
      toVirtualId(source, true),
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
      toVirtualId(source, true),
      "SSR resolution should upgrade client virtual IDs to SSR-specific virtual IDs",
    );
  }
}

console.log("vite-plugin-vize resolve tests passed!");
