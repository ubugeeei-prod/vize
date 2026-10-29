import assert from "node:assert/strict";
import {
  type JsonRecord,
  type LintProjectReport,
  type RegistryProject,
  type RunEvidence,
  exactIds,
  exactKeys,
  integer,
} from "./linter-deletion-schema.ts";

export interface MatrixRun extends JsonRecord {
  tool: string;
  status: string;
  command: string;
  cwd: string;
  outputPath: string;
  durationMs: number;
  exitCode: number;
  fileCount: number;
  coverage: null;
}

export interface MatrixProject extends JsonRecord {
  id: string;
  revision: string | null;
  fixturePath: string;
  runs: MatrixRun[];
}

export interface FixtureToolMatrix extends JsonRecord {
  schema: string;
  version: number;
  generatedAt: string;
  registryPath: string;
  evidence: RunEvidence;
  command: {
    vize: string;
    dryRun: boolean;
    timeoutMs: number;
    tools: string[];
    shardIndex: number;
    shardCount: number;
  } & JsonRecord;
  projects: MatrixProject[];
  summary: JsonRecord;
}

export interface RawLinterRun extends JsonRecord {
  schema: string;
  version: number;
  project: string;
  tool: string;
  exitCode: number;
  stdout: string;
  stderr: string;
  parsed: JsonRecord[];
}

export interface EvidenceFile<T> {
  value: T;
  sha256: string;
}

const matrixTools = ["compiler", "typechecker", "linter", "formatter"];
const matrixStatuses = ["planned", "ok", "findings", "failed", "missing-fixture"];
const lintExtensions = ".vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx",
  noFilesPrefix = `No ${lintExtensions} files found matching patterns: `,
  unmatchedPatternsPrefix = `Warning: no ${lintExtensions} files found matching patterns: `,
  recursiveVueGlob = /^(?:\*\*|[A-Za-z0-9_.@+-]+(?:\/[A-Za-z0-9_.@+-]+)*\/\*\*)\/\*\.vue$/;

function patternDiagnostic(prefix: string, patterns: string[]): string {
  return `${prefix}[${patterns.map((pattern) => JSON.stringify(pattern)).join(", ")}]\n`;
}

export function noFilesLinterStderr(patterns: string[]): string {
  return patternDiagnostic(noFilesPrefix, patterns);
}

export function unmatchedPatternsLinterStderr(patterns: string[]): string {
  return patternDiagnostic(unmatchedPatternsPrefix, patterns);
}

export function assertSupportedLinterVueGlob(
  pattern: unknown,
  label: string,
): asserts pattern is string {
  const literalSegments = typeof pattern === "string" ? pattern.split("/").slice(0, -2) : [];
  assert(
    typeof pattern === "string" &&
      recursiveVueGlob.test(pattern) &&
      literalSegments.every((segment) => segment !== "." && segment !== ".."),
    `${label} must use the pinned recursive Vue glob grammar`,
  );
}

function vueGlobMatchesFile(pattern: string, file: string): boolean {
  const suffix = "**/*.vue",
    prefix = pattern.slice(0, -suffix.length);
  return file.startsWith(prefix) && file.length > prefix.length;
}

function record(value: unknown, label: string): JsonRecord {
  assert(value && typeof value === "object" && !Array.isArray(value), `${label} is required`);
  return value as JsonRecord;
}

function nonempty(value: unknown, label: string): string {
  assert(typeof value === "string", `${label} must be a string`);
  assert(value.length > 0, `${label} must be nonempty`);
  return value;
}

function validateMatrixEvidence(matrix: FixtureToolMatrix, sourceRevision: string): void {
  exactKeys(matrix.evidence, ["commitSha", "runtime", "machine"], "matrix.evidence");
  assert.equal(matrix.evidence.commitSha, sourceRevision);
  const runtime = record(matrix.evidence.runtime, "matrix.evidence.runtime");
  exactKeys(runtime, ["name", "version"], "matrix.evidence.runtime");
  assert.equal(runtime.name, "rust-script");
  nonempty(runtime.version, "matrix.evidence.runtime.version");
  const machine = record(matrix.evidence.machine, "matrix.evidence.machine");
  exactKeys(
    machine,
    ["platform", "arch", "cpuModel", "logicalCpuCount", "totalMemoryBytes"],
    "matrix.evidence.machine",
  );
  nonempty(machine.platform, "matrix.evidence.machine.platform");
  nonempty(machine.arch, "matrix.evidence.machine.arch");
  nonempty(machine.cpuModel, "matrix.evidence.machine.cpuModel");
  assert(integer(machine.logicalCpuCount, "matrix.evidence.machine.logicalCpuCount") > 0);
  assert(integer(machine.totalMemoryBytes, "matrix.evidence.machine.totalMemoryBytes") > 0);
}

