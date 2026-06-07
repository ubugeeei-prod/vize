import { test } from "node:test";
import { createRequire } from "node:module";
import type { Compiler as WebpackCompiler } from "webpack";
import { injectWebpackVueDefines, vizeUnplugin } from "./unplugin.ts";

// A fake DefinePlugin that records the definitions it was constructed with and
// pushes them onto a shared log whenever `apply` runs. This mirrors the
// FakeDefinePlugin pattern in webpack.test.ts so we can assert on what would
// have been injected without running a real webpack build.
function createFakeDefinePlugin(log: Array<Record<string, string>>) {
  return class FakeDefinePlugin {
    definitions: Record<string, string>;

    constructor(definitions: Record<string, string>) {
      this.definitions = definitions;
    }

    apply() {
      log.push(this.definitions);
    }
  };
}

// Build a minimal fake compiler whose `webpack.DefinePlugin` is used by the
// Webpack 5 resolution branch (webpackVersion !== 4).
function createWebpack5Compiler(
  DefinePlugin: unknown,
  existingPlugins: Array<{ definitions?: Record<string, unknown> }> = [],
): WebpackCompiler {
  return {
    webpack: { DefinePlugin },
    options: { plugins: existingPlugins },
  } as unknown as WebpackCompiler;
}

// Whether the host environment can resolve real webpack with a DefinePlugin.
// This determines whether the "missing DefinePlugin" path can ever throw the
// vize error: when real webpack is resolvable, resolution always succeeds.
const hostWebpackHasDefinePlugin = (() => {
  try {
    const require = createRequire(import.meta.url);
    const w = require("webpack") as { DefinePlugin?: unknown };
    return typeof w.DefinePlugin === "function";
  } catch {
    return false;
  }
})();

void test("webpack 5 shape: injects all three Vue defines via compiler.webpack.DefinePlugin (dev)", (t) => {
  const applied: Array<Record<string, string>> = [];
  const FakeDefinePlugin = createFakeDefinePlugin(applied);
  const compiler = createWebpack5Compiler(FakeDefinePlugin);

  injectWebpackVueDefines(compiler, /* isProduction */ false, 5);

  // Applied exactly once with all three defines. With isProduction:false the two
  // prod flags become JSON.stringify(!false) === "true" and __VUE_OPTIONS_API__
  // is always JSON.stringify(true) === "true".
  t.assert.deepStrictEqual(applied, [
    {
      __VUE_OPTIONS_API__: "true",
      __VUE_PROD_DEVTOOLS__: "true",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "true",
    },
  ]);
});

void test("webpack 5 shape: production flips the prod-only defines to false", (t) => {
  const applied: Array<Record<string, string>> = [];
  const FakeDefinePlugin = createFakeDefinePlugin(applied);
  const compiler = createWebpack5Compiler(FakeDefinePlugin);

  injectWebpackVueDefines(compiler, /* isProduction */ true, 5);

  t.assert.deepStrictEqual(applied, [
    {
      __VUE_OPTIONS_API__: "true",
      __VUE_PROD_DEVTOOLS__: "false",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "false",
    },
  ]);
});

void test("full dedup: all three defines already present injects nothing", (t) => {
  const applied: Array<Record<string, string>> = [];
  const FakeDefinePlugin = createFakeDefinePlugin(applied);
  const compiler = createWebpack5Compiler(FakeDefinePlugin, [
    {
      definitions: {
        __VUE_OPTIONS_API__: "true",
        __VUE_PROD_DEVTOOLS__: "false",
        __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "false",
      },
    },
  ]);

  injectWebpackVueDefines(compiler, true, 5);

  // No missing defines => DefinePlugin is never constructed/applied.
  t.assert.strictEqual(applied.length, 0);
});

void test("full dedup across multiple existing plugins injects nothing", (t) => {
  const applied: Array<Record<string, string>> = [];
  const FakeDefinePlugin = createFakeDefinePlugin(applied);
  // The three keys are spread across separate existing define plugins; the
  // resolver unions every plugin's `definitions` keys before computing the gap.
  const compiler = createWebpack5Compiler(FakeDefinePlugin, [
    { definitions: { __VUE_OPTIONS_API__: "true" } },
    { definitions: { __VUE_PROD_DEVTOOLS__: "false" } },
    { definitions: { __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "false" } },
  ]);

  injectWebpackVueDefines(compiler, true, 5);

  t.assert.strictEqual(applied.length, 0);
});

void test("partial dedup: only the missing defines are injected (dev)", (t) => {
  const applied: Array<Record<string, string>> = [];
  const FakeDefinePlugin = createFakeDefinePlugin(applied);
  const compiler = createWebpack5Compiler(FakeDefinePlugin, [
    { definitions: { __VUE_OPTIONS_API__: "true" } },
  ]);

  injectWebpackVueDefines(compiler, /* isProduction */ false, 5);

  // __VUE_OPTIONS_API__ already exists, so only the two prod flags are injected.
  t.assert.deepStrictEqual(applied, [
    {
      __VUE_PROD_DEVTOOLS__: "true",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "true",
    },
  ]);
});

void test("plugins entries without `definitions` are ignored when computing the gap", (t) => {
  const applied: Array<Record<string, string>> = [];
  const FakeDefinePlugin = createFakeDefinePlugin(applied);
  // Unrelated plugins (no `definitions` field) must not affect dedup.
  const compiler = createWebpack5Compiler(FakeDefinePlugin, [
    { name: "some-other-plugin" } as unknown as { definitions?: Record<string, unknown> },
    { definitions: { __VUE_PROD_DEVTOOLS__: "false" } },
  ]);

  injectWebpackVueDefines(compiler, /* isProduction */ false, 5);

  // Only __VUE_PROD_DEVTOOLS__ pre-existed, so the other two are injected.
  t.assert.deepStrictEqual(applied, [
    {
      __VUE_OPTIONS_API__: "true",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "true",
    },
  ]);
});

