#!/usr/bin/env node
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { collectRunEvidence } from "./run-evidence.mjs";
import { resolveVizeLaunch, runToolWithHeartbeat } from "./tool-matrix-run.mjs";
import { validateTypecheckPerformanceTarget } from "./tool-matrix-typecheck-target.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const registryPath = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const schema = "vize.fixtureToolMatrixReport";
const supportedTools = ["compiler", "typechecker", "linter", "formatter"];

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const registry = readJson(registryPath);
  const selectedProjects = select(registry.projects, args.projects, "project");
  const projects = selectShard(selectedProjects, args.shardIndex, args.shardCount);
  const tools =
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

  const report = {
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
      failedRuns: 0,
      missingFixtureRuns: 0,
    },
    projects: [],
  };

  for (const project of projects) {
    const projectReport = {
      id: project.id,
      fixturePath: project.fixturePath,
      revision: project.revision,
      runs: [],
    };
    for (const tool of tools) {
      const run = await runToolWithHeartbeat(project, tool, args, launch, outputDir);
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

function parseArgs(argv) {
  const args = {
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

function printHelpAndExit() {
  process.stdout.write(`Usage: node tools/fixtures/tool-matrix-report.mjs [options]\n\n`);
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

function assertRegistryCoverage(registry, projects, tools) {
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

function renderMarkdown(report) {
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

function summaryKey(status) {
  return {
    planned: "plannedRuns",
    ok: "okRuns",
    findings: "findingsRuns",
    failed: "failedRuns",
    "missing-fixture": "missingFixtureRuns",
  }[status];
}

function select(items, selected, kind) {
  if (selected.length === 0) return items;
  const byId = new Map(items.map((item) => [typeof item === "string" ? item : item.id, item]));
  return selected.map((id) => {
    if (!byId.has(id)) throw new Error(`Unknown fixture ${kind}: ${id}`);
    return byId.get(id);
  });
}

function selectShard(projects, shardIndex, shardCount) {
  return projects.filter((_, index) => index % shardCount === shardIndex);
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}
function splitCsv(value) {
  return value
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);
}
function positiveInteger(value, name) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0)
    throw new Error(`${name} must be a positive integer`);
  return parsed;
}
function nonNegativeInteger(value, name) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0)
    throw new Error(`${name} must be a non-negative integer`);
  return parsed;
}
function timestampSlug(date) {
  return date
    .toISOString()
    .replace(/\.\d{3}Z$/, "Z")
    .replace(/[:.]/g, "-");
}
function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

try {
  await main();
} catch (error) {
  process.stderr.write(`${errorMessage(error)}\n`);
  process.exit(1);
}
