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

function rangeAt(source: string, offset: number, text: string) {
  assert.equal(source.slice(offset, offset + text.length), text);
  return {
    line: 1,
    column: offset + 1,
    endLine: 1,
    endColumn: offset + text.length + 1,
  };
}

function aliasRange(source: string, directive: string, alias: string) {
  const offset = source.indexOf(directive);
  assert.notEqual(offset, -1, `missing expected range target: ${directive}${alias}`);
  return rangeAt(source, offset + directive.length, alias);
}

const eventParameterShadowSource = `<template><button v-for="event in events" @click="(event) => handle(event)"></button></template>`;

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
  {
    id: "event-parameter-shadows-outer-alias",
    source: eventParameterShadowSource,
    diagnostics: ["'event' is defined but never used."],
    ranges: [aliasRange(eventParameterShadowSource, `v-for="`, "event")],
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
      .map((message) => ({
        message: message.message,
        range: {
          line: message.line,
          column: message.column,
          endLine: message.endLine,
          endColumn: message.endColumn,
        },
      }));
    const parserErrors = result.messages.filter((message) => message.ruleId == null);
    const messages = diagnostics.map((diagnostic) => diagnostic.message);
    const ranges = diagnostics.map((diagnostic) => diagnostic.range);

    assert.deepEqual(parserErrors, [], `${fixture.id}: parser errors`);
    assert.deepEqual(messages, fixture.diagnostics, fixture.id);
    if ("ranges" in fixture) {
      assert.deepEqual(ranges, fixture.ranges, `${fixture.id}: diagnostic ranges`);
    }
  }
});
