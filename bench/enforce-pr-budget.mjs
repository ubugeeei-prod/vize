#!/usr/bin/env node
/**
 * Fail CI when PR benchmark results exceed the configured regression budget.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { createBenchmarkBudget } from "./compare-pr.mjs";

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

function formatPercent(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  const sign = value > 0 ? "+" : "";
  return `${sign}${value.toFixed(2)}%`;
}

function formatRate(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  return `${value.toFixed(3)}x`;
}

export function enforceBenchmarkBudget(data) {
  if (data.skipped) {
    return {
      ok: true,
      message: `Benchmark budget skipped: ${data.reason ?? "benchmark skipped"}`,
    };
  }

  const budget = data.budget ?? createBenchmarkBudget(data.results ?? []);
  if (budget.status !== "failed") {
    return {
      ok: true,
      message: "Benchmark budget passed.",
    };
  }

  const failures = budget.regressions
    .map(
      (regression) =>
        `- ${regression.label}: ${formatRate(regression.rate)} (${formatPercent(regression.changePercent)})`,
    )
    .join("\n");

  return {
    ok: false,
    message: `Benchmark regression budget failed for ${budget.regressionCount} task(s):\n${failures}`,
  };
}

export function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const jsonPath = resolve(requireArg(args, "json"));
  const data = JSON.parse(readFileSync(jsonPath, "utf8"));
  const result = enforceBenchmarkBudget(data);

  console.log(result.message);
  if (!result.ok) {
    process.exitCode = 1;
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
