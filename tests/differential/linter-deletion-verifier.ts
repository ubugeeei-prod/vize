import assert from "node:assert/strict";
import {
  expectedLinterBlockers,
  validateLinterDeletionArtifact,
} from "./linter-deletion-artifact.ts";
import {
  type EvidenceFile,
  type FixtureToolMatrix,
  type RawLinterRun,
  assertSupportedLinterVueGlob,
  validateFixtureToolMatrix,
} from "./linter-deletion-matrix.ts";
import { validateLinterProjectEvidence } from "./linter-deletion-project.ts";
import {
  type BuildReceipt,
  type JsonRecord,
  type LintIndex,
  type LintProjectReport,
  type LinterDeletionArtifact,
  type LinterProjectEvidence,
  type RegistryProject,
  asRecord,
  canonicalSha256,
  commitPattern,
  exactIds,
  exactKeys,
  hashPattern,
  integer,
  projectPattern,
  requiredStages,
  totalFields,
  validateBuildReceipt,
} from "./linter-deletion-schema.ts";

export { validateLinterDeletionArtifact } from "./linter-deletion-artifact.ts";
export { lintDivergenceSha256 } from "./linter-deletion-schema.ts";
export type {
  FixtureToolMatrix,
  EvidenceFile,
  MatrixRun,
  RawLinterRun,
} from "./linter-deletion-matrix.ts";
export type { LintIndex, LintProjectReport, RegistryProject } from "./linter-deletion-schema.ts";

export interface LinterDeletionInput {
  sourceRevision: string;
  registrySha256: string;
  shardIndex: number;
  shardCount: number;
  registryProjects: RegistryProject[];
  matrix: EvidenceFile<FixtureToolMatrix>;
  rawLinterRuns: Map<string, EvidenceFile<RawLinterRun>>;
  index: EvidenceFile<LintIndex>;
  projectArtifacts: Map<string, EvidenceFile<LintProjectReport>>;
  ruleMap: EvidenceFile<unknown>;
  divergenceLedgerSha256: string;
  buildReceipt: BuildReceipt;
}

export function linterProjects(
  registryProjects: RegistryProject[],
  shardIndex: number,
  shardCount: number,
): RegistryProject[] {
  for (const project of registryProjects) {
    assert(Array.isArray(project.coverage), `${project.id}: coverage is required`);
    assert(
      project.coverage.includes("linter"),
      `${project.id}: every Matrix project must carry linter coverage`,
    );
  }
  return registryProjects.filter((_, index) => index % shardCount === shardIndex);
}

interface RuleMapCoverage {
  package: string;
  version: string;
  mappedRuleCount: number;
  skippedByPresetCount: number;
  comparedRuleCount: number;
}

function validateRuleMapIdentity(value: unknown): RuleMapCoverage {
  const ruleMap = asRecord(value, "linter rule map"),
    entries = asRecord(ruleMap.entries, "linter rule map entries"),
    summary = asRecord(ruleMap.summary, "linter rule map summary"),
    upstream = asRecord(ruleMap.upstream, "linter rule map upstream");
  exactKeys(ruleMap, ["entries", "schemaVersion", "summary", "upstream"], "linter rule map");
  assert.equal(ruleMap.schemaVersion, 1);
  assert(Object.keys(entries).length > 0, "linter rule map must not be empty");
  exactKeys(summary, ["intentionalDivergence", "mapped", "unimplemented"], "rule map summary");
  exactKeys(upstream, ["package", "ruleCount", "version"], "rule map upstream");
  const counts = { intentionalDivergence: 0, mapped: 0, unimplemented: 0 };
  let skippedByPresetCount = 0;
  for (const [ruleId, value] of Object.entries(entries)) {
    const entry = asRecord(value, `linter rule map ${ruleId}`),
      status = entry.status;
    if (status === "mapped") {
      counts.mapped++;
      assert.equal(typeof entry.patinaRule, "string", `${ruleId}: mapped rule target is required`);
      if (!Array.isArray(entry.patinaPresets) || !entry.patinaPresets.includes("ecosystem"))
        skippedByPresetCount++;
    } else if (status === "unimplemented") counts.unimplemented++;
    else if (status === "intentional-divergence") counts.intentionalDivergence++;
    else assert.fail(`${ruleId}: unsupported rule-map status`);
  }
  const ruleCount = integer(upstream.ruleCount, "rule map upstream ruleCount"),
    version = upstream.version;
  assert.equal(ruleCount, Object.keys(entries).length, "rule map ruleCount drifted");
  for (const [field, count] of Object.entries(counts))
    assert.equal(integer(summary[field], `rule map summary ${field}`), count);
  assert.equal(upstream.package, "eslint-plugin-vue");
  assert(typeof version === "string" && version.length > 0);
  return {
    package: "eslint-plugin-vue",
    version,
    mappedRuleCount: counts.mapped,
    skippedByPresetCount,
    comparedRuleCount: counts.mapped - skippedByPresetCount + counts.unimplemented,
  };
}

