import assert from "node:assert/strict";
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

void test("source-aware compilation refuses external CSS before cache insertion and retains inline output", (t) => {
  const options = normalizeOptions({ isProduction: true, root: "/project", sourceMap: true });
  const cases = [
    {
      id: "reported-src",
      source:
        '<template><button class="foo">Foo</button></template>\n<style scoped src="./foo.css"></style>',
      refused: true,
    },
    {
      id: "inline-import",
      source: '<template><button /></template><style scoped>@import "./foo.css";</style>',
      refused: true,
    },
    {
      id: "script-literal",
      source:
        "<script setup>const source = '<style src=\"./foo.css\"></style>'</script><template>{{ source }}</template><style scoped>.foo {color:red}</style>",
      refused: false,
    },
    {
      id: "comment-literal",
      source:
        '<template><button /></template><!-- <style src="./foo.css"></style> --><style>.foo {color:red}</style>',
      refused: false,
    },
    {
      id: "inline-scoped",
      source:
        '<template><button class="foo">Foo</button></template><style scoped>.foo { color: red; }</style>',
      refused: false,
    },
  ];
  const outcomes = cases.map((item) => {
    const cache = new Map<string, CachedCompiledModule>();
    try {
      const first = compileVueModule("/project/Foo.vue", item.source, options, cache);
      const second = compileVueModule("/project/Foo.vue", item.source, options, cache);
      const output = generateOutput(first.compiled, {
        isProduction: true,
        isDev: false,
        filePath: "/project/Foo.vue",
      });
      return { ...item, state: "returned", first, second, output, cache: [...cache] };
    } catch (error) {
      return {
        ...item,
        state: "threw",
        error:
          error instanceof Error
            ? { name: error.name, message: error.message, stack: error.stack }
            : error,
        cache: [...cache],
      };
    }
  });
  t.diagnostic(JSON.stringify({ schema: "vize.unplugin.styleDependencyCompilation.v1", outcomes }));
  for (const actual of outcomes) {
    if (actual.refused) {
      assert.equal(actual.state, "threw", actual.id);
      assert.deepEqual(actual.cache, [], actual.id);
      assert.match(JSON.stringify(actual.error), /not supported by @vizejs\/unplugin/);
    } else {
      assert.equal(actual.state, "returned", actual.id);
      assert.deepEqual(actual.second?.compiled, actual.first?.compiled, actual.id);
      assert.match(actual.output ?? "", /__vize_css__/);
      assert.match(actual.first?.compiled.css ?? "", /color:\s*red/);
      if (actual.id !== "comment-literal") {
        assert.match(actual.first?.compiled.css ?? "", /\[data-v-/);
      }
    }
  }
});
