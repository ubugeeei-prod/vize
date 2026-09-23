import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

// TS-34's corpus-shard lane (Davinci P4-3a). The committed planes (battery,
// fixture ladder, P2-15 matrix) ride `cargo test --workspace`; this gate runs
// the corpus half in the test-scripts job over exactly the submodules that
// job already hydrates for the CLI diagnostic fixture (the P2-15 shard), so
// it costs no extra checkout. Every Croquis fact group migrated by a P4-3
// wave is compared with its declarative spec's naive evaluator here.
const shard = ["tests/_fixtures/_git/ant-design-vue", "tests/_fixtures/_git/create-vue"];

function hydrated(relative: string): boolean {
  const dir = path.join(repoRoot, relative);
  return fs.existsSync(dir) && fs.readdirSync(dir).length > 0;
}

test("the Croquis fact specs agree with production over the corpus shard", (t) => {
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
      "vize_croquis",
      "--test",
      "fact_spec",
      "the_corpus_shard_agrees_with_the_specs",
      "--",
      "--exact",
      "--nocapture",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: { ...process.env, VIZE_DAVINCI_FACT_CORPUS: shard.join(",") },
      timeout: 1_500_000,
    },
  );
  const output = `${result.stdout}\n${result.stderr}`;
  assert.equal(result.status, 0, output.trim());
  // The scope proof travels in the printed lines: one per group, each must
  // have compared facts. The Rust verdict already fails a zero-fact run.
  for (const group of ["bindings", "undefined-refs", "reactivity"]) {
    const scope = output
      .split("\n")
      .find((line) => line.startsWith(`fact spec ${group} corpus shard: `));
    assert.notEqual(scope, undefined, output.trim());
    const compared = scope?.match(/ compared=(\d+)/);
    const facts = scope?.match(/ facts=(\d+)/);
    assert.notEqual(compared, null, scope);
    assert.notEqual(facts, null, scope);
    assert.equal(Number(compared?.[1]) > 0, true, scope);
    assert.equal(Number(facts?.[1]) > 0, true, scope);
  }
});
