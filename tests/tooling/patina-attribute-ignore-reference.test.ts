import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { baselineConfig } from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";
import {
  applyExpectedEdits,
  corpus,
  hash,
  original,
  referenceFor,
  root,
  rule,
  sourceFor,
  type ReferenceResult,
} from "./support/patina-attribute-ignore-contract.ts";

const require = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
const runtime = {
  ESLint: require("eslint").ESLint,
  plugin: require("eslint-plugin-vue"),
  vueParser: require("vue-eslint-parser"),
  scriptParser: require("@typescript-eslint/parser"),
};
function relocated(value: ReferenceResult[], filename: string) {
  const copy = structuredClone(value);
  assert.equal(copy.length, 1);
  assert.equal(copy[0]!.filePath, path.join(corpus.originalReference.sourceRoot, filename));
  copy[0]!.filePath = path.join(root, filename);
  return copy;
}

test("attribute ignore retains nine whole independent verify/fix/requery packets and all original bytes", async () => {
  for (const [name, version] of Object.entries(corpus.provider)) {
    assert.equal((require(name + "/package.json") as { version: string }).version, version);
  }
  const ruleFile = path.join(
    path.dirname(require.resolve("eslint-plugin-vue/package.json")),
    corpus.ruleSource.path,
  );
  const bytes = fs.readFileSync(ruleFile);
  assert.equal(bytes.length, corpus.ruleSource.bytes);
  assert.equal(hash(bytes), "ce050d17c802fb26ed9d8c7245ad3d55f28750705b86d97b59522c071aa46fbd");
  for (const row of corpus.valid) {
    const reference = referenceFor(row),
      source = sourceFor(row);
    const config = baselineConfig(runtime, { [rule]: ["warn", ...reference.options] });
    const lint = new runtime.ESLint({
      cwd: root,
      overrideConfigFile: true,
      overrideConfig: config,
    });
    const fixer = new runtime.ESLint({
      cwd: root,
      overrideConfigFile: true,
      overrideConfig: config,
      fix: true,
    });
    const options = { filePath: path.join(root, row.filename) };
    assert.deepEqual(
      await lint.lintText(source, options),
      relocated(reference.verify, row.filename),
      row.id,
    );
    assert.deepEqual(
      await fixer.lintText(source, options),
      relocated(reference.fix, row.filename),
      row.id,
    );
    assert.deepEqual(
      await lint.lintText(reference.fixedSource, options),
      relocated(reference.requery, row.filename),
      row.id,
    );
    assert.equal(applyExpectedEdits(row), reference.fixedSource, row.id);
  }
});

test("the same official provider independently refuses all six original malformed lists", async () => {
  const row = corpus.valid[0]!;
  for (const item of corpus.invalid) {
    const before = original.invalid[item.referenceRow]!;
    assert.deepEqual(before.ignore, item.ignore);
    let failure: unknown;
    try {
      const lint = new runtime.ESLint({
        cwd: root,
        overrideConfigFile: true,
        overrideConfig: baselineConfig(runtime, {
          [rule]: ["warn", "always", { ignore: item.ignore }],
        }),
      });
      await lint.lintText(sourceFor(row), { filePath: path.join(root, row.filename) });
    } catch (error) {
      failure = error;
    }
    assert(failure instanceof Error, item.id);
    assert.deepEqual({ name: failure.name, message: failure.message }, before.error, item.id);
  }
});
