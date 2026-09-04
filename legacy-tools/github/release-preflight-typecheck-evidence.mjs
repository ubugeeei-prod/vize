import { parseJsonText, readTextEntry, sha256 } from "./release-preflight-artifact-entries.mjs";

// Release evidence must prove exact typecheck parity. Ad hoc callers can still
// pass `enforceParity: false` when they intentionally inspect a broken artifact.
export const releaseTypecheckParityEnforced = true;

export function assertReleaseTypecheckShardArtifacts({
  artifactName,
  run,
  entries,
  expectedTypecheckProjects,
  observedTypecheckProjects,
  enforceParity = releaseTypecheckParityEnforced,
}) {
  const divergenceEntries = matchingEntries(
    entries,
    /(^|\/)[^/]+-typecheck-divergence\.json$/,
    `${artifactName} typecheck divergence artifact`,
  );
  const dependencyEntries = matchingEntries(
    entries,
    /(^|\/)[^/]+-typecheck-dependencies\.json$/,
    `${artifactName} typecheck dependency artifact`,
  );
  if (divergenceEntries.length !== dependencyEntries.length) {
    throw new Error(
      `${artifactName} typecheck dependency artifact count ${dependencyEntries.length} does not match divergence artifact count ${divergenceEntries.length}`,
    );
  }
  const dependencies = new Map();
  for (const entry of dependencyEntries) {
    const dependency = parseJsonText(entry.text, entry.name);
    if (dependencies.has(dependency.project)) {
      throw new Error(
        `${artifactName} duplicated typecheck dependency artifact for ${dependency.project}`,
      );
    }
    dependencies.set(dependency.project, { dependency, dependencySha256: sha256(entry.text) });
  }
  for (const entry of divergenceEntries) {
    const divergence = parseJsonText(entry.text, entry.name);
    const dependencyEvidence = dependencies.get(divergence.project);
    if (dependencyEvidence == null) {
      throw new Error(
        `${artifactName} has no typecheck dependency artifact for ${divergence.project}`,
      );
    }
    assertReleaseTypecheckDivergenceArtifact({
      artifactName,
      run,
      divergence,
      dependency: dependencyEvidence.dependency,
      dependencySha256: dependencyEvidence.dependencySha256,
      enforceParity,
    });
    if (!expectedTypecheckProjects.has(divergence.project)) {
      throw new Error(
        `${artifactName} includes unregistered typecheck performance project ${divergence.project}`,
      );
    }
    const previous = observedTypecheckProjects.get(divergence.project);
    if (previous != null) {
      throw new Error(
        `${artifactName} duplicates typecheck performance release evidence for ${divergence.project}; already seen in ${previous}`,
      );
    }
    observedTypecheckProjects.set(divergence.project, artifactName);
  }
}

export function assertReleaseTypecheckCoverage(
  expectedTypecheckProjects,
  observedTypecheckProjects,
) {
  const missing = [...expectedTypecheckProjects].filter(
    (project) => !observedTypecheckProjects.has(project),
  );
  if (missing.length > 0) {
    throw new Error(
      `Real Project Matrix release evidence is missing typecheck performance projects: ${missing.join(", ")}`,
    );
  }
}

function assertReleaseTypecheckDivergenceArtifact({
  artifactName,
  run,
  divergence,
  dependency,
  dependencySha256,
  enforceParity = releaseTypecheckParityEnforced,
}) {
  if (
    divergence.schema !== "vize.fixtureTypecheckDivergenceRun" ||
    divergence.version !== 6 ||
    divergence.evidence?.commitSha !== run.head_sha
  ) {
    throw new Error(
      `${artifactName} typecheck divergence artifact is not bound to ${run.head_sha}`,
    );
  }
  assertReleaseTypecheckParity({ artifactName, divergence, enforceParity });
  assertReleaseDependencyLink({ artifactName, divergence, dependency, dependencySha256 });
}

/**
 * The quality half of the proof. Throws when enforced; otherwise reports the
 * first failure so a waived release still says out loud what it shipped over.
 */
function assertReleaseTypecheckParity({ artifactName, divergence, enforceParity }) {
  try {
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
  } catch (error) {
    if (enforceParity) throw error;
    console.warn(`waived typecheck parity: ${error.message}`);
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

function matchingEntries(entries, pattern, label) {
  return entryNames(entries)
    .filter((name) => pattern.test(name))
    .sort((left, right) => left.localeCompare(right))
    .map((name) => ({ name, text: readTextEntry(entries, name, label) }));
}

function entryNames(entries) {
  if (entries instanceof Map) return [...entries.keys()];
  if (entries != null && typeof entries === "object") return Object.keys(entries);
  throw new Error("Real Project Matrix artifact entries must be a map or object");
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}
