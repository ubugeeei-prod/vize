import { noFilesLinterStderr } from "../../differential/linter-deletion-matrix.ts";
import type { LintSummary } from "../../differential/linter-deletion-schema.ts";
import {
  type EvidenceFile,
  type FixtureToolMatrix,
  type LintIndex,
  type LintProjectReport,
  type LinterDeletionInput,
  type RawLinterRun,
  type RegistryProject,
  lintDivergenceSha256,
} from "../../differential/linter-deletion-verifier.ts";

const sourceRevision = "a".repeat(40),
  registrySha256 = "b".repeat(64),
  binarySha256 = "c".repeat(64),
  reportSha256 = "d".repeat(64),
  rawSha256 = "e".repeat(64),
  matrixSha256 = "f".repeat(64),
  indexSha256 = "1".repeat(64),
  ruleMapSha256 = "2".repeat(64),
  ledgerSha256 = "3".repeat(64);
const totalFields = [
  "patinaFindingCount",
  "baselineFindingCount",
  "comparableBaselineCount",
  "sharedCount",
  "messageDifferenceCount",
  "documentedDivergenceCount",
  "ruleLocationDivergenceCount",
  "falsePositiveCount",
  "falseNegativeCount",
  "unimplementedCount",
  "intentionalDivergenceCount",
  "patinaOnlyRuleFindingCount",
  "baselineParseErrorCount",
  "baselineInvalidRangeCount",
] as const;

interface ProjectChanges {
  comparedFileCount?: number;
  reportedExpectedFileCount?: number | null;
  sourceRevision?: string;
  summary?: Partial<LintSummary>;
  unusableReason?: string | null;
  droppedConfigMessageCount?: number;
}
interface ControlChanges {
  projects?: Record<string, ProjectChanges>;
  indexSourceRevision?: string;
  input?: Partial<LinterDeletionInput>;
}

function summary(shared: number, changes: Partial<LintSummary> = {}): LintSummary {
  const values = {
    referenceCoverageIntegrityErrorCount: 0,
    sharedCount: shared,
    messageDifferenceCount: 0,
    documentedDivergenceCount: 0,
    ruleLocationDivergenceCount: 0,
    falsePositiveCount: 0,
    falseNegativeCount: 0,
    unimplementedCount: 0,
    intentionalDivergenceCount: 0,
    patinaOnlyRuleFindingCount: 0,
    patinaOutsideBaselineSurfaceCount: 0,
    baselineParseErrorCount: 0,
    baselineExcludedNonVueCount: 0,
    baselineInvalidRangeCount: 0,
    ...changes,
  };
  const comparablePatina =
      values.sharedCount + values.falsePositiveCount + values.ruleLocationDivergenceCount,
    comparableBaseline =
      values.sharedCount + values.falseNegativeCount + values.ruleLocationDivergenceCount;
  return {
    ...values,
    patinaFindingCount:
      comparablePatina +
      values.patinaOnlyRuleFindingCount +
      values.patinaOutsideBaselineSurfaceCount,
    baselineFindingCount:
      comparableBaseline + values.unimplementedCount + values.intentionalDivergenceCount,
    comparableBaselineCount: comparableBaseline,
    falsePositiveRatio: comparablePatina === 0 ? 0 : values.falsePositiveCount / comparablePatina,
    falseNegativeRatio:
      comparableBaseline === 0 ? 0 : values.falseNegativeCount / comparableBaseline,
  };
}

function records(count: number, value: Record<string, unknown> = {}): Record<string, unknown>[] {
  return Array.from({ length: count }, (_, index) => ({ index, ...value }));
}

