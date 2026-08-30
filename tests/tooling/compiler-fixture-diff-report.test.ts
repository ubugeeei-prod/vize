import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const toolPath = path.join(root, "tools", "commands", "fixtures", "compiler-diff-report.rs");

function writeFakeVizePreload(directory: string): string {
  const preloadPath = path.join(directory, "fake-vize.cjs");
  const report = JSON.stringify({
    files: [],
    summary: {
      fileCount: 0,
      changedFiles: 0,
      additions: 0,
      removals: 0,
      officialErrors: 0,
      vizeErrors: 0,
    },
  });
  fs.writeFileSync(
    preloadPath,
    `const path = require("node:path");
if (path.basename(process.argv[1] ?? "") === "inspector") {
  process.stdout.write(${JSON.stringify(report)});
  process.exit(0);
}
`,
  );
  return preloadPath;
}

function fakeVizeEnv(preloadPath: string): NodeJS.ProcessEnv {
  const preloadOption = `--require=${JSON.stringify(preloadPath)}`;
  return {
    ...process.env,
    NODE_OPTIONS: [process.env.NODE_OPTIONS, preloadOption].filter(Boolean).join(" "),
  };
}

function runCompilerFixtureDiff(vizeBin: string, invocationDirectory = root) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-compiler-diff-bin-"));
  const outputDir = path.join(tempDir, "report");
  const preloadPath = writeFakeVizePreload(tempDir);

  try {
    const result = spawnSync(
      "rust-script",
      [
        toolPath,
        "--project",
        "element-plus",
        "--target",
        "dom",
        "--max-files",
        "1",
        "--output-dir",
        outputDir,
        "--vize-bin",
        vizeBin,
      ],
      { cwd: invocationDirectory, encoding: "utf8", env: fakeVizeEnv(preloadPath) },
    );

    assert.equal(result.status, 0, result.stderr);
    return JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
}

test("compiler fixture diff reporter documents the shared artifact directory", () => {
  const result = spawnSync("rust-script", [toolPath, "--help"], {
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
      "rust-script",
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
      "rust-script",
      [toolPath, "--dry-run", "--project", "not-a-fixture", "--output-dir", outputDir],
      { cwd: root, encoding: "utf8" },
    );

    assert.equal(result.status, 1);
    assert.match(result.stderr, /Unknown fixture project: not-a-fixture/);
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("compiler fixture diff reporter resolves --vize-bin from the invocation directory", () => {
  const binPath = process.execPath;
  const invocationDirectory = path.dirname(binPath);
  const relativeBinPath = `.${path.sep}${path.basename(binPath)}`;
  const summary = runCompilerFixtureDiff(relativeBinPath, invocationDirectory);

  assert.equal(summary.command.vize, binPath);
  assert.equal(summary.projects[0].targets[0].status, "ok");
});

test("compiler fixture diff reporter preserves an absolute --vize-bin path", () => {
  const binPath = process.execPath;
  const summary = runCompilerFixtureDiff(binPath);

  assert.equal(summary.command.vize, binPath);
  assert.equal(summary.projects[0].targets[0].status, "ok");
});
