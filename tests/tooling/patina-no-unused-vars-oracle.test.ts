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
    id: "leading-value-alias-needed-for-index",
    source: `<template><div v-for="(entry, index) in 20" :key="index">{{ index }}</div></template>`,
    diagnostics: [],
  },
  {
    id: "leading-value-and-key-aliases-needed-for-index",
    source: `<template><div v-for="(entry, key, index) in items" :key="index">{{ index }}</div></template>`,
    diagnostics: [],
  },
  {
    id: "trailing-index-remains-reportable",
    source: `<template><div v-for="(entry, index) in items" :key="entry">{{ entry }}</div></template>`,
    diagnostics: ["'index' is defined but never used."],
  },
  {
    id: "destructured-value-bindings-remain-reportable",
    source: `<template><div v-for="({ id, label }, index) in items" :key="index">{{ index }}</div></template>`,
    diagnostics: ["'id' is defined but never used.", "'label' is defined but never used."],
  },
] as const;

test("no-unused-vars v-for tuple boundary stays pinned to eslint-plugin-vue 10.9.2", async () => {
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
        rules: { "vue/no-unused-vars": "warn" },
      },
    ],
  });

  for (const fixture of cases) {
    const [result] = await eslint.lintText(fixture.source, {
      filePath: path.join(repoRoot, "oracle", `${fixture.id}.vue`),
    });
    const diagnostics = result.messages
      .filter((message) => message.ruleId === "vue/no-unused-vars")
      .map((message) => message.message);
    const parserErrors = result.messages.filter((message) => message.ruleId == null);

    assert.deepEqual(parserErrors, [], `${fixture.id}: parser errors`);
    assert.deepEqual(diagnostics, fixture.diagnostics, fixture.id);
  }
});
