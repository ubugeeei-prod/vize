import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  readJson,
  run,
  setup,
  sharedBaselineOutput,
  unusableFailure,
  writeVueTsc,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

/**
 * #3513: the ledger has to be able to detect its own blindness.
 *
 * #3583 materialized the baseline as `files: [<Vize's exact Vue corpus>]`
 * extending the fixture's pinned config. That fixed the solution-style configs,
 * and it took the remaining failure out of reach of the file-coverage check
 * added in #3580: `--listFiles` prints every entry of `files` whether or not the
 * extended config resolved, so the two corpora now match by construction.
 *
 * What is left is a baseline that could not load the project. Reproduced against
 * real vue-tsc 3.3.4 / TypeScript 6.0.3, it says so and then carries on with
 * whatever options it salvaged — no `strict`, no `paths`, no `types` — checks
 * the files anyway, and hands the comparator a diagnostic stream that looks
 * exactly like a measurement:
 *
 *   error TS5083: Cannot read file '<root>/.nuxt/tsconfig.json'.
 *   tsconfig.json(2,18): error TS6053: File '<root>/.nuxt/tsconfig.app.json' not found.
 *
 * Both were dropped. The first went to `baselineExcludedProjectCount`, the second
 * to `baselineExcludedNonVueCount`, one against the materialized config itself to
 * `baselineExcludedExternalCount` — three counters, no verdict. A run in that
 * state reported `Budget passed: true`.
 */

const configuredBaselineReason = (detail: string) =>
  `vue-tsc could not load the fixture project configuration (1 error): ${detail}`;

function artifactPath(fixture: ReturnType<typeof setup>, extension: string) {
  return path.join(fixture.reportDir, `fixture-typecheck-divergence.${extension}`);
}

test("a baseline that could not read its extended config is unusable, not a pass", () => {
  // Everything else about this run is clean: one diagnostic per side, at the same
  // span, over the same single Vue file. Only the configuration line says the
  // baseline was never the fixture's project, and that alone has to sink it.
  const detail = "error TS5083: Cannot read file '/fixture/.nuxt/tsconfig.json'.";
  const fixture = setup({ baselineOutput: `${detail}\n${sharedBaselineOutput}` });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(result.stderr, `${unusableFailure(configuredBaselineReason(detail))}\n`);

    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.baseline.configuration, {
      diagnostics: [
        {
          code: 5083,
          column: null,
          file: null,
          line: null,
          message: "Cannot read file '/fixture/.nuxt/tsconfig.json'.",
          severity: "error",
        },
      ],
      errorCount: 1,
      unusableReason: configuredBaselineReason(detail),
      verdict: "unusable",
    });
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: true,
      falseNegativePassed: true,
      unusableReason: configuredBaselineReason(detail),
      verdict: "unusable",
      passed: false,
    });
    // The corpus check cannot see this: both sides carry the same one Vue file.
    assert.deepEqual(artifact.baseline.coverage.missingVueFiles, []);
    assert.deepEqual(artifact.baseline.coverage.unexpectedVueFiles, []);
    assert.equal(artifact.baseline.coverage.verdict, "usable");
    assert.equal(artifact.divergence.summary.sharedCount, 1);

    // Only the lines this guard owns: an unrelated field added to the shared
    // renderer must not fail the configuration test.
    const markdown = fs.readFileSync(artifactPath(fixture, "md"), "utf8").split("\n");
    assert.ok(markdown.includes("vue-tsc configuration errors: 1"));
    assert.ok(markdown.includes(`Budget verdict: unusable (${configuredBaselineReason(detail)})`));
    assert.ok(markdown.includes("Budget passed: false"));
  } finally {
    cleanup(fixture);
  }
});

test("a config error reported against the fixture tsconfig is unusable", () => {
  // The elk shape: a solution-style config whose references point at `.nuxt/*`
  // that `nuxt prepare` never generated. tsc reports it against the config file
  // with a span, which made it indistinguishable from an ordinary `.ts` error.
  const detail =
    "tsconfig.json(2,18): error TS6053: File '/fixture/.nuxt/tsconfig.app.json' not found.";
  const fixture = setup({ baselineOutput: `${detail}\n${sharedBaselineOutput}` });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(result.stderr, `${unusableFailure(configuredBaselineReason(detail))}\n`);
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.baseline.configuration, {
      diagnostics: [
        {
          code: 6053,
          column: 18,
          file: "tsconfig.json",
          line: 2,
          message: "File '/fixture/.nuxt/tsconfig.app.json' not found.",
          severity: "error",
        },
      ],
      errorCount: 1,
      unusableReason: configuredBaselineReason(detail),
      verdict: "unusable",
    });
    // The old lens still counts it as an ordinary excluded non-Vue diagnostic,
    // which is exactly why counting was never enough to fail the shard.
    assert.equal(artifact.divergence.summary.baselineExcludedNonVueCount, 1);
    assert.equal(artifact.divergence.summary.baselineExcludedProjectCount, 0);
  } finally {
    cleanup(fixture);
  }
});

