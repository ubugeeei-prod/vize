import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import {
  corpus,
  expected,
  filename,
  hash,
  root,
  repaired,
  sourceFor,
  type Finding,
} from "./support/patina-v-bind-prop-contract.ts";

interface Reference {
  verify(source: string, config: unknown[], options: { filename: string }): Finding[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}

test("v-bind property shorthand retains nine complete pinned upstream fixer and fixed-point records", () => {
  const require = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
  assert.deepEqual(corpus.provider, {
    eslint: "10.4.1",
    "eslint-plugin-vue": "10.9.2",
    "vue-eslint-parser": "10.4.1",
    "@typescript-eslint/parser": "8.65.0",
  });
  for (const [name, version] of Object.entries(corpus.provider)) {
    assert.equal((require(`${name}/package.json`) as { version: string }).version, version);
  }
  const { Linter } = require("eslint") as { Linter: new () => Reference };
  const plugin: unknown = require("eslint-plugin-vue");
  const parser: unknown = require("vue-eslint-parser");
  const scriptParser: unknown = require("@typescript-eslint/parser");
  assert.equal(
    hash(
      fs.readFileSync(
        path.join(
          path.dirname(require.resolve("eslint-plugin-vue/package.json")),
          corpus.ruleSource.path,
        ),
      ),
    ),
    corpus.ruleSource.sha256,
  );
  let findings = 0;
  for (const row of corpus.cases) {
    const source = sourceFor(row);
    const config = [
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser,
          parserOptions: { parser: scriptParser, ecmaVersion: 2022, sourceType: "module" },
        },
        plugins: { vue: plugin },
        rules: { "vue/v-bind-style": ["warn", ...(row.style === null ? [] : [row.style])] },
      },
    ];
    const linter = new Linter();
    assert.deepEqual(linter.verify(source, config, { filename }), row.reference.messages, row.id);
    assert.deepEqual(
      linter.verifyAndFix(source, config, { filename }),
      row.reference.fixResult,
      row.id,
    );
    assert.deepEqual(
      linter.verify(row.reference.fixResult.output, config, { filename }),
      row.reference.fixedRequery,
      row.id,
    );
    assert.deepEqual(
      linter.verifyAndFix(row.reference.fixResult.output, config, { filename }),
      row.reference.fixedPoint,
      row.id,
    );
    assert.equal(repaired(source, expected(row)), row.reference.fixResult.output, row.id);
    findings += row.reference.messages.length;
  }
  assert.equal(findings, 9);
});
