import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { getCompileOptionsForRequest, type VizePluginState } from "./state.ts";
import { loadSfcModule, transformCachedSfcModule } from "../test/sfc-module.ts";
import { resolveIdHook } from "./resolve.ts";
import { toPluginVisibleVirtualId } from "../virtual.ts";
import { compileFile } from "../compiler.ts";

function createState(root: string, cache: VizePluginState["cache"]): VizePluginState {
  return {
    cache,
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: true,
    root,
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
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
    logger: { log() {}, info() {}, warn() {}, error() {} } as never,
  };
}

void test("production template-only SFC is removable from a barrel", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pure-sfc-hook-"));
  const source = path.join(root, "Pure.vue");
  fs.writeFileSync(source, "<template><div /></template>");
  try {
    const state = createState(
      root,
      new Map([
        [
          source,
          {
            code: "export function render() { return null }",
            scopeId: "pure",
            hasScoped: true,
            styles: [],
            moduleShape: {
              hasDefaultExport: false,
              hasSfcMainDefined: false,
              hasNamedRenderExport: true,
              hasNamedSsrRenderExport: false,
              defaultExportIsSfcMain: false,
            },
          },
        ],
      ]),
    );
    const resolveContext = { resolve: async () => null };
    assert.deepEqual(await resolveIdHook(resolveContext, state, source), {
      id: toPluginVisibleVirtualId(source),
      moduleSideEffects: false,
    });
    assert.equal(
      await resolveIdHook(resolveContext, state, source, undefined, { scan: true }),
      source,
    );
    const loaded = await transformCachedSfcModule(state, toPluginVisibleVirtualId(source), {
      ssr: false,
    });
    assert.equal(loaded.moduleSideEffects, false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// #11544: the marker reads the shape real compiles carry. Only a template-only
// SFC that emits nothing beside its render export is side-effect free.
void test("real template-only output is marked only without styles", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pure-sfc-real-"));
  const sources: Record<string, string> = {
    Pure: "<template><div class='pure' /></template>",
    Theme: "<template><div /></template><style>body { color: red }</style>",
    Scoped: "<template><div class='s' /></template><style scoped>.s { color: red }</style>",
    Options: "<script>export default { name: 'Options' }</script><template><div /></template>",
  };
  try {
    const state = createState(root, new Map());
    const marked: Record<string, boolean> = {};
    for (const [name, source] of Object.entries(sources)) {
      const file = path.join(root, `${name}.vue`);
      fs.writeFileSync(file, source);
      compileFile(file, state.cache, getCompileOptionsForRequest(state, false, file), source);
      const resolved = await resolveIdHook({ resolve: async () => null }, state, file);
      const loaded = await loadSfcModule(state, toPluginVisibleVirtualId(file), { ssr: false });
      assert.ok(loaded);
      const byResolve = typeof resolved === "object" && resolved?.moduleSideEffects === false;
      assert.equal(
        byResolve,
        loaded.moduleSideEffects === false,
        `${name}: resolve and load agree`,
      );
      marked[name] = byResolve;
    }
    assert.deepEqual(marked, { Pure: true, Theme: false, Scoped: false, Options: false });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
