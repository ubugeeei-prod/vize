import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { compileVueModule } from "./compiler.ts";
import { assertSupportedStyleDependencies } from "./style-dependencies.ts";
import { generateOutput } from "./style.ts";
import { normalizeOptions } from "./unplugin.ts";
import type { CachedCompiledModule, StyleBlockNapi } from "./types.ts";

function block(content: string, extra: Partial<StyleBlockNapi> = {}): StyleBlockNapi {
  return { content, scoped: false, module: false, index: 0, ...extra };
}

void test("external style dependency guards preserve lexical and delegated CSS boundaries", (t) => {
  const cases = [
    { id: "scoped-src", style: block("", { scoped: true, src: "./foo.css" }), error: "src" },
    { id: "plain-src", style: block("", { src: "./foo.css" }), error: "src" },
    { id: "empty-src", style: block("", { src: "" }), error: "src" },
    { id: "module-src", style: block("", { src: "./foo.css", module: true }), error: "src" },
    { id: "preprocessor-src", style: block("", { src: "./foo.scss", lang: "scss" }), error: "src" },
    { id: "import", style: block('@import "./foo.css";'), error: "import" },
    {
      id: "scoped-import",
      style: block('@import url("./foo.css");', { scoped: true }),
      error: "import",
    },
    { id: "escaped-import", style: block('@\\69mport "./foo.css";'), error: "import" },
    { id: "uppercase-import", style: block('@IMPORT "./foo.css";'), error: "import" },
    { id: "plain-inline", style: block(".foo { color: red; }"), error: null },
    { id: "comment", style: block('/* @import "./foo.css"; */ .foo {color:red}'), error: null },
    { id: "string", style: block(".foo::after {content: '@import \"./foo.css\";'}"), error: null },
    { id: "media", style: block("@media (width > 1px) {.foo {color:red}}"), error: null },
    { id: "module-inline", style: block('@import "./foo.css";', { module: true }), error: null },
    { id: "scss-inline", style: block('@import "./foo.scss";', { lang: "scss" }), error: null },
  ] as const;
  const outcomes = cases.map((item) => {
    try {
      assertSupportedStyleDependencies([item.style], "/project/Foo.vue");
      return { ...item, outcome: { state: "returned" } };
    } catch (error) {
      return {
        ...item,
        outcome: {
          state: "threw",
          error:
            error instanceof Error
              ? { name: error.name, message: error.message, stack: error.stack }
              : error,
        },
      };
    }
  });
  t.diagnostic(JSON.stringify({ schema: "vize.unplugin.styleDependencyGuards.v1", outcomes }));
  for (const actual of outcomes) {
    if (actual.error === null) {
      assert.deepEqual(actual.outcome, { state: "returned" }, actual.id);
    } else {
      assert.equal(actual.outcome.state, "threw", actual.id);
      assert.match(
        JSON.stringify(actual.outcome),
        actual.error === "src" ? /<style src=/ : /CSS @import/,
      );
      assert.match(JSON.stringify(actual.outcome), /@vizejs\/vite-plugin/);
    }
  }
});

