import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

// TS-21's corpus-shard lane (Davinci P2-15). The metamorphic suite's
// matrix-plane half rides `cargo test --workspace` in clippy-and-test;
// this gate runs the corpus half in the test-scripts job over exactly
// the submodules that job already hydrates for the CLI diagnostic
// fixture (check.yml, "Hydrate CLI diagnostic fixture"), so the shard
// costs no extra checkout. The full-corpus recipe is the same env var
// pointed at tests/_fixtures/_git (see the P2-15 record).
const shard = ["tests/_fixtures/_git/ant-design-vue", "tests/_fixtures/_git/create-vue"];

function hydrated(relative: string): boolean {
  const dir = path.join(repoRoot, relative);
  return fs.existsSync(dir) && fs.readdirSync(dir).length > 0;
}

test("the metamorphic corpus shard holds with a scope proof", (t) => {
  const missing = shard.filter((dir) => !hydrated(dir));
  if (missing.length > 0) {
    t.skip(`corpus submodules are not hydrated: ${missing.join(", ")}`);
    return;
  }
  // CI installs the toolchain before any test runs (and the job builds
  // the vize CLI first, so a missing cargo would already have failed the
  // job); a cargo-less local checkout skips like an unhydrated one.
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
      "metamorphic",
      "the_corpus_shard_is_metamorphically_stable",
      "--",
      "--exact",
      "--nocapture",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: { ...process.env, VIZE_DAVINCI_METAMORPHIC_CORPUS: shard.join(",") },
      timeout: 1_500_000,
    },
  );
  const output = `${result.stdout}\n${result.stderr}`;
  assert.equal(result.status, 0, output.trim());
  // The scope proof travels in the printed line: a run that found no
  // files or applied no mutations must not pass silently. The Rust
  // verdict already fails on zero mutations; assert the proof surfaced.
  const scope = output.split("\n").find((line) => line.startsWith("metamorphic corpus shard: "));
  assert.notEqual(scope, undefined, output.trim());
  const mutations = scope?.match(/ mutations=(\d+)/);
  const files = scope?.match(/ files=(\d+)/);
  assert.notEqual(mutations, null, scope);
  assert.notEqual(files, null, scope);
  assert.equal(Number(files?.[1]) > 0, true, scope);
  assert.equal(Number(mutations?.[1]) > 0, true, scope);
});
