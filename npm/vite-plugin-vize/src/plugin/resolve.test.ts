import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizePluginState } from "./state.js";
import { resolveIdHook } from "./resolve.js";
import { toVirtualId } from "../virtual.js";

const tempRoots: string[] = [];

process.on("exit", () => {
  for (const root of tempRoots) {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

function writeFixtureFile(filePath: string, content = ""): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function createTempProject(): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vite-plugin-vize-resolve-"));
  tempRoots.push(root);

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
  const projectRoot = createTempProject();
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
  const projectRoot = createTempProject();
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
  const projectRoot = createTempProject();
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
  const projectRoot = createTempProject();
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

console.log("✅ vite-plugin-vize resolve tests passed!");
