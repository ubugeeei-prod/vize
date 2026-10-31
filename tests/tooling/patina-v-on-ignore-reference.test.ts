import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import {
  corpus,
  expected,
  hash,
  referenceBytes,
  referenceFor,
  referenceOptions,
  root,
  rowFor,
  rule,
  sourceFor,
} from "./support/patina-v-on-ignore-contract.ts";

interface Linter {
  verify(source: string, config: unknown[], options: { filename: string }): unknown[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}

test("event ignore retains thirteen whole pinned reference outcomes and ten whole schema refusals", () => {
  const require = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
  assert.deepEqual(corpus.provider, {
    eslint: "10.4.1",
    "eslint-plugin-vue": "10.9.2",
    "vue-eslint-parser": "10.4.1",
    "@typescript-eslint/parser": "8.65.0",
  });
  for (const [name, version] of Object.entries(corpus.provider))
    assert.equal((require(name + "/package.json") as { version: string }).version, version);
  const { Linter } = require("eslint") as { Linter: new () => Linter };
  const plugin: unknown = require("eslint-plugin-vue");
  const parser: unknown = require("vue-eslint-parser");
  const scriptParser: unknown = require("@typescript-eslint/parser");
  assert.equal(
    corpus.ruleSource.sha256,
    "f91e547b8e0275dac03081fab804190e619fe87ee376302b50814f929c503849",
  );
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
  const config = (policy: unknown) => [
    {
      files: ["**/*.vue"],
      languageOptions: {
        parser,
        parserOptions: { parser: scriptParser, ecmaVersion: 2022, sourceType: "module" },
      },
      plugins: { vue: plugin },
      rules: { [rule]: ["warn", ...referenceOptions(policy)] },
    },
  ];
  for (const row of corpus.valid) {
    const linter = new Linter();
    const source = sourceFor(row);
    const options = { filename: row.filename };
    const reference = referenceFor(row);
    assert.deepEqual(
      linter.verify(source, config(row.policy), options),
      reference.messages,
      row.id,
    );
    assert.deepEqual(
      linter.verifyAndFix(source, config(row.policy), options),
      reference.fixResult,
      row.id,
    );
    assert.deepEqual(linter.verify(source, config(row.policy), options), reference.requery, row.id);
  }
  const observe = (action: () => unknown) => {
    try {
      return { kind: "returned", value: action() };
    } catch (error) {
      assert.ok(error instanceof Error);
      return { kind: "threw", error: { name: error.name, message: error.message } };
    }
  };
  for (const row of corpus.invalid) {
    const linter = new Linter();
    const source = sourceFor(row);
    const options = { filename: row.filename };
    assert.deepEqual(
      {
        verify: observe(() => linter.verify(source, config(row.policy), options)),
        verifyAndFix: observe(() => linter.verifyAndFix(source, config(row.policy), options)),
      },
      referenceBytes(row),
      row.id,
    );
  }
});

test("original whole ranges, restoration and literal JS string boundaries remain independently pinned", () => {
  const baseline = expected(rowFor("original-default"));
  assert.deepEqual(
    baseline.diagnostics.map((finding) => [
      finding.location.start.offset,
      finding.location.end.offset,
    ]),
    [
      [135, 172],
      [279, 311],
    ],
  );
  assert.deepEqual(
    expected(rowFor("original-ignore-update")).diagnostics,
    baseline.diagnostics.slice(1),
  );
  assert.deepEqual(expected(rowFor("original-restored-empty")), baseline);
  assert.deepEqual(expected(rowFor("original-literal-not-glob")), baseline);
  assert.deepEqual(expected(rowFor("original-case-sensitive")), baseline);
  assert.equal(/:exit$/.test(":exit\n"), false);
  assert.equal(/^\s*$/.test("\uFEFF"), true);
  assert.equal(/^\s*$/.test("\u0085"), false);
});
