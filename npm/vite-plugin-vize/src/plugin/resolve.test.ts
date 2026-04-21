import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import type { VizePluginState } from "./state.js";
import { resolveIdHook } from "./resolve.js";
import { toVirtualId } from "../virtual.js";

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
