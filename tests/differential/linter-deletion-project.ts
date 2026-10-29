import assert from "node:assert/strict";
import {
  type EvidenceFile,
  type MatrixRun,
  type RawLinterRun,
  validateRawLinterRun,
} from "./linter-deletion-matrix.ts";
import {
  type JsonRecord,
  type LintProjectReport,
  type LintSummary,
  type LinterProjectEvidence,
  type RegistryProject,
  asRecord,
  canonicalSha256,
  exactKeys,
  integer,
  validateBudget,
  validateDivergence,
} from "./linter-deletion-schema.ts";

function expectedUnusableReason(
  summary: LintSummary,
  comparedFileCount: number,
  expectedFileCount: number | null,
  comparedRuleCount: number,
): string | null {
  if (summary.referenceCoverageIntegrityErrorCount > 0)
    return `expected exactly one native finding for ${summary.referenceCoverageIntegrityErrorCount} reviewed reference-coverage case(s)`;
  if (expectedFileCount === 0 && comparedFileCount !== 0)
    return "the project expected zero Vue files but selected a nonempty corpus";
  if (comparedFileCount === 0 && expectedFileCount !== 0)
    return "the project selected no Vue files";
  if (comparedRuleCount === 0)
    return "no mapped eslint-plugin-vue rule was comparable under the selected preset";
  if (summary.baselineParseErrorCount > 0)
    return `eslint-plugin-vue could not parse ${summary.baselineParseErrorCount} compared file(s)`;
  if (summary.baselineInvalidRangeCount > 0)
    return `eslint-plugin-vue reported ${summary.baselineInvalidRangeCount} finding(s) with invalid source ranges`;
  return null;
}

function parseErrorCount(results: unknown[], projectId: string): number {
  let count = 0;
  for (const [index, value] of results.entries()) {
    const result = asRecord(value, `${projectId}.baseline.parseErrorResults[${index}]`);
    assert(Array.isArray(result.messages));
    count += result.messages.filter(
      (message) => asRecord(message, `${projectId}.baseline.parseErrorMessage`).ruleId === null,
    ).length;
  }
  return count;
}

