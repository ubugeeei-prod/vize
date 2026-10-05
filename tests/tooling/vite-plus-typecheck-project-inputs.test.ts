import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  diagnostic,
  originalInputs,
  originalRoot,
} from "../fixtures/typechecker/vite-plus-relative-tsconfig/fixture.ts";

void test("issue8017 keeps every original fenced input and reporter identity", () => {
  const { inputs, manifest } = originalInputs();
  assert.equal(inputs["src/Counter.vue"].split("\n")[1], 'const count: number = "not a number";');
  assert.equal(
    inputs["vite.config.mts"].includes('typecheck: { tsconfig: "tsconfig.app.json" }'),
    true,
  );
  assert.deepEqual(JSON.parse(inputs["tsconfig.app.json"]).include, [
    "src/**/*.ts",
    "src/**/*.vue",
  ]);
  assert.equal(diagnostic, "error:2:7 [TS2322] Type 'string' is not assignable to type 'number'.");
  assert.equal(manifest.reportedVersions.vitePlus, "0.2.7");
});

for (const file of ["src/Counter.vue", "tsconfig.app.json", "vite.config.mts", "issue-body.md"]) {
  void test(`original input custody refuses changed ${file}`, () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-8017-inputs-"));
    try {
      fs.cpSync(originalRoot, root, { recursive: true });
      fs.appendFileSync(path.join(root, file), "\n");
      assert.throws(() => originalInputs(root));
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
}