function divergence(project: string, measured: LintSummary): Record<string, unknown> {
  const value = {
    schema: "vize.fixtureLintDivergence",
    version: 1,
    project,
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    summary: measured,
    referenceCoverageExpectations: records(measured.referenceCoverageIntegrityErrorCount, {
      passed: false,
    }),
    shared: records(measured.sharedCount),
    messageDifferences: records(measured.messageDifferenceCount),
    documentedDivergences: records(measured.documentedDivergenceCount),
    ruleLocationDivergences: records(measured.ruleLocationDivergenceCount),
    falsePositives: records(measured.falsePositiveCount),
    falseNegatives: records(measured.falseNegativeCount),
    unimplemented: records(measured.unimplementedCount),
    intentionalDivergences: records(measured.intentionalDivergenceCount),
    patinaOnlyRuleFindings: records(measured.patinaOnlyRuleFindingCount),
    patinaOutsideBaselineSurface: records(measured.patinaOutsideBaselineSurfaceCount),
  };
  return { ...value, sha256: lintDivergenceSha256(value) };
}

function message(index: number): Record<string, unknown> {
  return {
    ruleId: "vue/no-unused-vars",
    ruleDocsPath: "rules/no-unused-vars.md",
    severity: 2,
    message: `finding ${index}`,
    help: "remove it",
    line: index + 1,
    column: 1,
    endLine: index + 1,
    endColumn: 2,
  };
}

function patinaResults(fileCount: number, findingCount: number): Record<string, unknown>[] {
  return Array.from({ length: fileCount }, (_, index) => {
    const messages =
      index === 0 ? Array.from({ length: findingCount }, (__, item) => message(item)) : [];
    return {
      file: `src/File${index}.vue`,
      messages,
      errorCount: messages.length,
      warningCount: 0,
    };
  });
}

function flatten(results: Record<string, unknown>[]): Record<string, unknown>[] {
  return results.flatMap((result) =>
    (result.messages as Record<string, unknown>[]).map((entry) => ({
      ...entry,
      file: result.file,
    })),
  );
}

function expectedReason(
  measured: LintSummary,
  compared: number,
  expected: number | null,
): string | null {
  if (measured.referenceCoverageIntegrityErrorCount)
    return `expected exactly one native finding for ${measured.referenceCoverageIntegrityErrorCount} reviewed reference-coverage case(s)`;
  if (expected === 0 && compared !== 0)
    return "the project expected zero Vue files but selected a nonempty corpus";
  if (compared === 0 && expected !== 0) return "the project selected no Vue files";
  if (measured.baselineParseErrorCount)
    return `eslint-plugin-vue could not parse ${measured.baselineParseErrorCount} compared file(s)`;
  if (measured.baselineInvalidRangeCount)
    return `eslint-plugin-vue reported ${measured.baselineInvalidRangeCount} finding(s) with invalid source ranges`;
  return null;
}

function projectArtifact(
  project: RegistryProject,
  changes: ProjectChanges = {},
): LintProjectReport {
  const expected = project.expectedVueFileCount ?? null,
    compared = changes.comparedFileCount ?? expected ?? 2,
    measured = summary(compared === 0 ? 0 : 2, changes.summary),
    results = patinaResults(compared, measured.patinaFindingCount),
    unusableReason =
      changes.unusableReason === undefined
        ? expectedReason(measured, compared, changes.reportedExpectedFileCount ?? expected)
        : changes.unusableReason,
    falsePositivePassed = measured.falsePositiveCount === 0,
    falseNegativePassed = measured.falseNegativeCount === 0,
    verdict =
      unusableReason !== null
        ? "unusable"
        : falsePositivePassed && falseNegativePassed
          ? "passed"
          : "breached";
  return {
    schema: "vize.fixtureLintDivergenceRun",
    version: 2,
    project: project.id,
    revision: project.revision ?? null,
    preset: "ecosystem",
    evidence: {
      commitSha: changes.sourceRevision ?? sourceRevision,
      runtime: { name: "node", version: "v24.0.0" },
      machine: { platform: "linux", arch: "x64", logicalCpuCount: 8 },
    },
    files: {
      comparedCount: compared,
      expectedCount:
        changes.reportedExpectedFileCount === undefined
          ? expected
          : changes.reportedExpectedFileCount,
    },
    baseline: {
      package: "eslint-plugin-vue",
      version: "10.9.2",
      runtimeVersions: {
        eslint: "9.37.0",
        "eslint-plugin-vue": "10.9.2",
        "vue-eslint-parser": "10.4.1",
        "@typescript-eslint/parser": "8.46.1",
      },
      comparedRuleCount: 2,
      mappedRuleCount: 2,
      skippedByPresetCount: 1,
      droppedConfigMessageCount: changes.droppedConfigMessageCount ?? 0,
      durationMs: 1,
      parseErrorResults: records(measured.baselineParseErrorCount, {
        messages: [{ ruleId: null }],
      }),
    },
    raw: {
      patinaResults: results,
      patinaFindings: flatten(results),
      eslintResults: records(compared),
    },
    divergence: divergence(project.id, measured) as LintProjectReport["divergence"],
    budget: {
      maxFalsePositiveCount: 0,
      maxFalseNegativeCount: 0,
      falsePositivePassed,
      falseNegativePassed,
      unusableReason,
      verdict,
      passed: verdict === "passed",
    },
  };
}

