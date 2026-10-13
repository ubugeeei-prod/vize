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

import { laneEnvironment, laneThresholdPercent, makeTasks } from "./compare-pr-lanes.mjs";
import {
  confirmRegressions,
  formatPercent,
  formatRate,
  formatRegressionLine,
  summarizeBenchmarkRuns,
} from "./compare-pr-results.mjs";

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

function formatMs(ms) {
  if (!Number.isFinite(ms)) {
    return "n/a";
  }
  return `${ms.toLocaleString("en-US", {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3,
  })}ms`;
}

function formatRunList(values) {
  return values.map(formatMs).join(", ");
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
    env: laneEnvironment(options.threads ?? 1, {
      ...process.env,
      NO_COLOR: "1",
      PATH: options.path,
      VIZE_BENCH: "1",
    }),
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

  return summarizeBenchmarkRuns({
    id: task.id,
    label: task.label,
    baseRuns,
    headRuns,
    thresholdPercent: laneThresholdPercent(task, options.thresholdPercent),
  });
}

export function createBenchmarkBudget(results) {
  const regressions = results
    .filter((result) => result.status === "regression")
    .map((result) => ({
      id: result.id,
      label: result.label,
      rate: result.rate,
      changePercent: result.changePercent,
      thresholdPercent: result.thresholdPercent,
    }));

  return {
    status: regressions.length > 0 ? "failed" : "passed",
    regressionCount: regressions.length,
    regressions,
  };
}

/**
 * Lanes judged against something other than the shared default, so a report
 * never implies one budget covered every row. Derived from the results rather
 * than restated, so it cannot drift from what actually gated the run.
 */
export function laneBudgetOverrides(results, defaultThresholdPercent) {
  return results
    .filter(
      (result) =>
        result.thresholdPercent != null && result.thresholdPercent !== defaultThresholdPercent,
    )
    .map((result) => `${result.label} ${result.thresholdPercent}%`);
}

export function renderMarkdown(data) {
  const budget = data.budget ?? createBenchmarkBudget(data.results);
  const lines = [];
  lines.push("## PR Benchmark");
  lines.push("");
  lines.push(
    `Base: \`${data.baseLabel}\`  Head: \`${data.headLabel}\`  Input: ${data.fileCount.toLocaleString()} generated SFC files`,
  );
  const overrides = laneBudgetOverrides(data.results, data.thresholdPercent);
  lines.push(
    `Median of ${data.runs} adjacent head/base pairs after ${data.warmups} warmup pair(s). Any threshold breach receives ${data.runs} fresh confirmation pairs, and the final rate is the paired median over all samples. Times are shown in milliseconds to 0.001ms. Rate below 1.000x is faster. Regression threshold: ${data.thresholdPercent}%${overrides.length === 0 ? "" : `, except ${overrides.join(", ")}`}.`,
  );
  lines.push(
    `Budget: ${budget.status}${budget.regressionCount > 0 ? ` (${budget.regressionCount} regression${budget.regressionCount === 1 ? "" : "s"})` : ""}.`,
  );
  lines.push("");
  lines.push("| Task | Base median | Head median | Paired rate | Result |");
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
      lines.push(formatRegressionLine(regression));
    }
  }
  lines.push("");
  lines.push("<details>");
  lines.push("<summary>Raw run times</summary>");
  lines.push("");
  for (const result of data.results) {
    lines.push(`### ${result.label}`);
    lines.push("");
    const attempts = result.attempts ?? [result];
    for (const [index, attempt] of attempts.entries()) {
      const prefix = attempts.length === 1 ? "" : index === 0 ? "Initial " : "Confirmation ";
      lines.push(
        `- ${prefix}Result: ${formatRate(attempt.rate)} (${formatPercent(attempt.changePercent)}), ${attempt.status}`,
      );
      lines.push(`- ${prefix}Base: ${formatRunList(attempt.baseRuns)}`);
      lines.push(`- ${prefix}Head: ${formatRunList(attempt.headRuns)}`);
    }
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

  const measure = (task) =>
    measureTask(task, baseBin, headBin, {
      ...options,
      allowNonZeroExit: task.allowNonZeroExit,
      threads: task.threads ?? 1,
    });
  const results = confirmRegressions(
    tasks.map((task) => ({ task, result: measure(task) })),
    measure,
    thresholdPercent,
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
