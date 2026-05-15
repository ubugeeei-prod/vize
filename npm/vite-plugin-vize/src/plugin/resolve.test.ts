import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";
import { toVirtualId } from "../virtual.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const workspaceRoot = path.resolve(__dirname, "../../../..");

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
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-resolve-define-page-"));
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
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-resolve-js-macro-"));
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
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-resolve-alias-"));
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
  const projectRoot = path.join(workspaceRoot, "tests", "_fixtures", "_git", "npmx.dev");
  if (hasFixtureProject(projectRoot)) {
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
  if (hasFixtureProject(projectRoot)) {
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

console.log("✅ vite-plugin-vize resolve tests passed!");
