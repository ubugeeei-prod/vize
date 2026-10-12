import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  loadComponentRuntimeReadControls,
  validateComponentRuntimeReadControls,
} from "../differential/component-template-runtime-reads.ts";

const corpus = path.resolve(
  import.meta.dirname,
  "../_fixtures/differential/linter/component-template-runtime-reads",
);
const manifest = JSON.parse(fs.readFileSync(path.join(corpus, "cases.json"), "utf8")) as {
  cases: Array<{ source: string; expected: Array<{ start: number }> }>;
  rules: string[];
};
const read = (source: string) => fs.readFileSync(path.join(corpus, source));

void test("twenty complete passive runtime/type/write/mixed sources retain vectors without runtime credit", () => {
  const result = loadComponentRuntimeReadControls(corpus);
  assert.equal(result.cases.length, 20);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.differentialClosureCredit, false);
});
void test("each missing or changed whole input refuses", () => {
  for (const entry of manifest.cases) {
    assert.throws(() =>
      validateComponentRuntimeReadControls(manifest, (source) => {
        if (source === entry.source) throw Error("missing whole source");
        return read(source);
      }),
    );
    assert.throws(() =>
      validateComponentRuntimeReadControls(manifest, (source) =>
        source === entry.source ? Buffer.concat([read(source), Buffer.from(" ")]) : read(source),
      ),
    );
  }
});
void test("partial, duplicate, foreign, changed-vector and broadened-rule controls refuse", () => {
  const changes = [
    (value: typeof manifest) => value.cases.pop(),
    (value: typeof manifest) => value.cases.push(value.cases[0]),
    (value: typeof manifest) => (value.cases[0].source = "../foreign.vue.txt"),
    (value: typeof manifest) => (value.cases[0].expected = []),
    (value: typeof manifest) => value.cases[0].expected[0].start++,
    (value: typeof manifest) => value.rules.push("vue/no-unused-setup-bindings"),
  ];
  for (const change of changes) {
    const changed = structuredClone(manifest);
    change(changed);
    assert.throws(() => validateComponentRuntimeReadControls(changed, read));
  }
});
