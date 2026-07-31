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
  sharedVizeDiagnostic,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

/**
 * #3513: a divergence run only measures vize when the two tools actually met at
 * a diagnostic. On the v0.307.0 release run, 8 of 11 fixtures reported
 * `shared: 0` alongside hundreds of "false positives" because vue-tsc
 * typechecked nothing at all, and the other 3 reported 0/0/0. Scoring the first
 * shape as a breach blames vize for a broken instrument, and scoring the second
 * as a pass is the silent-success failure the budget gate exists to stop, so
 * both get a third verdict that is never a pass.
 */

const vizeOnly = "error:2:1 [TS2345] vize only";
const emptyBaselineReason =
  "vize reported 2 and vue-tsc reported 0 diagnostics with none in common";
const emptyBothReason = "neither vize nor vue-tsc reported a diagnostic, so nothing was compared";

function artifactPath(fixture: ReturnType<typeof setup>, extension: string) {
  return path.join(fixture.reportDir, `fixture-typecheck-divergence.${extension}`);
}

test("an empty vue-tsc baseline is unusable, not a false-positive breach", () => {
  const fixture = setup({ vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly], baselineOutput: "" });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(
      result.stderr,
      `Typecheck divergence baseline is unusable for fixture: ${emptyBaselineReason}\n`,
    );
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: false,
      falseNegativePassed: true,
      unusableReason: emptyBaselineReason,
      verdict: "unusable",
      passed: false,
    });
    assert.equal(
      fs.readFileSync(artifactPath(fixture, "md"), "utf8"),
      [
        "## fixture typecheck divergence",
        "",
        `Commit: ${commitSha}`,
        "Vize diagnostics: 2",
        "vue-tsc diagnostics: 0",
        "Shared: 0",
        "Message mismatches: 0",
        "Documented differences: 0",
        "False positives: 2 (1)",
        "False negatives: 0 (0)",
        "Vize excluded non-Vue: 0",
        "vue-tsc excluded non-Vue: 0",
        "vue-tsc excluded project-level: 0",
        "vue-tsc excluded external: 0",
        `Budget verdict: unusable (${emptyBaselineReason})`,
        "Budget passed: false",
        `Digest: ${artifact.divergence.sha256}`,
        "",
      ].join("\n"),
    );
  } finally {
    cleanup(fixture);
  }
});

test("two sides that never meet are unusable, not a breach of both budgets", () => {
  // Both tools reported a diagnostic, so neither side is empty, but they share
  // no position at all. That is a broken mapping, not a 100% error rate.
  const fixture = setup({
    vizeDiagnostics: [sharedVizeDiagnostic],
    baselineOutput: "src/App.vue(9,1): error TS2322: shared\n",
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(
      result.stderr,
      "Typecheck divergence baseline is unusable for fixture: " +
        "vize reported 1 and vue-tsc reported 1 diagnostics with none in common\n",
    );
    assert.deepEqual(readJson(artifactPath(fixture, "json")).budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: false,
      falseNegativePassed: false,
      unusableReason: "vize reported 1 and vue-tsc reported 1 diagnostics with none in common",
      verdict: "unusable",
      passed: false,
    });
  } finally {
    cleanup(fixture);
  }
});

test("zero diagnostics on both sides is unusable, never a vacuous pass", () => {
  const fixture = setup({ vizeDiagnostics: [], baselineOutput: "" });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(
      result.stderr,
      `Typecheck divergence baseline is unusable for fixture: ${emptyBothReason}\n`,
    );
    assert.deepEqual(readJson(artifactPath(fixture, "json")).budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: true,
      falseNegativePassed: true,
      unusableReason: emptyBothReason,
      verdict: "unusable",
      passed: false,
    });
  } finally {
    cleanup(fixture);
  }
});

test("--budget-mode record-only reports an unusable baseline as a warning", () => {
  // The release path still has to say so out loud: a green shard that measured
  // nothing is exactly what this verdict exists to make visible.
  const fixture = setup({ vizeDiagnostics: [], baselineOutput: "" });
  try {
    const result = run(fixture, {}, ["--budget-mode", "record-only"]);
    assert.equal(result.status, 0);
    assert.equal(result.stderr, "");
    assert.equal(
      result.stdout,
      [
        `Wrote ${path.relative(root, artifactPath(fixture, "json"))}`,
        `Wrote ${path.relative(root, artifactPath(fixture, "md"))}`,
        "::warning title=Typecheck divergence budget not enforced::" +
          `Typecheck divergence baseline is unusable for fixture: ${emptyBothReason}`,
        "",
      ].join("\n"),
    );
    assert.equal(readJson(artifactPath(fixture, "json")).budget.passed, false);
  } finally {
    cleanup(fixture);
  }
});
