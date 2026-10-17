import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { VIZE_BIN, requireVizeBin, type AppConfig } from "./apps.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../..");

export interface InspectorDiffBudget {
  additions: number;
  changedFiles: number;
  officialErrors: number;
  removals: number;
  target: "dom" | "ssr";
  vizeErrors: number;
}

interface InspectorCompareReport {
  schema: string;
  target: "dom" | "ssr";
  summary: {
    additions: number;
    changedFiles: number;
    fileCount: number;
    officialErrors: number;
    removals: number;
    vizeErrors: number;
  };
  files: Array<{
    changed: boolean;
    path: string;
    stats: {
      additions: number;
      removals: number;
      unchanged: number;
    };
  }>;
}

interface InspectorSourceIdentity {
  fileCount: number;
  pathsSha256: string;
}

export function assertInspectorCompareBudgets(
  app: AppConfig,
  budgets: InspectorDiffBudget[],
  inputs: InspectorSourceIdentity,
): void {
  requireVizeBin();

  for (const budget of budgets) {
    const report = runInspectorCompare(app, budget.target);
    assert.equal(report.schema, "vize.inspector.compare");
    assert.equal(report.target, budget.target);
    writeInspectorReport(app.name, budget.target, report);
    assert.ok(app.check, `${app.name} should define check fixtures`);
    assertInspectorFileCoverage(app.check, report, inputs);
    assertBudget(report, budget, app.name);
  }
}

/** Require exactly one comparison for every original input selected by the app. */
export function assertInspectorFileCoverage(
  check: NonNullable<AppConfig["check"]>,
  report: { summary: { fileCount: number }; files: ReadonlyArray<{ path: string }> },
  inputs: InspectorSourceIdentity,
): void {
  const expected = inspectorSourcePaths(check);
  assert.ok(expected.length > 0, "inspector fixtures must contain original source inputs");
  assert.equal(expected.length, inputs.fileCount, "pinned original source count");
  assert.equal(
    createHash("sha256")
      .update(`${expected.join("\n")}\n`)
      .digest("hex"),
    inputs.pathsSha256,
    "pinned original source paths",
  );
  assert.deepEqual(
    report.files.map((file) => file.path.replaceAll("\\", "/")).sort(),
    expected,
    "inspector must compare every original source exactly once",
  );
  assert.equal(report.summary.fileCount, expected.length, "inspector source count");
}

function inspectorSourcePaths(check: NonNullable<AppConfig["check"]>): string[] {
  const selected = new Set<string>();
  const walked = new Set<string>();
  for (const pattern of check.patterns) {
    const entry = fs.statSync(path.resolve(check.cwd, pattern), { throwIfNoEntry: false });
    const root = entry?.isDirectory()
      ? pattern
      : entry?.isFile()
        ? undefined
        : recursiveVueRoot(pattern);
    const candidates = entry?.isFile()
      ? [pattern]
      : root === undefined
        ? fs.globSync(pattern, { cwd: check.cwd })
        : fs
            .globSync("**/*.vue", { cwd: path.resolve(check.cwd, root) })
            .map((file) => path.join(root, file));
    for (const file of candidates) {
      const normalized = file.replaceAll("\\", "/");
      if (
        normalized.endsWith(".vue") &&
        !normalized.split("/").includes(".git") &&
        fs.statSync(path.resolve(check.cwd, file)).isFile()
      ) {
        (root === undefined ? selected : walked).add(normalized);
      }
    }
  }

  // Inspector's explicit files and generic globs bypass ignores. Only its
  // directory/simple recursive-glob branch uses the authored Git ignore rules.
  if (walked.size > 0) {
    const result = spawnSync("git", ["check-ignore", "--no-index", "--stdin", "-z"], {
      cwd: check.cwd,
      input: `${[...walked].join("\0")}\0`,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
      timeout: 10_000,
    });
    if (result.error) throw result.error;
    assert.ok(
      result.status === 0 || result.status === 1,
      `cannot determine inspector Git ignores (exit ${result.status}): ${result.stderr}`,
    );
    const ignored = new Set(result.stdout.split("\0").filter(Boolean));
    for (const file of walked) {
      if (!ignored.has(file)) selected.add(file);
    }
  }
  return [...selected].sort();
}

