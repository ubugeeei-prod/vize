import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  breachFailure,
  cleanup,
  commitSha,
  instrumentClassification,
  readJson,
  root,
  run,
  sha256,
  setup,
  sharedVizeDiagnostic,
  unusableFailure,
  updateVizeOutput,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

/**
 * #3513: a divergence run only measures vize when the two tools checked the same
 * Vue files. Diagnostics cannot prove that: a correct program can be clean, and
 * a broken config can still emit unrelated diagnostics. The report therefore
 * captures `vue-tsc --listFilesOnly` and compares that SFC set with Vize's
 * checked file set before interpreting the FP/FN result.
 */

const vizeOnly = "error:2:1 [TS2345] vize only";
const emptyBaselineReason =
  "vue-tsc checked 0 Vue files while Vize checked 1 (missing 1, unexpected 0)";

const artifactPath = (fixture: ReturnType<typeof setup>, extension: string) =>
  path.join(fixture.reportDir, `fixture-typecheck-divergence.${extension}`);

test("an empty vue-tsc baseline is unusable, not a false-positive breach", () => {
  const fixture = setup({
    vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly],
    baselineOutput: "",
    baselineFiles: [],
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(result.stderr, `${unusableFailure(emptyBaselineReason)}\n`);
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
        "vue-tsc excluded support Vue: 0",
        "vue-tsc excluded project-level: 0",
        "vue-tsc excluded external: 0",
        "vue-tsc configuration errors: 0",
        "vue-tsc ambient environment: isolated (no Vue runtime in program)",
        "Vize Vue files: 1",
        "vue-tsc Vue files: 0",
        "Shared Vue files: 0",
        "Missing Vue files: 1",
        "Unexpected Vue files: 0",
        "Ignored dependency Vue files: 0",
        "Ignored support Vue files: 0",
        `Seeded mutation oracle: unusable (${emptyBaselineReason})`,
        `Budget verdict: unusable (${emptyBaselineReason})`,
        `Classification: ${instrumentClassification}`,
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
      `${unusableFailure(
        "vize reported 1 and vue-tsc reported 1 diagnostics with none in common",
      )}\n`,
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

test("zero diagnostics on both sides passes when both checked the same Vue files", () => {
  const fixture = setup({ vizeDiagnostics: [], baselineOutput: "" });
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: true,
      falseNegativePassed: true,
      unusableReason: null,
      verdict: "passed",
      passed: true,
    });
    assert.equal(artifact.mutationOracle.verdict, "passed");
    assert.equal(artifact.mutationOracle.states[1].sharedCount, 1);
    assert.equal(artifact.mutationOracle.states[1].falsePositiveCount, 0);
    assert.equal(artifact.mutationOracle.states[1].falseNegativeCount, 0);
  } finally {
    cleanup(fixture);
  }
});

test("vue-tsc listFilesOnly evidence on stderr still proves baseline coverage", () => {
  const fixture = setup({
    coverageOutputStream: "stderr",
    vizeDiagnostics: [],
    baselineOutput: "",
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.equal(artifact.baseline.coverage.baselineVueFileCount, 1);
    assert.equal(artifact.baseline.coverage.sharedVueFileCount, 1);
    assert.equal(artifact.baseline.stdoutSha256, sha256(""));
    assert.notEqual(artifact.baseline.coverageStderrSha256, artifact.baseline.stdoutSha256);
    assert.equal(artifact.budget.verdict, "passed");
  } finally {
    cleanup(fixture);
  }
});

test("seeded mutation oracle tries the next deterministic candidate when a probe is invisible", () => {
  const fixture = setup({
    baselineFiles: ["src/App.vue", "src/Other.vue"],
    vizeDiagnostics: [],
    baselineOutput: "",
  });
  try {
    fs.writeFileSync(
      path.join(fixture.fixtureRoot, "src", "App.vue"),
      "<!-- vize-mutation-invisible -->\n<template />\n",
    );
    fs.writeFileSync(path.join(fixture.fixtureRoot, "src", "Other.vue"), "<template />\n");
    updateVizeOutput(fixture, (parsed) => {
      parsed.fileCount = 2;
      parsed.files = [
        { file: "src/App.vue", diagnostics: [] },
        { file: "src/Other.vue", diagnostics: [] },
      ];
    });

    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const oracle = readJson(artifactPath(fixture, "json")).mutationOracle;
    assert.equal(oracle.passed, true);
    assert.equal(oracle.file, "src/Other.vue");
    assert.equal(oracle.states[1].sharedCount, 1);
  } finally {
    cleanup(fixture);
  }
});

test("--budget-mode record-only reports an unusable baseline as a warning", () => {
  // The release path still has to say so out loud: a green shard that measured
  // nothing is exactly what this verdict exists to make visible.
  const fixture = setup({ vizeDiagnostics: [], baselineOutput: "", baselineFiles: [] });
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
          unusableFailure(emptyBaselineReason),
        "",
      ].join("\n"),
    );
    assert.equal(readJson(artifactPath(fixture, "json")).budget.passed, false);
  } finally {
    cleanup(fixture);
  }
});

