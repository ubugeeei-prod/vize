import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const registryPath = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const schema = "vize.fixtureCompilerDiffReport";
const defaultTargets = ["dom", "ssr"];

function main() {
  const args = parseArgs(process.argv.slice(2));
  const registry = readJson(registryPath);
  const selectedProjects = selectProjects(registry.projects, args.projects);
  const targets = args.targets.length === 0 ? defaultTargets : args.targets;
  const outputDir = resolve(
    repoRoot,
    args.outputDir ?? join(".vize", "artifacts", "compiler-diff-report", timestampSlug(new Date())),
  );
  const launch = resolveVizeLaunch(args.vizeBin);

  mkdirSync(outputDir, { recursive: true });

  const report = {
    schema,
    version: 1,
    generatedAt: new Date().toISOString(),
    registryPath: relative(repoRoot, registryPath),
    command: {
      vize: launch.label,
      targets,
      templateSyntax: args.templateSyntax,
      maxFiles: args.maxFiles,
      dryRun: args.dryRun,
    },
    summary: {
      projectCount: selectedProjects.length,
      targetCount: targets.length,
      plannedTargets: 0,
      okTargets: 0,
      failedTargets: 0,
      changedFiles: 0,
      additions: 0,
      removals: 0,
      officialErrors: 0,
      vizeErrors: 0,
    },
    projects: [],
  };

  for (const project of selectedProjects) {
    const projectReport = {
      id: project.id,
      displayName: project.displayName,
      fixturePath: project.fixturePath,
      revision: project.revision,
      vueGlobs: project.vueGlobs,
      diffMode: project.diff,
      targets: [],
    };

    for (const target of targets) {
      const targetReport = runProjectTarget(project, target, args, launch, outputDir);
      projectReport.targets.push(targetReport);
      if (targetReport.status === "planned") {
        report.summary.plannedTargets += 1;
      } else if (targetReport.status === "ok") {
        report.summary.okTargets += 1;
        report.summary.changedFiles += targetReport.summary.changedFiles;
        report.summary.additions += targetReport.summary.additions;
        report.summary.removals += targetReport.summary.removals;
        report.summary.officialErrors += targetReport.summary.officialErrors;
        report.summary.vizeErrors += targetReport.summary.vizeErrors;
      } else {
        report.summary.failedTargets += 1;
      }
    }

    report.projects.push(projectReport);
  }

  const jsonPath = join(outputDir, "summary.json");
  const markdownPath = join(outputDir, "summary.md");
  writeFileSync(jsonPath, `${JSON.stringify(report, null, 2)}\n`);
  writeFileSync(markdownPath, renderMarkdown(report));

  process.stdout.write(`Wrote ${relative(repoRoot, jsonPath)}\n`);
  process.stdout.write(`Wrote ${relative(repoRoot, markdownPath)}\n`);

  if (report.summary.failedTargets > 0 && !args.allowFailures) {
    process.exitCode = 1;
  }
}

