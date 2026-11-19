import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

// TS-34's corpus-shard lane (Davinci P4-3a). The committed planes (battery,
// fixture ladder, P2-15 matrix) ride `cargo test --workspace`; this gate runs
// the corpus half in the test-scripts job over the CLI diagnostic fixture
// (the P2-15 shard). Checking v-for collections removes its final positive
// unused rows. Keep its full scope and add the existing pinned layoutit-grid
// corpus as an independent positive witness. Exact equality and the existing
// zero-fact verdicts are unchanged. The committed LoopSource regression
// remains part of the separate fixture plane; it cannot replace real inputs.
const shard = ["tests/_fixtures/_git/ant-design-vue", "tests/_fixtures/_git/create-vue"];
const witness = ["tests/_fixtures/_git/layoutit-grid"];

function hydrated(relative: string): boolean {
  const dir = path.join(repoRoot, relative);
  return fs.existsSync(dir) && fs.readdirSync(dir).length > 0;
}

test("the Croquis fact specs agree with production over the corpus shard", (t) => {
  const missing = [...shard, ...witness].filter((dir) => !hydrated(dir));
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
      env: {
        ...process.env,
        VIZE_DAVINCI_FACT_CORPUS: shard.join(","),
        VIZE_DAVINCI_FACT_CORPUS_WITNESS: witness.join(","),
      },
      timeout: 1_500_000,
    },
  );
  const output = `${result.stdout}\n${result.stderr}`;
  assert.equal(result.status, 0, output.trim());
  const scopes = output.split("\n").filter((line) => line.startsWith("fact spec "));
  for (const scope of scopes) t.diagnostic(scope);
  const unusedScope = (label: string) => {
    const scope = scopes.find((line) => line.startsWith(`fact spec unused-bindings ${label}: `));
    assert.notEqual(scope, undefined, output.trim());
    return {
      artifacts: Number(scope?.match(/ artifacts=(\d+)/)?.[1]),
      compared: Number(scope?.match(/ compared=(\d+)/)?.[1]),
      facts: Number(scope?.match(/ facts=(\d+)/)?.[1]),
    };
  };
  const originals = unusedScope("corpus originals");
  const witnesses = unusedScope("corpus witnesses");
  const combined = unusedScope("corpus shard");
  // The unchanged original pins contain 731 + 42 SFCs. The supplemental
  // project's 112 SFCs cannot replace or hide any of those comparisons.
  assert.equal(originals.artifacts, 773);
  assert.equal(originals.compared, 773);
  assert.equal(witnesses.artifacts, 112);
  assert.equal(witnesses.compared, 112);
  assert.equal(witnesses.facts > 0, true);
  assert.deepEqual(combined, {
    artifacts: originals.artifacts + witnesses.artifacts,
    compared: originals.compared + witnesses.compared,
    facts: originals.facts + witnesses.facts,
  });
  // The scope proof travels in the printed lines: one per group, each must
  // have compared facts. The Rust verdict already fails a zero-fact run.
  for (const group of [
    "bindings",
    "undefined-refs",
    "unused-bindings",
    "reactivity",
    "race-conditions",
  ]) {
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
