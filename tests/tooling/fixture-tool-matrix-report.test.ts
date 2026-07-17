import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const toolPath = path.join(root, "tools", "fixtures", "tool-matrix-report.mjs");

function run(args: string[]) {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-matrix-"));
  const result = spawnSync(process.execPath, [toolPath, ...args, "--output-dir", outputDir], {
    cwd: root,
    encoding: "utf8",
  });
  return { outputDir, result };
}

test("fixture tool matrix plans every registered project across all four required tools", () => {
  const { outputDir, result } = run(["--dry-run"]);
  try {
    assert.equal(result.status, 0, result.stderr);
    const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
    assert.equal(report.schema, "vize.fixtureToolMatrixReport");
    assert.equal(report.summary.projectCount, 131);
    assert.equal(report.summary.toolCount, 4);
    assert.equal(report.summary.runCount, 524);
    assert.equal(report.summary.plannedRuns, 524);
    assert.equal(report.projects.length, 131);
    for (const project of report.projects) {
      assert.deepEqual(
        project.runs.map((entry: { tool: string }) => entry.tool),
        ["compiler", "typechecker", "linter", "formatter"],
        `${project.id} should exercise every requested tool`,
      );
    }
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix emits read-only commands with machine-readable diagnostics", () => {
  const { outputDir, result } = run([
    "--dry-run",
    "--project",
    "vue-vben-admin",
    "--tool",
    "compiler,typechecker,linter,formatter",
  ]);
  try {
    assert.equal(result.status, 0, result.stderr);
    const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
    const runs = Object.fromEntries(
      report.projects[0].runs.map((entry: { tool: string }) => [entry.tool, entry]),
    ) as Record<string, { command: string }>;
    assert.match(runs.compiler.command, /inspector .*--format json --template-syntax quirks/);
    assert.match(
      runs.typechecker.command,
      /check .*--format json --no-config --tsconfig playground\/tsconfig\.json/,
    );
    assert.match(runs.linter.command, /lint .*--format json --preset ecosystem --no-config/);
    assert.match(runs.formatter.command, /fmt .*--check --no-config/);
    for (const entry of Object.values(runs)) {
      assert.doesNotMatch(entry.command, /(?:^|\s)--write(?:\s|$)/);
      assert.doesNotMatch(entry.command, /(?:^|\s)-w(?:\s|$)/);
    }
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix rejects unknown projects, tools, and invalid timeouts", () => {
  for (const [args, message] of [
    [["--dry-run", "--project", "not-a-project"], /Unknown fixture project: not-a-project/],
    [["--dry-run", "--tool", "not-a-tool"], /Unknown fixture tool: not-a-tool/],
    [["--dry-run", "--timeout-ms", "0"], /--timeout-ms must be a positive integer/],
  ] as const) {
    const { outputDir, result } = run([...args]);
    try {
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
      assert.equal(fs.existsSync(path.join(outputDir, "summary.json")), false);
    } finally {
      fs.rmSync(outputDir, { recursive: true, force: true });
    }
  }
});
