import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { corpusFiles, materializeCorpus } from "./template-alias-paired-11822-corpus.ts";

void test("authored roots and negative controls include the last parent and are deterministic", () => {
  const first = corpusFiles(5, 2);
  assert.deepEqual(first, corpusFiles(5, 2));
  assert.equal([...first.files.keys()].filter((name) => name.endsWith(".vue")).length, 5);
  assert.deepEqual(
    first.expectedErrors.map((row) => row.file),
    ["src/generated/GeneratedComponent0000.vue", "src/generated/GeneratedComponent0003.vue"],
  );
  for (const error of first.expectedErrors) {
    const source = first.files.get(error.file)!;
    assert.equal(source.split("\n")[error.line - 1], "const oracle: number = 'wrong';");
    assert.equal(error.column, 7);
  }
  assert.match(first.files.get("src/index.ts")!, /GeneratedComponent0003/);
  assert.doesNotMatch(first.files.get("src/generated/GeneratedComponent0001.vue")!, /'wrong'/);
});

void test("materialized bytes match the manifest and overwrite is refused", (t) => {
  const directory = mkdtempSync(join(tmpdir(), "public-mapper-corpus-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const root = join(directory, "corpus");
  const manifest = materializeCorpus(root, 3, 1);
  for (const file of manifest.files)
    assert.equal(readFileSync(join(root, file.path)).length, file.bytes);
  assert.deepEqual(JSON.parse(readFileSync(join(root, "corpus.json"), "utf8")), manifest);
  assert.throws(() => materializeCorpus(root, 3, 1), /EEXIST/);
  const config = JSON.parse(readFileSync(join(root, "tsconfig.base.json"), "utf8"));
  assert.equal(config.compilerOptions.incremental, false);
  assert.equal(config.compilerOptions.skipLibCheck, false);
  assert.deepEqual(config.include, ["src/**/*.ts", "src/**/*.vue"]);
});
