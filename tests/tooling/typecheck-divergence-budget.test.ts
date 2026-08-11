import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  breachFailure,
  cleanup,
  commitSha,
  divergenceClassification,
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
 *
 * #3513: a comparison whose two sides never met is a measurement failure that
 * may never render as a pass.
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
      `${breachFailure(1, "1 false positives (ratio 0.5) exceed maxFalsePositiveRatio 0")}\n`,
    );
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0,
      maxFalseNegativeRatio: 0,
      messageMismatchPassed: true,
      falsePositivePassed: false,
      falseNegativePassed: true,
      unusableReason: null,
      verdict: "breached",
      passed: false,
    });
    // The breach is uploaded, not swallowed: both artifacts land before the throw.
    assert.equal(
      fs.readFileSync(artifactPath(fixture, "md"), "utf8"),
      [
        "## fixture typecheck divergence",
        "",
        `Commit: ${commitSha}`,
        "Vize version: vize 0.0.0",
        `Vize peak RSS: ${artifact.source.peakRssBytes} bytes`,
        `Vize seeded peak RSS: ${Math.max(
          ...artifact.seededMutation.states.map((state) => state.vize.peakRssBytes),
        )} bytes`,
        "vue-tsc version: 3.3.4",
        `vue-tsc peak RSS: ${artifact.baseline.peakRssBytes} bytes`,
        `vue-tsc seeded peak RSS: ${Math.max(
          ...artifact.seededMutation.states.map((state) => state.baseline.peakRssBytes),
        )} bytes`,
        "Seeded mutation: sfc-script-ts2322 clean/broken/repaired passed",
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
        "vue-tsc configuration errors: 0",
        "vue-tsc blocking configuration errors: 0",
        "vue-tsc ignored deprecation errors: 0",
        "Vize Vue files: 1",
        "vue-tsc Vue files: 1",
        "Shared Vue files: 1",
        "Missing Vue files: 0",
        "Unexpected Vue files: 0",
        "Ignored dependency Vue files: 0",
        "Budget verdict: breached",
        `Classification: ${divergenceClassification(1)}`,
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
      `${breachFailure(1, "1 false negatives (ratio 0.5) exceed maxFalseNegativeRatio 0")}\n`,
    );
    assert.deepEqual(readJson(artifactPath(fixture, "json")).budget, {
      maxFalsePositiveRatio: 0,
      maxFalseNegativeRatio: 0,
      messageMismatchPassed: true,
      falsePositivePassed: true,
      falseNegativePassed: false,
      unusableReason: null,
      verdict: "breached",
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
      `${breachFailure(
        1,
        "1 false positives (ratio 0.5) exceed maxFalsePositiveRatio 0; " +
          "1 false negatives (ratio 0.5) exceed maxFalseNegativeRatio 0",
      )}\n`,
    );
  } finally {
    cleanup(fixture);
  }
});

test("an undocumented message mismatch is release-blocking", () => {
  const fixture = setup({ baselineOutput: "src/App.vue(1,1): error TS2322: different\n" });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(
      result.stderr,
      `${breachFailure(
        1,
        "1 message mismatches require an explicit documented-difference entry",
      )}\n`,
    );
    const budget = readJson(artifactPath(fixture, "json")).budget;
    assert.equal(budget.messageMismatchPassed, false);
    assert.equal(budget.falsePositivePassed, true);
    assert.equal(budget.falseNegativePassed, true);
    assert.equal(budget.verdict, "breached");
  } finally {
    cleanup(fixture);
  }
});

test("one unexplained diagnostic fails even at the old ratio threshold", () => {
  // One false positive in 20 diagnostics used to fit the 5% allowance. Exact
  // parity is now monotonic: corpus size can never dilute an unexplained result.
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
    assert.equal(result.status, 1);
    assert.equal(
      result.stderr,
      `${breachFailure(1, "1 false positives (ratio 0.05) exceed maxFalsePositiveRatio 0")}\n`,
    );
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.equal(artifact.budget.verdict, "breached");
    assert.equal(artifact.divergence.summary.falsePositiveRatio, 0.05);
  } finally {
    cleanup(fixture);
  }
});

test("--budget-mode enforce is the default and fails the job", () => {
  const fixture = setup({ vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly] });
  try {
    const expected = `${breachFailure(
      1,
      "1 false positives (ratio 0.5) exceed maxFalsePositiveRatio 0",
    )}\n`;
    const explicit = run(fixture, {}, ["--budget-mode", "enforce"]);
    assert.equal(explicit.status, 1);
    assert.equal(explicit.stderr, expected);
    const implicit = run(fixture);
    assert.equal(implicit.status, 1);
    assert.equal(implicit.stderr, expected);
  } finally {
    cleanup(fixture);
  }
});

test("an unrecognised budget mode is rejected instead of disarming the gate", () => {
  const fixture = setup({ vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly] });
  try {
    const result = run(fixture, {}, ["--budget-mode", "off"]);
    assert.equal(result.status, 1);
    assert.equal(result.stderr, "--budget-mode must be one of: enforce\n");
    assert.equal(fs.existsSync(artifactPath(fixture, "json")), false);
  } finally {
    cleanup(fixture);
  }
});

test("the removed record-only escape hatch is rejected", () => {
  const fixture = setup({ vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly] });
  try {
    const result = run(fixture, {}, ["--budget-mode", "record-only"]);
    assert.equal(result.status, 1);
    assert.equal(result.stderr, "--budget-mode must be one of: enforce\n");
    assert.equal(fs.existsSync(artifactPath(fixture, "json")), false);
  } finally {
    cleanup(fixture);
  }
});
