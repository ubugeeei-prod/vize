import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  renderNativeAcceptanceMarkdown,
  summarizeNativeAcceptance,
} from "../differential/acceptance-rates.mjs";
import { loadFormatterManifest } from "../differential/manifest.mjs";
import { runFormatterPack } from "../differential/formatter.mjs";
import { runFormatterHistoryCli } from "../differential/formatter-history-cli.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("source-built formatter matches every exact reference and reaches a fixed point", (t) => {
  const revision = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(revision.status, 0, revision.stderr);
  const manifestPath = path.join(root, "tests/_fixtures/differential/formatter/manifest.json");
  const loaded = loadFormatterManifest(manifestPath);
  const count = loaded.cases.length;
  const report = runFormatterPack({
    manifestPath,
    binaryPath: path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize"),
    sourceRevision: revision.stdout.trim(),
    repoRoot: root,
  });
  const artifact = path.join(root, "target/differential/formatter.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
  t.diagnostic(`Raw CLI observations: ${artifact}`);
  const acceptance = summarizeNativeAcceptance(loaded, report, {
    sourceRevision: revision.stdout.trim(),
  });
  assert.deepEqual(acceptance.total, {
    planned: count,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: count,
    legacyBacked: 0,
    unverified: 0,
  });
  const acceptanceArtifact = path.join(root, "target/differential/native-acceptance.json");
  const markdown = renderNativeAcceptanceMarkdown([acceptance]);
  fs.writeFileSync(acceptanceArtifact, `${JSON.stringify(acceptance, null, 2)}\n`);
  t.diagnostic(markdown);
  t.diagnostic(`Native acceptance: ${acceptanceArtifact}`);
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, markdown);
  assert.deepEqual(
    report.summary,
    {
      plannedCases: count,
      legacyMatches: count - 1,
      currentReferenceMatches: 1,
      legacyFailures: 0,
      baselineDrift: 0,
      nativeUnsupported: count,
      pairedComparisons: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
    },
    JSON.stringify(report.rows, null, 2),
  );
});

void test("formatter history CLI checks preserve files and compare complete verdict streams", (t) => {
  const report = runFormatterHistoryCli({
    repoRoot: root,
    binaryPath: path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize"),
  });
  const artifact = path.join(root, "target/differential/formatter-api/cli-history-report.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
  t.diagnostic(`Raw historical CLI check, dry-run, write and recheck observations: ${artifact}`);
  assert.deepEqual(
    report.summary,
    {
      plannedCases: 5,
      legacyMatches: 5,
      legacyFailures: 0,
      nativeUnsupported: 5,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
    JSON.stringify(report.rows, null, 2),
  );
});
