import {
  exactMatchingEntries,
  parseJsonText,
  readJsonEntry,
  readTextEntry,
  sha256,
} from "./release-preflight-artifact-entries.mjs";

export const requiredRealProjectMatrixShardCount = 11;
export const realProjectMatrixWorkflowName = "Real Project Matrix";

// Release evidence for the full corpus is mandatory: a selection that omits the
// workflow must fail the gate instead of silently skipping all shard artifacts.
export function requireRealProjectMatrixRun(selectedRuns) {
  const run = selectedRuns?.get?.(realProjectMatrixWorkflowName);
  if (run == null) {
    throw new Error(
      `${realProjectMatrixWorkflowName} release evidence is required; no exact-SHA run was selected`,
    );
  }
  return run;
}

export async function assertRealProjectMatrixReleaseArtifacts({
  run,
  artifacts,
  readArtifactEntries,
}) {
  if (typeof readArtifactEntries !== "function") {
    throw new Error("Real Project Matrix artifact reader is required");
  }
  const expectedNames = Array.from(
    { length: requiredRealProjectMatrixShardCount },
    (_, shard) => `real-project-matrix-${shard}`,
  );
  for (const artifactName of expectedNames) {
    const matches = artifacts.filter((artifact) => artifact.name === artifactName);
    if (matches.length !== 1) {
      throw new Error(
        `Real Project Matrix release evidence must contain exactly one ${artifactName} artifact; found ${matches.length}`,
      );
    }
    const artifact = matches[0];
    assertArtifactBoundToRun(run, artifact);
    const entries = await readArtifactEntries(artifact);
    assertRealProjectShardArtifact({
      run,
      artifactName,
      shard: Number(artifactName.slice("real-project-matrix-".length)),
      entries,
    });
  }
}

function assertRealProjectShardArtifact({ run, artifactName, shard, entries }) {
  const summary = readJsonEntry(entries, "summary.json", artifactName);
  if (
    summary.schema !== "vize.fixtureToolMatrixReport" ||
    summary.version !== 3 ||
    summary.evidence?.commitSha !== run.head_sha ||
    summary.command?.shardIndex !== shard ||
    summary.command?.shardCount !== requiredRealProjectMatrixShardCount
  ) {
    throw new Error(`${artifactName} summary is not exact release evidence for ${run.head_sha}`);
  }
  const selectedFixtures = readTextEntry(entries, "selected-fixtures.txt", artifactName)
    .split(/\r?\n/)
    .filter(Boolean);
  if (selectedFixtures.length === 0) {
    throw new Error(`${artifactName} selected no authored fixture corpus`);
  }

  const surface = readJsonEntry(entries, "surface-verdict.json", artifactName);
  if (surface.status !== "success") {
    throw new Error(`${artifactName} surface verdict is ${String(surface.status)}`);
  }
  assertReleaseLintDivergenceSummary({
    artifactName,
    run,
    summary: readJsonEntry(entries, "lint-divergence-summary.json", artifactName),
  });

  const [divergenceEntry] = exactMatchingEntries(
    entries,
    /(^|\/)[^/]+-typecheck-divergence\.json$/,
    `${artifactName} typecheck divergence artifact`,
  );
  const [dependencyEntry] = exactMatchingEntries(
    entries,
    /(^|\/)[^/]+-typecheck-dependencies\.json$/,
    `${artifactName} typecheck dependency artifact`,
  );
  const divergence = parseJsonText(divergenceEntry.text, divergenceEntry.name);
  const dependency = parseJsonText(dependencyEntry.text, dependencyEntry.name);
  assertReleaseTypecheckDivergenceArtifact({
    artifactName,
    run,
    divergence,
    dependency,
    dependencySha256: sha256(dependencyEntry.text),
  });
}

function assertReleaseTypecheckDivergenceArtifact({
  artifactName,
  run,
  divergence,
  dependency,
  dependencySha256,
}) {
  if (
    divergence.schema !== "vize.fixtureTypecheckDivergenceRun" ||
    divergence.version !== 5 ||
    divergence.evidence?.commitSha !== run.head_sha
  ) {
    throw new Error(
      `${artifactName} typecheck divergence artifact is not bound to ${run.head_sha}`,
    );
  }
  if (divergence.enforcement?.budgetMode !== "enforce") {
    throw new Error(
      `${artifactName} typecheck divergence artifact used ${String(divergence.enforcement?.budgetMode)} mode; release evidence must not be record-only`,
    );
  }
  if (divergence.budget?.passed !== true || divergence.budget?.verdict !== "passed") {
    throw new Error(
      `${artifactName} typecheck divergence budget is ${String(divergence.budget?.verdict)}`,
    );
  }

  const summary = divergence.divergence?.summary;
  if (summary?.falsePositiveCount !== 0 || summary?.falseNegativeCount !== 0) {
    throw new Error(
      `${artifactName} typecheck divergence must have zero unexplained false positives and false negatives; got ${String(summary?.falsePositiveCount)} FP and ${String(summary?.falseNegativeCount)} FN`,
    );
  }
  assertReleaseVueCoverage(artifactName, divergence.baseline?.coverage);
  assertReleaseMutationOracle(artifactName, divergence.mutationOracle);
  assertReleaseDependencyLink({ artifactName, divergence, dependency, dependencySha256 });
}