test("a diagnostic-free baseline is a real breach when it covered every Vue file", () => {
  const fixture = setup({
    vizeDiagnostics: [sharedVizeDiagnostic, vizeOnly],
    baselineOutput: "",
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(
      result.stderr,
      `${breachFailure(1, "2 false positives (ratio 1) exceed maxFalsePositiveRatio 0.05")}\n`,
    );
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.equal(artifact.baseline.coverage.verdict, "usable");
    assert.equal(artifact.budget.verdict, "breached");
  } finally {
    cleanup(fixture);
  }
});

test("a partially covered Vue corpus is unusable even when diagnostics overlap", () => {
  const fixture = setup();
  try {
    fs.writeFileSync(path.join(fixture.fixtureRoot, "src", "Other.vue"), "<template />\n");
    updateVizeOutput(fixture, (parsed) => {
      parsed.fileCount = 2;
      parsed.files.splice(1, 0, { file: "src/Other.vue", diagnostics: [] });
    });

    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /vue-tsc checked 1 Vue files while Vize checked 2/);
    const coverage = readJson(artifactPath(fixture, "json")).baseline.coverage;
    assert.deepEqual(coverage.missingVueFiles, ["src/Other.vue"]);
    assert.deepEqual(coverage.unexpectedVueFiles, []);
  } finally {
    cleanup(fixture);
  }
});

test("same-sized but different Vue corpora are unusable", () => {
  const fixture = setup({ baselineFiles: ["src/Other.vue"] });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(
      result.stderr,
      /vue-tsc checked 1 Vue files while Vize checked 1 \(missing 1, unexpected 1\)/,
    );
    const coverage = readJson(artifactPath(fixture, "json")).baseline.coverage;
    assert.deepEqual(coverage.missingVueFiles, ["src/App.vue"]);
    assert.deepEqual(coverage.unexpectedVueFiles, ["src/Other.vue"]);
  } finally {
    cleanup(fixture);
  }
});

test("transitive authored TypeScript sources stay out of the Vue corpus comparison", () => {
  const fixture = setup();
  try {
    fs.writeFileSync(
      path.join(fixture.fixtureRoot, "src", "helper.ts"),
      "export const helper = 1;\n",
    );
    updateVizeOutput(fixture, (parsed) => {
      parsed.fileCount = 2;
      parsed.files.push({ file: "src/helper.ts", diagnostics: [] });
    });

    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const coverage = readJson(artifactPath(fixture, "json")).baseline.coverage;
    assert.equal(coverage.verdict, "usable");
    assert.equal(coverage.vizeVueFileCount, 1);
    assert.deepEqual(coverage.missingVueFiles, []);
    assert.deepEqual(coverage.unexpectedVueFiles, []);
  } finally {
    cleanup(fixture);
  }
});

test("transitive Vue dependencies do not expand the authored fixture corpus", () => {
  const fixture = setup({
    baselineFiles: ["src/App.vue", "node_modules/pkg/RuntimeComponent.vue"],
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const coverage = readJson(artifactPath(fixture, "json")).baseline.coverage;
    assert.equal(coverage.verdict, "usable");
    assert.equal(coverage.baselineVueFileCount, 1);
    assert.equal(coverage.ignoredDependencyVueFileCount, 1);
    assert.deepEqual(coverage.missingVueFiles, []);
    assert.deepEqual(coverage.unexpectedVueFiles, []);
  } finally {
    cleanup(fixture);
  }
});
