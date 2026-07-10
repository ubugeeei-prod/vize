import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";

const scratchRoot = path.join(repoRoot, "target", "vize-tests", "tooling-tests");

function createReport(totals: unknown): string {
  fs.mkdirSync(scratchRoot, { recursive: true });
  const root = fs.mkdtempSync(path.join(scratchRoot, "rust-source-coverage-"));
  const reportPath = path.join(root, "summary.json");
  fs.writeFileSync(reportPath, `${JSON.stringify({ data: [{ totals }] })}\n`);
  return reportPath;
}

test("rust source coverage script passes when every metric clears its minimum", () => {
  const reportPath = createReport({
    lines: { count: 1000, covered: 850, percent: 85 },
    functions: { count: 200, covered: 150, percent: 75 },
    regions: { count: 500, covered: 333, percent: 66.66666666666667 },
  });

  const result = runMoonScript("enforce_rust_source_coverage", [
    "--json",
    reportPath,
    "--min-lines",
    "70",
    "--min-functions",
    "70",
    "--min-regions",
    "60",
  ]);

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.equal(
    result.stdout,
    `## Rust Source Coverage

| Metric | Covered | Total | Percent | Minimum | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| Lines | 850 | 1000 | 85.00% | 70.00% | pass |
| Functions | 150 | 200 | 75.00% | 70.00% | pass |
| Regions | 333 | 500 | 66.67% | 60.00% | pass |
`,
  );
  assert.equal(result.stderr, "");
});

test("rust source coverage script fails and lists every metric below its minimum", () => {
  const reportPath = createReport({
    lines: { count: 1000, covered: 850, percent: 85 },
    functions: { count: 200, covered: 130, percent: 65 },
    regions: { count: 500, covered: 300, percent: 60 },
  });

  const result = runMoonScript("enforce_rust_source_coverage", [
    "--json",
    reportPath,
    "--min-lines",
    "70",
    "--min-functions",
    "70",
    "--min-regions",
    "70",
  ]);

  assert.equal(result.status, 1, `${result.stderr}\n${result.stdout}`.trim());
  assert.equal(
    result.stdout,
    `## Rust Source Coverage

| Metric | Covered | Total | Percent | Minimum | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| Lines | 850 | 1000 | 85.00% | 70.00% | pass |
| Functions | 130 | 200 | 65.00% | 70.00% | fail |
| Regions | 300 | 500 | 60.00% | 70.00% | fail |

Rust source coverage budget failed:
Functions coverage 65.00% < 70.00%
Regions coverage 60.00% < 70.00%
`,
  );
});

test("rust source coverage script appends its table to the requested summary file", () => {
  const reportPath = createReport({
    lines: { count: 4, covered: 3, percent: 75 },
  });
  const summaryPath = `${reportPath}.summary.md`;
  fs.writeFileSync(summaryPath, "# Job Summary\n");

  const result = runMoonScript("enforce_rust_source_coverage", [
    "--json",
    reportPath,
    "--markdown",
    summaryPath,
    "--min-lines",
    "70",
  ]);

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.equal(
    fs.readFileSync(summaryPath, "utf8"),
    `# Job Summary
## Rust Source Coverage

| Metric | Covered | Total | Percent | Minimum | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| Lines | 3 | 4 | 75.00% | 70.00% | pass |
`,
  );
});

test("rust source coverage script rejects reports without cargo llvm-cov totals", () => {
  fs.mkdirSync(scratchRoot, { recursive: true });
  const root = fs.mkdtempSync(path.join(scratchRoot, "rust-source-coverage-bad-"));
  const reportPath = path.join(root, "summary.json");
  fs.writeFileSync(reportPath, `${JSON.stringify({ data: [] })}\n`);

  const result = runMoonScript("enforce_rust_source_coverage", [
    "--json",
    reportPath,
    "--min-lines",
    "70",
  ]);

  assert.equal(result.status, 1, `${result.stderr}\n${result.stdout}`.trim());
  assert.equal(result.stdout, `${reportPath} is not a cargo llvm-cov summary JSON report\n`);
});
