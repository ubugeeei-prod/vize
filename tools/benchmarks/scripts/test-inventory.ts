#!/usr/bin/env node
/**
 * Build a static inventory of repository test cases.
 */

import { appendFileSync, writeFileSync } from "node:fs";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { collectRustTests, customRustTestTargets } from "./rust-test-inventory.ts";
import type { TestCase, TestGroup } from "./rust-test-inventory.ts";
import type { CliArgs } from "./script-types.ts";
import { isRecord } from "./script-types.ts";

type AreaSummary = { area: string; files: number; cases: number };
type Metric = { covered: number; count: number };
type ComponentSummary = {
  component: string;
  files: number;
  cases: number;
  coverage?: { lines: Metric; regions: Metric; functions: Metric };
};
export type TestInventory = {
  totalCases: number;
  totalFiles: number;
  areas: AreaSummary[];
  groups: TestGroup[];
  components: ComponentSummary[];
  coverageSource?: { headSha: string };
};

function componentFor(file: string): string {
  return file.split("/").slice(0, 2).join("/");
}

const IGNORED_DIRS = new Set([
  ".git",
  ".moon",
  ".turbo",
  ".vite",
  "coverage",
  "dist",
  "dialect-guard-out",
  "node_modules",
  "playwright-report",
  "target",
  "test-results",
  "__snapshots__",
]);

const JS_EXTENSIONS = new Set([".js", ".mjs", ".cjs", ".ts", ".mts", ".cts", ".tsx", ".jsx"]);

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

function walkFiles(root: string): string[] {
  const files: string[] = [];
  const stack = [root];
  while (stack.length) {
    const current = stack.pop()!;
    const entries = fs.readdirSync(current, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.name.startsWith(".") && entry.name !== ".github" && entry.name !== ".cargo") {
        continue;
      }
      if (IGNORED_DIRS.has(entry.name)) {
        continue;
      }
      const absolute = path.join(current, entry.name);
      if (entry.isDirectory()) {
        stack.push(absolute);
      } else if (entry.isFile()) {
        files.push(absolute);
      }
    }
  }
  return files.sort((a, b) => a.localeCompare(b));
}

function normalizePath(root: string, absolute: string): string {
  return path.relative(root, absolute).split(path.sep).join("/");
}

function lineNumberForIndex(content: string, index: number): number {
  let line = 1;
  for (let i = 0; i < index; i++) {
    if (content.charCodeAt(i) === 10) {
      line++;
    }
  }
  return line;
}

