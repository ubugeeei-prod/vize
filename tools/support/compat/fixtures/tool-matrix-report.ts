import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { collectRunEvidence, type RunEvidence } from "./run-evidence.ts";
import { resolveVizeLaunch, runToolWithHeartbeat } from "./tool-matrix-run.mjs";
import { validateTypecheckPerformanceTarget } from "./tool-matrix-typecheck-target.ts";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..", "..", "..");
const registryPath = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const schema = "vize.fixtureToolMatrixReport";
const supportedTools = ["compiler", "typechecker", "linter", "formatter"] as const;

type Tool = (typeof supportedTools)[number];
type RunStatus = "planned" | "ok" | "findings" | "configuration" | "failed" | "missing-fixture";
type SummaryCounter =
  | "plannedRuns"
  | "okRuns"
  | "findingsRuns"
  | "configurationRuns"
  | "failedRuns"
  | "missingFixtureRuns";

interface FixtureProject {
  readonly id: string;
  readonly fixturePath: string;
  readonly revision: string;
  readonly coverage: readonly Tool[];
  readonly [field: string]: unknown;
}

interface FixtureRegistry {
  readonly projects: readonly FixtureProject[];
  readonly requiredToolCoverage: readonly Tool[];
}

interface Arguments {
  dryRun: boolean;
  listFixturePaths: boolean;
  heartbeatMs: number;
  outputDir: string | null;
  projects: string[];
  shardCount: number;
  shardIndex: number;
  timeoutMs: number;
  tools: string[];
  vizeBin: string | null;
}

interface MatrixCoverage {
  readonly requestedFileCount?: number;
  readonly transitiveAuthoredFileCount?: number;
  readonly transitiveDependencyFileCount?: number;
}

interface MatrixRun {
  readonly tool: Tool;
  readonly status: RunStatus;
  readonly exitCode: number | null;
  readonly fileCount: number | null;
  readonly coverage: MatrixCoverage | null;
  readonly durationMs: number;
  readonly outputPath: string | null;
}

interface MatrixReport {
  readonly schema: string;
  readonly version: number;
  readonly generatedAt: string;
  readonly evidence: RunEvidence;
  readonly registryPath: string;
  readonly command: {
    readonly vize: string;
    readonly dryRun: boolean;
    readonly timeoutMs: number;
    readonly tools: readonly Tool[];
    readonly shardIndex: number;
    readonly shardCount: number;
  };
  readonly summary: {
    readonly projectCount: number;
    readonly toolCount: number;
    readonly runCount: number;
    plannedRuns: number;
    okRuns: number;
    findingsRuns: number;
    configurationRuns: number;
    failedRuns: number;
    missingFixtureRuns: number;
  };
  readonly projects: Array<{
    readonly id: string;
    readonly fixturePath: string;
    readonly revision: string;
    readonly runs: MatrixRun[];
  }>;
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  const registry = readJson<FixtureRegistry>(registryPath);
  const selectedProjects = select(registry.projects, args.projects, "project");
  const projects = selectShard(selectedProjects, args.shardIndex, args.shardCount);
  const tools: readonly Tool[] =
    args.tools.length === 0 ? supportedTools : select(supportedTools, args.tools, "tool");
  assertRegistryCoverage(registry, projects, tools);

  if (args.listFixturePaths) {
    process.stdout.write(`${projects.map((project) => project.fixturePath).join("\n")}\n`);
    return;
  }
  if (!args.dryRun && tools.includes("typechecker")) {
    for (const project of projects) {
      const fixtureRoot = resolve(repoRoot, project.fixturePath);
      if (existsSync(fixtureRoot)) validateTypecheckPerformanceTarget(project, fixtureRoot);
    }
  }

  const outputDir = resolve(
    repoRoot,
    args.outputDir ?? join(".vize", "fixture-tool-matrix", timestampSlug(new Date())),
  );
  const launch = resolveVizeLaunch(args.vizeBin, args.dryRun);
  mkdirSync(outputDir, { recursive: true });

  const report: MatrixReport = {
    schema,
    version: 3,
    generatedAt: new Date().toISOString(),
    evidence: collectRunEvidence(),
    registryPath: relative(repoRoot, registryPath),
    command: {
      vize: launch.label,
      dryRun: args.dryRun,
      timeoutMs: args.timeoutMs,
      tools,
      shardIndex: args.shardIndex,
      shardCount: args.shardCount,
    },
    summary: {
      projectCount: projects.length,
      toolCount: tools.length,
      runCount: projects.length * tools.length,
      plannedRuns: 0,
      okRuns: 0,
      findingsRuns: 0,
      configurationRuns: 0,
      failedRuns: 0,
      missingFixtureRuns: 0,
    },
    projects: [],
  };