function evidence<T>(value: T, digest: string): EvidenceFile<T> {
  return { value, sha256: digest };
}

function rawRun(report: LintProjectReport, project: RegistryProject): RawLinterRun {
  const parsed = structuredClone(report.raw.patinaResults) as Record<string, unknown>[],
    hasErrors = parsed.some((result) => Number(result.errorCount) > 0);
  return {
    schema: "vize.fixtureToolRun",
    version: 1,
    project: report.project,
    tool: "linter",
    exitCode: hasErrors ? 1 : 0,
    stdout: JSON.stringify(parsed),
    stderr: report.files.comparedCount === 0 ? noFilesLinterStderr(project.vueGlobs) : "",
    parsed,
  };
}

function index(projects: LintProjectReport[], revision = sourceRevision): LintIndex {
  const failed = projects.filter((project) => !project.budget.passed);
  return {
    schema: "vize.fixtureLintDivergenceIndex",
    version: 1,
    evidence: {
      commitSha: revision,
      runtime: { name: "node", version: "v24.0.0" },
      machine: { platform: "linux", arch: "x64", logicalCpuCount: 8 },
    },
    preset: "ecosystem",
    projectCount: projects.length,
    budget: {
      status: failed.length ? "failure" : "success",
      passed: failed.length === 0,
      projectCount: projects.length,
      passedCount: projects.length - failed.length,
      failedCount: failed.length,
      unusableCount: failed.filter((project) => project.budget.verdict === "unusable").length,
      breachedCount: failed.filter((project) => project.budget.verdict === "breached").length,
      failedProjects: failed.map((project) => project.project),
    },
    totals: Object.fromEntries(
      totalFields.map((field) => [
        field,
        projects.reduce((sum, project) => sum + project.divergence.summary[field], 0),
      ]),
    ) as LintIndex["totals"],
    projects: projects.map((project) => ({
      project: project.project,
      ...project.divergence.summary,
    })),
  };
}