function unescapeName(value: string): string {
  return value.replace(/\\([\\'"`nrt])/g, (_: string, escaped: string) => {
    switch (escaped) {
      case "n":
        return "\\n";
      case "r":
        return "\\r";
      case "t":
        return "\\t";
      default:
        return escaped;
    }
  });
}

function classifyJs(relativePath: string): { area: string; runner: string } {
  if (
    relativePath.includes("/e2e/vrt/") ||
    relativePath.includes("playwright.vrt.") ||
    relativePath.endsWith(".vrt.spec.ts")
  ) {
    return { area: "VRT", runner: "Playwright VRT" };
  }
  if (
    relativePath.includes("/e2e/") ||
    relativePath.startsWith("tests/app/") ||
    relativePath.includes("playwright.config.")
  ) {
    return { area: "E2E", runner: "Playwright / Vitest e2e" };
  }
  return { area: "JS / TS", runner: "node:test / vitest" };
}

function shouldScanJs(relativePath: string, content: string): boolean {
  const basename = path.posix.basename(relativePath);
  return (
    /\.test\.[cm]?[jt]sx?$/.test(basename) ||
    /\.spec\.[cm]?[jt]sx?$/.test(basename) ||
    content.includes("node:test") ||
    content.includes("@playwright/test") ||
    content.includes("import.meta.vitest")
  );
}

function collectJsTests(root: string, absolute: string): TestGroup | null {
  const relativePath = normalizePath(root, absolute);
  const content = fs.readFileSync(absolute, "utf8");
  if (!shouldScanJs(relativePath, content)) {
    return null;
  }

  const tests: TestCase[] = [];
  const pattern = /(^|[^\w$.])(?:void\s+)?(?:test|it)\s*\(\s*(["'`])((?:\\.|(?!\2)[\s\S])*?)\2/gm;
  for (const match of content.matchAll(pattern)) {
    tests.push({
      name: unescapeName(match[3].trim()),
      line: lineNumberForIndex(content, match.index + match[1].length),
    });
  }

  if (tests.length === 0) {
    return null;
  }

  const classification = classifyJs(relativePath);
  return {
    ...classification,
    file: relativePath,
    count: tests.length,
    tests,
  };
}

function collectFixtureCases(root: string, absolute: string): TestGroup | null {
  const relativePath = normalizePath(root, absolute);
  if (!relativePath.startsWith("tests/fixtures/")) {
    return null;
  }

  const content = fs.readFileSync(absolute, "utf8");
  let tests: TestCase[];
  if (relativePath.endsWith(".toml")) {
    tests = collectTomlFixtureCases(content);
  } else if (relativePath.endsWith(".pkl")) {
    tests = collectPklFixtureCases(content);
  } else {
    return null;
  }

  if (tests.length === 0) {
    return null;
  }

  return {
    area: "Compiler Fixtures",
    runner: "vize_test_runner",
    file: relativePath,
    count: tests.length,
    tests,
  };
}

function collectTomlFixtureCases(content: string): TestCase[] {
  const tests: TestCase[] = [];
  const casePattern = /^\[\[cases\]\]/gm;
  for (const match of content.matchAll(casePattern)) {
    const blockStart = match.index;
    const blockEnd = content.indexOf("\n[[cases]]", blockStart + 1);
    const block = content.slice(blockStart, blockEnd === -1 ? undefined : blockEnd);
    const nameMatch = /^\s*name\s*=\s*(["'])(.*?)\1/m.exec(block);
    tests.push({
      name: nameMatch ? unescapeName(nameMatch[2]) : `case ${tests.length + 1}`,
      line: lineNumberForIndex(content, blockStart),
    });
  }
  return tests;
}

// Typed Pkl fixtures emit each case as `new { name = "..." ... }`. A schema
// module declares `name: String` (a colon, not `=`), so a `name = <string>`
// assignment uniquely identifies a case and skips the schema files.
function collectPklFixtureCases(content: string): TestCase[] {
  const tests: TestCase[] = [];
  const casePattern = /^[ \t]*name\s*=\s*(#*)(["'])(.*?)\2\1/gm;
  for (const match of content.matchAll(casePattern)) {
    tests.push({
      name: unescapeName(match[3]),
      line: lineNumberForIndex(content, match.index),
    });
  }
  return tests;
}

export function collectInventory(root: string = process.cwd()): TestInventory {
  const files = walkFiles(root);
  const customTargets = customRustTestTargets(files);
  const groups: TestGroup[] = [];

  for (const absolute of files) {
    const relativePath = normalizePath(root, absolute);
    const extension = path.extname(relativePath);
    let group: TestGroup | null = null;

    if (
      relativePath.startsWith("tests/fixtures/") &&
      (extension === ".toml" || extension === ".pkl")
    ) {
      group = collectFixtureCases(root, absolute);
    } else if (extension === ".rs") {
      group = collectRustTests(root, absolute, customTargets, lineNumberForIndex);
    } else if (JS_EXTENSIONS.has(extension)) {
      group = collectJsTests(root, absolute);
    }

    if (group) {
      groups.push(group);
    }
  }

  groups.sort((a, b) => a.area.localeCompare(b.area) || a.file.localeCompare(b.file));

  const areas: AreaSummary[] = [];
  const byArea = new Map<string, AreaSummary>();
  for (const group of groups) {
    const area = byArea.get(group.area) ?? { area: group.area, files: 0, cases: 0 };
    area.files++;
    area.cases += group.count;
    byArea.set(group.area, area);
  }
  areas.push(...byArea.values());
  areas.sort((a, b) => a.area.localeCompare(b.area));

  const components = new Map<string, ComponentSummary>();
  // Include workspace components with no statically recognized cases.
  for (const absolute of files) {
    const file = normalizePath(root, absolute);
    if (/^(?:crates|davinci|npm)\/[^/]+\/(?:Cargo\.toml|package\.json)$/.test(file)) {
      const component = componentFor(file);
      components.set(component, { component, files: 0, cases: 0 });
    }
  }
  for (const group of groups) {
    const component = componentFor(group.file);
    const summary = components.get(component) ?? { component, files: 0, cases: 0 };
    summary.files++;
    summary.cases += group.count;
    components.set(component, summary);
  }

  return {
    totalCases: groups.reduce((sum, group) => sum + group.count, 0),
    totalFiles: groups.length,
    areas,
    groups,
    components: [...components.values()].sort((a, b) => a.component.localeCompare(b.component)),
  };
}

/** Join measured LLVM totals by repository component, never average file percentages. */
export function attachSourceCoverage(
  inventory: TestInventory,
  report: unknown,
  headSha: string,
): void {
  if (!/^[a-f0-9]{40}$/i.test(headSha)) {
    throw new Error("--source-sha must identify the measured report's full commit SHA");
  }
  if (
    !isRecord(report) ||
    report.type !== "llvm.coverage.json.export" ||
    !Array.isArray(report.data) ||
    report.data.length !== 1 ||
    !isRecord(report.data[0]) ||
    !Array.isArray(report.data[0].files)
  ) {
    throw new Error("Expected one cargo llvm-cov summary JSON export");
  }
  const components = new Map(inventory.components.map((entry) => [entry.component, entry]));
  for (const file of report.data[0].files) {
    if (!isRecord(file) || typeof file.filename !== "string" || !isRecord(file.summary)) {
      throw new Error("Invalid LLVM file summary");
    }
    const normalized = file.filename.replaceAll("\\", "/");
    const match = /(?:^|\/)((?:crates|davinci)\/[^/]+)\//.exec(normalized);
    if (!match) continue;
    const component = match[1];
    const summary = components.get(component) ?? { component, files: 0, cases: 0 };
    const coverage = summary.coverage ?? {
      lines: { covered: 0, count: 0 },
      regions: { covered: 0, count: 0 },
      functions: { covered: 0, count: 0 },
    };
    for (const key of ["lines", "regions", "functions"] as const) {
      const metric = file.summary[key];
      if (
        !isRecord(metric) ||
        typeof metric.covered !== "number" ||
        typeof metric.count !== "number" ||
        !Number.isSafeInteger(metric.covered) ||
        !Number.isSafeInteger(metric.count) ||
        metric.covered < 0 ||
        metric.covered > metric.count
      ) {
        throw new Error(`Invalid LLVM ${key} totals for ${file.filename}`);
      }
      coverage[key].covered += metric.covered;
      coverage[key].count += metric.count;
    }
    summary.coverage = coverage;
    components.set(component, summary);
  }
  inventory.components = [...components.values()].sort((a, b) =>
    a.component.localeCompare(b.component),
  );
  inventory.coverageSource = { headSha };
}

function escapeTable(value: string): string {
  return String(value ?? "")
    .replace(/\|/g, "\\|")
    .replace(/\r?\n/g, "<br>");
}

export function renderInventoryMarkdown(inventory: TestInventory): string {
  const lines = [
    "## Test Inventory",
    "",
    `Total tracked cases: **${inventory.totalCases}** across **${inventory.totalFiles}** files.`,
    "",
    "| Area | Files | Cases |",
    "| --- | ---: | ---: |",
  ];

  for (const area of inventory.areas) {
    lines.push(`| ${escapeTable(area.area)} | ${area.files} | ${area.cases} |`);
  }

  lines.push(
    "",
    "### Components",
    "",
    "Cases are static declarations, not executed tests or measured coverage. Parameterized cases, fixtures and custom harnesses can execute many checks per declaration.",
    "Components use the first two repository path segments; test files belong to their containing component. Cross-component integration tests stay under their own test suite.",
    "",
  );
  if (inventory.coverageSource) {
    lines.push(
      `Measured coverage source: \`${inventory.coverageSource.headSha}\`. Counts describe the scanned checkout; coverage describes this separate commit.`,
      "",
    );
  }
  lines.push(
    "| Component | Test files | Static cases | Measured lines | Measured regions | Measured functions |",
    "| --- | ---: | ---: | ---: | ---: | ---: |",
  );
  const metricText = (metric: Metric | undefined): string =>
    metric == null
      ? "unmeasured"
      : `${metric.covered}/${metric.count} (${metric.count === 0 ? "n/a" : `${((100 * metric.covered) / metric.count).toFixed(2)}%`})`;
  for (const entry of inventory.components) {
    lines.push(
      `| ${escapeTable(entry.component)} | ${entry.files} | ${entry.cases} | ${metricText(entry.coverage?.lines)} | ${metricText(entry.coverage?.regions)} | ${metricText(entry.coverage?.functions)} |`,
    );
  }

  // File rows only: a per-test table exceeds GitHub's 1024k step-summary cap.
  lines.push("");
  lines.push("<details>");
  lines.push("<summary>Files</summary>");
  lines.push("");
  lines.push("| Area | File | Cases |");
  lines.push("| --- | --- | ---: |");
  for (const group of inventory.groups) {
    lines.push(`| ${escapeTable(group.area)} | \`${escapeTable(group.file)}\` | ${group.count} |`);
  }
  lines.push("");
  lines.push("</details>");
  return lines.join("\n");
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  const root = path.resolve(args.root ?? process.cwd());
  const inventory = collectInventory(root);
  if (args["source-coverage"]) {
    const report: unknown = JSON.parse(fs.readFileSync(args["source-coverage"], "utf8"));
    attachSourceCoverage(inventory, report, args["source-sha"] ?? "");
  } else if (args["source-sha"]) {
    throw new Error("--source-sha requires --source-coverage");
  }

  if (args.json) {
    writeFileSync(args.json, `${JSON.stringify(inventory, null, 2)}\n`);
  }

  if (args.markdown) {
    appendFileSync(args.markdown, `${renderInventoryMarkdown(inventory)}\n`);
  }

  if (!args.json && !args.markdown) {
    console.log(renderInventoryMarkdown(inventory));
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
