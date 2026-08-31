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
    id: "legacy-slot-attribute",
    source: `<template><MyComponent><template slot="header">Title</template></MyComponent></template>`,
    diagnostics: 0,
  },
  {
    id: "legacy-slot-scope-attribute",
    source: `<template><MyComponent><template slot-scope="props">{{ props.title }}</template></MyComponent></template>`,
    diagnostics: 0,
  },
  {
    id: "legacy-scope-attribute",
    source: `<template><MyComponent><template scope="props">{{ props.title }}</template></MyComponent></template>`,
    diagnostics: 0,
  },
  {
    id: "static-bound-slot",
    source: `<template><MyComponent><template v-bind:slot="name">Title</template></MyComponent></template>`,
    diagnostics: 0,
  },
  {
    id: "static-bound-slot-shorthand",
    source: `<template><MyComponent><template :slot="name">Title</template></MyComponent></template>`,
    diagnostics: 0,
  },
  {
    id: "dynamic-bound-slot",
    source: `<template><MyComponent><template v-bind:[slot]="name">Title</template></MyComponent></template>`,
    diagnostics: 1,
  },
] as const;

test("no-lone-template slot boundary stays pinned to eslint-plugin-vue 10.9.2", async () => {
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
        rules: { "vue/no-lone-template": "warn" },
      },
    ],
  });

  for (const fixture of cases) {
    const [result] = await eslint.lintText(fixture.source, {
      filePath: path.join(repoRoot, "oracle", `${fixture.id}.vue`),
    });
    const diagnostics = result.messages.filter(
      (message) => message.ruleId === "vue/no-lone-template",
    );
    const parserErrors = result.messages.filter((message) => message.ruleId == null);

    assert.deepEqual(parserErrors, [], `${fixture.id}: parser errors`);
    assert.equal(diagnostics.length, fixture.diagnostics, fixture.id);
  }
});