  for (const project of projects) {
    const projectReport: MatrixReport["projects"][number] = {
      id: project.id,
      fixturePath: project.fixturePath,
      revision: project.revision,
      runs: [],
    };
    for (const tool of tools) {
      const run = (await runToolWithHeartbeat(project, tool, args, launch, outputDir)) as MatrixRun;
      projectReport.runs.push(run);
      report.summary[summaryKey(run.status)] += 1;
    }
    report.projects.push(projectReport);
  }

  const jsonPath = join(outputDir, "summary.json");
  const markdownPath = join(outputDir, "summary.md");
  writeFileSync(jsonPath, `${JSON.stringify(report, null, 2)}\n`);
  writeFileSync(markdownPath, renderMarkdown(report));
  process.stdout.write(`Wrote ${relative(repoRoot, jsonPath)}\n`);
  process.stdout.write(`Wrote ${relative(repoRoot, markdownPath)}\n`);

  if (report.summary.failedRuns > 0 || report.summary.missingFixtureRuns > 0) {
    process.exitCode = 1;
  }
}

function parseArgs(argv: readonly string[]): Arguments {
  const args: Arguments = {
    dryRun: false,
    listFixturePaths: false,
    heartbeatMs: 30_000,
    outputDir: null,
    projects: [],
    shardCount: 1,
    shardIndex: 0,
    timeoutMs: 300_000,
    tools: [],
    vizeBin: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const value = () => {
      if (argv[index + 1] == null) throw new Error(`${arg} requires a value`);
      return argv[++index];
    };
    if (arg === "--dry-run") args.dryRun = true;
    else if (arg === "--list-fixture-paths") args.listFixturePaths = true;
    else if (arg === "--help" || arg === "-h") return printHelpAndExit();
    else if (arg === "--heartbeat-ms") args.heartbeatMs = positiveInteger(value(), arg);
    else if (arg.startsWith("--heartbeat-ms="))
      args.heartbeatMs = positiveInteger(arg.slice(15), "--heartbeat-ms");
    else if (arg === "--output-dir") args.outputDir = value();
    else if (arg.startsWith("--output-dir=")) args.outputDir = arg.slice(13);
    else if (arg === "--project") args.projects.push(...splitCsv(value()));
    else if (arg.startsWith("--project=")) args.projects.push(...splitCsv(arg.slice(10)));
    else if (arg === "--shard-count") args.shardCount = positiveInteger(value(), arg);
    else if (arg.startsWith("--shard-count="))
      args.shardCount = positiveInteger(arg.slice(14), "--shard-count");
    else if (arg === "--shard-index") args.shardIndex = nonNegativeInteger(value(), arg);
    else if (arg.startsWith("--shard-index="))
      args.shardIndex = nonNegativeInteger(arg.slice(14), "--shard-index");
    else if (arg === "--timeout-ms") args.timeoutMs = positiveInteger(value(), arg);
    else if (arg.startsWith("--timeout-ms="))
      args.timeoutMs = positiveInteger(arg.slice(13), "--timeout-ms");
    else if (arg === "--tool") args.tools.push(...splitCsv(value()));
    else if (arg.startsWith("--tool=")) args.tools.push(...splitCsv(arg.slice(7)));
    else if (arg === "--vize-bin") args.vizeBin = value();
    else if (arg.startsWith("--vize-bin=")) args.vizeBin = arg.slice(11);
    else throw new Error(`Unknown argument: ${arg}`);
  }
  args.projects = [...new Set(args.projects)];
  args.tools = [...new Set(args.tools)];
  if (args.shardIndex >= args.shardCount) {
    throw new Error("--shard-index must be less than --shard-count");
  }
  return args;
}

