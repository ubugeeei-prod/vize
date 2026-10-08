import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { toolingTestCommand } from "./run-tooling-tests.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const planPath = process.env.VIZE_TOOLING_TEST_PLAN;
const selected = planPath
  ? toolingTestCommand(
      JSON.parse(readFileSync(planPath, "utf8")),
      undefined,
      process.env.VIZE_TOOLING_TEST_SHARD,
    )
  : ["tests/tooling/vue-benchmarks-current-lint.test.ts"];
if (selected.includes("tests/tooling/vue-benchmarks-current-lint.test.ts")) {
  const run = spawnSync(
    "git",
    ["submodule", "update", "--init", "--depth", "1", "--", "tests/_fixtures/_git/vue-benchmarks"],
    { cwd: root, stdio: "inherit" },
  );
  assert.ifError(run.error);
  assert.equal(run.status, 0, "pinned upstream benchmark hydration failed");
  assert.equal(run.signal, null);
}
