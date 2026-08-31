import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  createAppE2ePlanEvidence,
  validateAppE2eTarget,
} from "../../legacy-tools/github/app-e2e-plan.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("dispatch target validation binds the requested ref to one exact SHA", () => {
  const sha = "a".repeat(40);
  assert.equal(validateAppE2eTarget("all", sha, sha), sha);
  assert.equal(validateAppE2eTarget("dev", "", sha), null);
  assert.throws(() => validateAppE2eTarget("all", "", sha), /required when suite=all/);
  assert.throws(() => validateAppE2eTarget("all", "A".repeat(40), sha), /full lowercase/);
  assert.throws(() => validateAppE2eTarget("all", sha, "b".repeat(40)), /dispatch ref/);
  const evidence = createAppE2ePlanEvidence("full", "all", sha);
  assert.deepEqual(
    {
      schema: evidence.schema,
      version: evidence.version,
      targetSha: evidence.targetSha,
      sourceHeadSha: evidence.sourceHeadSha,
      rowCount: evidence.rowCount,
    },
    {
      schema: "vize.appE2ePlanEvidence",
      version: 1,
      targetSha: sha,
      sourceHeadSha: null,
      rowCount: 17,
    },
  );
  assert.throws(() => createAppE2ePlanEvidence("full", "all", "main"), /exact target SHA/);
});

test("planner CLI rejects unknown suites with a nonzero exit", () => {
  const result = spawnSync(
    "rust-script",
    ["tools/commands/ci/github/app-e2e-plan.rs", "--profile", "full", "--suite", "missing"],
    { cwd: root, encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Unknown full App E2E suite/);
});
