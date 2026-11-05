import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";
import type { SsrEnvironmentResolve } from "./ssr-modules.ts";
import { toVirtualId } from "../virtual.ts";

const testRoot = fs.mkdtempSync(
  path.join(fs.realpathSync(os.tmpdir()), "vize-vue-runtime-resolve-"),
);

function writeFixtureFile(filePath: string, content = ""): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
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

{
  const projectRoot = fs.mkdtempSync(path.join(testRoot, "vuetify-runtime-"));
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");

  const importer = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vuetify@3.11.0_vue@3.6.0",
    "node_modules",
    "vuetify",
    "lib",
    "components",
    "VCard",
    "VCard.mjs",
  );
  writeFixtureFile(importer, "import { withDirectives } from 'vue';");

  const vuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0",
    "node_modules",
    "vue",
  );
  const projectVueHoist = path.join(projectRoot, "node_modules", ".pnpm", "node_modules", "vue");
  const vueBundlerEntry = path.join(vuePackage, "dist", "vue.runtime.esm-bundler.js");
  writeFixtureFile(
    path.join(vuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }, null, 2),
  );
  writeFixtureFile(path.join(vuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(vueBundlerEntry, "export const withDirectives = () => null;");
  fs.mkdirSync(path.dirname(projectVueHoist), { recursive: true });
  fs.symlinkSync(vuePackage, projectVueHoist, "dir");

  const optimizedVueEntry = path.join(projectRoot, "node_modules", ".vite", "deps", "vue.js");
  writeFixtureFile(optimizedVueEntry, "export const withDirectives = () => null;");

  const resolved = await resolveIdHook(
    {
      resolve: async (id) => (id === "vue" ? { id: `${optimizedVueEntry}?v=abc123` } : null),
    },
    createState(projectRoot),
    "vue",
    importer,
    undefined,
  );

  assert.equal(
    resolved,
    null,
    "Dev Vue imports from dependencies should stay with Vite's optimized runtime to avoid duplicate Vue instances",
  );
}

{
  // Vitest pre-bundles under `<root>/node_modules/.vite/vitest/<hash>/deps`, not
  // Vite's default `.vite/deps`. A project SFC must still keep Vite's optimized
  // Vue there; pinning the raw pnpm `vue` file instead loads a second Vue
  // runtime beside the pre-bundled one, and Vapor effects stop seeing updates.
  const projectRoot = fs.mkdtempSync(path.join(testRoot, "vitest-cache-dir-"));
  writeFixtureFile(path.join(projectRoot, "package.json"), "{}");
  const importer = path.join(projectRoot, "src", "Counter.vue");
  writeFixtureFile(importer, "<template><div /></template>");

  const vuePackage = path.join(
    projectRoot,
    "node_modules",
    ".pnpm",
    "vue@3.6.0",
    "node_modules",
    "vue",
  );
  writeFixtureFile(
    path.join(vuePackage, "package.json"),
    JSON.stringify({ name: "vue", main: "index.js" }),
  );
  writeFixtureFile(path.join(vuePackage, "index.js"), "module.exports = {};");
  writeFixtureFile(path.join(vuePackage, "dist", "vue.runtime.esm-bundler.js"), "export {};");
  fs.mkdirSync(path.join(projectRoot, "node_modules"), { recursive: true });
  fs.symlinkSync(vuePackage, path.join(projectRoot, "node_modules", "vue"), "dir");

  const cacheDir = path.join(projectRoot, "node_modules", ".vite", "vitest", "da39a3ee5e6b4b0d");
  const optimizedVueEntry = path.join(cacheDir, "deps", "vue.js");
  writeFixtureFile(optimizedVueEntry, "export {};");

  const state = { ...createState(projectRoot), viteCacheDir: cacheDir };
  const resolved = await resolveIdHook(
    { resolve: async (id) => (id === "vue" ? { id: `${optimizedVueEntry}?v=abc123` } : null) },
    state,
    "vue",
    importer,
    undefined,
  );

  assert.equal(
    resolved?.id,
    `${optimizedVueEntry}?v=abc123`,
    "A Vue entry in Vite's configured cacheDir (Vitest's .vite/vitest/<hash>) is the optimized runtime and must not be replaced",
  );
}

{
  // A workerd environment (Void, @cloudflare/vite-plugin) bundles everything:
  // an external `vue` becomes `No such module "assets/vue"` at request time.
  const projectRoot = fs.mkdtempSync(path.join(testRoot, "workerd-ssr-vue-"));
  const importer = toVirtualId(path.join(projectRoot, "pages", "index.vue"), true);
  const state = { ...createState(projectRoot), server: null };
  const bundledVue = path.join(projectRoot, "node_modules", "vue", "index.mjs");
  const resolveVue = async (id: string) => (id === "vue" ? { id: bundledVue } : null);
  const resolveIn = (resolve: SsrEnvironmentResolve | undefined) =>
    resolveIdHook(
      { environment: { name: "ssr", config: { resolve } }, resolve: resolveVue },
      state,
      "vue",
      importer,
      { ssr: true },
    );

  assert.deepEqual(await resolveIn(undefined), { id: "vue", external: true });
  assert.deepEqual(await resolveIn({ noExternal: true }), { id: bundledVue });
  assert.deepEqual(await resolveIn({ noExternal: ["vue"] }), { id: bundledVue });
  assert.deepEqual(await resolveIn({ noExternal: true, external: ["vue"] }), {
    id: "vue",
    external: true,
  });
}

{
  // #11527: a build resolves the project's Vue runtime once per environment,
  // request and importer directory; the importing SFC does not change it.
  const projectRoot = fs.mkdtempSync(path.join(testRoot, "vue-runtime-per-directory-"));
  const vueEntry = (base: string, version: string) => {
    const pkg = path.join(base, "node_modules", ".pnpm", `vue@${version}`, "node_modules", "vue");
    writeFixtureFile(path.join(pkg, "package.json"), '{"name":"vue"}');
    const entry = path.join(pkg, "dist", "vue.runtime.esm-bundler.js");
    writeFixtureFile(entry, "export {};");
    return entry;
  };
  const rootVue = vueEntry(projectRoot, "3.6.0");
  const nestedVue = vueEntry(path.join(projectRoot, "nested"), "3.5.0");
  const importers = ["src/A.vue", "src/B.vue", "nested/C.vue"].map((file) => {
    writeFixtureFile(path.join(projectRoot, file), "<template><div /></template>");
    return path.join(projectRoot, file);
  });

  for (const server of [null, {} as never]) {
    const state = { ...createState(projectRoot), server };
    const resolved: string[] = [];
    const ctx = {
      resolve: async (id: string, importer?: string) => {
        if (id !== "vue") return null;
        resolved.push(path.relative(projectRoot, importer ?? ""));
        return { id: importer?.includes("/nested/") ? nestedVue : rootVue };
      },
    };
    const results = [];
    for (const importer of importers)
      results.push(await resolveIdHook(ctx, state, "vue", importer));

    assert.deepEqual(results, [rootVue, rootVue, nestedVue], "each directory keeps its own Vue");
    assert.deepEqual(
      resolved,
      server === null ? ["src/A.vue", "nested/C.vue"] : ["src/A.vue", "src/B.vue", "nested/C.vue"],
      server === null
        ? "a build resolves Vue once per importer directory"
        : "the dev server, where installs can change it, resolves per import",
    );
  }
}
