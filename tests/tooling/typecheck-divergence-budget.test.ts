import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  commitSha,
  readJson,
  root,
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
 * #3513: the same gate then had to learn two more things. It is the *weekly*
 * gate, so `--budget-mode record-only` records a verdict on the release-evidence
 * dispatch without failing it; and a comparison whose two sides never met is a
 * measurement failure that may never render as a pass.
 */

const vizeOnly = "error:2:1 [TS2345] vize only";
const baselineOnly = "src/App.vue(3,1): error TS2345: baseline only\n";
const passingBudget = {
  maxFalsePositiveRatio: 0.05,
  maxFalseNegativeRatio: 0.05,
  falsePositivePassed: true,
  falseNegativePassed: true,
  unusableReason: null,
  verdict: "passed",
  passed: true,
};

function artifactPath(fixture: ReturnType<typeof setup>, extension: string) {
  return path.join(fixture.reportDir, `fixture-typecheck-divergence.${extension}`);
}

/** Everything the script prints on the happy path, before any verdict line. */
function wroteLines(fixture: ReturnType<typeof setup>) {
  return [
    `Wrote ${path.relative(root, artifactPath(fixture, "json"))}`,
    `Wrote ${path.relative(root, artifactPath(fixture, "md"))}`,
    "",
  ].join("\n");
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
        "Vize Vue files: 1",
        "vue-tsc Vue files: 1",
        "Shared Vue files: 1",
        "Missing Vue files: 0",
        "Unexpected Vue files: 0",
        "Budget verdict: breached",
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
    assert.deepEqual(artifact.budget, passingBudget);
    assert.equal(artifact.divergence.summary.falsePositiveRatio, 0.05);
  } finally {
    cleanup(fixture);
  }
});

test("--budget-mode record-only records a breach without failing the job", () => {
  // The release path. `real-project-matrix.yml` is dispatched as a required
  // release gate as well as run weekly, so a breach has to stay visible without
  // blocking a release: the artifacts and the warning land, the exit code is 0.
  const fixture = setup({ vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly] });
  try {
    const result = run(fixture, {}, ["--budget-mode", "record-only"]);
    assert.equal(result.status, 0);
    assert.equal(result.stderr, "");
    assert.equal(
      result.stdout,
      `${wroteLines(fixture)}::warning title=Typecheck divergence budget not enforced::` +
        "Typecheck divergence budget breached for fixture: " +
        "1 false positives (ratio 0.5) exceed maxFalsePositiveRatio 0.05\n",
    );
    // Recording is not forgiving: the artifact still carries the failing verdict.
    assert.deepEqual(readJson(artifactPath(fixture, "json")).budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: false,
      falseNegativePassed: true,
      unusableReason: null,
      verdict: "breached",
      passed: false,
    });
  } finally {
    cleanup(fixture);
  }
});

test("--budget-mode enforce is the default and fails the job", () => {
  const fixture = setup({ vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly] });
  try {
    const expected =
      "Typecheck divergence budget breached for fixture: " +
      "1 false positives (ratio 0.5) exceed maxFalsePositiveRatio 0.05\n";
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
    assert.equal(result.stderr, "--budget-mode must be one of: enforce, record-only\n");
    assert.equal(fs.existsSync(artifactPath(fixture, "json")), false);
  } finally {
    cleanup(fixture);
  }
});