function parseArgs(argv) {
  const args = {
    allowFailures: false,
    dryRun: false,
    maxFiles: null,
    outputDir: null,
    projects: [],
    targets: [],
    templateSyntax: "quirks",
    timeoutMs: 300_000,
    vizeBin: null,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const value = () => {
      const next = argv[index + 1];
      if (next == null) {
        throw new Error(`${arg} requires a value`);
      }
      index += 1;
      return next;
    };

    if (arg === "--allow-failures") {
      args.allowFailures = true;
    } else if (arg === "--dry-run") {
      args.dryRun = true;
    } else if (arg === "--help" || arg === "-h") {
      printHelp();
      process.exit(0);
    } else if (arg === "--max-files") {
      args.maxFiles = parsePositiveInteger(value(), "--max-files");
    } else if (arg.startsWith("--max-files=")) {
      args.maxFiles = parsePositiveInteger(arg.slice("--max-files=".length), "--max-files");
    } else if (arg === "--output-dir") {
      args.outputDir = value();
    } else if (arg.startsWith("--output-dir=")) {
      args.outputDir = arg.slice("--output-dir=".length);
    } else if (arg === "--project") {
      args.projects.push(...splitCsv(value()));
    } else if (arg.startsWith("--project=")) {
      args.projects.push(...splitCsv(arg.slice("--project=".length)));
    } else if (arg === "--target") {
      args.targets.push(...splitCsv(value()).map(parseTarget));
    } else if (arg.startsWith("--target=")) {
      args.targets.push(...splitCsv(arg.slice("--target=".length)).map(parseTarget));
    } else if (arg === "--template-syntax") {
      args.templateSyntax = parseTemplateSyntax(value());
    } else if (arg.startsWith("--template-syntax=")) {
      args.templateSyntax = parseTemplateSyntax(arg.slice("--template-syntax=".length));
    } else if (arg === "--timeout-ms") {
      args.timeoutMs = parsePositiveInteger(value(), "--timeout-ms");
    } else if (arg.startsWith("--timeout-ms=")) {
      args.timeoutMs = parsePositiveInteger(arg.slice("--timeout-ms=".length), "--timeout-ms");
    } else if (arg === "--vize-bin") {
      args.vizeBin = value();
    } else if (arg.startsWith("--vize-bin=")) {
      args.vizeBin = arg.slice("--vize-bin=".length);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  args.projects = [...new Set(args.projects)];
  args.targets = [...new Set(args.targets)];
  return args;
}

function printHelp() {
  process.stdout.write(`Usage: rust-script tools/commands/fixtures/compiler-diff-report.rs [options]

Compare every Vue ecosystem fixture project against the official Vue compiler.

Options:
  --project <id[,id]>       Limit to one or more registry project ids.
  --target <dom|ssr>        Limit target; repeat or comma-separate. Defaults to dom,ssr.
  --max-files <n>           Forward a per-project file limit to vize inspector.
  --template-syntax <mode>  Forward template syntax mode. Defaults to quirks.
  --output-dir <dir>        Report directory. Defaults under .vize/artifacts/compiler-diff-report.
  --vize-bin <path>         vize binary. Defaults to VIZE_BIN, target/ci, target/debug, or cargo.
  --timeout-ms <n>          Per project/target timeout. Defaults to 300000.
  --dry-run                 Write the planned report without invoking vize.
  --allow-failures          Keep exit code 0 even if some project/target runs fail.
`);
}

function runProjectTarget(project, target, args, launch, outputDir) {
  const cwd = resolve(repoRoot, project.fixturePath);
  const commandArgs = [
    ...launch.prefix,
    "inspector",
    ...project.vueGlobs,
    "--format",
    "compare",
    "--target",
    target,
    "--template-syntax",
    args.templateSyntax,
  ];
  if (args.maxFiles != null) {
    commandArgs.push("--max-files", String(args.maxFiles));
  }

  const rawPath = join(outputDir, `${project.id}-${target}.json`);
  const startedAt = Date.now();
  if (args.dryRun) {
    return {
      target,
      status: "planned",
      durationMs: 0,
      command: displayCommand(launch.command, commandArgs),
      cwd: relative(repoRoot, cwd),
      outputPath: relative(repoRoot, rawPath),
      summary: emptySummary(),
      largestDiffs: [],
    };
  }

  const result = spawnSync(launch.command, commandArgs, {
    cwd,
    encoding: "utf8",
    env: {
      ...process.env,
      LANG: "C",
      LC_ALL: "C",
    },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: args.timeoutMs,
  });
  const durationMs = Date.now() - startedAt;

  if (result.error != null) {
    return failedTarget(
      project,
      target,
      launch,
      commandArgs,
      cwd,
      rawPath,
      durationMs,
      result.error,
    );
  }
  if (result.status !== 0) {
    return {
      target,
      status: "failed",
      durationMs,
      command: displayCommand(launch.command, commandArgs),
      cwd: relative(repoRoot, cwd),
      outputPath: relative(repoRoot, rawPath),
      summary: emptySummary(),
      largestDiffs: [],
      failure: {
        exitCode: result.status,
        stderr: truncate(result.stderr),
        stdout: truncate(result.stdout),
      },
    };
  }

  let rawReport;
  try {
    rawReport = JSON.parse(result.stdout);
  } catch (error) {
    return failedTarget(project, target, launch, commandArgs, cwd, rawPath, durationMs, error);
  }

  writeFileSync(rawPath, `${JSON.stringify(rawReport, null, 2)}\n`);
  return {
    target,
    status: "ok",
    durationMs,
    command: displayCommand(launch.command, commandArgs),
    cwd: relative(repoRoot, cwd),
    outputPath: relative(repoRoot, rawPath),
    summary: rawReport.summary,
    largestDiffs: largestDiffs(rawReport),
  };
}

function failedTarget(project, target, launch, commandArgs, cwd, rawPath, durationMs, error) {
  return {
    target,
    status: "failed",
    durationMs,
    command: displayCommand(launch.command, commandArgs),
    cwd: relative(repoRoot, cwd),
    outputPath: relative(repoRoot, rawPath),
    summary: emptySummary(),
    largestDiffs: [],
    failure: {
      message: error instanceof Error ? error.message : String(error),
      project: project.id,
    },
  };
}

function largestDiffs(rawReport) {
  return rawReport.files
    .filter((file) => file.changed)
    .map((file) => ({
      path: file.path,
      additions: file.stats.additions,
      removals: file.stats.removals,
      officialError: file.official.error,
      vizeError: file.vize.error,
    }))
    .sort((a, b) => b.additions + b.removals - (a.additions + a.removals))
    .slice(0, 20);
}

function renderMarkdown(report) {
  const lines = [
    "# Vize Fixture Compiler Diff Report",
    "",
    `Generated: ${report.generatedAt}`,
    `Registry: \`${report.registryPath}\``,
    `Targets: ${report.command.targets.map((target) => `\`${target}\``).join(", ")}`,
    "",
    "## Summary",
    "",
    "| Projects | Targets planned | Targets OK | Targets failed | Changed files | Additions | Removals | Official errors | Vize errors |",
    "| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    `| ${report.summary.projectCount} | ${report.summary.plannedTargets} | ${report.summary.okTargets} | ${report.summary.failedTargets} | ${report.summary.changedFiles} | ${report.summary.additions} | ${report.summary.removals} | ${report.summary.officialErrors} | ${report.summary.vizeErrors} |`,
    "",
    "## Project Targets",
    "",
    "| Project | Target | Status | Files | Changed | Additions | Removals | Official errors | Vize errors | Report |",
    "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |",
  ];

  for (const project of report.projects) {
    for (const target of project.targets) {
      lines.push(
        [
          project.id,
          target.target,
          target.status,
          target.summary.fileCount,
          target.summary.changedFiles,
          target.summary.additions,
          target.summary.removals,
          target.summary.officialErrors,
          target.summary.vizeErrors,
          `\`${target.outputPath}\``,
        ]
          .join(" | ")
          .replace(/^/, "| ")
          .replace(/$/, " |"),
      );
    }
  }

  lines.push("", "## Largest Diffs", "");
  for (const project of report.projects) {
    for (const target of project.targets) {
      if (target.largestDiffs.length === 0) {
        continue;
      }
      lines.push(`### ${project.id} ${target.target}`, "");
      for (const file of target.largestDiffs.slice(0, 10)) {
        const total = file.additions + file.removals;
        lines.push(`- \`${file.path}\` (+${file.additions}/-${file.removals}, total ${total})`);
      }
      lines.push("");
    }
  }

  if (report.summary.failedTargets > 0) {
    lines.push("## Failures", "");
    for (const project of report.projects) {
      for (const target of project.targets) {
        if (target.status !== "failed") {
          continue;
        }
        lines.push(
          `- ${project.id}:${target.target} failed: ${target.failure?.message ?? target.failure?.stderr ?? "unknown error"}`,
        );
      }
    }
    lines.push("");
  }

  return `${lines.join("\n")}\n`;
}

