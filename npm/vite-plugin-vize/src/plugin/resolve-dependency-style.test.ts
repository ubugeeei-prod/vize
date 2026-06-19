import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";

const testRoot = fs.mkdtempSync(
  path.join(fs.realpathSync(os.tmpdir()), "vize-vite-plugin-dependency-style-"),
);

function writeFixtureFile(filePath: string, content = ""): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function createTempProject(prefix: string): string {
  const root = fs.mkdtempSync(path.join(testRoot, prefix + "-"));
  writeFixtureFile(path.join(root, "package.json"), JSON.stringify({ private: true }, null, 2));
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
    filter: (id) => !id.includes("node_modules") && id.endsWith(".vue"),
    scanPatterns: ["app/**/*.vue", "design/**/*.vue"],
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {
      handleNodeModulesVue: false,
      include: ["app/**/*.vue", "design/**/*.vue"],
      exclude: ["node_modules/**", "**/node_modules/**"],
    },
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

function createDependencyVuePath(projectRoot: string): string {
  return path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "@nuxtjs+mdc@0.19.2",
    "node_modules",
    "@nuxtjs",
    "mdc",
    "dist",
    "runtime",
    "components",
    "prose",
    "ProsePre.vue",
  );
}

function expectResolvedId(resolved: Awaited<ReturnType<typeof resolveIdHook>>): string | null {
  if (resolved == null) return null;
  return typeof resolved === "string" ? resolved : resolved.id;
}

{
  const projectRoot = createTempProject("style-query");
  const source = createDependencyVuePath(projectRoot);
  writeFixtureFile(
    source,
    "<template><pre /></template><style>pre code .line{display:block}</style>",
  );

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
    "Dependency Vue style queries should stay CSS-visible when dependency SFC compilation is disabled",
  );

  const fsResolved = await resolveIdHook(
    nullResolveContext,
    createState(projectRoot),
    `/@fs${source}?vue=&type=style&index=0&lang=css`,
    undefined,
    undefined,
  );

  assert.equal(
    expectResolvedId(fsResolved),
    `${source}?vue=&type=style&index=0&lang=css.css`,
    "Dependency Vue style queries from /@fs requests should normalize into the CSS pipeline",
  );
}

{
  const projectRoot = createTempProject("component-query");
  const source = createDependencyVuePath(projectRoot);
  writeFixtureFile(source, "<template><pre /></template>");

  assert.equal(
    await resolveIdHook(nullResolveContext, createState(projectRoot), source, undefined, undefined),
    null,
    "Dependency Vue component requests should stay on the host Nuxt/Vite route",
  );

  const query = "?nuxt_component=async&nuxt_component_name=ProsePre&nuxt_component_export=default";
  assert.equal(
    await resolveIdHook(
      nullResolveContext,
      createState(projectRoot),
      `${source}${query}`,
      undefined,
      undefined,
    ),
    null,
    "Nuxt dependency component wrapper queries should not be Vize-compiled",
  );
}

console.log("✅ vite-plugin-vize dependency style resolve tests passed!");
