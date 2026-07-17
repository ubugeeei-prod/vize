#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const registryPath = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const schema = "vize.fixtureToolMatrixReport";
const supportedTools = ["compiler", "typechecker", "linter", "formatter"];

function main() {
  const args = parseArgs(process.argv.slice(2));
  const registry = readJson(registryPath);
  const projects = select(registry.projects, args.projects, "project");
  const tools =
    args.tools.length === 0 ? supportedTools : select(supportedTools, args.tools, "tool");
  assertRegistryCoverage(registry, projects, tools);

  const outputDir = resolve(
    repoRoot,
    args.outputDir ?? join("__agent_only", "fixture-tool-matrix", timestampSlug(new Date())),
  );
  const launch = resolveVizeLaunch(args.vizeBin, args.dryRun);
  mkdirSync(outputDir, { recursive: true });

  const report = {
    schema,
    version: 1,
    generatedAt: new Date().toISOString(),
    registryPath: relative(repoRoot, registryPath),
    command: {
      vize: launch.label,
      dryRun: args.dryRun,
      timeoutMs: args.timeoutMs,
      tools,
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
      const run = runTool(project, tool, args, launch, outputDir);
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
    outputDir: null,
    projects: [],
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
    else if (arg === "--help" || arg === "-h") return printHelpAndExit();
    else if (arg === "--output-dir") args.outputDir = value();
    else if (arg.startsWith("--output-dir=")) args.outputDir = arg.slice(13);
    else if (arg === "--project") args.projects.push(...splitCsv(value()));
    else if (arg.startsWith("--project=")) args.projects.push(...splitCsv(arg.slice(10)));
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
  return args;
}

function printHelpAndExit() {
  process.stdout.write(`Usage: node tools/fixtures/tool-matrix-report.mjs [options]\n\n`);
  process.stdout.write(
    `Exercise every registered real project with compiler, typechecker, linter, and formatter.\n\n`,
  );
  process.stdout.write(`  --project <id[,id]>  Limit registry projects\n`);
  process.stdout.write(`  --tool <name[,name]> Limit tool surfaces\n`);
  process.stdout.write(`  --output-dir <dir>   Report directory\n`);
  process.stdout.write(`  --vize-bin <path>    Vize executable\n`);
  process.stdout.write(`  --timeout-ms <n>     Per-run timeout\n`);
  process.stdout.write(`  --dry-run            Plan without invoking Vize\n`);
  process.exit(0);
}

function runTool(project, tool, args, launch, outputDir) {
  const cwd = resolve(repoRoot, project.fixturePath);
  const commandArgs = [...launch.prefix, ...toolArgs(project, tool)];
  const base = {
    tool,
    command: displayCommand(launch.command, commandArgs),
    cwd: relative(repoRoot, cwd),
    durationMs: 0,
    exitCode: null,
    outputPath: null,
  };
  if (args.dryRun) return { ...base, status: "planned" };
  if (!existsSync(cwd)) return { ...base, status: "missing-fixture" };

  const startedAt = Date.now();
  const result = spawnSync(launch.command, commandArgs, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: args.timeoutMs,
  });
  const rawPath = join(outputDir, `${project.id}-${tool}.json`);
  const completed = {
    ...base,
    durationMs: Date.now() - startedAt,
    exitCode: result.status,
    outputPath: relative(repoRoot, rawPath),
  };
  const payload = {
    schema: "vize.fixtureToolRun",
    version: 1,
    project: project.id,
    tool,
    exitCode: result.status,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
  if (result.error != null) {
    payload.spawnError = errorMessage(result.error);
  } else if (tool !== "formatter" && (result.status === 0 || result.status === 1)) {
    try {
      payload.parsed = JSON.parse(result.stdout);
    } catch (error) {
      payload.parseError = errorMessage(error);
    }
  }
  writeFileSync(rawPath, `${JSON.stringify(payload, null, 2)}\n`);

  if (result.error != null) {
    return { ...completed, status: "failed", failure: errorMessage(result.error) };
  }
  if (result.status !== 0 && result.status !== 1) {
    return { ...completed, status: "failed", failure: failureOutput(result) };
  }
  if (payload.parseError != null) {
    return {
      ...completed,
      status: "failed",
      failure: `invalid JSON output: ${payload.parseError}`,
    };
  }
  return { ...completed, status: result.status === 0 ? "ok" : "findings" };
}

function toolArgs(project, tool) {
  if (tool === "compiler") {
    return ["inspector", ...project.vueGlobs, "--format", "json", "--template-syntax", "quirks"];
  }
  if (tool === "linter") {
    return [
      "lint",
      ...project.vueGlobs,
      "--format",
      "json",
      "--preset",
      "ecosystem",
      "--no-config",
    ];
  }
  if (tool === "typechecker") {
    const args = ["check", ...project.vueGlobs, "--format", "json", "--no-config"];
    if (project.tsconfig != null) args.push("--tsconfig", project.tsconfig);
    return args;
  }
  return ["fmt", ...project.vueGlobs, "--check", "--no-config"];
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

function resolveVizeLaunch(vizeBin, dryRun) {
  const candidates = [
    vizeBin,
    process.env.VIZE_BIN,
    join(repoRoot, "target", "ci", executableName("vize")),
    join(repoRoot, "target", "debug", executableName("vize")),
    join(repoRoot, "target", "release", executableName("vize")),
  ]
    .filter(Boolean)
    .map((candidate) => resolve(candidate));
  for (const candidate of candidates) {
    if (!existsSync(candidate)) continue;
    if (dryRun) return { command: candidate, prefix: [], label: candidate };
    const probe = spawnSync(candidate, ["--version"], {
      encoding: "utf8",
      timeout: 10_000,
    });
    if (probe.status === 0) return { command: candidate, prefix: [], label: candidate };
  }
  if (vizeBin != null && !dryRun) throw new Error(`Vize executable is not runnable: ${vizeBin}`);
  return {
    command: "cargo",
    prefix: ["run", "-q", "-p", "vize", "--"],
    label: "cargo run -q -p vize --",
  };
}

function renderMarkdown(report) {
  const lines = [
    "# Vize Fixture Tool Matrix Report",
    "",
    `Projects: ${report.summary.projectCount}`,
    `Tools: ${report.command.tools.join(", ")}`,
    `Runs: ${report.summary.runCount}`,
    "",
    "| Project | Tool | Status | Exit | Duration (ms) | Output |",
    "| --- | --- | --- | ---: | ---: | --- |",
  ];
  for (const project of report.projects) {
    for (const run of project.runs) {
      lines.push(
        `| ${project.id} | ${run.tool} | ${run.status} | ${run.exitCode ?? "-"} | ${run.durationMs} | ${run.outputPath == null ? "-" : `\`${run.outputPath}\``} |`,
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
function timestampSlug(date) {
  return date
    .toISOString()
    .replace(/\.\d{3}Z$/, "Z")
    .replace(/[:.]/g, "-");
}
function executableName(name) {
  return process.platform === "win32" ? `${name}.exe` : name;
}
function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}
function failureOutput(result) {
  return { stdout: truncate(result.stdout), stderr: truncate(result.stderr) };
}
function truncate(value) {
  return value.length <= 4000 ? value : `${value.slice(0, 4000)}\n...<truncated>`;
}
function displayCommand(command, args) {
  return [command, ...args].map(shellQuote).join(" ");
}
function shellQuote(value) {
  return /^[A-Za-z0-9_./:=@*-]+$/.test(value) ? value : `'${value.replaceAll("'", "'\\''")}'`;
}

try {
  main();
} catch (error) {
  process.stderr.write(`${errorMessage(error)}\n`);
  process.exit(1);
}
