import assert from "node:assert/strict";
import { test } from "node:test";
import type { LintSummary } from "../differential/linter-deletion-schema.ts";
import {
  assessLinterDeletionShard,
  validateLinterDeletionArtifact,
} from "../differential/linter-deletion-verifier.ts";
import { unmatchedPatternsLinterStderr } from "../differential/linter-deletion-matrix.ts";
import { control } from "./_helpers/davinci-linter-deletion-fixture.ts";

void test("linter preflight binds Matrix diagnostics but remains deletion-blocked", () => {
  const input = control(),
    zeroInputStderr = input.rawLinterRuns.get("without-vue")!.value.stderr,
    artifact = assessLinterDeletionShard(input);
  assert.equal(
    zeroInputStderr,
    'No .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx files found matching patterns: ["**/*.vue"]\n',
  );
  assert.equal(artifact.status, "blocked");
  for (const code of [
    "linter-native-route",
    "linter-actions-custody",
    "linter-candidate-evidence",
    "linter-stability-history",
    "linter-project-membership",
  ])
    assert(artifact.blockers.some((blocker) => blocker.code === code));
  assert.deepEqual(artifact.totals, {
    projectCount: 2,
    comparedFileCount: 2,
    falsePositiveCount: 0,
    falseNegativeCount: 0,
    messageDifferenceCount: 0,
  });
  assert.doesNotThrow(() => validateLinterDeletionArtifact(artifact));
});

void test("linter preflight records every diagnostic and corpus gap", () => {
  for (const summaryChanges of [
    { falsePositiveCount: 1 },
    { falseNegativeCount: 1 },
    { messageDifferenceCount: 1 },
    { ruleLocationDivergenceCount: 1 },
    { unimplementedCount: 1 },
    { intentionalDivergenceCount: 1 },
    { patinaOnlyRuleFindingCount: 1 },
    { patinaOutsideBaselineSurfaceCount: 1 },
    { referenceCoverageIntegrityErrorCount: 1 },
    { baselineParseErrorCount: 1 },
  ] satisfies Partial<LintSummary>[]) {
    const artifact = assessLinterDeletionShard(
      control({ projects: { "with-vue": { summary: summaryChanges } } }),
    );
    assert(artifact.blockers.some((blocker) => blocker.code === "linter-divergence"));
  }
  const reduced = assessLinterDeletionShard(
    control({ projects: { "with-vue": { comparedFileCount: 0 } } }),
  );
  assert(reduced.blockers.some((blocker) => blocker.code === "linter-project-scope"));
  const forgedZero = assessLinterDeletionShard(
    control({ projects: { "without-vue": { comparedFileCount: 1 } } }),
  );
  assert(forgedZero.blockers.some((blocker) => blocker.code === "linter-project-scope"));
  const droppedConfig = assessLinterDeletionShard(
    control({ projects: { "with-vue": { droppedConfigMessageCount: 1 } } }),
  );
  assert.equal(droppedConfig.projects[0].droppedConfigMessageCount, 1);
  assert(droppedConfig.blockers.some((blocker) => blocker.code === "linter-baseline-config"));

  const unmatchedInput = control();
  unmatchedInput.registryProjects[0].vueGlobs = [
    "missing-a/**/*.vue",
    "missing-b/**/*.vue",
    "src/**/*.vue",
  ];
  unmatchedInput.rawLinterRuns.get("with-vue")!.value.stderr = unmatchedPatternsLinterStderr([
    "missing-a/**/*.vue",
    "missing-b/**/*.vue",
  ]);
  const unmatched = assessLinterDeletionShard(unmatchedInput);
  assert.deepEqual(unmatched.projects[0].unmatchedPatterns, [
    "missing-a/**/*.vue",
    "missing-b/**/*.vue",
  ]);
  assert(
    unmatched.blockers.some(
      (blocker) =>
        blocker.code === "linter-unmatched-patterns" &&
        blocker.detail ===
          'with-vue: unmatched explicit lint patterns ["missing-a/**/*.vue","missing-b/**/*.vue"]',
    ),
  );
});
