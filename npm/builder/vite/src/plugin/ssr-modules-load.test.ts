import assert from "node:assert/strict";
import { test } from "node:test";

import type { VizePluginState } from "./state.ts";
import { loadHook } from "./load.ts";
import { toVirtualId } from "../virtual.ts";

const ROOT = "/project";
const PAGE = "/project/app/pages/index.vue";

const COMPILED = {
  code: 'const _sfc_main = { name: "Index" };\nexport default _sfc_main;',
  scopeId: "ssr12345",
  hasScoped: false,
  styles: [],
};

function stateFor(environment: "client" | "ssr"): VizePluginState {
  const compiled = new Map([[PAGE, COMPILED]]);
  return {
    cache: environment === "client" ? compiled : new Map(),
    ssrCache: environment === "ssr" ? compiled : new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: true,
    root: ROOT,
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
    logger: { log() {}, info() {}, warn() {}, error() {} },
  } as unknown as VizePluginState;
}

function loadedCode(environment: "client" | "ssr"): string {
  const result = loadHook(stateFor(environment), toVirtualId(PAGE), {
    ssr: environment === "ssr",
  });
  assert.ok(result, `${environment} load returned nothing`);
  return typeof result === "string" ? result : result.code;
}

void test("an SSR-loaded SFC registers itself in ssrContext.modules", () => {
  // Without this, `vue-bundle-renderer` finds no module to intersect with the
  // client manifest, emits no stylesheet link, and the page renders unstyled
  // until the route chunk loads on the client (#3868).
  const code = loadedCode("ssr");

  assert.match(code, /useSSRContext as __vize_useSSRContext/);
  assert.match(code, /\(ssrContext\.modules \|\| \(ssrContext\.modules = new Set\(\)\)\)\.add\(/);
  assert.match(code, /"app\/pages\/index\.vue"/);
  assert.ok(
    code.includes(COMPILED.code.split("\n")[0]),
    `the compiled module must survive the append:\n${code}`,
  );
});

void test("a client-loaded SFC is left alone", () => {
  const code = loadedCode("client");

  assert.ok(!code.includes("__vize_useSSRContext"), `client output must not register:\n${code}`);
  assert.ok(!code.includes("ssrContext"), `client output must not reference ssrContext:\n${code}`);
});
