import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
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

export function assertInspectorCompareBudgets(
  app: AppConfig,
  budgets: InspectorDiffBudget[],
): void {
  requireVizeBin();

  for (const budget of budgets) {
    const report = runInspectorCompare(app, budget.target);
    assert.equal(report.schema, "vize.inspector.compare");
    assert.equal(report.target, budget.target);
    assert.ok(report.summary.fileCount > 0, `${app.name}:${budget.target} should inspect files`);
    assertBudget(report, budget, app.name);
    writeInspectorReport(app.name, budget.target, report);
  }
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
      "--vue-parser-quirks",
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
  assert.ok(report.summary.changedFiles <= budget.changedFiles, `${label} changed file budget`);
  assert.ok(report.summary.additions <= budget.additions, `${label} addition budget`);
  assert.ok(report.summary.removals <= budget.removals, `${label} removal budget`);
}

function writeInspectorReport(
  appName: string,
  target: "dom" | "ssr",
  report: InspectorCompareReport,
): void {
  const outputRoot =
    process.env.VIZE_INSPECT_OUTPUT_DIR ??
    path.join(REPO_ROOT, "__agent_only", "inspect", sanitizeName(appName));
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

function sanitizeName(name: string): string {
  return name.replace(/[^a-z0-9._-]+/gi, "-").replace(/^-|-$/g, "");
}
