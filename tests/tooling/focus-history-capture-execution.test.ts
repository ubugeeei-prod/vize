import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { buildProductObserver } from "../differential/observer-build.ts";
import { FOCUS_OBSERVER } from "../differential/focus-history.ts";
import { runFocusCapture } from "../differential/focus-history-capture.ts";
import { compareFocusCurrentOutputs } from "../differential/focus-history-current-oracle.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("source-built focus witnesses match reviewed whole current outputs and retain genuine native refusals", (t) => {
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
    const report = runFocusCapture({ repoRoot: root, ...built });
    fs.writeFileSync(
      path.join(evidenceDir, "unaccepted-capture.json"),
      `${JSON.stringify(report, null, 2)}\n`,
    );
    const comparison = compareFocusCurrentOutputs(root, report, built.receipt);
    fs.writeFileSync(comparisonPath, `${JSON.stringify(comparison, null, 2)}\n`);
    t.diagnostic(
      `Complete current-output comparison and unchanged raw capture/source receipt: ${evidenceDir}`,
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
