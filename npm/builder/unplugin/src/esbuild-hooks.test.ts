import { test } from "node:test";
import { vizeUnplugin } from "./unplugin.ts";
import { parseVueRequest } from "./request.ts";
import type { VizeUnpluginOptions } from "./types.ts";

type EsbuildHooks = {
  onResolveFilter: RegExp;
  onLoadFilter: RegExp;
  loader(this: void, code: string, id: string): string;
  config(this: void, buildOptions: { define?: Record<string, string> }): void;
};

function esbuildHooks(options: VizeUnpluginOptions): EsbuildHooks {
  const plugin = vizeUnplugin.raw(options, { framework: "esbuild" });
  const esbuild = plugin.esbuild;
  if (!esbuild) {
    throw new Error("expected esbuild hook object");
  }
  return esbuild as unknown as EsbuildHooks;
}

void test("esbuild hook object exposes the expected members", (t) => {
  const esbuild = esbuildHooks({ isProduction: true });
  t.assert.ok(esbuild.onResolveFilter instanceof RegExp);
  t.assert.ok(esbuild.onLoadFilter instanceof RegExp);
  t.assert.strictEqual(typeof esbuild.loader, "function");
  t.assert.strictEqual(typeof esbuild.config, "function");
});

void test("onResolveFilter matches .vue and .vue style requests but not .ts", (t) => {
  const { onResolveFilter } = esbuildHooks({ isProduction: true });
  t.assert.strictEqual(onResolveFilter.test("App.vue"), true);
  t.assert.strictEqual(onResolveFilter.test("App.vue?vue&type=style"), true);
  t.assert.strictEqual(onResolveFilter.test("App.ts"), false);
});

void test("onLoadFilter matches .vue and .vue style requests but not .ts", (t) => {
  const { onLoadFilter } = esbuildHooks({ isProduction: true });
  t.assert.strictEqual(onLoadFilter.test("App.vue"), true);
  t.assert.strictEqual(onLoadFilter.test("App.vue?vue&type=style"), true);
  t.assert.strictEqual(onLoadFilter.test("App.ts"), false);
});

void test("onResolveFilter/onLoadFilter are anchored on the .vue boundary", (t) => {
  const { onResolveFilter, onLoadFilter } = esbuildHooks({ isProduction: true });
  // `.vue` must be followed by end-of-string or `?`; a longer extension such as
  // `.vuex` should not match.
  t.assert.strictEqual(onResolveFilter.test("store.vuex"), false);
  t.assert.strictEqual(onLoadFilter.test("store.vuex"), false);
  // Path prefixes are fine as long as a `.vue` boundary appears.
  t.assert.strictEqual(onResolveFilter.test("/abs/path/App.vue"), true);
  t.assert.strictEqual(onLoadFilter.test("/abs/path/App.vue?vue&type=template"), true);
});

void test("parseVueRequest treats module as false only when the param is absent", (t) => {
  // No `module` param at all -> module === false (boolean).
  const absent = parseVueRequest("App.vue?vue&type=style").query;
  t.assert.strictEqual(absent.type, "style");
  t.assert.strictEqual(absent.module, false);

  // `module=false` -> the param IS present, so module is the truthy string
  // "false" (NOT the boolean false). This is the subtle parsing behavior the
  // loader branch keys off of.
  const present = parseVueRequest("App.vue?vue&type=style&module=false").query;
  t.assert.strictEqual(present.type, "style");
  t.assert.strictEqual(present.module, "false");
  t.assert.strictEqual(typeof present.module, "string");

  // Present-but-empty `module` -> boolean true.
  const empty = parseVueRequest("App.vue?vue&type=style&module").query;
  t.assert.strictEqual(empty.module, true);
});

void test('loader returns "css" for a plain style request (module param absent)', (t) => {
  const { loader } = esbuildHooks({ isProduction: true });
  // With no `module` param, parseVueRequest yields query.module === false, so
  // the `module !== false` test is false -> "css".
  t.assert.strictEqual(loader("", "App.vue?vue&type=style"), "css");
});

void test('loader returns "local-css" when the module param is present', (t) => {
  const { loader } = esbuildHooks({ isProduction: true });
  // `module=false` keeps the param PRESENT, so query.module is the string
  // "false" which is `!== false` -> "local-css".
  t.assert.strictEqual(loader("", "App.vue?vue&type=style&module=false"), "local-css");
  // present-but-empty `module` -> query.module === true -> "local-css".
  t.assert.strictEqual(loader("", "App.vue?vue&type=style&module"), "local-css");
});

void test('loader returns "js" for a non-style request', (t) => {
  const { loader } = esbuildHooks({ isProduction: true });
  t.assert.strictEqual(loader("", "App.vue"), "js");
  t.assert.strictEqual(loader("", "App.vue?vue&type=template"), "js");
  t.assert.strictEqual(loader("", "App.vue?vue&type=script&setup=true"), "js");
});

void test("config injects production Vue defines when hostCompiler is off", (t) => {
  const { config } = esbuildHooks({ isProduction: true });
  const buildOptions: { define?: Record<string, string> } = {};
  config(buildOptions);

  t.assert.deepStrictEqual(buildOptions.define, {
    __VUE_OPTIONS_API__: JSON.stringify(true),
    __VUE_PROD_DEVTOOLS__: JSON.stringify(false),
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: JSON.stringify(false),
  });
});

void test("config injects development Vue defines when isProduction is false", (t) => {
  const { config } = esbuildHooks({ isProduction: false });
  const buildOptions: { define?: Record<string, string> } = {};
  config(buildOptions);

  t.assert.deepStrictEqual(buildOptions.define, {
    __VUE_OPTIONS_API__: JSON.stringify(true),
    __VUE_PROD_DEVTOOLS__: JSON.stringify(true),
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: JSON.stringify(true),
  });
});

void test("config preserves existing user define entries and lets the user win on conflict", (t) => {
  const { config } = esbuildHooks({ isProduction: true });
  const buildOptions: { define?: Record<string, string> } = {
    define: {
      // User overrides one Vue define; the user value must win because
      // `...buildOptions.define` is spread last.
      __VUE_OPTIONS_API__: JSON.stringify(false),
      CUSTOM_FLAG: JSON.stringify("kept"),
    },
  };
  config(buildOptions);

  t.assert.deepStrictEqual(buildOptions.define, {
    __VUE_OPTIONS_API__: JSON.stringify(false),
    __VUE_PROD_DEVTOOLS__: JSON.stringify(false),
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: JSON.stringify(false),
    CUSTOM_FLAG: JSON.stringify("kept"),
  });
});

void test("config returns early and leaves define untouched when hostCompiler is on", (t) => {
  const { config } = esbuildHooks({ vueVersion: 2 });

  // No pre-existing define stays undefined.
  const empty: { define?: Record<string, string> } = {};
  const result = config(empty);
  t.assert.strictEqual(result, undefined);
  t.assert.strictEqual(empty.define, undefined);

  // A pre-existing user define is left exactly as-is.
  const withDefine: { define?: Record<string, string> } = {
    define: { CUSTOM_FLAG: JSON.stringify("kept") },
  };
  config(withDefine);
  t.assert.deepStrictEqual(withDefine.define, {
    CUSTOM_FLAG: JSON.stringify("kept"),
  });
});

void test("config respects explicit compatibility.hostCompiler:true on Vue 3", (t) => {
  const { config } = esbuildHooks({ compatibility: { hostCompiler: true } });
  const buildOptions: { define?: Record<string, string> } = {};
  config(buildOptions);
  t.assert.strictEqual(buildOptions.define, undefined);
});