function assertReleaseLintDivergenceSummary({ artifactName, run, summary }) {
  if (
    summary.schema !== "vize.fixtureLintDivergenceIndex" ||
    summary.version !== 1 ||
    summary.evidence?.commitSha !== run.head_sha ||
    !Number.isSafeInteger(summary.projectCount) ||
    summary.projectCount <= 0 ||
    !Array.isArray(summary.projects) ||
    summary.projects.length !== summary.projectCount
  ) {
    throw new Error(`${artifactName} lint divergence summary is not exact release evidence`);
  }
}

function assertReleaseVueCoverage(artifactName, coverage) {
  if (
    coverage?.verdict !== "usable" ||
    !Number.isSafeInteger(coverage.sharedVueFileCount) ||
    !Number.isSafeInteger(coverage.vizeVueFileCount) ||
    !Number.isSafeInteger(coverage.baselineVueFileCount) ||
    coverage.sharedVueFileCount <= 0 ||
    coverage.vizeVueFileCount !== coverage.baselineVueFileCount ||
    coverage.sharedVueFileCount !== coverage.vizeVueFileCount ||
    coverage.vizeVueFilesSha256 !== coverage.baselineVueFilesSha256 ||
    !isSha256(coverage.vizeVueFilesSha256) ||
    !Array.isArray(coverage.missingVueFiles) ||
    !Array.isArray(coverage.unexpectedVueFiles) ||
    coverage.missingVueFiles.length !== 0 ||
    coverage.unexpectedVueFiles.length !== 0
  ) {
    throw new Error(
      `${artifactName} did not prove both tools checked the same non-empty authored Vue corpus`,
    );
  }
}

function assertReleaseMutationOracle(artifactName, mutationOracle) {
  const states = mutationOracle?.states ?? [];
  const [clean, broken, repaired] = states;
  if (
    mutationOracle?.schema !== "vize.fixtureTypecheckSeededMutationOracle" ||
    mutationOracle.version !== 1 ||
    mutationOracle.passed !== true ||
    mutationOracle.verdict !== "passed" ||
    mutationOracle.cleanExpectedDiagnosticPresent !== false ||
    mutationOracle.expectedDiagnosticMatched !== true ||
    mutationOracle.repairedExpectedDiagnosticPresent !== false ||
    clean?.name !== "clean" ||
    broken?.name !== "broken" ||
    repaired?.name !== "repaired" ||
    !hasMutationStateEvidence(clean) ||
    !hasMutationStateEvidence(broken) ||
    !hasMutationStateEvidence(repaired) ||
    !isSha256(clean.sourceSha256) ||
    !isSha256(broken.sourceSha256) ||
    !isSha256(repaired.sourceSha256) ||
    clean.sharedCount !== 0 ||
    clean.falsePositiveCount !== 0 ||
    clean.falseNegativeCount !== 0 ||
    broken.sourceSha256 === clean.sourceSha256 ||
    broken.sharedCount !== 1 ||
    broken.messageMismatchCount !== 0 ||
    broken.documentedDifferenceCount !== 0 ||
    broken.falsePositiveCount !== 0 ||
    broken.falseNegativeCount !== 0 ||
    repaired.sourceSha256 !== clean.sourceSha256 ||
    repaired.sharedCount !== 0 ||
    repaired.messageMismatchCount !== 0 ||
    repaired.documentedDifferenceCount !== 0 ||
    repaired.falsePositiveCount !== 0 ||
    repaired.falseNegativeCount !== 0
  ) {
    throw new Error(`${artifactName} has no passing seeded mutation oracle`);
  }
}

function hasMutationStateEvidence(state) {
  return (
    hasSummaryEvidence(state.observed) &&
    hasRunEvidence(state.vize) &&
    hasRunEvidence(state.baseline)
  );
}

function hasSummaryEvidence(summary) {
  return [
    "vizeDiagnosticCount",
    "baselineDiagnosticCount",
    "sharedCount",
    "messageMismatchCount",
    "documentedDifferenceCount",
    "falsePositiveCount",
    "falseNegativeCount",
  ].every((key) => Number.isSafeInteger(summary?.[key]) && summary[key] >= 0);
}

function hasRunEvidence(run) {
  return (
    typeof run?.command === "string" &&
    run.command.length > 0 &&
    Number.isSafeInteger(run.exitCode) &&
    isSha256(run.stdoutSha256) &&
    isSha256(run.stderrSha256)
  );
}

function assertReleaseDependencyLink({ artifactName, divergence, dependency, dependencySha256 }) {
  if (
    dependency.schema !== "vize.fixtureTypecheckDependencyInstall" ||
    dependency.version !== 2 ||
    dependency.project !== divergence.project ||
    dependency.revision !== divergence.revision ||
    dependency.evidence?.commitSha !== divergence.evidence?.commitSha
  ) {
    throw new Error(`${artifactName} typecheck dependency evidence is not bound to divergence`);
  }
  if (
    divergence.preparation?.schema !== "vize.fixtureTypecheckPreparationEvidence" ||
    divergence.preparation.version !== 1 ||
    divergence.preparation.payloadSha256 !== dependencySha256
  ) {
    throw new Error(
      `${artifactName} divergence artifact is missing dependency preparation linkage`,
    );
  }
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function assertArtifactBoundToRun(run, artifact) {
  if (artifact.expired === true) {
    throw new Error(`Real Project Matrix artifact ${String(artifact.name)} has expired`);
  }
  const source = artifact.workflow_run;
  if (
    source == null ||
    Number(source.id) !== Number(run.id) ||
    source.head_sha !== run.head_sha ||
    source.head_branch !== run.head_branch
  ) {
    throw new Error(
      `Real Project Matrix artifact ${String(artifact.name)} is not bound to run ${String(run.id)}`,
    );
  }
}
