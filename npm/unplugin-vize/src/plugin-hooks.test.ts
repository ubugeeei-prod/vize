import path from "node:path";
import { test } from "node:test";
import { vizeUnplugin } from "./unplugin.ts";
import { isVirtualStyleId } from "./request.ts";
import { packageRoot } from "./test/helpers.ts";
import type { VizeUnpluginOptions } from "./types.ts";

// The bundler-agnostic raw hooks of the unplugin factory. For `framework:
// "rollup"` the factory yields a Rollup-format plugin whose hooks are bare
// functions (verified: resolveId/loadInclude/load/transformInclude/transform/
// watchChange are all `typeof === "function"`), so each is invoked with
// `.call(ctx, ...args)` exactly like the sibling raw-hook tests.
type RawCtx = { warn(message: string): void };

function rawPlugin(options: VizeUnpluginOptions) {
  return vizeUnplugin.raw(options, { framework: "rollup" });
}

const ctx: RawCtx = { warn() {} };

// A synthetic id is enough: the source is passed straight to transform(), so the
// .vue file never has to exist on disk. The path only has to end in `.vue` and
// avoid `node_modules` to satisfy the plugin's request matching.
const APP_VUE = path.join(packageRoot, "App.vue");
const BASIC_SFC = `<template><div>Hello</div></template>\n<script setup>const n = 1</script>`;

// ---------------------------------------------------------------------------
// transformInclude (hostCompiler OFF)
// ---------------------------------------------------------------------------

void test("transformInclude is true for a plain .vue id", (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });
  t.assert.strictEqual(plugin.transformInclude?.call(ctx as never, "/p/App.vue" as never), true);
});

void test("transformInclude is false for a .vue style sub-request (query.vue set)", (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });
  // `?vue&type=style` carries query.vue === true, so the `!request.query.vue`
  // guard excludes the request even though the filename is a .vue file.
  t.assert.strictEqual(
    plugin.transformInclude?.call(ctx as never, "/p/App.vue?vue&type=style" as never),
    false,
  );
});

void test("transformInclude fast-paths to false for any id without .vue (no throw)", (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });
  // The `!id.includes(".vue")` fast-path returns false before any
  // URLSearchParams parsing, so a plain .ts import never throws.
  t.assert.strictEqual(plugin.transformInclude?.call(ctx as never, "/p/main.ts" as never), false);
});

void test("transformInclude is false for a node_modules .vue (default exclude)", (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });
  // The id contains `.vue` and is a .vue file, but the default exclude
  // `/node_modules/` makes the filter reject it.
  t.assert.strictEqual(
    plugin.transformInclude?.call(ctx as never, "/p/node_modules/foo/App.vue" as never),
    false,
  );
});

// ---------------------------------------------------------------------------
// transform (hostCompiler OFF)
// ---------------------------------------------------------------------------

void test("transform returns null for a non-.vue id", async (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });
  const result = await plugin.transform?.call(ctx as never, "const x = 1;", "/p/main.ts");
  t.assert.strictEqual(result, null);
});

void test("transform returns null for a node_modules .vue id", async (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });
  // isVueFile(id) is true here, but filter(id) rejects node_modules -> null.
  const result = await plugin.transform?.call(
    ctx as never,
    "<template><div>x</div></template>",
    "/p/node_modules/foo/App.vue",
  );
  t.assert.strictEqual(result, null);
});

// ---------------------------------------------------------------------------
// watchChange (cache eviction)
// ---------------------------------------------------------------------------

void test("watchChange evicts a cached .vue module so the next transform recompiles", async (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });

  // First transform caches the compiled module.
  const first = await plugin.transform?.call(ctx as never, BASIC_SFC, APP_VUE);
  t.assert.ok(first && typeof first === "object" && typeof first.code === "string");

  // watchChange is a void no-op return for a .vue id (it deletes the cache entry
  // internally) and must not throw.
  t.assert.strictEqual(plugin.watchChange?.call(ctx as never, APP_VUE), undefined);

  // A second transform after the eviction still succeeds (recompiles cleanly).
  const second = await plugin.transform?.call(ctx as never, BASIC_SFC, APP_VUE);
  t.assert.ok(second && typeof second === "object" && typeof second.code === "string");
  // Deterministic source -> identical output across the recompile.
  t.assert.strictEqual(second.code, first.code);
});

void test("watchChange is a no-op for a non-.vue id (no throw)", (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });
  t.assert.strictEqual(plugin.watchChange?.call(ctx as never, "/p/main.ts"), undefined);
});

// ---------------------------------------------------------------------------
// hostCompiler short-circuit (legacy Vue delegates everything to the host)
// ---------------------------------------------------------------------------

void test("hostCompiler short-circuits every bundler hook (vueVersion: 2)", async (t) => {
  const plugin = rawPlugin({ vueVersion: 2, isProduction: true, root: packageRoot });

  const styleRequest = "/p/App.vue?vue&type=style&index=0&lang=css";

  // resolveId returns null instead of a virtual style id.
  t.assert.strictEqual(plugin.resolveId?.call(ctx as never, styleRequest as never), null);

  // loadInclude returns false for anything.
  t.assert.strictEqual(plugin.loadInclude?.call(ctx as never, styleRequest as never), false);
  t.assert.strictEqual(plugin.loadInclude?.call(ctx as never, "/p/App.vue" as never), false);

  // load returns null even for what would otherwise be a virtual style id.
  t.assert.strictEqual(await plugin.load?.call(ctx as never, styleRequest), null);

  // transformInclude returns false for a plain .vue id.
  t.assert.strictEqual(plugin.transformInclude?.call(ctx as never, "/p/App.vue" as never), false);

  // transform resolves to null for a .vue source.
  t.assert.strictEqual(
    await plugin.transform?.call(ctx as never, "<template><div>x</div></template>", "/p/App.vue"),
    null,
  );
});

// ---------------------------------------------------------------------------
// resolveId / loadInclude with hostCompiler OFF
// ---------------------------------------------------------------------------

void test("resolveId maps a .vue style request to a virtual id string (hostCompiler off)", (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });
  const resolved = plugin.resolveId?.call(
    ctx as never,
    "/p/App.vue?vue&type=style&index=0&lang=css" as never,
  );
  t.assert.strictEqual(typeof resolved, "string");
  t.assert.strictEqual(isVirtualStyleId(resolved as string), true);
});

void test("loadInclude is true for a virtual style id and false for a plain id (hostCompiler off)", (t) => {
  const plugin = rawPlugin({ isProduction: true, root: packageRoot });
  const virtualId = plugin.resolveId?.call(
    ctx as never,
    "/p/App.vue?vue&type=style&index=0&lang=css" as never,
  ) as string;

  t.assert.strictEqual(plugin.loadInclude?.call(ctx as never, virtualId as never), true);
  t.assert.strictEqual(plugin.loadInclude?.call(ctx as never, "/p/App.vue" as never), false);
});
