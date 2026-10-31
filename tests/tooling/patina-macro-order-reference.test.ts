import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";

import {
  corpus,
  filename,
  hash,
  root,
  sourceFor,
  type Finding,
} from "./support/patina-macro-order-contract.ts";
import {
  corpus as exposeCorpus,
  filename as exposeFilename,
  sourceFor as exposeSourceFor,
} from "./support/patina-expose-last-contract.ts";

interface Reference {
  verify(source: string, config: unknown[], options: { filename: string }): Finding[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}

test("original macro bodies retain complete pinned reference diagnostics and fixes", () => {
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
  const plugin = require("eslint-plugin-vue") as {
    rules: Record<string, { meta: { fixable: string; hasSuggestions: boolean } }>;
  };
  const parser: unknown = require("vue-eslint-parser");
  const scriptParser: unknown = require("@typescript-eslint/parser");
  assert.equal(plugin.rules["define-macros-order"]!.meta.fixable, "code");
  assert.equal(plugin.rules["define-macros-order"]!.meta.hasSuggestions, true);
  const directory = path.dirname(require.resolve("eslint-plugin-vue/package.json"));
  assert.equal(
    hash(fs.readFileSync(path.join(directory, corpus.ruleSource.path))),
    corpus.ruleSource.sha256,
  );
  const original = fs.readFileSync(path.join(root, corpus.provenance.originalTests), "utf8");
  for (const pin of corpus.provenance.sources) {
    const body = original
      .split(`fn ${pin.originalTest}()`)[1]!
      .split('let source = r#"')[1]!
      .split('"#;')[0]!;
    assert.equal(hash(body), pin.originalScriptSha256);
    const row = corpus.cases.find((entry) => entry.source === pin.file)!;
    assert.equal(
      sourceFor(row),
      '<script setup lang="ts">' + body + "</script>\n<template><span /></template>\n",
    );
  }
  let fixed = 0;
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
        rules: {
          "vue/define-macros-order": ["error", ...(row.options === null ? [] : [row.options])],
        },
      },
    ];
    const linter = new Linter();
    assert.deepEqual(linter.verify(source, config, { filename }), row.reference.messages, row.id);
    assert.deepEqual(
      linter.verifyAndFix(source, config, { filename }),
      row.reference.fixResult,
      row.id,
    );
    fixed += Number(row.reference.fixResult.fixed);
    assert.equal(hash(sourceFor(row)), hash(source), row.id);
  }
  assert.equal(fixed, 3, "real upstream fix outputs must not be erased to claim Vize fixer parity");
});

test("defineExposeLast retains all seven complete independent messages and suggestion/fix records", () => {
  const require = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
  assert.deepEqual(exposeCorpus.provider, corpus.provider);
  for (const [name, version] of Object.entries(exposeCorpus.provider)) {
    assert.equal((require(`${name}/package.json`) as { version: string }).version, version);
  }
  const { Linter } = require("eslint") as { Linter: new () => Reference };
  const plugin: unknown = require("eslint-plugin-vue");
  const parser: unknown = require("vue-eslint-parser");
  const scriptParser: unknown = require("@typescript-eslint/parser");
  const providerDirectory = path.dirname(require.resolve("eslint-plugin-vue/package.json"));
  assert.equal(
    hash(fs.readFileSync(path.join(providerDirectory, exposeCorpus.ruleSource.path))),
    exposeCorpus.ruleSource.sha256,
  );
  let fixed = 0;
  for (const row of exposeCorpus.cases) {
    const source = exposeSourceFor(row);
    const config = [
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser,
          parserOptions: { parser: scriptParser, ecmaVersion: 2022, sourceType: "module" },
        },
        plugins: { vue: plugin },
        rules: { "vue/define-macros-order": ["error", row.options] },
      },
    ];
    const linter = new Linter();
    assert.deepEqual(
      linter.verify(source, config, { filename: exposeFilename }),
      row.reference.messages,
      row.id,
    );
    assert.deepEqual(
      linter.verifyAndFix(source, config, { filename: exposeFilename }),
      row.reference.fixResult,
      row.id,
    );
    fixed += Number(row.reference.fixResult.fixed);
    assert.equal(hash(exposeSourceFor(row)), hash(source), row.id);
  }
  assert.equal(fixed, 2, "only the independent prefix-order findings have automatic fixes");
});