function expectedIndexBudget(projects: LinterProjectEvidence[]): JsonRecord {
  const failed = projects.filter((project) => !project.budget.passed);
  return {
    status: projects.length > 0 && failed.length === 0 ? "success" : "failure",
    passed: projects.length > 0 && failed.length === 0,
    projectCount: projects.length,
    passedCount: projects.length - failed.length,
    failedCount: failed.length,
    unusableCount: failed.filter((project) => project.budget.verdict === "unusable").length,
    breachedCount: failed.filter((project) => project.budget.verdict === "breached").length,
    failedProjects: failed.map((project) => project.id),
  };
}

function validateIndex(
  index: LintIndex,
  projects: LinterProjectEvidence[],
  sourceRevision: string,
): void {
  exactKeys(
    index,
    ["schema", "version", "evidence", "preset", "projectCount", "budget", "totals", "projects"],
    "lint divergence index",
  );
  assert.equal(index.schema, "vize.fixtureLintDivergenceIndex");
  assert.equal(index.version, 1);
  exactKeys(index.evidence, ["commitSha", "runtime", "machine"], "lint index evidence");
  assert.equal(index.evidence?.commitSha, sourceRevision);
  const runtime = asRecord(index.evidence.runtime, "lint index runtime"),
    machine = asRecord(index.evidence.machine, "lint index machine");
  exactKeys(runtime, ["name", "version"], "lint index runtime");
  assert.equal(runtime.name, "node");
  assert(typeof runtime.version === "string");
  assert(runtime.version.length > 0);
  exactKeys(machine, ["platform", "arch", "logicalCpuCount"], "lint index machine");
  assert.equal(typeof machine.platform, "string");
  assert.equal(typeof machine.arch, "string");
  assert(integer(machine.logicalCpuCount, "lint index logicalCpuCount") > 0);
  assert.equal(index.preset, "ecosystem");
  assert.equal(index.projectCount, projects.length);
  exactKeys(index.totals, [...totalFields], "lint divergence index totals");
  exactIds(
    index.projects.map((project) => project.project),
    projects.map((project) => project.id),
    "lint divergence index projects",
  );
  for (const project of projects)
    assert.deepEqual(
      index.projects.find((item) => item.project === project.id),
      { project: project.id, ...project.summary },
      `${project.id}: lint index summary drifted from its report`,
    );
  for (const field of totalFields)
    assert.equal(
      index.totals[field],
      projects.reduce((sum, project) => sum + project.summary[field], 0),
      `lint divergence index total drifted: ${field}`,
    );
  assert.deepEqual(index.budget, expectedIndexBudget(projects));
}

