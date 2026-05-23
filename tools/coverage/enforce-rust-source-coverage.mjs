#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

function parseArgs(argv) {
  const args = {
    json: "",
    markdown: process.env.GITHUB_STEP_SUMMARY ?? "",
    thresholds: {},
  };

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    const next = argv[i + 1];
    if (arg === "--json") {
      args.json = requireValue(arg, next);
      i++;
    } else if (arg === "--markdown") {
      args.markdown = requireValue(arg, next);
      i++;
    } else if (arg.startsWith("--min-")) {
      const key = arg.slice("--min-".length);
      args.thresholds[key] = Number.parseFloat(requireValue(arg, next));
      i++;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.json) {
    throw new Error("Usage: node tools/coverage/enforce-rust-source-coverage.mjs --json <path>");
  }

  return args;
}

function requireValue(flag, value) {
  if (value == null || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function readTotals(jsonPath) {
  const report = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
  const totals = report.data?.[0]?.totals;
  if (totals == null || typeof totals !== "object") {
    throw new Error(`${jsonPath} is not a cargo llvm-cov summary JSON report`);
  }
  return totals;
}

function metricPercent(totals, metric) {
  const value = totals[metric];
  if (value == null || typeof value.percent !== "number") {
    throw new Error(`cargo llvm-cov report is missing ${metric}.percent`);
  }
  return value.percent;
}

function metricCount(totals, metric) {
  const value = totals[metric];
  return typeof value?.count === "number" ? value.count : 0;
}

function formatPercent(value) {
  return `${value.toFixed(2)}%`;
}

function renderMarkdown(rows) {
  return [
    "## Rust Source Coverage",
    "",
    "| Metric | Covered | Total | Percent | Minimum | Status |",
    "| --- | ---: | ---: | ---: | ---: | --- |",
    ...rows.map(
      (row) =>
        `| ${row.label} | ${row.covered} | ${row.total} | ${formatPercent(row.percent)} | ${formatPercent(row.minimum)} | ${row.passed ? "pass" : "fail"} |`,
    ),
    "",
  ].join("\n");
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const jsonPath = path.resolve(args.json);
  const totals = readTotals(jsonPath);
  const rows = [];
  const failures = [];
  const metricLabels = {
    branches: "Branches",
    functions: "Functions",
    lines: "Lines",
    regions: "Regions",
  };

  for (const [metric, minimum] of Object.entries(args.thresholds)) {
    const parsedMinimum = Number(minimum);
    if (!Number.isFinite(parsedMinimum)) {
      throw new Error(`Invalid minimum for ${metric}: ${String(minimum)}`);
    }

    const percent = metricPercent(totals, metric);
    const total = metricCount(totals, metric);
    const covered = typeof totals[metric]?.covered === "number" ? totals[metric].covered : 0;
    const passed = total > 0 && percent >= parsedMinimum;
    const label = metricLabels[metric] ?? metric;
    rows.push({ covered, label, minimum: parsedMinimum, passed, percent, total });

    if (!passed) {
      failures.push(`${label} coverage ${formatPercent(percent)} < ${formatPercent(minimum)}`);
    }
  }

  const markdown = renderMarkdown(rows);
  process.stdout.write(markdown);

  if (args.markdown) {
    fs.appendFileSync(args.markdown, markdown);
  }

  if (failures.length > 0) {
    throw new Error(`Rust source coverage budget failed:\n${failures.join("\n")}`);
  }
}

main();
