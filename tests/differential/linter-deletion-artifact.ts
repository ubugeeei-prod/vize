import assert from "node:assert/strict";
import {
  type Blocker,
  type LinterDeletionArtifact,
  type LinterProjectEvidence,
  asRecord,
  canonicalSha256,
  commitPattern,
  exactKeys,
  hashPattern,
  integer,
  parityFields,
  projectPattern,
  requiredStages,
  validateSummary,
} from "./linter-deletion-schema.ts";

const requiredBlockers: Blocker[] = [
  {
    code: "linter-native-route",
    detail: "Matrix lint artifacts have no native-only parse/lint/output route receipt",
  },
  {
    code: "linter-actions-custody",
    detail: "No Actions run and uploaded-artifact custody receipt binds this shard evidence",
  },
  {
    code: "linter-candidate-evidence",
    detail: "No exact-candidate T1/T2 deletion evidence consumes this shard artifact",
  },
  {
    code: "linter-stability-history",
    detail: "No scheduled stable-history receipt consumes this shard artifact",
  },
  {
    code: "linter-project-membership",
    detail: "Linter acceptance fixtures carry no real-project projectId membership",
  },
];
const artifactFields = [
  "schema",
  "version",
  "sourceRevision",
  "target",
  "registry",
  "shard",
  "buildReceiptSha256",
  "binarySha256",
  "requiredStages",
  "measurement",
  "inputs",
  "nativeRouteReceipt",
  "actionsCustodyReceipt",
  "candidateEvidenceReceipt",
  "stabilityHistoryReceipt",
  "fixtureProjectMembershipReceipt",
  "projects",
  "totals",
  "status",
  "blockers",
  "sha256",
];
const projectFields = [
  "id",
  "fixturePath",
  "revision",
  "expectedFileCount",
  "reportedExpectedFileCount",
  "comparedFileCount",
  "unmatchedPatterns",
  "droppedConfigMessageCount",
  "preset",
  "matrixRunSha256",
  "rawRunSha256",
  "reportSha256",
  "divergenceSha256",
  "budget",
  "summary",
];

export function expectedLinterBlockers(projects: LinterProjectEvidence[]): Blocker[] {
  const blockers = requiredBlockers.map((blocker) => ({ ...blocker }));
  for (const project of projects) {
    if (
      project.reportedExpectedFileCount !== project.expectedFileCount ||
      (project.expectedFileCount === null
        ? project.comparedFileCount === 0
        : project.comparedFileCount !== project.expectedFileCount)
    )
      blockers.push({
        code: "linter-project-scope",
        detail:
          `${project.id}: compared ${project.comparedFileCount} file(s), expected ` +
          (project.expectedFileCount === null
            ? "a nonzero real corpus"
            : project.expectedFileCount),
      });
    if (!project.budget.passed)
      blockers.push({
        code: "linter-budget",
        detail: `${project.id}: ${project.budget.verdict}${
          project.budget.unusableReason ? ` (${project.budget.unusableReason})` : ""
        }`,
      });
    if (project.droppedConfigMessageCount > 0)
      blockers.push({
        code: "linter-baseline-config",
        detail: `${project.id}: baseline discarded ${project.droppedConfigMessageCount} config message(s)`,
      });
    if (project.unmatchedPatterns.length > 0)
      blockers.push({
        code: "linter-unmatched-patterns",
        detail: `${project.id}: unmatched explicit lint patterns ${JSON.stringify(project.unmatchedPatterns)}`,
      });
    const gaps = parityFields.filter((field) => project.summary[field] !== 0);
    if (gaps.length)
      blockers.push({
        code: "linter-divergence",
        detail: `${project.id}: ${gaps
          .map((field) => `${field}=${project.summary[field]}`)
          .join(", ")}`,
      });
  }
  return blockers;
}

