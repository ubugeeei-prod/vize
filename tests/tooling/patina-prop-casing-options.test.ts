import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { vueConfigOptions } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

interface Finding {
  ruleId: string;
  severity: number;
  message: string;
  line: number;
  column: number;
  messageId: string;
  endLine: number;
  endColumn: number;
}
interface Reference {
  verify(source: string, config: unknown[], options: { filename: string }): Finding[];
  verifyAndFix(
    source: string,
    config: unknown[],
    options: { filename: string },
  ): {
    fixed: boolean;
    output: string;
    messages: Finding[];
  };
}
interface Plugin {
  rules: Record<string, { meta: { fixable: string | null } }>;
}
interface Corpus {
  provider: Record<string, string>;
  cases: Array<{ id: string; source: string; casing: string | null; findings: Finding[] }>;
}
const require = createRequire(path.join(repoRoot, "tools/benchmarks/scripts/package.json"));
const { Linter } = require("eslint") as { Linter: new () => Reference };
const plugin = require("eslint-plugin-vue") as Plugin;
const parser: unknown = require("vue-eslint-parser");
const inputRoot = path.join(
  repoRoot,
  "tests/_fixtures/differential/linter/prop-name-casing-options-6097",
);
const corpus = JSON.parse(fs.readFileSync(path.join(inputRoot, "cases.json"), "utf8")) as Corpus;

test("authored prop casing inputs retain their pinned public-option and no-fix observations", () => {
  assert.deepEqual(corpus.provider, {
    eslint: "10.4.1",
    "eslint-plugin-vue": "10.9.2",
    "vue-eslint-parser": "10.4.1",
  });
  for (const [name, version] of Object.entries(corpus.provider)) {
    assert.equal((require(`${name}/package.json`) as { version: string }).version, version);
  }
  assert.equal(plugin.rules["prop-name-casing"]!.meta.fixable, null);
  assert.equal(corpus.cases.length, 5);
  for (const row of corpus.cases) {
    const source = fs.readFileSync(path.join(inputRoot, row.source), "utf8");
    const config = [
      {
        files: ["**/*.vue"],
        languageOptions: { parser, parserOptions: { ecmaVersion: 2022, sourceType: "module" } },
        plugins: { vue: plugin },
        rules: { "vue/prop-name-casing": ["error", ...(row.casing === null ? [] : [row.casing])] },
      },
    ];
    const linter = new Linter();
    assert.deepEqual(
      linter.verify(source, config, { filename: "Probe.vue" }),
      row.findings,
      row.id,
    );
    assert.deepEqual(
      linter.verifyAndFix(source, config, { filename: "Probe.vue" }),
      {
        fixed: false,
        output: source,
        messages: row.findings,
      },
      row.id,
    );
  }
});

test("canonical public config accepts only the declared prop casing policy", () => {
  for (const casing of ["camelCase", "snake_case"]) {
    const config = { vue: { linter: { ruleOptions: { "vue/prop-name-casing": { casing } } } } };
    const original = JSON.stringify(config);
    assert.deepEqual(vueConfigOptions(config), config.vue);
    assert.equal(JSON.stringify(config), original);
  }
  assert.doesNotThrow(() =>
    vueConfigOptions({ vue: { linter: { ruleOptions: { "vue/prop-name-casing": {} } } } }),
  );
  for (const [option, reason] of [
    ["snake_case", "must be object"],
    [{ casing: "snake-case" }, "casing: unsupported native option value"],
    [{ casing: false }, "casing: unsupported native option value"],
    [{ casing: null }, "casing: unsupported native option value"],
    [{ ignoreProps: ["authoredName"] }, 'unknown option "ignoreProps"'],
  ] as const) {
    const config = { vue: { linter: { ruleOptions: { "vue/prop-name-casing": option } } } };
    const separator = reason.startsWith("casing:") ? "." : ": ";
    assert.throws(() => vueConfigOptions(config), {
      name: "TypeError",
      message: `vue.linter.ruleOptions.vue/prop-name-casing${separator}${reason}`,
    });
  }
});
