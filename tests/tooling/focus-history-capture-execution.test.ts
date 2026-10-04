import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { buildProductObserver } from "../differential/observer-build.ts";
import { FOCUS_OBSERVER } from "../differential/focus-history.ts";
import { runFocusCapture } from "../differential/focus-history-capture.ts";
import {
  compareFocusCurrentOutputs,
  validateFocusCurrentReport,
} from "../differential/focus-history-current-output.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("source-built focus witnesses retain exploratory captures and match reviewed current outputs/refusals", (t) => {
  const evidenceDir = path.resolve(
    root,
    process.env.VIZE_FOCUS_HISTORY_EVIDENCE_DIR ?? "target/differential/focus-history",
  );
  fs.mkdirSync(evidenceDir, { recursive: true });
  try {
    const built = buildProductObserver({
      spec: FOCUS_OBSERVER,
      repoRoot: root,
      targetDir: path.resolve(root, process.env.CARGO_TARGET_DIR ?? "target"),
      evidenceDir,
      profile: process.env.CI ? "ci" : "dev",
      offline: !process.env.CI,
    });
    const report = runFocusCapture({ repoRoot: root, ...built });
    fs.writeFileSync(
      path.join(evidenceDir, "unaccepted-capture.json"),
      `${JSON.stringify(report, null, 2)}\n`,
    );
    const current = compareFocusCurrentOutputs(root, report, built.receipt);
    fs.writeFileSync(
      path.join(evidenceDir, "current-output-report.json"),
      `${JSON.stringify(current, null, 2)}\n`,
    );
    validateFocusCurrentReport(root, current, built.receipt);
    t.diagnostic(
      `Raw captures/source receipt and separate current-output-only comparisons: ${evidenceDir}`,
    );
    assert.deepEqual(
      report.summary,
      {
        plannedOriginalWitnesses: 8,
        legacyCaptured: 8,
        nativeRefused: 8,
        captureFailures: 0,
        acceptedCompleteOracles: 0,
        nativeHandled: 0,
        nativeEquivalent: 0,
        pairedComparisons: 0,
      },
      JSON.stringify(report.rows, null, 2),
    );
    assert.deepEqual(
      current.summary,
      {
        plannedOriginalInputs: 8,
        legacyCurrentMatches: 8,
        nativeRefusalMatches: 8,
        currentOutputDrift: 0,
        captureFailures: 0,
        wholeCurrentComparisons: 32,
        historicalCompleteOutputAuthorities: 0,
        nativeHandled: 0,
        nativeEquivalent: 0,
        pairedComparisons: 0,
      },
      JSON.stringify(current.rows, null, 2),
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