export function validateLinterDeletionArtifact(value: unknown): LinterDeletionArtifact {
  const record = asRecord(value, "linter deletion artifact");
  exactKeys(record, artifactFields, "linter deletion artifact");
  const artifact = record as LinterDeletionArtifact,
    registry = asRecord(artifact.registry, "linter deletion registry"),
    shard = asRecord(artifact.shard, "linter deletion shard"),
    measurement = asRecord(artifact.measurement, "linter deletion measurement"),
    inputs = asRecord(artifact.inputs, "linter deletion inputs");
  exactKeys(registry, ["path", "sha256"], "linter deletion registry");
  exactKeys(shard, ["index", "count"], "linter deletion shard");
  exactKeys(measurement, ["preset", "measureCoverageGap"], "linter deletion measurement");
  exactKeys(
    inputs,
    ["matrix", "divergenceIndex", "ruleMap", "divergenceLedger"],
    "linter deletion inputs",
  );
  assert.equal(artifact.schema, "vize.deletionReadiness.linterShard");
  assert.equal(artifact.version, 1);
  assert.match(artifact.sourceRevision, commitPattern);
  assert.equal(artifact.target, "lint");
  assert.equal(artifact.registry?.path, "tests/_fixtures/vue-ecosystem-fixtures.json");
  assert.match(artifact.registry?.sha256, hashPattern);
  assert.match(artifact.buildReceiptSha256, hashPattern);
  assert.match(artifact.binarySha256, hashPattern);
  assert.deepEqual(artifact.requiredStages, requiredStages);
  assert.deepEqual(artifact.measurement, { preset: "ecosystem", measureCoverageGap: true });
  const inputPaths = {
    matrix: "summary.json",
    divergenceIndex: "lint-divergence-summary.json",
    ruleMap: "tests/_fixtures/patina-eslint-vue-rule-map.json",
    divergenceLedger: "tests/_fixtures/patina-lint-documented-divergences.json",
  };
  for (const [name, expectedPath] of Object.entries(inputPaths)) {
    const input = asRecord(inputs[name], `linter deletion ${name} input`),
      digest = input.sha256;
    exactKeys(input, ["path", "sha256"], `linter deletion ${name} input`);
    assert.equal(input.path, expectedPath);
    assert(typeof digest === "string");
    assert.match(digest, hashPattern);
  }
  for (const receipt of [
    artifact.nativeRouteReceipt,
    artifact.actionsCustodyReceipt,
    artifact.candidateEvidenceReceipt,
    artifact.stabilityHistoryReceipt,
    artifact.fixtureProjectMembershipReceipt,
  ])
    assert.equal(receipt, null);
  integer(artifact.shard.index, "linter deletion shard index");
  assert(
    Number.isSafeInteger(artifact.shard?.count) && artifact.shard.count > artifact.shard.index,
    "linter deletion shard count is invalid",
  );
  assert(Array.isArray(artifact.projects) && artifact.projects.length > 0);
  assert.equal(
    new Set(artifact.projects.map((project) => project.id)).size,
    artifact.projects.length,
  );
  const totals = {
    projectCount: artifact.projects.length,
    comparedFileCount: 0,
    falsePositiveCount: 0,
    falseNegativeCount: 0,
    messageDifferenceCount: 0,
  };
  for (const project of artifact.projects) {
    exactKeys(
      asRecord(project, `${project.id} deletion project`),
      projectFields,
      `${project.id} deletion project`,
    );
    assert.match(project.id, projectPattern);
    assert.equal(typeof project.fixturePath, "string");
    assert(project.fixturePath.length > 0);
    assert(project.revision === null || commitPattern.test(project.revision));
    for (const count of [project.expectedFileCount, project.reportedExpectedFileCount])
      assert(count === null || (Number.isSafeInteger(count) && count >= 0));
    const compared = integer(project.comparedFileCount, `${project.id}.comparedFileCount`);
    assert(Array.isArray(project.unmatchedPatterns));
    assert(
      project.unmatchedPatterns.every(
        (pattern) => typeof pattern === "string" && pattern.length > 0,
      ),
      `${project.id}: unmatched patterns must be nonempty strings`,
    );
    assert.equal(
      new Set(project.unmatchedPatterns).size,
      project.unmatchedPatterns.length,
      `${project.id}: unmatched patterns must be unique`,
    );
    integer(project.droppedConfigMessageCount, `${project.id}.droppedConfigMessageCount`);
    assert.equal(project.preset, artifact.measurement.preset);
    for (const digest of [
      project.matrixRunSha256,
      project.rawRunSha256,
      project.reportSha256,
      project.divergenceSha256,
    ])
      assert.match(digest, hashPattern);
    const summary = validateSummary(project.summary, `${project.id}.summary`),
      budget = asRecord(project.budget, `${project.id}.budget`),
      verdict = budget.verdict,
      passed = budget.passed,
      unusableReason = budget.unusableReason;
    exactKeys(budget, ["verdict", "passed", "unusableReason"], `${project.id}.budget`);
    assert(typeof verdict === "string" && ["passed", "breached", "unusable"].includes(verdict));
    assert(typeof passed === "boolean");
    assert(
      unusableReason === null || (typeof unusableReason === "string" && unusableReason.length > 0),
    );
    assert.equal(passed, verdict === "passed", `${project.id}: projected budget passed drifted`);
    assert.equal(
      unusableReason !== null,
      verdict === "unusable",
      `${project.id}: projected unusable budget drifted`,
    );
    totals.comparedFileCount += compared;
    totals.falsePositiveCount += summary.falsePositiveCount;
    totals.falseNegativeCount += summary.falseNegativeCount;
    totals.messageDifferenceCount += summary.messageDifferenceCount;
  }
  exactKeys(
    asRecord(artifact.totals, "linter deletion totals"),
    [
      "projectCount",
      "comparedFileCount",
      "falsePositiveCount",
      "falseNegativeCount",
      "messageDifferenceCount",
    ],
    "linter deletion totals",
  );
  assert.deepEqual(artifact.totals, totals, "linter deletion totals drifted");
  assert.equal(artifact.status, "blocked");
  assert(Array.isArray(artifact.blockers));
  for (const blocker of artifact.blockers) {
    exactKeys(
      asRecord(blocker, "linter deletion blocker"),
      ["code", "detail"],
      "linter deletion blocker",
    );
    assert.equal(typeof blocker?.code, "string");
    assert(blocker.code.length > 0);
    assert.equal(typeof blocker?.detail, "string");
    assert(blocker.detail.length > 0);
  }
  assert.deepEqual(
    artifact.blockers,
    expectedLinterBlockers(artifact.projects),
    "linter deletion blocker projection drifted",
  );
  for (const { code } of requiredBlockers)
    assert(
      artifact.blockers.some((blocker) => blocker.code === code),
      `${code} must be explicit`,
    );
  const { sha256: recorded, ...unsigned } = artifact;
  assert.match(recorded, hashPattern);
  assert.equal(recorded, canonicalSha256(unsigned), "linter deletion artifact digest mismatch");
  return artifact;
}
