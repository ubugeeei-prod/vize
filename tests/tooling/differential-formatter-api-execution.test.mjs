import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { buildFormatterObserver } from "../differential/formatter-api-build.mjs";
import { runFormatterApiPack } from "../differential/formatter-api.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const packs = [
  ["script", 6, 4, 0, 2],
  ["prepared", 82, 81, 1, 0],
  ["literal", 84, 84, 0, 0],
  ["literal-extra", 34, 34, 0, 0],
  ["vue-version", 14, 14, 0, 0],
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
});