function expectedMatrixSummary(matrix: FixtureToolMatrix): JsonRecord {
  const runs = matrix.projects.flatMap((project) => project.runs);
  const count = (status: string): number => runs.filter((run) => run.status === status).length;
  return {
    projectCount: matrix.projects.length,
    toolCount: matrix.command.tools.length,
    runCount: runs.length,
    okRuns: count("ok"),
    findingsRuns: count("findings"),
    failedRuns: count("failed"),
    missingFixtureRuns: count("missing-fixture"),
    plannedRuns: count("planned"),
  };
}

function linterRun(project: MatrixProject): MatrixRun {
  assert(Array.isArray(project.runs), `${project.id}: matrix runs are required`);
  const matches = project.runs.filter((run) => run.tool === "linter");
  assert.equal(matches.length, 1, `${project.id}: exactly one matrix linter run is required`);
  return matches[0];
}

export function validateFixtureToolMatrix(
  matrix: FixtureToolMatrix,
  selected: RegistryProject[],
  sourceRevision: string,
  shardIndex: number,
  shardCount: number,
): Map<string, MatrixRun> {
  record(matrix, "fixture tool matrix");
  exactKeys(
    matrix,
    [
      "schema",
      "version",
      "generatedAt",
      "registryPath",
      "evidence",
      "command",
      "projects",
      "summary",
    ],
    "fixture tool matrix",
  );
  assert.equal(matrix.schema, "vize.fixtureToolMatrixReport");
  assert.equal(matrix.version, 3);
  assert.equal(matrix.registryPath, "tests/_fixtures/vue-ecosystem-fixtures.json");
  assert(Number.isFinite(Date.parse(matrix.generatedAt)), "matrix.generatedAt must be RFC3339");
  validateMatrixEvidence(matrix, sourceRevision);
  exactKeys(
    matrix.command,
    ["vize", "dryRun", "timeoutMs", "tools", "shardIndex", "shardCount"],
    "matrix.command",
  );
  assert.match(matrix.command.vize, /(?:^|\/)target\/ci\/vize(?:\.exe)?$/);
  assert.equal(matrix.command.dryRun, false);
  assert(integer(matrix.command.timeoutMs, "matrix.command.timeoutMs") > 0);
  assert.deepEqual(matrix.command.tools, matrixTools);
  assert.equal(matrix.command.shardIndex, shardIndex);
  assert.equal(matrix.command.shardCount, shardCount);
  assert(Array.isArray(matrix.projects));
  exactIds(
    matrix.projects.map((project) => project.id),
    selected.map((project) => project.id),
    "matrix projects",
  );
  const runs = new Map<string, MatrixRun>();
  for (const registryProject of selected) {
    const project = matrix.projects.find((entry) => entry.id === registryProject.id);
    assert(project, `${registryProject.id}: matrix project is missing`);
    exactKeys(project, ["id", "revision", "fixturePath", "runs"], `${project.id}.matrix`);
    assert.equal(project.revision ?? null, registryProject.revision ?? null);
    assert.equal(project.fixturePath, registryProject.fixturePath);
    assert.equal(project.runs.length, matrixTools.length);
    exactIds(
      project.runs.map((run) => run.tool),
      matrixTools,
      `${project.id} matrix tools`,
    );
    for (const run of project.runs)
      assert(
        matrixStatuses.includes(run.status),
        `${project.id}/${run.tool}: invalid matrix status`,
      );
    const run = linterRun(project);
    exactKeys(
      run,
      [
        "tool",
        "status",
        "command",
        "cwd",
        "outputPath",
        "durationMs",
        "exitCode",
        "fileCount",
        "coverage",
      ],
      `${project.id}.matrix.linter`,
    );
    assert(["ok", "findings"].includes(run.status), `${project.id}: matrix linter run failed`);
    nonempty(run.command, `${project.id}.matrix.linter.command`);
    assert.equal(run.cwd, registryProject.fixturePath);
    nonempty(run.outputPath, `${project.id}.matrix.linter.outputPath`);
    integer(run.durationMs, `${project.id}.matrix.linter.durationMs`);
    assert([0, 1].includes(run.exitCode));
    integer(run.fileCount, `${project.id}.matrix.linter.fileCount`);
    assert.equal(run.coverage, null);
    runs.set(project.id, run);
  }
  exactKeys(
    matrix.summary,
    [
      "projectCount",
      "toolCount",
      "runCount",
      "okRuns",
      "findingsRuns",
      "failedRuns",
      "missingFixtureRuns",
      "plannedRuns",
    ],
    "matrix.summary",
  );
  assert.deepEqual(matrix.summary, expectedMatrixSummary(matrix));
  return runs;
}

