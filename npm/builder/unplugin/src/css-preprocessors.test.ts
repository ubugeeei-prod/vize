import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import "./test/setup.ts";
import { vizeUnplugin } from "./unplugin.ts";
import { createFakePreprocessorRoot } from "./test/fake-preprocessors.ts";
import { packageRoot } from "./test/helpers.ts";

// A synthetic id is enough: the SFC source is passed directly to transform(),
// which caches the compiled module + styles in-memory, so the .vue file never
// needs to exist on disk. The path only has to end in `.vue` and avoid
// `node_modules` to satisfy the plugin's request matching.
const SYNTHETIC_ID = path.join(packageRoot, "App.vue");

function createPlugin(root = createFakePreprocessorRoot("css-preprocessors")) {
  return vizeUnplugin.raw(
    {
      isProduction: true,
      root,
    },
    {
      framework: "rollup",
    },
  );
}

// Drives the full virtual-style delegation pipeline against ONE fresh plugin
// instance so the in-memory transform cache is clean:
//   transform(source, id)  -> caches compiled module + style blocks
//   resolveId(importId)    -> the same id, claimed by the plugin
//   load(virtualId)        -> { code } read back from the cache
async function runPipeline(source: string, importRegex: RegExp, id = SYNTHETIC_ID) {
  const plugin = createPlugin(id === SYNTHETIC_ID ? undefined : path.dirname(id));
  const watched: string[] = [];
  const ctx = { warn() {}, addWatchFile: (file: string) => watched.push(file) };

  const transformed = await plugin.transform?.call(ctx as never, source, id);
  if (!transformed || typeof transformed !== "object") {
    throw new Error("transform() returned no result");
  }

  const match = transformed.code.match(importRegex);
  if (!match) {
    throw new Error(`no style import matched ${importRegex} in:\n${transformed.code}`);
  }
  const importId = match[1];

  const resolvedId = await plugin.resolveId?.call({} as never, importId, id, {
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
    watched,
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
  // Named as plain CSS: load() runs the project's preprocessor before scoping.
  t.assert.match(importId, /&lang\.css$/);
  // Scoped preprocessor styles carry a scoped=data-v-<id> query.
  t.assert.match(importId, /scoped=data-v-[0-9a-f]+/);
  const scopeMatch = importId.match(/scoped=(data-v-[0-9a-f]+)/);
  t.assert.ok(scopeMatch, "expected a data-v scope id in the import query");
  const scopeId = scopeMatch[1];

  t.assert.equal(resolvedId, importId);
  t.assert.equal(loadInclude, true);

  t.assert.ok(loadedCss.includes(`.a[${scopeId}]`), "compiled rule scoped in place");
  t.assert.ok(!loadedCss.includes("$c"), "preprocessor ran before scoping");
});

void test("less style (non-scoped) is delegated and returned verbatim", async (t) => {
  const { importId, resolvedId, loadInclude, loadedCss } = await runPipeline(
    `<template><div class="a">x</div></template>
<style lang="less">@c: red; .a { color: @c; }</style>`,
    /import "([^"]+)";/,
  );

  t.assert.match(importId, /type=style/);
  t.assert.match(importId, /&lang\.less$/);
  // Not scoped: no scope query is emitted.
  t.assert.ok(!/scoped=/.test(importId), "non-scoped style has no scoped query");

  t.assert.equal(resolvedId, importId);
  t.assert.equal(loadInclude, true);

  // Non-scoped preprocessor content passes through untouched (no scope wrapper).
  t.assert.ok(loadedCss.includes("@c: red"), "raw less content preserved");
  t.assert.ok(!/\[data-v-/.test(loadedCss), "no scope wrapper for a non-scoped block");
});

