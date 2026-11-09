import fs from "node:fs";
import path from "node:path";

interface CoverageRow {
  label: string;
  covered: number;
  total: number;
  percent: number;
  minimum: number;
  passed: boolean;
}

interface Threshold {
  metric: string;
  minimum: number;
}

interface Arguments {
  jsonPath: string;
  markdownPath: string;
  thresholds: Threshold[];
}

type JsonObject = Record<string, unknown>;

function isObject(value: unknown): value is JsonObject {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function usage(): string {
  return "Usage: node tools/commands/ci/source-coverage.ts --json <path> [--markdown <path>] [--min-<metric> N]";
}

function readValue(args: readonly string[], index: number, flag: string): string {
  const value = args[index + 1];
  if (value === undefined) throw new Error(`${flag} requires a value`);
  return value;
}

function parseMinimum(metric: string, value: string): number {
  const minimum = Number(value);
  if (value === "" || value.trim() !== value || !Number.isFinite(minimum)) {
    throw new Error(`Invalid minimum for ${metric}: ${value}`);
  }
  if (minimum < 0 || minimum > 100) {
    throw new Error(`Invalid minimum for ${metric}: ${value}`);
  }
  return minimum;
}

function parseArguments(args: readonly string[]): Arguments {
  let jsonPath = "";
  let markdownPath = process.env.GITHUB_STEP_SUMMARY ?? "";
  const thresholds: Threshold[] = [];

  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index]!;
    if (argument === "--json") {
      jsonPath = readValue(args, index, argument);
      index += 1;
    } else if (argument === "--markdown") {
      markdownPath = readValue(args, index, argument);
      index += 1;
    } else if (argument.startsWith("--min-")) {
      const metric = argument.slice("--min-".length);
      const value = readValue(args, index, argument);
      thresholds.push({ metric, minimum: parseMinimum(metric, value) });
      index += 1;
    } else {
      throw new Error(`Unknown argument: ${argument}`);
    }
  }

  if (jsonPath === "") throw new Error(usage());
  return { jsonPath, markdownPath, thresholds };
}

function readReport(file: string): unknown {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function reportTotals(report: unknown): JsonObject | undefined {
  if (!isObject(report) || !Array.isArray(report.data)) return undefined;
  const first = report.data[0];
  if (!isObject(first) || !isObject(first.totals)) return undefined;
  return first.totals;
}

function metricEntry(totals: JsonObject, metric: string): JsonObject | undefined {
  const entry = totals[metric];
  return isObject(entry) ? entry : undefined;
}

function metricPercent(totals: JsonObject, metric: string): number | undefined {
  const percent = metricEntry(totals, metric)?.percent;
  return typeof percent === "number" ? percent : undefined;
}

function metricCount(totals: JsonObject, metric: string, field: string): number {
  const value = metricEntry(totals, metric)?.[field];
  if (typeof value !== "number" || !Number.isFinite(value)) return 0;
  return Math.max(0, Math.trunc(value));
}

function metricLabel(metric: string): string {
  const labels: Record<string, string> = {
    branches: "Branches",
    functions: "Functions",
    lines: "Lines",
    regions: "Regions",
  };
  return labels[metric] ?? metric;
}

function formatPercent(value: number): string {
  const scaled = Math.max(0, Math.floor(value * 100 + 0.5));
  const whole = Math.floor(scaled / 100);
  const fraction = String(scaled % 100).padStart(2, "0");
  return `${whole}.${fraction}%`;
}

function renderMarkdown(rows: readonly CoverageRow[]): string {
  const lines = [
    "## Rust Source Coverage",
    "",
    "| Metric | Covered | Total | Percent | Minimum | Status |",
    "| --- | ---: | ---: | ---: | ---: | --- |",
  ];
  for (const row of rows) {
    lines.push(
      `| ${row.label} | ${row.covered} | ${row.total} | ${formatPercent(row.percent)} | ${formatPercent(row.minimum)} | ${row.passed ? "pass" : "fail"} |`,
    );
  }
  lines.push("");
  return lines.join("\n");
}

function main(): number {
  let args: Arguments;
  try {
    args = parseArguments(process.argv.slice(2));
  } catch (error) {
    console.log(error instanceof Error ? error.message : String(error));
    return 1;
  }

  const reportPath = path.resolve(args.jsonPath);
  let report: unknown;
  try {
    report = readReport(reportPath);
  } catch {
    console.log(`Failed to read or parse ${reportPath}`);
    return 1;
  }
  const totals = reportTotals(report);
  if (totals === undefined) {
    console.log(`${reportPath} is not a cargo llvm-cov summary JSON report`);
    return 1;
  }

  const rows: CoverageRow[] = [];
  const failures: string[] = [];
  for (const threshold of args.thresholds) {
    const percent = metricPercent(totals, threshold.metric);
    if (percent === undefined) {
      console.log(`cargo llvm-cov report is missing ${threshold.metric}.percent`);
      return 1;
    }
    const total = metricCount(totals, threshold.metric, "count");
    const covered = metricCount(totals, threshold.metric, "covered");
    const passed = total > 0 && percent >= threshold.minimum;
    const label = metricLabel(threshold.metric);
    rows.push({ label, covered, total, percent, minimum: threshold.minimum, passed });
    if (!passed) {
      failures.push(
        `${label} coverage ${formatPercent(percent)} < ${formatPercent(threshold.minimum)}`,
      );
    }
  }

  const markdown = renderMarkdown(rows);
  process.stdout.write(markdown);
  if (args.markdownPath !== "") {
    try {
      fs.appendFileSync(args.markdownPath, markdown);
    } catch {
      console.log(`Failed to append summary to ${args.markdownPath}`);
      return 1;
    }
  }
  if (failures.length > 0) {
    console.log("\nRust source coverage budget failed:");
    failures.forEach((failure) => console.log(failure));
    return 1;
  }
  return 0;
}

process.exitCode = main();
