import path from "node:path";
import { test } from "node:test";
import "./test/setup.ts";
import { vizeUnplugin } from "./unplugin.ts";
import { packageRoot } from "./test/helpers.ts";

const STYLE_MARKER = ".__vize_style_";

// A synthetic id is enough: the SFC source is passed directly to transform(),
// which caches the compiled module + styles in-memory, so the .vue file never
// needs to exist on disk. The path only has to end in `.vue` and avoid
// `node_modules` to satisfy the plugin's request matching.
const SYNTHETIC_ID = path.join(packageRoot, "App.vue");

function createPlugin() {
  return vizeUnplugin.raw(
    {
      isProduction: true,
      root: packageRoot,
    },
    {
      framework: "rollup",
    },
  );
}

// Drives the full virtual-style delegation pipeline against ONE fresh plugin
// instance so the in-memory transform cache is clean:
//   transform(source, id)  -> caches compiled module + style blocks
//   resolveId(importId)    -> STYLE_MARKER virtual id (carries vize-file)
//   load(virtualId)        -> { code } read back from the cache
async function runPipeline(source: string, importRegex: RegExp) {
  const plugin = createPlugin();
  const ctx = { warn() {} };

  const transformed = await plugin.transform?.call(ctx as never, source, SYNTHETIC_ID);
  if (!transformed || typeof transformed !== "object") {
    throw new Error("transform() returned no result");
  }

  const match = transformed.code.match(importRegex);
  if (!match) {
    throw new Error(`no style import matched ${importRegex} in:\n${transformed.code}`);
  }
  const importId = match[1];

  const resolvedId = await plugin.resolveId?.call({} as never, importId, SYNTHETIC_ID, {
    isEntry: false,
  });
  if (typeof resolvedId !== "string") {
    throw new Error(`resolveId() returned non-string: ${String(resolvedId)}`);
  }

  const loaded = await plugin.load?.call({} as never, resolvedId);
  if (!loaded || typeof loaded !== "object") {
    throw new Error("load() returned no result");
  }

  return {
    transformCode: transformed.code,
    importId,
    resolvedId,
    loadInclude: plugin.loadInclude?.call({} as never, resolvedId),
    loadedCss: loaded.code,
  };
}

void test("scss scoped style is delegated through transform -> resolveId -> load", async (t) => {
  const { transformCode, importId, resolvedId, loadInclude, loadedCss } = await runPipeline(
    `<template><div class="a">x</div></template>
<style scoped lang="scss">$c: red; .a { color: $c; }</style>`,
    /import "([^"]+)";/,
  );

  // transform() emits a side-effect style import with the right query.
  t.assert.match(transformCode, /import "[^"]+\.vue\?[^"]*type=style[^"]*";/);
  t.assert.match(importId, /type=style/);
  t.assert.match(importId, /lang=scss/);
  // Scoped preprocessor styles carry a scoped=data-v-<id> query.
  t.assert.match(importId, /scoped=data-v-[0-9a-f]+/);
  const scopeMatch = importId.match(/scoped=(data-v-[0-9a-f]+)/);
  t.assert.ok(scopeMatch, "expected a data-v scope id in the import query");
  const scopeId = scopeMatch[1];

  // resolveId() hands back the STYLE_MARKER virtual id.
  t.assert.ok(resolvedId.includes(STYLE_MARKER), "resolved id contains the STYLE_MARKER");
  t.assert.match(resolvedId, /\.scss\?/);
  t.assert.equal(loadInclude, true);

  // load() returns the wrapScopedPreprocessorStyle output: the original
  // (un-compiled) scss is wrapped in the scope-attribute selector so the
  // downstream preprocessor sees the scope.
  t.assert.ok(loadedCss.includes(`[${scopeId}]`), "scoped selector present in loaded css");
  t.assert.ok(loadedCss.includes("$c: red"), "raw scss content preserved for the preprocessor");
});

