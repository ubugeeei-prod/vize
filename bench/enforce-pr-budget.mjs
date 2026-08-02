#!/usr/bin/env node
/**
 * Fail CI when PR benchmark results exceed the configured regression budget.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { formatRegressionLine } from "./compare-pr-results.mjs";
import { createBenchmarkBudget } from "./compare-pr.mjs";

export const DEFAULT_SKIP_OVERRIDE_LABEL = "ci:allow-skipped-benchmark";

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

function parseLabelsJson(value) {
  if (!value) {
    return [];
  }

  const labels = JSON.parse(value);
  if (!Array.isArray(labels) || labels.some((label) => typeof label !== "string")) {
    throw new Error("--labels-json must be a JSON array of label names");
  }
  return labels;
}

export function enforceBenchmarkBudget(data, options = {}) {
  if (data.skipped) {
    const skipOverrideLabel = options.skipOverrideLabel ?? DEFAULT_SKIP_OVERRIDE_LABEL;
    const labels = options.labels ?? data.labels ?? [];
    if (labels.includes(skipOverrideLabel)) {
      return {
        ok: true,
        message: `Benchmark budget skipped with override label '${skipOverrideLabel}': ${data.reason ?? "benchmark skipped"}`,
      };
    }

    return {
      ok: false,
      message: `Benchmark budget skipped without override label '${skipOverrideLabel}': ${data.reason ?? "benchmark skipped"}`,
    };
  }

  const budget = data.budget ?? createBenchmarkBudget(data.results ?? []);
  if (budget.status !== "failed") {
    const confirmed = (data.results ?? []).filter((result) => result.attempts?.length === 2);
    return {
      ok: true,
      message:
        confirmed.length === 0
          ? "Benchmark budget passed."
          : `Benchmark budget passed after extending ${confirmed.length} initial breach(es) with paired confirmation samples: ${confirmed.map((result) => result.label).join(", ")}.`,
    };
  }

  const failures = budget.regressions.map(formatRegressionLine).join("\n");

  return {
    ok: false,
    message: `Benchmark regression budget failed for ${budget.regressionCount} task(s) after paired confirmation:\n${failures}`,
  };
}

export function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const jsonPath = resolve(requireArg(args, "json"));
  const data = JSON.parse(readFileSync(jsonPath, "utf8"));
  const result = enforceBenchmarkBudget(data, {
    labels: parseLabelsJson(args["labels-json"]),
    skipOverrideLabel: args["skip-override-label"],
  });

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
