import assert from "node:assert/strict";
import { test } from "node:test";
import { canonicalSha256, type LintSummary } from "../differential/linter-deletion-schema.ts";
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
    'No .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, or .tsrx files found matching patterns: ["**/*.vue"]\n',
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

void test("linter preflight validates later typecheck targets without treating them as lint projects", () => {
  const input = control(),
    project = input.registryProjects[0],
    target = {
      id: `${project.id}@docs`,
      revision: project.revision ?? null,
      fixturePath: project.fixturePath,
      runs: [
        {
          tool: "typechecker",
          status: "configuration",
          command: "target/ci/vize typechecker",
          cwd: project.fixturePath,
          outputPath: `real-project-results/shard-0/${project.id}@docs-typechecker.json`,
          durationMs: 1,
          exitCode: 1,
          fileCount: 2,
          coverage: null,
        },
      ],
    };
  project.typecheckPerformance = {
    targets: [
      { id: "app", tsconfig: "tsconfig.json", corpusGlobs: ["src/**/*.vue"] },
      { id: "docs", tsconfig: "docs/tsconfig.json", corpusGlobs: ["docs/**/*.vue"] },
    ],
  };
  input.matrix.value.projects.splice(1, 0, target);
  input.matrix.value.summary.runCount++;
  input.matrix.value.summary.configurationRuns++;

  const artifact = assessLinterDeletionShard(input);
  assert.deepEqual(
    artifact.projects.map((entry) => entry.id),
    ["with-vue", "without-vue"],
  );
});

void test("linter preflight retains explicitly excluded Matrix-only Vue files", () => {
  const input = control(),
    project = input.registryProjects.find((entry) => entry.id === "with-vue")!,
    raw = input.rawLinterRuns.get("with-vue")!.value,
    matrixProject = input.matrix.value.projects.find((entry) => entry.id === "with-vue")!,
    matrixRun = matrixProject.runs.find((entry) => entry.tool === "linter")!;
  project.lintDivergenceExcludePaths = ["src/Excluded.vue"];
  raw.parsed.push({
    file: "src/Excluded.vue",
    messages: [],
    errorCount: 0,
    warningCount: 0,
  });
  raw.stdout = JSON.stringify(raw.parsed);
  matrixRun.fileCount++;

  const artifact = assessLinterDeletionShard(input);
  assert.equal(artifact.projects[0].comparedFileCount, 2);
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

void test("a declared zero-Vue project records a zero-count artifact without project blockers", () => {
  const artifact = assessLinterDeletionShard(control()),
    zero = artifact.projects.find((project) => project.id === "without-vue")!;
  assert.equal(zero.expectedFileCount, 0);
  assert.equal(zero.reportedExpectedFileCount, 0);
  assert.equal(zero.comparedFileCount, 0);
  assert.deepEqual(zero.unmatchedPatterns, ["**/*.vue"]);
  assert.deepEqual(zero.budget, { verdict: "passed", passed: true, unusableReason: null });
  assert.deepEqual(
    artifact.blockers.filter((blocker) => blocker.detail.startsWith("without-vue: ")),
    [],
  );
  assert.doesNotThrow(() => validateLinterDeletionArtifact(artifact));

  const forged = structuredClone(artifact);
  forged.blockers.push({
    code: "linter-unmatched-patterns",
    detail: 'without-vue: unmatched explicit lint patterns ["**/*.vue"]',
  });
  forged.sha256 = canonicalSha256(
    Object.fromEntries(Object.entries(forged).filter(([key]) => key !== "sha256")),
  );
  assert.throws(
    () => validateLinterDeletionArtifact(forged),
    /linter deletion blocker projection drifted/u,
  );

  const scopeDrift = assessLinterDeletionShard(
    control({ projects: { "without-vue": { comparedFileCount: 1 } } }),
  );
  assert(
    scopeDrift.blockers.some(
      (blocker) =>
        blocker.code === "linter-project-scope" &&
        blocker.detail === "without-vue: compared 1 file(s), expected 0",
    ),
  );
});