void test("less style (non-scoped) is delegated and returned verbatim", async (t) => {
  const { importId, resolvedId, loadInclude, loadedCss } = await runPipeline(
    `<template><div class="a">x</div></template>
<style lang="less">@c: red; .a { color: @c; }</style>`,
    /import "([^"]+)";/,
  );

  t.assert.match(importId, /type=style/);
  t.assert.match(importId, /lang=less/);
  // Not scoped: no scope query is emitted.
  t.assert.ok(!/scoped=/.test(importId), "non-scoped style has no scoped query");

  t.assert.ok(resolvedId.includes(STYLE_MARKER), "resolved id contains the STYLE_MARKER");
  t.assert.match(resolvedId, /\.less\?/);
  t.assert.equal(loadInclude, true);

  // Non-scoped preprocessor content passes through untouched (no scope wrapper).
  t.assert.ok(loadedCss.includes("@c: red"), "raw less content preserved");
  t.assert.ok(!/\[data-v-/.test(loadedCss), "no scope wrapper for a non-scoped block");
});

void test("stylus scoped style is delegated and scope-wrapped", async (t) => {
  const { importId, resolvedId, loadInclude, loadedCss } = await runPipeline(
    `<template><div class="a">x</div></template>
<style scoped lang="stylus">.a
  color red</style>`,
    /import "([^"]+)";/,
  );

  t.assert.match(importId, /type=style/);
  t.assert.match(importId, /lang=stylus/);
  t.assert.match(importId, /scoped=data-v-[0-9a-f]+/);
  const scopeMatch = importId.match(/scoped=(data-v-[0-9a-f]+)/);
  t.assert.ok(scopeMatch, "expected a data-v scope id in the import query");
  const scopeId = scopeMatch[1];

  t.assert.ok(resolvedId.includes(STYLE_MARKER), "resolved id contains the STYLE_MARKER");
  t.assert.match(resolvedId, /\.stylus\?/);
  t.assert.equal(loadInclude, true);

  t.assert.ok(loadedCss.includes(`[${scopeId}]`), "scoped selector present in loaded stylus");
  t.assert.ok(loadedCss.includes("color red"), "raw stylus content preserved");
});

void test("css-module style is delegated via a default import binding", async (t) => {
  const { transformCode, importId, resolvedId, loadInclude, loadedCss } = await runPipeline(
    `<template><div :class="$style.root">x</div></template>
<style module>.root { color: seagreen; }</style>`,
    /import \$style from "([^"]+)";/,
  );

  // css-modules produce a default import bound to $style, not a side-effect import.
  t.assert.match(transformCode, /import \$style from "[^"]+";/);
  // The compiled module also registers the binding on __cssModules.
  t.assert.match(transformCode, /__cssModules\["\$style"\] = \$style;/);

  t.assert.match(importId, /type=style/);
  t.assert.match(importId, /module=/);

  t.assert.ok(resolvedId.includes(STYLE_MARKER), "resolved id contains the STYLE_MARKER");
  t.assert.match(resolvedId, /\.module\.css\?/);
  t.assert.equal(loadInclude, true);

  // The loaded css is the (plain css) module content, non-empty.
  t.assert.ok(loadedCss.length > 0, "css-module load returns non-empty css");
  t.assert.ok(loadedCss.includes(".root"), "css-module selector present in loaded css");
});

void test("plain non-module css is inlined, not delegated", async (t) => {
  const plugin = createPlugin();
  const ctx = { warn() {} };

  const transformed = await plugin.transform?.call(
    ctx as never,
    `<template><div class="a">x</div></template>
<style>.a{color:red}</style>`,
    SYNTHETIC_ID,
  );
  t.assert.ok(transformed && typeof transformed === "object");

  // Plain css is injected inline via the __vize_css__ runtime, not delegated.
  t.assert.match(transformed.code, /export const __vize_css__ = "\.a\{color:red\}";/);
  t.assert.match(transformed.code, /__vize_css_id__/);

  // No delegated style import / css-module import is emitted.
  t.assert.ok(
    !/\?[^"]*type=style/.test(transformed.code),
    "plain css emits no delegated style import",
  );
  t.assert.ok(
    !/import \$style from/.test(transformed.code),
    "plain css emits no css-module import",
  );

  // And resolveId on a would-be style request for this plain css is moot: the
  // transform never produced one, confirming the inline path was taken.
});
