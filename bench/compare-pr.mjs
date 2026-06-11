#!/usr/bin/env node
/**
 * Compare PR benchmark timings between a base and head vize binary.
 *
 * The script is intentionally small and dependency-free so GitHub Actions can
 * run it after checking out both commits.
 */

import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, writeFileSync } from "node:fs";
import { basename, delimiter, dirname, join, parse, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";

const DEFAULT_RUNS = 5;
const DEFAULT_WARMUPS = 1;
const DEFAULT_THRESHOLD_PERCENT = 5;

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (!arg.startsWith("--")) {
      continue;
    }
    const key = arg.slice(2);
    const next = argv[i + 1];
    if (next == null || next.startsWith("--")) {
      args[key] = "true";
    } else {
      args[key] = next;
      i++;
    }
  }
  return args;
}

function requireArg(args, key) {
  const value = args[key];
  if (!value) {
    throw new Error(`Missing required argument: --${key}`);
  }
  return value;
}

function parsePositiveInt(value, fallback) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function parseNonNegativeInt(value, fallback) {
  const parsed = Number.parseInt(value ?? "", 10);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : fallback;
}

function parsePositiveFloat(value, fallback) {
  const parsed = Number.parseFloat(value ?? "");
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 1) {
    return sorted[mid];
  }
  return (sorted[mid - 1] + sorted[mid]) / 2;
}

function formatMs(ms) {
  if (!Number.isFinite(ms)) {
    return "n/a";
  }
  return `${ms.toLocaleString("en-US", {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3,
  })}ms`;
}

function formatRate(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  return `${value.toFixed(3)}x`;
}

function formatRunList(values) {
  return values.map(formatMs).join(", ");
}

function formatPercent(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  const sign = value > 0 ? "+" : "";
  return `${sign}${value.toFixed(2)}%`;
}

function pathWithNodeBins(cwd) {
  const dirs = [];
  let current = cwd;
  const root = parse(current).root;
  while (true) {
    const candidate = join(current, "node_modules", ".bin");
    if (existsSync(candidate)) {
      dirs.push(candidate);
    }
    if (current === root) {
      break;
    }
    current = dirname(current);
  }
  return [...dirs.reverse(), process.env.PATH ?? ""].join(delimiter);
}

