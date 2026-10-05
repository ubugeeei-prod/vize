import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { buildProductObserver } from "../differential/observer-build.ts";
import { FOCUS_OBSERVER } from "../differential/focus-history.ts";
import { runFocusCurrentCapture } from "../differential/focus-history-current-capture.ts";
import {
  compareFocusNativeOutputs,
  validateFocusNativeComparison,
} from "../differential/focus-history-native-comparison.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("source-built focus native observations match all eight immutable whole legacy outputs", (t) => {
  const evidenceDir = path.resolve(
    root,
    process.env.VIZE_FOCUS_HISTORY_EVIDENCE_DIR ?? "target/differential/focus-history",
  );
  fs.mkdirSync(evidenceDir, { recursive: true });
  const comparisonPath = path.join(evidenceDir, "current-output-comparison.json");
  fs.rmSync(comparisonPath, { force: true });
  try {
    const built = buildProductObserver({
      spec: FOCUS_OBSERVER,
      repoRoot: root,
      targetDir: path.resolve(root, process.env.CARGO_TARGET_DIR ?? "target"),
      evidenceDir,
      profile: process.env.CI ? "ci" : "dev",
      offline: !process.env.CI,
    });
    const report = runFocusCurrentCapture({ repoRoot: root, ...built });
    fs.writeFileSync(
      path.join(evidenceDir, "unaccepted-capture.json"),
      `${JSON.stringify(report, null, 2)}\n`,
    );
    const comparison = compareFocusNativeOutputs(root, report, built.receipt);
    fs.writeFileSync(comparisonPath, `${JSON.stringify(comparison, null, 2)}\n`);
    validateFocusNativeComparison(root, comparison, built.receipt);
    t.diagnostic(
      `Complete current-output comparison and unchanged raw capture/source receipt: ${evidenceDir}`,
    );
    assert.deepEqual(
      report.summary,
      {
        plannedOriginalWitnesses: 8,
        legacyCaptured: 8,
        nativeHandled: 8,
        captureFailures: 0,
        acceptedCompleteOracles: 0,
        nativeEquivalent: 0,
        pairedComparisons: 0,
      },
      JSON.stringify(report.rows, null, 2),
    );
    assert.deepEqual(
      comparison.summary,
      {
        reviewedCurrentOutputOracles: 8,
        completeLegacyMatches: 8,
        nativeHandled: 8,
        nativeEquivalent: 8,
        pairedComparisons: 8,
        currentOutputDrift: 0,
        captureFailures: 0,
        wholeCurrentComparisons: 32,
        wholePairedComparisons: 32,
        historicalCompleteOutputAuthorities: 0,
      },
      JSON.stringify(comparison.rows, null, 2),
    );
  } catch (error) {
    // Compile logs already retained by the shared source-build helper are not
    // replaced with a fabricated successful receipt or partial observation.
    fs.writeFileSync(
      path.join(evidenceDir, "execution-failure.json"),
      `${JSON.stringify({ acceptance: "unreviewed", error: String(error) }, null, 2)}\n`,
    );
    throw error;
  }
});
