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
const matrices = [
  {
    name: "croquis consumption matrix",
    generator: "tools/davinci/croquis-consumers.mjs",
    artifact: "davinci-road/plan/croquis-consumption.md",
  },
  {
    name: "rule-parity matrix (SFC × JSX)",
    generator: "tools/davinci/rule-parity.mjs",
    artifact: "davinci-road/plan/rule-parity.md",
  },
  {
    name: "SourceLocation consumer inventory",
    generator: "tools/davinci/sourcelocation-inventory.mjs",
    artifact: "davinci-road/plan/sourcelocation-inventory.md",
  },
  {
    name: "consumer migration surface inventory",
    generator: "tools/davinci/consumer-migration-surfaces.mjs",
    artifact: "davinci-road/plan/consumer-migration-surfaces.md + .tsv",
  },
  {
    name: "construct-matrix fixture plane (element kind × directive)",
    generator: "tools/davinci/matrix-gen.mjs",
    artifact: "tests/fixtures/davinci-matrix/",
  },
];

function runCheck(generator: string, extraArgs: string[] = []) {
  return spawnSync(process.execPath, [path.join(repoRoot, generator), "--check", ...extraArgs], {
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
        `  node ${matrix.generator} --write\n\n` +
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
      process.execPath,
      [path.join(repoRoot, "tools/davinci/matrix-gen.mjs"), "--write", "--out-dir", scratch],
      { cwd: repoRoot, encoding: "utf8" },
    );
    assert.equal(write.status, 0, `${write.stdout}${write.stderr}`.trim());
    const clean = runCheck("tools/davinci/matrix-gen.mjs", ["--out-dir", scratch]);
    assert.equal(clean.status, 0, `${clean.stdout}${clean.stderr}`.trim());
    const victim = path.join(scratch, "native--v-if.vue");
    fs.appendFileSync(victim, "<!-- injected edit -->\n");
    const stale = runCheck("tools/davinci/matrix-gen.mjs", ["--out-dir", scratch]);
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
