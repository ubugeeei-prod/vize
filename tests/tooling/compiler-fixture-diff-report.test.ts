import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const toolPath = path.join(root, "tools", "fixtures", "compiler-diff-report.mjs");

test("compiler fixture diff reporter documents the shared artifact directory", () => {
  const result = spawnSync(process.execPath, [toolPath, "--help"], {
    cwd: root,
    encoding: "utf8",
  });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /\.vize\/artifacts\/compiler-diff-report/);
});

test("compiler fixture diff reporter dry-runs selected registry projects", () => {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-compiler-diff-report-"));
  try {
    const result = spawnSync(
      process.execPath,
      [
        toolPath,
        "--dry-run",
        "--project",
        "element-plus,directus",
        "--target",
        "dom",
        "--max-files",
        "2",
        "--output-dir",
        outputDir,
      ],
      { cwd: root, encoding: "utf8" },
    );

    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /summary\.json/);
    assert.match(result.stdout, /summary\.md/);

    const summary = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
    assert.equal(summary.schema, "vize.fixtureCompilerDiffReport");
    assert.equal(summary.summary.projectCount, 2);
    assert.equal(summary.summary.targetCount, 1);
    assert.deepEqual(
      summary.projects.map((project: { id: string }) => project.id),
      ["element-plus", "directus"],
    );
    assert.equal(summary.projects[0].targets[0].status, "planned");
    assert.match(summary.projects[0].targets[0].command, /inspector/);
    assert.match(summary.projects[0].targets[0].command, /--format compare/);
    assert.match(summary.projects[0].targets[0].command, /--max-files 2/);

    const markdown = fs.readFileSync(path.join(outputDir, "summary.md"), "utf8");
    assert.match(markdown, /Vize Fixture Compiler Diff Report/);
    assert.match(markdown, /element-plus/);
    assert.match(markdown, /directus/);
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("compiler fixture diff reporter rejects unknown fixture ids", () => {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-compiler-diff-report-"));
  try {
    const result = spawnSync(
      process.execPath,
      [toolPath, "--dry-run", "--project", "not-a-fixture", "--output-dir", outputDir],
      { cwd: root, encoding: "utf8" },
    );

    assert.equal(result.status, 1);
    assert.match(result.stderr, /Unknown fixture project: not-a-fixture/);
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});