export function validateLinterProjectEvidence(
  reportFile: EvidenceFile<LintProjectReport>,
  rawRunFile: EvidenceFile<RawLinterRun>,
  matrixRun: MatrixRun,
  registryProject: RegistryProject,
  sourceRevision: string,
  indexEvidence: JsonRecord,
  ruleMapIdentity: {
    package: string;
    version: string;
    mappedRuleCount: number;
    skippedByPresetCount: number;
    comparedRuleCount: number;
  },
): LinterProjectEvidence {
  const raw = reportFile.value,
    projectId = registryProject.id;
  asRecord(raw, `${projectId} lint report`);
  exactKeys(
    raw,
    [
      "schema",
      "version",
      "project",
      "revision",
      "preset",
      "evidence",
      "files",
      "baseline",
      "raw",
      "divergence",
      "budget",
    ],
    `${projectId} lint report`,
  );
  assert.equal(raw.schema, "vize.fixtureLintDivergenceRun");
  assert.equal(raw.version, 2);
  assert.equal(raw.project, projectId);
  assert.equal(raw.revision ?? null, registryProject.revision ?? null);
  assert.equal(raw.preset, "ecosystem");
  assert.equal(raw.evidence?.commitSha, sourceRevision);
  assert.deepEqual(raw.evidence, indexEvidence, `${projectId}: index evidence drifted`);
  exactKeys(raw.files, ["comparedCount", "expectedCount"], `${projectId}.files`);
  const comparedFileCount = integer(raw.files.comparedCount, `${projectId}.files.comparedCount`),
    expectedFileCount =
      registryProject.expectedVueFileCount == null
        ? null
        : integer(registryProject.expectedVueFileCount, `${projectId}.expectedVueFileCount`);
  assert(
    raw.files.expectedCount === null ||
      (Number.isSafeInteger(raw.files.expectedCount) && Number(raw.files.expectedCount) >= 0),
  );
  exactKeys(
    raw.baseline,
    [
      "package",
      "version",
      "runtimeVersions",
      "comparedRuleCount",
      "mappedRuleCount",
      "skippedByPresetCount",
      "droppedConfigMessageCount",
      "durationMs",
      "parseErrorResults",
    ],
    `${projectId}.baseline`,
  );
  assert.equal(
    raw.baseline.package,
    ruleMapIdentity.package,
    `${projectId}: baseline package drifted`,
  );
  assert.equal(
    raw.baseline.version,
    ruleMapIdentity.version,
    `${projectId}: baseline version drifted`,
  );
  const comparedRuleCount = integer(
    raw.baseline.comparedRuleCount,
    `${projectId}.comparedRuleCount`,
  );
  assert.equal(
    comparedRuleCount,
    ruleMapIdentity.comparedRuleCount,
    `${projectId}: compared rule count drifted from coverage-gap selection`,
  );
  assert.equal(
    integer(raw.baseline.mappedRuleCount, `${projectId}.mappedRuleCount`),
    ruleMapIdentity.mappedRuleCount,
    `${projectId}: mapped rule count drifted from the rule map`,
  );
  assert.equal(
    integer(raw.baseline.skippedByPresetCount, `${projectId}.skippedByPresetCount`),
    ruleMapIdentity.skippedByPresetCount,
    `${projectId}: preset-skipped rule count drifted from the rule map`,
  );
  const droppedConfigMessageCount = integer(
    raw.baseline.droppedConfigMessageCount,
    `${projectId}.droppedConfigMessageCount`,
  );
  integer(raw.baseline.durationMs, `${projectId}.baseline.durationMs`);
  assert(Array.isArray(raw.baseline.parseErrorResults));
  const runtimeVersions = asRecord(raw.baseline.runtimeVersions, `${projectId}.runtimeVersions`);
  exactKeys(
    runtimeVersions,
    ["eslint", "eslint-plugin-vue", "vue-eslint-parser", "@typescript-eslint/parser"],
    `${projectId}.runtimeVersions`,
  );
  for (const [runtime, version] of Object.entries(runtimeVersions)) {
    assert(typeof version === "string", `${projectId}: ${runtime} version must be a string`);
    assert(version.length > 0, `${projectId}: ${runtime} version must be nonempty`);
  }
  assert.equal(
    runtimeVersions[ruleMapIdentity.package],
    ruleMapIdentity.version,
    `${projectId}: baseline runtime version drifted`,
  );
  exactKeys(raw.raw, ["patinaResults", "patinaFindings", "eslintResults"], `${projectId}.raw`);
  assert(Array.isArray(raw.raw.patinaResults));
  assert(Array.isArray(raw.raw.patinaFindings));
  assert(Array.isArray(raw.raw.eslintResults));
  assert.equal(raw.raw.patinaResults.length, comparedFileCount);
  assert.equal(raw.raw.eslintResults.length, comparedFileCount);
  const { divergence, summary } = validateDivergence(raw.divergence, projectId);
  assert.deepEqual(
    divergence.upstream,
    { package: ruleMapIdentity.package, version: ruleMapIdentity.version },
    `${projectId}: divergence upstream identity drifted`,
  );
  assert.equal(raw.raw.patinaFindings.length, summary.patinaFindingCount);
  assert.equal(
    parseErrorCount(raw.baseline.parseErrorResults, projectId),
    summary.baselineParseErrorCount,
  );
  const budget = validateBudget(raw.budget, summary, projectId);
  assert.equal(
    budget.unusableReason,
    expectedUnusableReason(summary, comparedFileCount, raw.files.expectedCount, comparedRuleCount),
    `${projectId}: linter budget reason drifted`,
  );
  const unmatchedPatterns = validateRawLinterRun(
    rawRunFile.value,
    matrixRun,
    raw,
    registryProject.vueGlobs,
  );
  return {
    id: projectId,
    fixturePath: registryProject.fixturePath,
    revision: registryProject.revision ?? null,
    expectedFileCount,
    reportedExpectedFileCount: raw.files.expectedCount,
    comparedFileCount,
    unmatchedPatterns,
    droppedConfigMessageCount,
    preset: raw.preset,
    matrixRunSha256: canonicalSha256(matrixRun),
    rawRunSha256: rawRunFile.sha256,
    reportSha256: reportFile.sha256,
    divergenceSha256: divergence.sha256,
    budget: {
      verdict: budget.verdict,
      passed: budget.passed,
      unusableReason: budget.unusableReason,
    },
    summary,
  };
}
