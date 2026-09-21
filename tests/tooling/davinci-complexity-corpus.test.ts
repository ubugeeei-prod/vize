import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

// TS-34's corpus-shard lane for the template-complexity fact group
// (Davinci P4-9a). The matrix-plane half and the hand-computed pins ride
// `cargo test --workspace` in clippy-and-test; this gate runs the corpus
// half in the test-scripts job over exactly the submodules that job
// already hydrates (check.yml, "Hydrate CLI diagnostic fixture"), the
// metamorphic lane's shard. The full-corpus recipe — and the distribution
// the metric spec pins its thresholds from — is the same env var pointed at
// tests/_fixtures/_git (davinci-road/plan/complexity-metrics.md).
const shard = ["tests/_fixtures/_git/ant-design-vue", "tests/_fixtures/_git/create-vue"];

function hydrated(relative: string): boolean {
  const dir = path.join(repoRoot, relative);
  return fs.existsSync(dir) && fs.readdirSync(dir).length > 0;
}

test("the complexity corpus shard agrees with the naive evaluator, with a scope proof", (t) => {
  const missing = shard.filter((dir) => !hydrated(dir));
  if (missing.length > 0) {
    t.skip(`corpus submodules are not hydrated: ${missing.join(", ")}`);
    return;
  }
  if (spawnSync("cargo", ["--version"], { encoding: "utf8" }).status !== 0) {
    t.skip("cargo is not on PATH");
    return;
  }
  const result = spawnSync(
    "cargo",
    [
      "test",
      "-p",
      "vize_s1_to_s2",
      "--test",
      "cfg_complexity_oracle",
      "the_corpus_shard_agrees_with_the_naive_evaluator",
      "--",
      "--exact",
      "--nocapture",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: { ...process.env, VIZE_DAVINCI_COMPLEXITY_CORPUS: shard.join(",") },
      timeout: 1_500_000,
    },
  );
  const output = `${result.stdout}\n${result.stderr}`;
  assert.equal(result.status, 0, output.trim());
  // The scope proof travels in the printed line: a run that scored no
  // template must not pass silently (the Rust verdict already fails on
  // zero; assert the proof surfaced).
  const scope = output.split("\n").find((line) => line.startsWith("complexity corpus shard: "));
  assert.notEqual(scope, undefined, output.trim());
  const templates = scope?.match(/ templates=(\d+)/);
  const rows = scope?.match(/ rows=(\d+)/);
  assert.notEqual(templates, null, scope);
  assert.notEqual(rows, null, scope);
  assert.equal(Number(templates?.[1]) > 0, true, scope);
  assert.equal(Number(rows?.[1]) > 0, true, scope);
});