export function assessLinterDeletionShard(input: LinterDeletionInput): LinterDeletionArtifact {
  const {
    sourceRevision,
    registrySha256,
    shardIndex,
    shardCount,
    registryProjects,
    matrix,
    rawLinterRuns,
    index,
    projectArtifacts,
    ruleMap,
    divergenceLedgerSha256,
    buildReceipt,
  } = input;
  assert.match(sourceRevision, commitPattern);
  for (const digest of [
    registrySha256,
    matrix.sha256,
    index.sha256,
    ruleMap.sha256,
    divergenceLedgerSha256,
  ])
    assert.match(digest, hashPattern, "linter input digests must be sha256");
  const ruleMapIdentity = validateRuleMapIdentity(ruleMap.value);
  integer(shardIndex, "shardIndex");
  assert(Number.isSafeInteger(shardCount) && shardCount > 0 && shardIndex < shardCount);
  const registryIds = registryProjects.map((project) => project.id);
  assert.equal(new Set(registryIds).size, registryIds.length);
  for (const project of registryProjects) {
    assert.match(project.id, projectPattern);
    assert.equal(typeof project.fixturePath, "string");
    assert(project.fixturePath.length > 0);
    assert(
      Array.isArray(project.vueGlobs) && project.vueGlobs.length > 0,
      `${project.id}: vueGlobs are required`,
    );
    for (const [index, pattern] of project.vueGlobs.entries())
      assertSupportedLinterVueGlob(pattern, `${project.id}.vueGlobs[${index}]`);
    assert.equal(
      new Set(project.vueGlobs).size,
      project.vueGlobs.length,
      `${project.id}: vueGlobs must be unique`,
    );
  }
  const selected = linterProjects(registryProjects, shardIndex, shardCount);
  assert(selected.length > 0, "linter verifier refuses an empty shard");
  for (const [label, ids] of [
    ["linter project artifacts", [...projectArtifacts.keys()]],
    ["raw Matrix linter artifacts", [...rawLinterRuns.keys()]],
  ] satisfies [string, string[]][])
    exactIds(
      ids,
      selected.map((project) => project.id),
      label,
    );
  const matrixRuns = validateFixtureToolMatrix(
    matrix.value,
    selected,
    sourceRevision,
    shardIndex,
    shardCount,
  );
  assert.equal(
    index.value.evidence?.commitSha,
    sourceRevision,
    "lint divergence index source SHA drifted",
  );
  const matrixMachine = asRecord(matrix.value.evidence.machine, "matrix machine"),
    lintMachine = asRecord(index.value.evidence.machine, "lint divergence machine");
  for (const field of ["platform", "arch", "logicalCpuCount"])
    assert.equal(
      lintMachine[field],
      matrixMachine[field],
      `Matrix and lint-divergence machine identity drifted: ${field}`,
    );
  const build = validateBuildReceipt(buildReceipt, sourceRevision);
  const projects = selected.map((project) => {
    const report = projectArtifacts.get(project.id),
      rawRun = rawLinterRuns.get(project.id),
      matrixRun = matrixRuns.get(project.id);
    assert(report && rawRun && matrixRun, `${project.id}: complete linter evidence is required`);
    return validateLinterProjectEvidence(
      report,
      rawRun,
      matrixRun,
      project,
      sourceRevision,
      index.value.evidence,
      ruleMapIdentity,
    );
  });
  validateIndex(index.value, projects, sourceRevision);
  const blockers = expectedLinterBlockers(projects);
  const totals = {
    projectCount: projects.length,
    comparedFileCount: projects.reduce((sum, project) => sum + project.comparedFileCount, 0),
    falsePositiveCount: projects.reduce(
      (sum, project) => sum + project.summary.falsePositiveCount,
      0,
    ),
    falseNegativeCount: projects.reduce(
      (sum, project) => sum + project.summary.falseNegativeCount,
      0,
    ),
    messageDifferenceCount: projects.reduce(
      (sum, project) => sum + project.summary.messageDifferenceCount,
      0,
    ),
  };
  const unsigned = {
    schema: "vize.deletionReadiness.linterShard",
    version: 1,
    sourceRevision,
    target: "lint" as const,
    registry: { path: "tests/_fixtures/vue-ecosystem-fixtures.json", sha256: registrySha256 },
    shard: { index: shardIndex, count: shardCount },
    buildReceiptSha256: build.sha256,
    binarySha256: build.receipt.binarySha256,
    requiredStages,
    measurement: { preset: "ecosystem" as const, measureCoverageGap: true as const },
    inputs: {
      matrix: { path: "summary.json" as const, sha256: matrix.sha256 },
      divergenceIndex: {
        path: "lint-divergence-summary.json" as const,
        sha256: index.sha256,
      },
      ruleMap: {
        path: "tests/_fixtures/patina-eslint-vue-rule-map.json" as const,
        sha256: ruleMap.sha256,
      },
      divergenceLedger: {
        path: "tests/_fixtures/patina-lint-documented-divergences.json" as const,
        sha256: divergenceLedgerSha256,
      },
    },
    nativeRouteReceipt: null,
    actionsCustodyReceipt: null,
    candidateEvidenceReceipt: null,
    stabilityHistoryReceipt: null,
    fixtureProjectMembershipReceipt: null,
    projects,
    totals,
    status: "blocked" as const,
    blockers,
  };
  return validateLinterDeletionArtifact({ ...unsigned, sha256: canonicalSha256(unsigned) });
}
