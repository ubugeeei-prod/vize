import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { summarizeNativeAcceptance } from "../differential/acceptance-rates.mjs";
import { loadCompilerManifest, runCompilerPack } from "../differential/compiler.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("ordinary source-built compiler rejects legacy-only Vue 1 rows without credit", (t) => {
  const revision = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(revision.status, 0, revision.stderr);
  const manifestPath = path.join(root, "tests/_fixtures/differential/compiler/manifest.json");
  const loaded = loadCompilerManifest(manifestPath);
  const report = runCompilerPack({
    manifestPath,
    binaryPath: path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize"),
    sourceRevision: revision.stdout.trim(),
    repoRoot: root,
  });
  const artifact = path.join(root, "target/differential/compiler-default.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
  t.diagnostic(`Compiler observation: ${artifact}`);
  const acceptance = summarizeNativeAcceptance(loaded, report, {
    sourceRevision: revision.stdout.trim(),
  });
  assert.deepEqual(acceptance.total, {
    planned: loaded.cases.length,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: loaded.cases.length,
    legacyBacked: 0,
    unverified: 0,
  });
  const vue1 = report.rows.filter((row) => row.id.startsWith("compiler/dialect/vue1-"));
  assert.equal(vue1.length, 3);
  for (const row of vue1) {
    assert.equal(row.legacy.state, "failed");
    assert.equal(row.legacy.observation, null);
    assert.match(row.legacy.error, /requires source-build recipe .* --features legacy/);
  }
  assert.equal(report.summary.legacyMatches, loaded.cases.length - vue1.length);
  assert.equal(report.summary.legacyFailures, vue1.length);
  assert.equal(report.summary.baselineDrift, 0);
  assert(
    report.rows
      .filter((row) => !row.id.startsWith("compiler/dialect/vue1-"))
      .every((row) => row.legacy.verdict === "matched-reference"),
    JSON.stringify(report.rows, null, 2),
  );
});
