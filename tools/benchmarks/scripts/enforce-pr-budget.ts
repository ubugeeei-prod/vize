#!/usr/bin/env node
/**
 * Fail CI when PR benchmark results exceed the configured regression budget.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { formatRegressionLine } from "./compare-pr-results.ts";
import { createBenchmarkBudget } from "./compare-pr.ts";
import { isBenchmarkArtifact } from "./pr-benchmark-artifact.ts";
import type { BenchmarkArtifact } from "./pr-benchmark-artifact.ts";
import { isStringArray } from "./script-types.ts";
import type { CliArgs } from "./script-types.ts";

export const DEFAULT_SKIP_OVERRIDE_LABEL = "ci:allow-skipped-benchmark";

export type EnforceOptions = { labels?: string[]; skipOverrideLabel?: string };

function parseArgs(argv: string[]): CliArgs {
  const args: CliArgs = {};
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

function requireArg(args: CliArgs, key: string): string {
  const value = args[key];
  if (!value) {
    throw new Error(`Missing required argument: --${key}`);
  }
  return value;
}

function parseLabelsJson(value: string | undefined): string[] {
  if (!value) {
    return [];
  }

  const labels: unknown = JSON.parse(value);
  if (!isStringArray(labels)) {
    throw new Error("--labels-json must be a JSON array of label names");
  }
  return labels;
}

export function enforceBenchmarkBudget(
  data: BenchmarkArtifact,
  options: EnforceOptions = {},
): { ok: boolean; message: string } {
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

export function main(argv: string[] = process.argv.slice(2)): void {
  const args = parseArgs(argv);
  const jsonPath = resolve(requireArg(args, "json"));
  const data: unknown = JSON.parse(readFileSync(jsonPath, "utf8"));
  if (!isBenchmarkArtifact(data)) {
    throw new Error(`${jsonPath} is not a PR benchmark artifact`);
  }
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
