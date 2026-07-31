import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  commitSha,
  readJson,
  run,
  setup,
  sharedBaselineOutput,
  sharedVizeDiagnostic,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

/**
 * #3460: `budget.passed` was computed, embedded in the artifact, and printed in
 * the step summary while nothing read it, so a matrix-wide false-positive or
 * false-negative breach printed `Budget passed: false` and the weekly Real
 * Project Matrix job still went green. These tests execute the failure path the
 * gate had never taken.
 */

const vizeOnly = "error:2:1 [TS2345] vize only";
const baselineOnly = "src/App.vue(3,1): error TS2345: baseline only\n";

function artifactPath(fixture: ReturnType<typeof setup>, extension: string) {
  return path.join(fixture.reportDir, `fixture-typecheck-divergence.${extension}`);
}

test("a false-positive budget breach fails the divergence report", () => {
  const fixture = setup({ vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly] });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(
      result.stderr,
      "Typecheck divergence budget breached for fixture: 1 false positives (ratio 0.5) exceed maxFalsePositiveRatio 0.05\n",
    );
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: false,
      falseNegativePassed: true,
      passed: false,
    });
    // The breach is uploaded, not swallowed: both artifacts land before the throw.
    assert.equal(
      fs.readFileSync(artifactPath(fixture, "md"), "utf8"),
      [
        "## fixture typecheck divergence",
        "",
        `Commit: ${commitSha}`,
        "Vize diagnostics: 2",
        "vue-tsc diagnostics: 1",
        "Shared: 1",
        "Message mismatches: 0",
        "Documented differences: 0",
        "False positives: 1 (0.5)",
        "False negatives: 0 (0)",
        "Vize excluded non-Vue: 0",
        "vue-tsc excluded non-Vue: 0",
        "vue-tsc excluded project-level: 0",
        "vue-tsc excluded external: 0",
        "Budget passed: false",
        `Digest: ${artifact.divergence.sha256}`,
        "",
      ].join("\n"),
    );
  } finally {
    cleanup(fixture);
  }
});

test("a false-negative budget breach fails the divergence report", () => {
  const fixture = setup({ baselineOutput: `${sharedBaselineOutput}${baselineOnly}` });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(
      result.stderr,
      "Typecheck divergence budget breached for fixture: 1 false negatives (ratio 0.5) exceed maxFalseNegativeRatio 0.05\n",
    );
    assert.deepEqual(readJson(artifactPath(fixture, "json")).budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: true,
      falseNegativePassed: false,
      passed: false,
    });
  } finally {
    cleanup(fixture);
  }
});

test("a breach of both budgets reports both sides", () => {
  const fixture = setup({
    vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly],
    baselineOutput: `${sharedBaselineOutput}${baselineOnly}`,
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(
      result.stderr,
      "Typecheck divergence budget breached for fixture: " +
        "1 false positives (ratio 0.5) exceed maxFalsePositiveRatio 0.05; " +
        "1 false negatives (ratio 0.5) exceed maxFalseNegativeRatio 0.05\n",
    );
  } finally {
    cleanup(fixture);
  }
});

test("a ratio exactly at the budget still passes", () => {
  // 1 false positive in 20 vize diagnostics is exactly maxFalsePositiveRatio,
  // so this pins the comparison as `<=` and proves the gate is not always-fail.
  const shared = Array.from(
    { length: 19 },
    (_unused, index) => `error:${index + 1}:1 [TS2322] shared`,
  );
  const fixture = setup({
    vizeDiagnostics: [...shared, "error:20:1 [TS2345] vize only"],
    baselineOutput: shared
      .map((_unused, index) => `src/App.vue(${index + 1},1): error TS2322: shared\n`)
      .join(""),
  });
  try {
    const result = run(fixture);
    assert.equal(result.stderr, "");
    assert.equal(result.status, 0);
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: true,
      falseNegativePassed: true,
      passed: true,
    });
    assert.equal(artifact.divergence.summary.falsePositiveRatio, 0.05);
  } finally {
    cleanup(fixture);
  }
});
