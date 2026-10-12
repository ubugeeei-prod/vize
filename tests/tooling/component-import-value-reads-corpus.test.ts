import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  loadComponentImportValueCorpus,
  validateComponentImportValueCorpus,
} from "../differential/component-import-value-reads.ts";

const corpus = path.resolve(
  import.meta.dirname,
  "../_fixtures/differential/linter/component-import-value-reads",
);
type MutableContract = {
  cases: Array<{
    source: string;
    expected: Array<{ start: number }>;
    primarySource?: { revision: string };
  }>;
  rules: string[];
  unknown?: boolean;
};
const manifest = JSON.parse(
  fs.readFileSync(path.join(corpus, "cases.json"), "utf8"),
) as MutableContract;
const read = (file: string) => fs.readFileSync(path.join(corpus, file));

void test("all17 complete authored and pinned sources retain full original vectors without native credit", () => {
  const result = loadComponentImportValueCorpus(corpus);
  assert.equal(result.cases.length, 17);
  assert.equal(result.cases.filter((entry) => entry.primarySource).length, 3);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.differentialClosureCredit, false);
});

void test("every missing or changed whole source and original packet refuses", () => {
  for (const file of [
    ...manifest.cases.map((entry: { source: string }) => entry.source),
    "original-current447-whole-vectors.json",
  ]) {
    assert.throws(() =>
      validateComponentImportValueCorpus(manifest, (name) => {
        if (name === file) throw Error("retained source missing");
        return read(name);
      }),
    );
    assert.throws(() =>
      validateComponentImportValueCorpus(manifest, (name) =>
        name === file ? Buffer.concat([read(name), Buffer.from(" ")]) : read(name),
      ),
    );
  }
});

void test("missing, duplicate, foreign, changed tuple and changed source-pin contracts refuse", () => {
  const mutations = [
    (value: typeof manifest) => value.cases.pop(),
    (value: typeof manifest) => value.cases.push(value.cases[0]),
    (value: typeof manifest) => (value.cases[0].source = "../foreign.vue.txt"),
    (value: typeof manifest) => value.cases[2].expected[0].start++,
    (value: typeof manifest) => (value.cases[2].expected = []),
    (value: typeof manifest) => (value.cases[14].primarySource!.revision = "f".repeat(40)),
    (value: typeof manifest) => value.rules.push("vue/no-unused-setup-bindings"),
    (value: typeof manifest) => (value.unknown = true),
  ];
  for (const mutate of mutations) {
    const changed = structuredClone(manifest);
    mutate(changed);
    assert.throws(() => validateComponentImportValueCorpus(changed, read));
  }
});

void test("physical unknown and symlink corpus sources cannot masquerade as the reviewed set", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-component-value-custody-"));
  try {
    fs.cpSync(corpus, temporary, { recursive: true });
    fs.writeFileSync(path.join(temporary, "foreign.vue.txt"), "<template />");
    assert.throws(() => loadComponentImportValueCorpus(temporary));
    fs.unlinkSync(path.join(temporary, "foreign.vue.txt"));
    const source = manifest.cases[0].source;
    fs.unlinkSync(path.join(temporary, source));
    fs.symlinkSync(path.join(corpus, source), path.join(temporary, source));
    assert.throws(() => loadComponentImportValueCorpus(temporary));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