void test("stylus scoped style is preprocessed, then scoped", async (t) => {
  const { importId, resolvedId, loadInclude, loadedCss } = await runPipeline(
    `<template><div class="a">x</div></template>
<style scoped lang="stylus">.a
  color red</style>`,
    /import "([^"]+)";/,
  );

  t.assert.match(importId, /type=style/);
  t.assert.match(importId, /&lang\.css$/);
  t.assert.match(importId, /scoped=data-v-[0-9a-f]+/);
  const scopeMatch = importId.match(/scoped=(data-v-[0-9a-f]+)/);
  t.assert.ok(scopeMatch, "expected a data-v scope id in the import query");
  const scopeId = scopeMatch[1];

  t.assert.equal(resolvedId, importId);
  t.assert.equal(loadInclude, true);

  t.assert.ok(loadedCss.includes(`.a[${scopeId}]`), "compiled stylus rule scoped in place");
  t.assert.ok(loadedCss.includes("color: red"), "stylus compiled to CSS");
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
  t.assert.match(importId, /&module&lang\.module\.css$/);
  t.assert.equal(resolvedId, importId);
  t.assert.equal(loadInclude, true);

  // The loaded css is the (plain css) module content, non-empty.
  t.assert.ok(loadedCss.length > 0, "css-module load returns non-empty css");
  t.assert.ok(loadedCss.includes(".root"), "css-module selector present in loaded css");
});

void test("plain css is delegated through the same style module", async (t) => {
  const { transformCode, importId, resolvedId, loadInclude, loadedCss } = await runPipeline(
    `<template><div class="a">x</div></template>
<style scoped>.a{color:red}</style>`,
    /import "([^"]+)";/,
  );

  t.assert.match(importId, /\?vue&type=style&index=0&scoped=data-v-[0-9a-f]+&lang\.css$/);
  t.assert.equal(resolvedId, importId);
  t.assert.equal(loadInclude, true);
  // The host writes the CSS; nothing is injected at run time.
  t.assert.doesNotMatch(transformCode, /__vize_css__|document\.createElement/);
  const scopeId = importId.match(/scoped=(data-v-[0-9a-f]+)/)?.[1];
  t.assert.ok(loadedCss.includes(`.a[${scopeId}]{color:red}`), loadedCss);
});

void test("#11456 scss and css-module src blocks delegate the file's content", async (t) => {
  const root = createFakePreprocessorRoot("css-preprocessors-src");
  fs.mkdirSync(path.join(root, "styles"), { recursive: true });
  fs.writeFileSync(path.join(root, "styles", "a.scss"), "$c: red; .a { color: $c; }");
  fs.writeFileSync(path.join(root, "styles", "m.css"), ".root { color: seagreen; }");
  const id = path.join(root, "App.vue");

  const scss = await runPipeline(
    `<template><div class="a">x</div></template>
<style scoped lang="scss" src="./styles/a.scss"></style>`,
    /import "([^"]+)";/,
    id,
  );
  const scopeId = scss.importId.match(/scoped=(data-v-[0-9a-f]+)/)?.[1];
  t.assert.ok(scopeId, "scoped query on the delegated request");
  t.assert.match(scss.importId, /&lang\.css$/);
  t.assert.ok(scss.loadedCss.includes(`.a[${scopeId}]`), "file content preprocessed and scoped");
  t.assert.ok(!scss.loadedCss.includes("$c"), "preprocessor ran on the file content");
  t.assert.deepEqual(scss.watched, [path.join(root, "styles", "a.scss")]);

  const cssModule = await runPipeline(
    `<template><div :class="$style.root">x</div></template>
<style module src="./styles/m.css"></style>`,
    /import \$style from "([^"]+)";/,
    id,
  );
  t.assert.match(cssModule.transformCode, /__cssModules\["\$style"\] = \$style;/);
  t.assert.match(cssModule.resolvedId, /&lang\.module\.css$/);
  t.assert.ok(
    cssModule.loadedCss.includes(".root { color: seagreen; }"),
    "module file content loaded",
  );
  t.assert.deepEqual(cssModule.watched, [path.join(root, "styles", "m.css")]);
});
