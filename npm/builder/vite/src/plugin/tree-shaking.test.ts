import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import type { VizePluginState } from "./state.ts";
import { loadHook } from "./load.ts";
import { resolveIdHook } from "./resolve.ts";
import { toPluginVisibleVirtualId } from "../virtual.ts";

void test("production template-only SFC is removable from a barrel", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pure-sfc-hook-"));
  const source = path.join(root, "Pure.vue");
  fs.writeFileSync(source, "<template><div /></template>");
  try {
    const state: VizePluginState = {
      cache: new Map([
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
    const resolveContext = { resolve: async () => null };
    assert.deepEqual(await resolveIdHook(resolveContext, state, source), {
      id: toPluginVisibleVirtualId(source),
      moduleSideEffects: false,
    });
    assert.equal(
      await resolveIdHook(resolveContext, state, source, undefined, { scan: true }),
      source,
    );
    const loaded = loadHook(state, toPluginVisibleVirtualId(source), { ssr: false });
    assert.ok(loaded && typeof loaded === "object");
    assert.equal(loaded.moduleSideEffects, false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