void test("explicit constructor is preferred over compiler.webpack.DefinePlugin", (t) => {
  const compilerLog: Array<Record<string, string>> = [];
  const explicitLog: Array<Record<string, string>> = [];
  const CompilerDefinePlugin = createFakeDefinePlugin(compilerLog);
  const ExplicitDefinePlugin = createFakeDefinePlugin(explicitLog);
  const compiler = createWebpack5Compiler(CompilerDefinePlugin);

  injectWebpackVueDefines(compiler, false, 5, ExplicitDefinePlugin);

  // The explicit constructor short-circuits resolution entirely.
  t.assert.strictEqual(compilerLog.length, 0);
  t.assert.strictEqual(explicitLog.length, 1);
});

void test("missing DefinePlugin: resolution falls through to host webpack", (t) => {
  // Webpack 4 path with no explicit constructor and a compiler that lacks
  // `webpack.DefinePlugin`. The v4 branch skips compiler.webpack and resolves
  // via host require("webpack"). Behavior depends on the environment:
  //  - if host webpack is NOT resolvable => throws the vize error;
  //  - if host webpack IS resolvable => resolution succeeds (no vize error).
  // We use the full-dedup shape so that, when resolution succeeds, the real
  // DefinePlugin is never constructed/applied (no missing defines) and the call
  // completes without touching the fake compiler's (absent) webpack hooks.
  const compiler = {
    options: {
      plugins: [
        {
          definitions: {
            __VUE_OPTIONS_API__: "true",
            __VUE_PROD_DEVTOOLS__: "false",
            __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "false",
          },
        },
      ],
    },
  } as unknown as WebpackCompiler;

  if (hostWebpackHasDefinePlugin) {
    // Resolution succeeds; with all defines present nothing is injected and it
    // does NOT throw the vize "Could not resolve" error.
    t.assert.doesNotThrow(() => injectWebpackVueDefines(compiler, true, 4));
  } else {
    t.assert.throws(
      () => injectWebpackVueDefines(compiler, true, 4),
      /\[vize\] Could not resolve webpack DefinePlugin/,
    );
  }
});

void test("missing DefinePlugin: vize error only when host webpack is unresolvable", (t) => {
  // A bare compiler (no webpack.DefinePlugin) with missing defines. When host
  // webpack is unresolvable this is the canonical "Could not resolve" throw.
  // When it IS resolvable, the real DefinePlugin resolves successfully (so the
  // vize error never fires) but its `.apply` requires a real compiler with
  // hooks; on this fake compiler that surfaces as a non-vize TypeError. Either
  // way, the vize "Could not resolve" message must not appear when resolution
  // succeeded.
  const compiler = {
    options: { plugins: [] },
  } as unknown as WebpackCompiler;

  if (hostWebpackHasDefinePlugin) {
    let caught: unknown;
    try {
      injectWebpackVueDefines(compiler, false, 4);
    } catch (error) {
      caught = error;
    }
    // Resolution succeeded => never the vize "Could not resolve" error.
    const message = caught instanceof Error ? caught.message : String(caught);
    t.assert.strictEqual(message.includes("[vize] Could not resolve webpack DefinePlugin"), false);
  } else {
    t.assert.throws(
      () => injectWebpackVueDefines(compiler, false, 4),
      /\[vize\] Could not resolve webpack DefinePlugin/,
    );
  }
});

void test("webpack hook: hostCompiler (vueVersion 2) does not inject defines", (t) => {
  const applied: Array<Record<string, string>> = [];
  const SpyDefinePlugin = createFakeDefinePlugin(applied);
  const compiler = createWebpack5Compiler(SpyDefinePlugin);

  // vueVersion 2 is a legacy version => hostCompiler defaults to true, so the
  // `webpack` hook's `if (!options.hostCompiler)` guard skips injection.
  const raw = (
    vizeUnplugin.raw as (
      options: unknown,
      meta: unknown,
    ) => { webpack?: (compiler: WebpackCompiler) => void }
  )({ vueVersion: 2 }, { framework: "webpack", webpack: {} });
  const plugin = Array.isArray(raw) ? raw[0] : raw;

  t.assert.strictEqual(typeof plugin.webpack, "function");
  plugin.webpack?.(compiler);

  t.assert.strictEqual(applied.length, 0);
});

void test("webpack hook: non-host (vueVersion 3) injects all three defines", (t) => {
  const applied: Array<Record<string, string>> = [];
  const SpyDefinePlugin = createFakeDefinePlugin(applied);
  const compiler = createWebpack5Compiler(SpyDefinePlugin);

  // vueVersion 3 (default) => hostCompiler false => the hook injects. The hook
  // forwards compatibility.webpackVersion (undefined here), so the resolver uses
  // compiler.webpack.DefinePlugin (the webpackVersion !== 4 branch).
  const raw = (
    vizeUnplugin.raw as (
      options: unknown,
      meta: unknown,
    ) => { webpack?: (compiler: WebpackCompiler) => void }
  )({ vueVersion: 3, isProduction: false }, { framework: "webpack", webpack: {} });
  const plugin = Array.isArray(raw) ? raw[0] : raw;

  plugin.webpack?.(compiler);

  t.assert.deepStrictEqual(applied, [
    {
      __VUE_OPTIONS_API__: "true",
      __VUE_PROD_DEVTOOLS__: "true",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "true",
    },
  ]);
});
