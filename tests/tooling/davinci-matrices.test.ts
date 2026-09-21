import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

// Davinci plan matrices are generated artifacts committed to the repo
// (davinci-road/plan/*.md, and since P2-15 the fixture plane under
// tests/fixtures/). Each generator supports `--check`, which regenerates
// in memory and byte-compares against the committed artifact.
// Add one entry per matrix (P0-8 rule-parity joins this list).
//
// The croquis consumption matrix and the consumer migration surfaces are
// sharded (an index page plus one file per crate / consumer) and commit no
// cross-crate totals, so parallel PRs touching different crates never
// conflict on them. `--check` is as strict as before: every shard is
// byte-compared, and a shard the generator no longer produces is stale.
// Totals are printed on demand by each generator's `--summary`.
const matrices = [
  {
    name: "croquis consumption matrix",
    generator: "tools/commands/davinci/croquis-consumers.rs",
    artifact: "davinci-road/plan/croquis-consumption.md + croquis-consumption/<crate>.md",
  },
  {
    name: "rule-parity matrix (SFC × JSX)",
    generator: "tools/commands/davinci/rule-parity.rs",
    artifact: "davinci-road/plan/rule-parity.md",
  },
  {
    name: "SourceLocation consumer inventory",
    generator: "tools/commands/davinci/sourcelocation-inventory.rs",
    artifact: "davinci-road/plan/sourcelocation-inventory.md",
  },
  {
    name: "consumer migration surface inventory",
    generator: "tools/commands/davinci/consumer-migration-surfaces.rs",
    artifact:
      "davinci-road/plan/consumer-migration-surfaces.md + consumer-migration-surfaces/<consumer>/<crate>.tsv",
  },
  {
    name: "storage summary (aggregates of the per-file storage ledger)",
    generator: "tools/commands/davinci/storage-summary.rs",
    artifact: "davinci-road/plan/storage-summary.md",
  },
  {
    name: "construct-matrix fixture plane (element kind × directive)",
    generator: "tools/commands/davinci/matrix-gen.rs",
    artifact: "tests/fixtures/davinci-matrix/",
  },
  {
    name: "HTML content-model fact table (P4-11a, pinned WHATWG snapshot)",
    generator: "tools/commands/davinci/html-content-model.rs",
    artifact: "crates/vize_patina/src/html_content_model/whatwg.tsv",
  },
];

function runCheck(generator: string, extraArgs: string[] = []) {
  return spawnSync("rust-script", [path.join(repoRoot, generator), "--check", ...extraArgs], {
    cwd: repoRoot,
    encoding: "utf8",
  });
}

for (const matrix of matrices) {
  test(`${matrix.name} is current (${matrix.artifact})`, () => {
    const result = runCheck(matrix.generator);
    assert.equal(
      result.status,
      0,
      `${matrix.artifact} is stale. Regenerate it with:\n` +
        `  rust-script ${matrix.generator} --write\n\n` +
        `${result.stdout}${result.stderr}`.trim(),
    );
  });
}

// The P0-7 discipline, kept continuous: a staleness check that cannot
// fail is no check at all, so prove `--check` rejects an injected edit
// on a throwaway copy of the fixture plane (the committed tree is never
// touched).
test("the fixture-plane staleness check fails on an injected edit", () => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-matrix-check-"));
  try {
    const write = spawnSync(
      "rust-script",
      [
        path.join(repoRoot, "tools/commands/davinci/matrix-gen.rs"),
        "--write",
        "--out-dir",
        scratch,
      ],
      { cwd: repoRoot, encoding: "utf8" },
    );
    assert.equal(write.status, 0, `${write.stdout}${write.stderr}`.trim());
    const clean = runCheck("tools/commands/davinci/matrix-gen.rs", ["--out-dir", scratch]);
    assert.equal(clean.status, 0, `${clean.stdout}${clean.stderr}`.trim());
    const victim = path.join(scratch, "native--v-if.vue");
    fs.appendFileSync(victim, "<!-- injected edit -->\n");
    const stale = runCheck("tools/commands/davinci/matrix-gen.rs", ["--out-dir", scratch]);
    assert.equal(
      stale.status,
      1,
      `--check accepted a stale fixture plane:\n${stale.stdout}${stale.stderr}`.trim(),
    );
    assert.equal(stale.stdout.includes("stale: native--v-if.vue"), true, stale.stdout);
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});

// Same discipline for the P4-11a content-model table: regenerate it into a
// throwaway copy, then prove `--check` rejects a one-member edit there.
test("the content-model table staleness check fails on an injected edit", () => {
  const generator = "tools/commands/davinci/html-content-model.rs";
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-html-content-model-"));
  const table = path.join(scratch, "whatwg.tsv");
  try {
    const write = spawnSync(
      "rust-script",
      [path.join(repoRoot, generator), "--write", "--table", table],
      { cwd: repoRoot, encoding: "utf8" },
    );
    assert.equal(write.status, 0, `${write.stdout}${write.stderr}`.trim());
    const clean = runCheck(generator, ["--table", table]);
    assert.equal(clean.status, 0, `${clean.stdout}${clean.stderr}`.trim());
    const rows = fs.readFileSync(table, "utf8");
    const edited = rows.replace(/^(set\tvoid\t[^\t]+\t)area /mu, "$1");
    assert.notEqual(edited, rows, "the injected edit must change the void row");
    fs.writeFileSync(table, edited);
    const stale = runCheck(generator, ["--table", table]);
    assert.equal(
      stale.status,
      1,
      `--check accepted a stale content-model table:\n${stale.stdout}${stale.stderr}`.trim(),
    );
    assert.equal(stale.stdout.includes("- set\tvoid\t"), true, stale.stdout);
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});
