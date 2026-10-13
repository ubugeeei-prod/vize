import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  loadComponentCliControls,
  validateComponentCliControls,
} from "../differential/component-cli-runtime-reads.ts";
const root = path.resolve(
  import.meta.dirname,
  "../_fixtures/differential/linter/component-cli-runtime-reads",
);
const manifest = JSON.parse(fs.readFileSync(path.join(root, "cases.json"), "utf8")) as {
  inputs: Array<{ source: string }>;
  cases: Array<{ config: string | null; argv: string[]; expected: unknown }>;
};
const read = (file: string) => fs.readFileSync(path.join(root, file));
void test("five complete passive public CLI/config/formatter vectors retain every raw source without runtime credit", () => {
  const result = loadComponentCliControls(root);
  assert.equal(result.cases.length, 5);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.originalMatrixCredit, false);
});
void test("missing/changed source or config and omitted/foreign command or whole diagnostic vectors refuse", () => {
  for (const entry of manifest.inputs) {
    assert.throws(() =>
      validateComponentCliControls(manifest, (file) => {
        if (file === entry.source) throw Error("missing");
        return read(file);
      }),
    );
    assert.throws(() =>
      validateComponentCliControls(manifest, (file) =>
        file === entry.source ? Buffer.concat([read(file), Buffer.from(" ")]) : read(file),
      ),
    );
  }
  for (const mutate of [
    (x: typeof manifest) => x.cases.pop(),
    (x: typeof manifest) => x.cases.push(x.cases[0]),
    (x: typeof manifest) => x.cases[0].argv.push("--quiet"),
    (x: typeof manifest) => (x.cases[3].config = "foreign.json"),
    (x: typeof manifest) => (x.cases[1].expected = []),
  ]) {
    const copy = structuredClone(manifest);
    mutate(copy);
    assert.throws(() => validateComponentCliControls(copy, read));
  }
});