test("a config error against the materialized baseline project is unusable", () => {
  // The materialized config lives in the report directory, outside the fixture,
  // so a diagnostic against it normalizes out of the workspace and was counted
  // as `baselineExcludedExternalCount` — the third counter nothing read.
  const fixture = setup();
  try {
    const baselineProject = path.join(fixture.reportDir, "fixture-vue-tsc.tsconfig.json");
    const detail = `${baselineProject}(3,3): error TS18002: The 'files' list in config file is empty.`;
    writeVueTsc(
      fixture.vueTsc,
      `process.stdout.write(${JSON.stringify(
        `${detail}\n${sharedBaselineOutput}${path.join(fixture.fixtureRoot, "src/App.vue")}\n`,
      )}); process.exit(2);`,
    );
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(result.stderr, `${unusableFailure(configuredBaselineReason(detail))}\n`);
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.baseline.configuration, {
      diagnostics: [
        {
          code: 18002,
          column: 3,
          file: baselineProject,
          line: 3,
          message: "The 'files' list in config file is empty.",
          severity: "error",
        },
      ],
      errorCount: 1,
      unusableReason: configuredBaselineReason(detail),
      verdict: "unusable",
    });
    assert.equal(artifact.divergence.summary.baselineExcludedExternalCount, 1);
  } finally {
    cleanup(fixture);
  }
});

test("an ordinary non-Vue diagnostic is not a configuration failure", () => {
  // The precision half. Misskey's baseline emits 94 excluded non-Vue diagnostics
  // alongside 582 scoreable `.vue` ones; a guard that failed the shard on those
  // would make the only measurable fixture unmeasurable. A positionless *warning*
  // is recorded for the same reason it does not gate: it did not change what the
  // baseline checked.
  const warning = "warning TS5102: Option 'charset' has been removed.";
  const fixture = setup({
    baselineOutput: `src/util.ts(3,1): error TS2345: baseline only\n${warning}\n${sharedBaselineOutput}`,
  });
  try {
    const result = run(fixture);
    assert.equal(result.stderr, "");
    assert.equal(result.status, 0);
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.baseline.configuration, {
      diagnostics: [
        {
          code: 5102,
          column: null,
          file: null,
          line: null,
          message: "Option 'charset' has been removed.",
          severity: "warning",
        },
      ],
      errorCount: 0,
      unusableReason: null,
      verdict: "usable",
    });
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: true,
      falseNegativePassed: true,
      unusableReason: null,
      verdict: "passed",
      passed: true,
    });
    assert.equal(artifact.divergence.summary.baselineExcludedNonVueCount, 1);
    assert.equal(artifact.divergence.summary.sharedCount, 1);
  } finally {
    cleanup(fixture);
  }
});

test("a file-list error against a Vue file is unusable, not a false negative", () => {
  // #3738, the element-plus shape. `TS6307` is reported against the source file,
  // so it read as an ordinary type error while saying the opposite: the file is
  // in the program without being in the project's file list. On run 30738583070
  // element-plus's baseline emitted 208 of these and the ledger scored every one
  // as a Vize false negative — 88% of that shard's false-negative breach — while
  // the real finding was that the materialized project never listed the `.ts`
  // files its SFCs import.
  const detail =
    "src/App.vue(1,1): error TS6307: File '/fixture/src/util.ts' is not listed within the " +
    "file list of project '/fixture/fixture-vue-tsc.tsconfig.json'. Projects must list all " +
    "files or use an 'include' pattern.";
  const fixture = setup({ baselineOutput: `${detail}\n${sharedBaselineOutput}` });
  try {
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.equal(result.stderr, `${unusableFailure(configuredBaselineReason(detail))}\n`);
    const artifact = readJson(artifactPath(fixture, "json"));
    assert.deepEqual(artifact.baseline.configuration, {
      diagnostics: [
        {
          code: 6307,
          column: 1,
          file: "src/App.vue",
          line: 1,
          message:
            "File '/fixture/src/util.ts' is not listed within the file list of project " +
            "'/fixture/fixture-vue-tsc.tsconfig.json'. Projects must list all files or use " +
            "an 'include' pattern.",
          severity: "error",
        },
      ],
      errorCount: 1,
      unusableReason: configuredBaselineReason(detail),
      verdict: "unusable",
    });
    assert.deepEqual(artifact.budget, {
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
      falsePositivePassed: true,
      falseNegativePassed: false,
      unusableReason: configuredBaselineReason(detail),
      verdict: "unusable",
      passed: false,
    });
    // Still counted as a scoreable false negative in the divergence lens: the
    // verdict is what refuses to read it as one, so the evidence stays intact.
    assert.equal(artifact.divergence.summary.falseNegativeCount, 1);
  } finally {
    cleanup(fixture);
  }
});
