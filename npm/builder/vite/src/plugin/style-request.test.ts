import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { compileFile } from "../compiler.ts";
import { generateOutput } from "../utils/index.ts";
import { transformScopedPreprocessorCss } from "./compat.ts";
import { loadHook } from "./load.ts";
import { resolveIdHook } from "./resolve.ts";
import { getCompileOptionsForRequest, type VizePluginState } from "./state.ts";

function createState(root: string): VizePluginState {
  return {
    cache: new Map(),
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
    scanPatterns: null,
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

void test("Nuxt repeated inline style resolution loads the original scoped SFC blocks", async () => {
  const root = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), "vize-nuxt-style-"));
  try {
    const filename = path.join(root, "日本語 Component.vue");
    fs.writeFileSync(path.join(root, "external.css"), "aside { color: green; }");
    fs.writeFileSync(
      filename,
      `<template><p>hi</p></template>
<style scoped>p { color: red; }</style>
<style scoped module="theme" lang="scss">.button { color: blue; }</style>
<style lang="less">.less { color: black; }</style>
<style scoped src="./external.css"></style>`,
    );
    const state = createState(root);
    const compiled = compileFile(filename, state.cache, getCompileOptionsForRequest(state, false));
    compileFile(filename, state.ssrCache, getCompileOptionsForRequest(state, true));
    const module = generateOutput(compiled, {
      filePath: filename,
      isProduction: true,
      isDev: false,
      extractCss: false,
    });
    const imports = [...module.matchAll(/(?:import "|from ")([^"\n]+\?vue=[^"\n]+)"/g)]
      .map((match) => match[1])
      .sort(
        (a, b) =>
          Number(new URLSearchParams(a.split("?")[1]).get("index")) -
          Number(new URLSearchParams(b.split("?")[1]).get("index")),
      );
    assert.equal(imports.length, 4);
    const loaded: string[] = [];
    for (const [index, imported] of imports.entries()) {
      const expectedLang = ["css", "scss", "less", "css"][index];
      const params = new URLSearchParams(imported.slice(imported.indexOf("?") + 1));
      assert.equal(params.get("index"), String(index));
      assert.equal(params.get("lang"), expectedLang);
      assert.equal(params.get("vize-file"), filename);
      assert.equal(params.get("module"), index === 1 ? "theme" : null);
      assert.equal(
        imported.slice(0, imported.indexOf("?")),
        `${filename}.__vize_style_${index}${index === 1 ? ".module" : ""}.${expectedLang}`,
      );
      const id = `${imported}&inline&used`;
      for (let round = 0; round < 5; round++) {
        assert.equal(
          await resolveIdHook({ resolve: async () => null }, state, id, filename, { ssr: true }),
          id,
        );
      }
      for (const ssr of [false, true]) {
        const result = loadHook(state, id, { ssr });
        assert.ok(result && typeof result === "object");
        loaded.push(result.code);
      }
      if (index === 1) {
        assert.equal(
          transformScopedPreprocessorCss(".button { color: blue; }", id),
          `.button[data-v-${compiled.scopeId}]{color: blue;}`,
        );
      }
    }
    assert.deepEqual(loaded, [
      `p[data-v-${compiled.scopeId}]{color: red;}`,
      `p[data-v-${compiled.scopeId}]{color: red;}`,
      ".button { color: blue; }",
      ".button { color: blue; }",
      ".less { color: black; }",
      ".less { color: black; }",
      `aside[data-v-${compiled.scopeId}]{color: green;}`,
      `aside[data-v-${compiled.scopeId}]{color: green;}`,
    ]);
    const raw = `${filename}?vue=&type=style&index=0&scoped=data-v-${compiled.scopeId}&lang=css&inline&used`;
    const expected = `${filename}.__vize_style_0.css?${raw.split("?")[1]}&${new URLSearchParams({ "vize-file": filename }).toString()}`;
    assert.equal(
      await resolveIdHook({ resolve: async () => null }, state, raw, filename, undefined),
      expected,
    );
    const fsRequest = `/@fs${raw}`;
    assert.equal(
      await resolveIdHook({ resolve: async () => null }, state, fsRequest, filename, undefined),
      expected,
    );
    const namedModule = `${filename}?vue=&type=style&index=1&lang=scss&module=theme.module&inline&used`;
    const resolvedModule = await resolveIdHook(
      { resolve: async () => null },
      state,
      namedModule,
      filename,
      { ssr: true },
    );
    assert.equal(typeof resolvedModule, "string");
    const moduleParams = new URLSearchParams((resolvedModule as string).split("?")[1]);
    assert.equal(moduleParams.get("module"), "theme.module");
    assert.equal(moduleParams.get("lang"), "scss");
    assert.equal(moduleParams.has("inline") && moduleParams.has("used"), true);
    const moduleStyle = loadHook(state, resolvedModule as string, { ssr: true });
    assert.ok(moduleStyle && typeof moduleStyle === "object");
    assert.equal(moduleStyle.code, ".button { color: blue; }");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
