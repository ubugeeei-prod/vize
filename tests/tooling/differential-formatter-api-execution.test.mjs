import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { buildFormatterObserver } from "../differential/formatter-api-build.mjs";
import { runFormatterApiPack } from "../differential/formatter-api.mjs";
import { runSortingConfigPack } from "../differential/formatter-sorting-config.mjs";
import { runCssGroupingRegressions } from "../differential/formatter-css-grouping.ts";
import { validateFormatterHistoryExecution } from "../differential/formatter-history-audit.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const packs = [
  ["script", 6, 4, 0, 2],
  ["prepared", 82, 81, 1, 0],
  ["literal", 84, 84, 0, 0],
  ["literal-extra", 34, 34, 0, 0],
  ["vue-version", 14, 14, 0, 0],
  ["capture", 51, 31, 20, 0],
  ["capture-extra", 25, 23, 0, 1],
  ["capture-final", 4, 4, 0, 0],
];

void test("shared formatter API history observes complete source-built output and typed errors", (t) => {
  const evidenceDir = path.resolve(
    root,
    process.env.VIZE_FORMATTER_API_EVIDENCE_DIR ?? "target/differential/formatter-api",
  );
  const built = buildFormatterObserver({
    repoRoot: root,
    targetDir: path.resolve(root, process.env.CARGO_TARGET_DIR ?? "target"),
    evidenceDir,
    profile: process.env.CI ? "ci" : "dev",
    offline: !process.env.CI,
  });
  for (const argv of [
    ["--options-json", "--options", '{"unknown":true}'],
    ["--options-json", "--options", '{"skipScriptStabilization":true}'],
    ["--options-json", "--vue-version", "4"],
    ["--options-json", "--vue-version", "2", "--vue-version", "3"],
    ["--style", "--vue-version", "2"],
  ]) {
    const observed = spawnSync(built.binaryPath, argv, { input: "", timeout: 30_000 });
    assert.equal(observed.error, undefined);
    assert.equal(observed.signal, null);
    assert.equal(observed.status, 1);
    assert.equal(observed.stdout.length, 0);
    assert.match(observed.stderr.toString(), /^Error: IoError/);
  }
  const reports = packs.map(([name, count, bytes, errors, internal]) => {
    const report = runFormatterApiPack({
      manifestPath: path.join(
        root,
        "tests/_fixtures/differential/formatter-history",
        name + "-manifest.json",
      ),
      repoRoot: root,
      ...built,
    });
    const artifact = name === "script" ? "report.json" : name + "-report.json";
    fs.writeFileSync(path.join(evidenceDir, artifact), JSON.stringify(report, null, 2) + "\n");
    return {
      report,
      expected: {
        plannedCases: count,
        legacyByteMatches: bytes,
        legacyInternalObservations: internal,
        legacyErrorMatches: errors,
        legacyFailures: 0,
        ...(name === "capture-extra" ? { currentReferenceMatches: 1 } : {}),
        nativeUnsupported: count,
        nativeHandled: 0,
        nativeEquivalent: 0,
        pairedComparisons: 0,
      },
    };
  });
  t.diagnostic("Raw public API observations and build receipt: " + evidenceDir);
  for (const { report, expected } of reports)
    assert.deepEqual(report.summary, expected, JSON.stringify(report.rows, null, 2));
  validateFormatterHistoryExecution(
    root,
    reports.map(({ report }) => report),
  );
  const sorting = runFormatterApiPack({
    manifestPath: path.join(
      root,
      "tests/_fixtures/differential/formatter-history/import-sorting-manifest.json",
    ),
    repoRoot: root,
    ...built,
  });
  fs.writeFileSync(
    path.join(evidenceDir, "import-sorting-report.json"),
    JSON.stringify(sorting, null, 2) + "\n",
  );
  assert.deepEqual(
    sorting.summary,
    {
      plannedCases: 14,
      legacyByteMatches: 11,
      legacyInternalObservations: 0,
      legacyErrorMatches: 3,
      legacyFailures: 0,
      nativeUnsupported: 14,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
    JSON.stringify(sorting.rows, null, 2),
  );
  const configuration = runSortingConfigPack({
    repoRoot: root,
    binaryPath: path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize"),
  });
  fs.writeFileSync(
    path.join(evidenceDir, "import-sorting-config-report.json"),
    JSON.stringify(configuration, null, 2) + "\n",
  );
  assert.deepEqual(
    configuration.summary,
    {
      plannedCases: 6,
      legacyMatches: 6,
      legacyFailures: 0,
      nativeUnsupported: 6,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
    JSON.stringify(configuration.rows, null, 2),
  );
  const malformed = runSortingConfigPack({
    repoRoot: root,
    binaryPath: path.join(root, "target/ci", process.platform === "win32" ? "vize.exe" : "vize"),
    manifestPath:
      "tests/_fixtures/differential/formatter-history/import-sorting-malformed-config-manifest.json",
  });
  fs.writeFileSync(
    path.join(evidenceDir, "import-sorting-malformed-config-report.json"),
    JSON.stringify(malformed, null, 2) + "\n",
  );
  assert.deepEqual(
    malformed.summary,
    {
      plannedCases: 8,
      legacyMatches: 8,
      legacyFailures: 0,
      nativeUnsupported: 8,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
    JSON.stringify(malformed.rows, null, 2),
  );
  runCssGroupingRegressions({ repoRoot: root, ...built, evidenceDir });
});
