import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { runFormatterPack } from "../differential/formatter.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("source-built formatter matches both exact references and reaches a fixed point", (t) => {
  const revision = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(revision.status, 0, revision.stderr);
  const report = runFormatterPack({
    manifestPath: path.join(root, "tests/_fixtures/differential/formatter/manifest.json"),
    binaryPath: path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize"),
    sourceRevision: revision.stdout.trim(),
    repoRoot: root,
  });
  const artifact = path.join(root, "target/differential/formatter.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
  t.diagnostic(`Raw CLI observations: ${artifact}`);
  assert.deepEqual(
    report.summary,
    {
      plannedCases: 2,
      legacyMatches: 2,
      legacyFailures: 0,
      baselineDrift: 0,
      nativeUnsupported: 2,
      pairedComparisons: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
    },
    JSON.stringify(report.rows, null, 2),
  );
});