function resolveVizeLaunch(vizeBin) {
  const envBin = process.env.VIZE_BIN || null;
  const candidates = [
    vizeBin,
    envBin,
    join(repoRoot, "target", "ci", executableName("vize")),
    join(repoRoot, "target", "debug", executableName("vize")),
    join(repoRoot, "target", "release", executableName("vize")),
  ]
    .filter(Boolean)
    .map((candidate) => resolve(candidate));

  for (const candidate of candidates) {
    if (!existsSync(candidate)) {
      continue;
    }
    const probe = spawnSync(candidate, ["--version"], { cwd: repoRoot, encoding: "utf8" });
    if (probe.status === 0) {
      return { command: candidate, prefix: [], label: candidate };
    }
  }

  return {
    command: "cargo",
    prefix: ["run", "-q", "-p", "vize", "--"],
    label: "cargo run -q -p vize --",
  };
}

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function selectProjects(projects, selectedIds) {
  if (selectedIds.length === 0) {
    return projects;
  }

  const byId = new Map(projects.map((project) => [project.id, project]));
  return selectedIds.map((id) => {
    const project = byId.get(id);
    if (project == null) {
      throw new Error(`Unknown fixture project: ${id}`);
    }
    return project;
  });
}

function parseTarget(target) {
  if (target !== "dom" && target !== "ssr") {
    throw new Error(`Unsupported target: ${target}`);
  }
  return target;
}

function parseTemplateSyntax(value) {
  if (!["standard", "strict", "quirks"].includes(value)) {
    throw new Error(`Unsupported template syntax mode: ${value}`);
  }
  return value;
}

function parsePositiveInteger(value, name) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return parsed;
}

function splitCsv(value) {
  return value
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);
}

function emptySummary() {
  return {
    fileCount: 0,
    changedFiles: 0,
    additions: 0,
    removals: 0,
    officialErrors: 0,
    vizeErrors: 0,
  };
}

function timestampSlug(date) {
  return date
    .toISOString()
    .replace(/\.\d{3}Z$/, "Z")
    .replace(/[:.]/g, "-");
}

function displayCommand(command, args) {
  return [command, ...args].map(shellQuote).join(" ");
}

function shellQuote(value) {
  if (/^[A-Za-z0-9_./:=@-]+$/.test(value)) {
    return value;
  }
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function executableName(name) {
  return process.platform === "win32" ? `${name}.exe` : name;
}

function truncate(value) {
  if (value == null || value.length <= 4000) {
    return value ?? "";
  }
  return `${value.slice(0, 4000)}\n...<truncated>`;
}

try {
  main();
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
}
