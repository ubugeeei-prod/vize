import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { summarizeNativeAcceptance } from "../differential/acceptance-rates.mjs";
import { loadLspManifest, runLspPack } from "../differential/lsp.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("source-built LSP registers complete fix-history response sessions with zero native credit", async (t) => {
  const revision = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(revision.status, 0, revision.stderr);
  const manifestPath = path.join(root, "tests/_fixtures/differential/lsp/manifest.json");
  const loaded = loadLspManifest(manifestPath);
  const report = await runLspPack({
    manifestPath,
    binaryPath: path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize"),
    sourceRevision: revision.stdout.trim(),
    repoRoot: root,
  });
  const artifact = path.join(root, "target/differential/lsp.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
  t.diagnostic(`LSP raw-wire observation: ${artifact}`);
  const acceptance = summarizeNativeAcceptance(loaded, report, {
    sourceRevision: revision.stdout.trim(),
  });
  assert.deepEqual(acceptance.total, {
    planned: 7,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: 7,
    legacyBacked: 0,
    unverified: 0,
  });
  assert.equal(report.summary.legacyMatches, 7, JSON.stringify(report.rows, null, 2));
  assert.equal(report.summary.legacyFailures, 0);
  assert.equal(report.summary.baselineDrift, 0);
});