function validateMessage(value: unknown, label: string): 1 | 2 {
  const message = record(value, label);
  const keys = [
    "ruleId",
    "ruleDocsPath",
    "severity",
    "message",
    "line",
    "column",
    "endLine",
    "endColumn",
  ];
  if ("help" in message) keys.push("help");
  exactKeys(message, keys, label);
  for (const field of ["ruleId", "ruleDocsPath", "message"])
    nonempty(message[field], `${label}.${field}`);
  if ("help" in message) nonempty(message.help, `${label}.help`);
  assert(message.severity === 1 || message.severity === 2, `${label}.severity must be 1 or 2`);
  const line = integer(message.line, `${label}.line`),
    column = integer(message.column, `${label}.column`),
    endLine = integer(message.endLine, `${label}.endLine`),
    endColumn = integer(message.endColumn, `${label}.endColumn`);
  assert(line > 0 && column > 0 && endLine > 0 && endColumn > 0, `${label} range must be positive`);
  assert(endLine > line || (endLine === line && endColumn >= column), `${label} range is inverted`);
  return message.severity;
}

function validateParsedResults(parsed: JsonRecord[], projectId: string): JsonRecord[] {
  const flattened: JsonRecord[] = [];
  const files = new Set<string>();
  for (const [fileIndex, value] of parsed.entries()) {
    const label = `${projectId}.parsed[${fileIndex}]`,
      result = record(value, label);
    exactKeys(result, ["file", "messages", "errorCount", "warningCount"], label);
    const file = nonempty(result.file, `${label}.file`);
    assert(!file.startsWith("/") && !file.includes("\\") && !file.split("/").includes(".."));
    assert(file.endsWith(".vue"), `${label}.file must be a Vue SFC`);
    assert(!files.has(file), `${projectId}: duplicate parsed file ${file}`);
    files.add(file);
    assert(Array.isArray(result.messages), `${label}.messages must be an array`);
    let errors = 0,
      warnings = 0;
    for (const [messageIndex, message] of result.messages.entries()) {
      const severity = validateMessage(message, `${label}.messages[${messageIndex}]`);
      if (severity === 2) errors++;
      else warnings++;
      flattened.push({ ...(message as JsonRecord), file });
    }
    assert.equal(result.errorCount, errors, `${label}.errorCount drifted`);
    assert.equal(result.warningCount, warnings, `${label}.warningCount drifted`);
  }
  return flattened;
}

export function validateRawLinterRun(
  raw: RawLinterRun,
  matrixRun: MatrixRun,
  report: LintProjectReport,
  vueGlobs: string[],
): string[] {
  const projectId = report.project;
  record(raw, `${projectId} raw matrix linter run`);
  exactKeys(
    raw,
    ["schema", "version", "project", "tool", "exitCode", "stdout", "stderr", "parsed"],
    `${projectId}.rawRun`,
  );
  assert.equal(raw.schema, "vize.fixtureToolRun");
  assert.equal(raw.version, 1);
  assert.equal(raw.project, projectId);
  assert.equal(raw.tool, "linter");
  assert([0, 1].includes(raw.exitCode));
  assert.equal(raw.exitCode, matrixRun.exitCode);
  assert(Array.isArray(raw.parsed));
  assert.deepEqual(
    JSON.parse(raw.stdout),
    raw.parsed,
    `${projectId}: stdout and parsed output drifted`,
  );
  assert.deepEqual(
    raw.parsed,
    report.raw.patinaResults,
    `${projectId}: Matrix and divergence runs drifted`,
  );
  assert.equal(raw.parsed.length, matrixRun.fileCount);
  assert.equal(raw.parsed.length, report.files.comparedCount);
  const flattened = validateParsedResults(raw.parsed, projectId);
  assert.deepEqual(
    flattened,
    report.raw.patinaFindings,
    `${projectId}: flattened findings drifted`,
  );
  assert.equal(flattened.length, report.divergence.summary.patinaFindingCount);
  const parsedFiles = raw.parsed.map((result) => result.file as string);
  for (const file of parsedFiles)
    assert(
      vueGlobs.some((pattern) => vueGlobMatchesFile(pattern, file)),
      `${projectId}: parsed file falls outside registry vueGlobs: ${file}`,
    );
  const unmatchedPatterns = vueGlobs.filter(
      (pattern) => !parsedFiles.some((file) => vueGlobMatchesFile(pattern, file)),
    ),
    expectedStderr =
      parsedFiles.length === 0
        ? noFilesLinterStderr(vueGlobs)
        : unmatchedPatterns.length > 0
          ? unmatchedPatternsLinterStderr(unmatchedPatterns)
          : "";
  assert.equal(raw.stderr, expectedStderr, `${projectId}: raw linter stderr drifted`);
  const errors = raw.parsed.reduce(
    (sum, result) => sum + integer(result.errorCount, `${projectId}.parsed.errorCount`),
    0,
  );
  const expectedExitCode = errors > 0 ? 1 : 0;
  assert.equal(raw.exitCode, expectedExitCode, `${projectId}: linter exit code drifted`);
  assert.equal(matrixRun.status, expectedExitCode === 0 ? "ok" : "findings");
  return unmatchedPatterns;
}