function runCommand(binary, commandArgs, options) {
  const start = performance.now();
  const result = spawnSync(binary, commandArgs, {
    cwd: options.cwd,
    env: {
      ...process.env,
      NO_COLOR: "1",
      PATH: options.path,
      RAYON_NUM_THREADS: "1",
      VIZE_BENCH: "1",
    },
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  const elapsedMs = performance.now() - start;

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0 && !options.allowNonZeroExit) {
    const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`.trim();
    throw new Error(
      `${basename(binary)} ${commandArgs.join(" ")} exited with ${result.status}\n${output}`,
    );
  }

  return elapsedMs;
}

function measureTask(task, baseBin, headBin, options) {
  for (let i = 0; i < options.warmups; i++) {
    runCommand(baseBin, task.args, options);
    runCommand(headBin, task.args, options);
  }

  const baseRuns = [];
  const headRuns = [];
  for (let i = 0; i < options.runs; i++) {
    if (i % 2 === 0) {
      baseRuns.push(runCommand(baseBin, task.args, options));
      headRuns.push(runCommand(headBin, task.args, options));
    } else {
      headRuns.push(runCommand(headBin, task.args, options));
      baseRuns.push(runCommand(baseBin, task.args, options));
    }
  }

  const baseMs = median(baseRuns);
  const headMs = median(headRuns);
  const rate = baseMs === 0 ? Number.NaN : headMs / baseMs;
  const changePercent = Number.isFinite(rate) ? (rate - 1) * 100 : Number.NaN;
  const status =
    changePercent >= options.thresholdPercent
      ? "regression"
      : changePercent <= -options.thresholdPercent
        ? "faster"
        : "stable";

  return {
    id: task.id,
    label: task.label,
    baseMs,
    headMs,
    rate,
    changePercent,
    status,
    baseRuns,
    headRuns,
  };
}

export function createBenchmarkBudget(results) {
  const regressions = results
    .filter((result) => result.status === "regression")
    .map((result) => ({
      id: result.id,
      label: result.label,
      rate: result.rate,
      changePercent: result.changePercent,
    }));

  return {
    status: regressions.length > 0 ? "failed" : "passed",
    regressionCount: regressions.length,
    regressions,
  };
}

export function makeTasks(inputDir, taskFilter) {
  const tsconfig = join(inputDir, "tsconfig.json");
  const pattern = ".";
  const allTasks = [
    {
      id: "compile",
      label: "Compile SFC",
      args: ["build", pattern, "--format", "stats", "--threads", "1", "--continue-on-error"],
      allowNonZeroExit: false,
    },
    {
      id: "lint",
      label: "Lint",
      args: ["lint", pattern, "--quiet"],
      allowNonZeroExit: true,
    },
    {
      id: "fmt",
      label: "Format",
      // `*.vue` instead of `.`: fmt expands `.` into a gitignore-aware walk
      // and bench/__in__ is gitignored, so that walk finds zero files; the
      // plain glob matches the corpus directly (same invocation as
      // bench/fmt.ts and compare-tools.mjs). `--check` formats in memory and
      // never writes, so the corpus stays byte-identical between the
      // alternating base/head runs. The generated corpus is intentionally
      // unformatted, so the non-zero "would reformat" exit is expected.
      // fmt has no --threads flag; the RAYON_NUM_THREADS=1 set by runCommand
      // pins it to a single thread like the other lanes.
      args: ["fmt", "*.vue", "--check"],
      allowNonZeroExit: true,
    },
    {
      id: "check",
      label: "Type check",
      // --servers 1 pins the lane to a single Corsa server so it isolates
      // single-program performance and stays deterministic on shared CI
      // runners. The trade-off is that multi-server sharding regressions are
      // not covered here; a dedicated multi-server lane is tracked in #1386
      // (PR3) and needs its own noise validation before it can gate PRs.
      args: ["check", pattern, "--quiet", "--servers", "1", "--tsconfig", tsconfig],
      allowNonZeroExit: true,
      enabled: existsSync(tsconfig),
    },
  ];

  const requested = new Set(
    taskFilter
      .split(",")
      .map((task) => task.trim())
      .filter(Boolean),
  );
  return allTasks.filter((task) => {
    if (task.enabled === false) {
      return false;
    }
    return requested.size === 0 || requested.has(task.id);
  });
}

export function renderMarkdown(data) {
  const budget = data.budget ?? createBenchmarkBudget(data.results);
  const lines = [];
  lines.push("## PR Benchmark");
  lines.push("");
  lines.push(
    `Base: \`${data.baseLabel}\`  Head: \`${data.headLabel}\`  Input: ${data.fileCount.toLocaleString()} generated SFC files`,
  );
  lines.push(
    `Median of ${data.runs} measured run(s) after ${data.warmups} warmup run(s). Times are shown in milliseconds to 0.001ms. Rate is head/base, so below 1.000x is faster. Regression threshold: ${data.thresholdPercent}%.`,
  );
  lines.push(
    `Budget: ${budget.status}${budget.regressionCount > 0 ? ` (${budget.regressionCount} regression${budget.regressionCount === 1 ? "" : "s"})` : ""}.`,
  );
  lines.push("");
  lines.push("| Task | Base | Head | Rate | Result |");
  lines.push("| --- | ---: | ---: | ---: | --- |");
  for (const result of data.results) {
    lines.push(
      `| ${result.label} | ${formatMs(result.baseMs)} | ${formatMs(result.headMs)} | ${formatRate(result.rate)} | ${result.status} |`,
    );
  }
  if (budget.regressionCount > 0) {
    lines.push("");
    lines.push("Regression budget failures:");
    for (const regression of budget.regressions) {
      lines.push(
        `- ${regression.label}: ${formatRate(regression.rate)} (${formatPercent(regression.changePercent)})`,
      );
    }
  }
  lines.push("");
  lines.push("<details>");
  lines.push("<summary>Raw run times</summary>");
  lines.push("");
  for (const result of data.results) {
    lines.push(`### ${result.label}`);
    lines.push("");
    lines.push(`- Base: ${formatRunList(result.baseRuns)}`);
    lines.push(`- Head: ${formatRunList(result.headRuns)}`);
    lines.push("");
  }
  lines.push("</details>");
  lines.push("");
  return `${lines.join("\n")}\n`;
}

export function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const inputDir = resolve(requireArg(args, "input"));
  const baseBin = resolve(requireArg(args, "base-bin"));
  const headBin = resolve(requireArg(args, "head-bin"));
  const runs = parsePositiveInt(args.runs, DEFAULT_RUNS);
  const warmups = parseNonNegativeInt(args.warmups, DEFAULT_WARMUPS);
  const thresholdPercent = parsePositiveFloat(args.threshold, DEFAULT_THRESHOLD_PERCENT);
  const taskFilter = args.tasks ?? "";

  if (!existsSync(inputDir)) {
    throw new Error(`Input directory not found: ${inputDir}`);
  }
  if (!existsSync(baseBin)) {
    throw new Error(`Base binary not found: ${baseBin}`);
  }
  if (!existsSync(headBin)) {
    throw new Error(`Head binary not found: ${headBin}`);
  }

  const fileCount = readdirSync(inputDir).filter((entry) => entry.endsWith(".vue")).length;
  if (fileCount === 0) {
    throw new Error(`No .vue files found in ${inputDir}`);
  }

  const tasks = makeTasks(inputDir, taskFilter);
  if (tasks.length === 0) {
    throw new Error(`No benchmark tasks selected. Requested: ${taskFilter || "(default)"}`);
  }

  const options = {
    cwd: inputDir,
    path: pathWithNodeBins(inputDir),
    runs,
    warmups,
    thresholdPercent,
    allowNonZeroExit: false,
  };

  const results = tasks.map((task) =>
    measureTask(task, baseBin, headBin, {
      ...options,
      allowNonZeroExit: task.allowNonZeroExit,
    }),
  );

  const data = {
    baseLabel: args["base-label"] ?? "base",
    headLabel: args["head-label"] ?? "head",
    inputDir,
    fileCount,
    runs,
    warmups,
    thresholdPercent,
    results,
    budget: createBenchmarkBudget(results),
  };

  const markdown = renderMarkdown(data);
  if (args.out) {
    writeFileSync(resolve(args.out), markdown);
  } else {
    process.stdout.write(markdown);
  }
  if (args.json) {
    writeFileSync(resolve(args.json), `${JSON.stringify(data, null, 2)}\n`);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
