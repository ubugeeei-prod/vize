import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

// Davinci plan matrices are generated artifacts committed to the repo
// (davinci-road/plan/*.md). Each generator supports `--check`, which
// regenerates in memory and byte-compares against the committed file.
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
];

function runCheck(generator: string) {
  return spawnSync(process.execPath, [path.join(repoRoot, generator), "--check"], {
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