function recursiveVueRoot(pattern: string): string | undefined {
  const normalized = pattern.replaceAll("\\", "/");
  if (normalized === "**/*.vue" || normalized === "./**/*.vue") return ".";
  const root = normalized.match(/^(.*)\/\*\*\/\*\.vue$/)?.[1];
  return root && !/[?*[\]{}]/.test(root) ? root : undefined;
}

function runInspectorCompare(app: AppConfig, target: "dom" | "ssr"): InspectorCompareReport {
  const check = app.check;
  assert.ok(check, `${app.name} should define check fixtures`);

  const result = spawnSync(
    VIZE_BIN,
    [
      "inspector",
      ...check.patterns,
      "--format",
      "compare",
      "--target",
      target,
      "--template-syntax",
      "quirks",
    ],
    {
      cwd: check.cwd,
      encoding: "utf8",
      env: {
        ...process.env,
        LANG: "C",
        LC_ALL: "C",
      },
      maxBuffer: 512 * 1024 * 1024,
      timeout: 300_000,
    },
  );

  if (result.error != null) {
    throw result.error;
  }
  assert.equal(
    result.status,
    0,
    `${app.name}:${target} inspector compare failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`,
  );

  return JSON.parse(result.stdout) as InspectorCompareReport;
}

function assertBudget(
  report: InspectorCompareReport,
  budget: InspectorDiffBudget,
  appName: string,
): void {
  const label = `${appName}:${budget.target}`;
  assert.equal(report.summary.officialErrors, budget.officialErrors, `${label} official errors`);
  assert.equal(report.summary.vizeErrors, budget.vizeErrors, `${label} vize errors`);
  assert.ok(
    report.summary.changedFiles <= budget.changedFiles,
    `${label} changed file budget: ${report.summary.changedFiles} > ${budget.changedFiles}`,
  );
  assert.ok(
    report.summary.additions <= budget.additions,
    `${label} addition budget: ${report.summary.additions} > ${budget.additions}`,
  );
  assert.ok(
    report.summary.removals <= budget.removals,
    `${label} removal budget: ${report.summary.removals} > ${budget.removals}`,
  );
}

function writeInspectorReport(
  appName: string,
  target: "dom" | "ssr",
  report: InspectorCompareReport,
): void {
  const outputRoot = inspectorReportDirectory(appName, process.env.VIZE_INSPECT_OUTPUT_DIR);
  fs.mkdirSync(outputRoot, { recursive: true });

  fs.writeFileSync(path.join(outputRoot, `${target}.json`), JSON.stringify(report, null, 2) + "\n");
  fs.writeFileSync(
    path.join(outputRoot, `${target}-summary.json`),
    JSON.stringify(
      {
        summary: report.summary,
        largestDiffs: report.files
          .filter((file) => file.changed)
          .map((file) => ({
            path: file.path,
            additions: file.stats.additions,
            removals: file.stats.removals,
          }))
          .sort((a, b) => b.additions + b.removals - (a.additions + a.removals))
          .slice(0, 50),
      },
      null,
      2,
    ) + "\n",
  );
}

/**
 * Resolve the directory used for one application's inspector reports.
 *
 * @param outputDirectory - Explicit directory supplied by the test environment.
 * @default `<repository>/.vize/artifacts/inspect/<application>`
 */
export function inspectorReportDirectory(appName: string, outputDirectory?: string): string {
  return (
    outputDirectory ?? path.join(REPO_ROOT, ".vize", "artifacts", "inspect", sanitizeName(appName))
  );
}

function sanitizeName(name: string): string {
  return name.replace(/[^a-z0-9._-]+/gi, "-").replace(/^-|-$/g, "");
}