function printHelpAndExit(): never {
  process.stdout.write(
    `Usage: node tools/support/compat/fixtures/tool-matrix-report.ts [options]\n\n`,
  );
  process.stdout.write(
    `Exercise every registered real project with compiler, typechecker, linter, and formatter.\n\n`,
  );
  process.stdout.write(`  --project <id[,id]>  Limit registry projects\n`);
  process.stdout.write(`  --tool <name[,name]> Limit tool surfaces\n`);
  process.stdout.write(`  --shard-index <n>    Zero-based project shard index\n`);
  process.stdout.write(`  --shard-count <n>    Total balanced project shards\n`);
  process.stdout.write(`  --list-fixture-paths Print selected fixture paths and exit\n`);
  process.stdout.write(`  --output-dir <dir>   Report directory\n`);
  process.stdout.write(`  --vize-bin <path>    Vize executable\n`);
  process.stdout.write(`  --timeout-ms <n>     Per-run timeout\n`);
  process.stdout.write(`  --heartbeat-ms <n>   Progress heartbeat interval for child runs\n`);
  process.stdout.write(`  --dry-run            Plan without invoking Vize\n`);
  process.exit(0);
}

function assertRegistryCoverage(
  registry: FixtureRegistry,
  projects: readonly FixtureProject[],
  tools: readonly Tool[],
): void {
  for (const tool of tools) {
    if (!registry.requiredToolCoverage.includes(tool)) {
      throw new Error(`Registry does not require tool coverage: ${tool}`);
    }
  }
  for (const project of projects) {
    for (const tool of tools) {
      if (!project.coverage.includes(tool)) {
        throw new Error(`${project.id} does not declare ${tool} coverage`);
      }
    }
  }
}

function renderMarkdown(report: MatrixReport): string {
  const lines = [
    "# Vize Fixture Tool Matrix Report",
    "",
    `Projects: ${report.summary.projectCount}`,
    `Tools: ${report.command.tools.join(", ")}`,
    `Runs: ${report.summary.runCount}`,
    `Commit: ${report.evidence.commitSha}`,
    `Runtime: ${report.evidence.runtime.name} ${report.evidence.runtime.version}`,
    `Machine: ${report.evidence.machine.platform}/${report.evidence.machine.arch}, ${report.evidence.machine.logicalCpuCount} logical CPUs, ${report.evidence.machine.totalMemoryBytes} bytes memory`,
    "",
    "| Project | Tool | Status | Exit | Files | Requested | Transitive Authored | Transitive Dependencies | Duration (ms) | Output |",
    "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |",
  ];
  for (const project of report.projects) {
    for (const run of project.runs) {
      lines.push(
        `| ${project.id} | ${run.tool} | ${run.status} | ${run.exitCode ?? "-"} | ${run.fileCount ?? "-"} | ${run.coverage?.requestedFileCount ?? "-"} | ${run.coverage?.transitiveAuthoredFileCount ?? "-"} | ${run.coverage?.transitiveDependencyFileCount ?? "-"} | ${run.durationMs} | ${run.outputPath == null ? "-" : `\`${run.outputPath}\``} |`,
      );
    }
  }
  return `${lines.join("\n")}\n`;
}

function summaryKey(status: RunStatus): SummaryCounter {
  const counters: Record<RunStatus, SummaryCounter> = {
    planned: "plannedRuns",
    ok: "okRuns",
    findings: "findingsRuns",
    configuration: "configurationRuns",
    failed: "failedRuns",
    "missing-fixture": "missingFixtureRuns",
  };
  return counters[status];
}

function select<T extends string | { readonly id: string }>(
  items: readonly T[],
  selected: readonly string[],
  kind: string,
): T[] {
  if (selected.length === 0) return [...items];
  const byId = new Map(items.map((item) => [typeof item === "string" ? item : item.id, item]));
  return selected.map((id) => {
    if (!byId.has(id)) throw new Error(`Unknown fixture ${kind}: ${id}`);
    return byId.get(id)!;
  });
}

function selectShard(
  projects: readonly FixtureProject[],
  shardIndex: number,
  shardCount: number,
): FixtureProject[] {
  return projects.filter((_, index) => index % shardCount === shardIndex);
}

function readJson<T>(path: string): T {
  return JSON.parse(readFileSync(path, "utf8")) as T;
}
function splitCsv(value: string): string[] {
  return value
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);
}
function positiveInteger(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0)
    throw new Error(`${name} must be a positive integer`);
  return parsed;
}
function nonNegativeInteger(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0)
    throw new Error(`${name} must be a non-negative integer`);
  return parsed;
}
function timestampSlug(date: Date): string {
  return date
    .toISOString()
    .replace(/\.\d{3}Z$/, "Z")
    .replace(/[:.]/g, "-");
}
function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

try {
  await main();
} catch (error) {
  process.stderr.write(`${errorMessage(error)}\n`);
  process.exit(1);
}