function matrix(
  registryProjects: RegistryProject[],
  reports: Map<string, LintProjectReport>,
): FixtureToolMatrix {
  const tools = ["compiler", "typechecker", "linter", "formatter"],
    projects = registryProjects.map((project) => {
      const report = reports.get(project.id) as LintProjectReport,
        raw = rawRun(report, project),
        runs = tools.map((tool) => ({
          tool,
          status: tool === "linter" && raw.exitCode === 1 ? "findings" : "ok",
          command: `target/ci/vize ${tool}`,
          cwd: project.fixturePath,
          outputPath: `real-project-results/shard-0/${project.id}-${tool}.json`,
          durationMs: 1,
          exitCode: tool === "linter" ? raw.exitCode : 0,
          fileCount: report.files.comparedCount,
          coverage: null,
        }));
      return {
        id: project.id,
        revision: project.revision ?? null,
        fixturePath: project.fixturePath,
        runs,
      };
    }),
    runs = projects.flatMap((project) => project.runs),
    count = (status: string): number => runs.filter((run) => run.status === status).length;
  return {
    schema: "vize.fixtureToolMatrixReport",
    version: 3,
    generatedAt: "2026-10-28T00:00:00Z",
    registryPath: "tests/_fixtures/vue-ecosystem-fixtures.json",
    evidence: {
      commitSha: sourceRevision,
      runtime: { name: "rust-script", version: "rustc 1.90.0" },
      machine: {
        platform: "linux",
        arch: "x64",
        cpuModel: "test cpu",
        logicalCpuCount: 8,
        totalMemoryBytes: 1024,
      },
    },
    command: {
      vize: "target/ci/vize",
      dryRun: false,
      timeoutMs: 600000,
      tools,
      shardIndex: 0,
      shardCount: 1,
    },
    projects,
    summary: {
      projectCount: projects.length,
      toolCount: tools.length,
      runCount: runs.length,
      okRuns: count("ok"),
      findingsRuns: count("findings"),
      failedRuns: 0,
      missingFixtureRuns: 0,
      plannedRuns: 0,
    },
  };
}

export function control(changes: ControlChanges = {}): LinterDeletionInput {
  const registryProjects: RegistryProject[] = [
      {
        id: "with-vue",
        fixturePath: "tests/_fixtures/_git/with-vue",
        revision: "4".repeat(40),
        coverage: ["linter", "formatter"],
        vueGlobs: ["src/**/*.vue"],
      },
      {
        id: "without-vue",
        fixturePath: "tests/_fixtures/_git/without-vue",
        revision: "5".repeat(40),
        coverage: ["linter"],
        vueGlobs: ["**/*.vue"],
        expectedVueFileCount: 0,
      },
    ],
    reports = new Map(
      registryProjects.map((project) => [
        project.id,
        projectArtifact(project, changes.projects?.[project.id]),
      ]),
    ),
    matrixArtifact = matrix(registryProjects, reports);
  return {
    sourceRevision,
    registrySha256,
    shardIndex: 0,
    shardCount: 1,
    registryProjects,
    matrix: evidence(matrixArtifact, matrixSha256),
    rawLinterRuns: new Map(
      registryProjects.map((project) => [
        project.id,
        evidence(rawRun(reports.get(project.id) as LintProjectReport, project), rawSha256),
      ]),
    ),
    index: evidence(index([...reports.values()], changes.indexSourceRevision), indexSha256),
    projectArtifacts: new Map(
      [...reports].map(([project, report]) => [project, evidence(report, reportSha256)]),
    ),
    ruleMap: evidence(
      {
        entries: {
          "vue/intentional": { status: "intentional-divergence" },
          "vue/mapped": {
            status: "mapped",
            patinaRule: "vue/mapped",
            patinaPresets: ["ecosystem"],
          },
          "vue/skipped": {
            status: "mapped",
            patinaRule: "vue/skipped",
            patinaPresets: ["essential"],
          },
          "vue/unimplemented": { status: "unimplemented" },
        },
        schemaVersion: 1,
        summary: { intentionalDivergence: 1, mapped: 2, unimplemented: 1 },
        upstream: { package: "eslint-plugin-vue", ruleCount: 4, version: "10.9.2" },
      },
      ruleMapSha256,
    ),
    divergenceLedgerSha256: ledgerSha256,
    buildReceipt: {
      schema: "vize.differential.build",
      version: 1,
      recipe: "cargo build --profile ci -p vize",
      sourceRevision,
      binaryPath: "target/ci/vize",
      binarySha256,
      cliVersion: "vize 0.500.0-dev",
    },
    ...changes.input,
  };
}
