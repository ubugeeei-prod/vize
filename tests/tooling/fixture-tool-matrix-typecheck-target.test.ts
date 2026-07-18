import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { validateTypecheckPerformanceTarget } from "../../tools/fixtures/tool-matrix-typecheck-target.mjs";

test("fixture tool matrix requires an exact baseline tsconfig for performance targets", () => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typecheck-target-"));
  const project = {
    id: "performance-fixture",
    tsconfig: "configs/tsconfig.check.json",
    typecheckPerformance: { enabled: true, compareTo: "vue-tsc" },
  };
  try {
    fs.mkdirSync(path.join(fixtureRoot, "configs"));
    fs.writeFileSync(path.join(fixtureRoot, project.tsconfig), "{}\n");
    assert.doesNotThrow(() => validateTypecheckPerformanceTarget(project, fixtureRoot));
    assert.doesNotThrow(() =>
      validateTypecheckPerformanceTarget(
        { ...project, tsconfig: undefined, typecheckPerformance: { enabled: false } },
        fixtureRoot,
      ),
    );

    for (const [candidate, message] of [
      [{ ...project, typecheckPerformance: { enabled: true, compareTo: "tsc" } }, /compareTo/],
      [{ ...project, tsconfig: undefined }, /normalized relative path/],
      [{ ...project, tsconfig: "../tsconfig.json" }, /normalized relative path/],
      [{ ...project, tsconfig: "./tsconfig.json" }, /normalized relative path/],
      [{ ...project, tsconfig: "/tmp/tsconfig.json" }, /normalized relative path/],
      [{ ...project, tsconfig: "configs/missing.json" }, /does not exist/],
      [{ ...project, tsconfig: "configs" }, /is not a file/],
    ] as const) {
      assert.throws(() => validateTypecheckPerformanceTarget(candidate, fixtureRoot), message);
    }
  } finally {
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