void test("#11456 compilation inlines style src, keeps inline output and refuses imports before caching", (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-style-src-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.writeFileSync(path.join(root, "foo.css"), ".foo {\n  color: red;\n}\n");
  const filePath = path.join(root, "Foo.vue");
  const options = normalizeOptions({ isProduction: true, root, sourceMap: true });
  const cases: Array<
    { id: string; source: string } & (
      | { expected: { scoped: boolean; dependencies: string[] } }
      | { error: RegExp }
    )
  > = [
    {
      id: "reported-src",
      source:
        '<template><button class="foo">Foo</button></template>\n<style scoped src="./foo.css"></style>',
      expected: { scoped: true, dependencies: [path.join(root, "foo.css")] },
    },
    {
      id: "plain-src",
      source: '<template><button class="foo" /></template><style src="./foo.css"></style>',
      expected: { scoped: false, dependencies: [path.join(root, "foo.css")] },
    },
    {
      id: "self-closing-src",
      source:
        "<template><button class=\"foo\" /></template><style scoped src='./foo.css' /><style>.b {color:red}</style>",
      expected: { scoped: true, dependencies: [path.join(root, "foo.css")] },
    },
    {
      id: "missing-src",
      source: '<template><button /></template><style scoped src="./missing.css"></style>',
      error: /<style src="\.\/missing\.css"> not found/,
    },
    {
      id: "inline-import",
      source: '<template><button /></template><style scoped>@import "./foo.css";</style>',
      error: /CSS @import in <style> is not supported/,
    },
    {
      id: "script-literal",
      source:
        "<script setup>const source = '<style src=\"./missing.css\"></style>'</script><template>{{ source }}</template><style scoped>.foo {color:red}</style>",
      expected: { scoped: true, dependencies: [] },
    },
    {
      id: "comment-literal",
      source:
        '<template><button /></template><!-- <style src="./missing.css"></style> --><style>.foo {color:red}</style>',
      expected: { scoped: false, dependencies: [] },
    },
    {
      id: "inline-scoped",
      source:
        '<template><button class="foo">Foo</button></template><style scoped>.foo { color: red; }</style>',
      expected: { scoped: true, dependencies: [] },
    },
  ];
  const outcomes = cases.map((item) => {
    const cache = new Map<string, CachedCompiledModule>();
    try {
      const first = compileVueModule(filePath, item.source, options, cache);
      const second = compileVueModule(filePath, item.source, options, cache);
      const output = generateOutput(first.compiled, { isProduction: true, isDev: false, filePath });
      return { item, first, second, output, cacheSize: cache.size, message: null };
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      return { item, first: null, second: null, output: "", cacheSize: cache.size, message };
    }
  });
  t.diagnostic(
    JSON.stringify({
      schema: "vize.unplugin.styleSrcCompilation.v2",
      outcomes: outcomes.map(({ item, first, message }) => ({
        id: item.id,
        css: first?.compiled.css ?? null,
        dependencies: first?.dependencies ?? null,
        message,
      })),
    }),
  );
  for (const { item, first, second, output, cacheSize, message } of outcomes) {
    if ("error" in item) {
      assert.match(message ?? "", item.error, item.id);
      assert.equal(cacheSize, 0, item.id);
      continue;
    }
    assert.equal(message, null, item.id);
    assert.ok(first && second, item.id);
    assert.equal(second.compiled, first.compiled, item.id);
    assert.deepEqual(first.dependencies, item.expected.dependencies, item.id);
    assert.deepEqual(second.dependencies, item.expected.dependencies, item.id);
    assert.ok(
      first.compiled.styles.every((style) => style.src === null),
      item.id,
    );
    assert.match(output, /__vize_css__/, item.id);
    assert.match(first.compiled.css ?? "", /color:\s*red/, item.id);
    assert.equal(/\.foo\[data-v-/.test(first.compiled.css ?? ""), item.expected.scoped, item.id);
  }
});

void test("#11456 an edited style src file invalidates the compile cache", (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-style-src-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const css = path.join(root, "foo.css");
  const filePath = path.join(root, "Foo.vue");
  const source = '<template><b class="foo" /></template><style scoped src="./foo.css"></style>';
  const options = normalizeOptions({ isProduction: true, root });
  const cache = new Map<string, CachedCompiledModule>();
  fs.writeFileSync(css, ".foo { color: red; }");
  const first = compileVueModule(filePath, source, options, cache).compiled;
  fs.writeFileSync(css, ".foo { color: blue; }");
  const second = compileVueModule(filePath, source, options, cache).compiled;
  assert.match(first.css ?? "", /red/);
  assert.match(second.css ?? "", /blue/);
  assert.doesNotMatch(second.css ?? "", /red/);
  assert.equal(second.scopeId, first.scopeId);
});
