import assert from "node:assert/strict";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const requireFromBench = createRequire(
  path.join(repoRoot, "tools", "benchmarks", "scripts", "package.json"),
);
const { ESLint } = requireFromBench("eslint") as typeof import("eslint");
const pluginVue = requireFromBench("eslint-plugin-vue");
const vueParser = requireFromBench("vue-eslint-parser");
const typescriptParser = requireFromBench("@typescript-eslint/parser");

const cases = [
  {
    id: "filename-inferred-html-component",
    source: `<script setup></script><template><div /></template>`,
    diagnostics: 0,
  },
  {
    id: "explicit-html-name",
    source: `<script>export default { name: "button" }</script><template><div /></template>`,
    diagnostics: 1,
  },
  {
    id: "static-template-literal-name",
    source: "<script>export default { [`name`]: `button` }</script><template><div /></template>",
    diagnostics: 1,
  },
  {
    id: "registered-pascal-html-components",
    source: `<script>
export default {
  components: {
    Title,
    Link,
    Header: SiteHeader,
  },
}
</script>
<template><div /></template>`,
    diagnostics: 3,
  },
  {
    id: "registered-static-expression-keys",
    source:
      "<script>export default { [`components`]: { 'font-face': FontFace, [`missing-glyph`]: MissingGlyph } }</script><template><div /></template>",
    diagnostics: 2,
  },
  {
    id: "registered-dynamic-expression-keys",
    source:
      "<script>const Button = 'CustomButton'; export default { components: { [Button]: Button, [`button-${kind}`]: Dynamic } }</script><template><div /></template>",
    diagnostics: 0,
  },
  {
    id: "vue-component-static-registration",
    source: "<script>Vue.component(`button`, {})</script><template><div /></template>",
    diagnostics: 1,
  },
  {
    id: "identifier-component-static-registration",
    source:
      "<script>const app = createApp({}); app.component('button', {}); foo.component('Title', {})</script><template><div /></template>",
    diagnostics: 2,
  },
  {
    id: "vue-component-dynamic-registration",
    source: "<script>Vue.component(`button-${kind}`, {})</script><template><div /></template>",
    diagnostics: 0,
  },
] as const;

test("no-reserved-component-names stays pinned to eslint-plugin-vue 10.9.2", async () => {
  assert.equal(requireFromBench("eslint-plugin-vue/package.json").version, "10.9.2");
  assert.equal(requireFromBench("vue-eslint-parser/package.json").version, "10.4.1");

  const eslint = new ESLint({
    cwd: repoRoot,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser: vueParser,
          parserOptions: {
            parser: typescriptParser,
            ecmaVersion: "latest",
            sourceType: "module",
            extraFileExtensions: [".vue"],
          },
        },
        plugins: { vue: pluginVue },
        rules: { "vue/no-reserved-component-names": "error" },
      },
    ],
  });

  for (const fixture of cases) {
    const [result] = await eslint.lintText(fixture.source, {
      filePath: path.join(repoRoot, "oracle", `${fixture.id}.vue`),
    });
    const diagnostics = result.messages.filter(
      (message) => message.ruleId === "vue/no-reserved-component-names",
    );
    const parserErrors = result.messages.filter((message) => message.ruleId == null);

    assert.deepEqual(parserErrors, [], `${fixture.id}: parser errors`);
    assert.equal(diagnostics.length, fixture.diagnostics, fixture.id);
  }
});
