import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs, { mkdtempSync, rmSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";
import { toolingProgram } from "./support/program-path.ts";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const commandPath = "tools/support/github/run-many.ts";

function runMany(args: readonly string[], cwd = repoRoot) {
  return spawnSync(process.execPath, [path.join(repoRoot, commandPath), ...args], {
    cwd,
    encoding: "utf8",
  });
}

test("github/run-many executes command groups in order", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "rust-run-many-"));
  const outputPath = path.join(tempDir, "output.txt");

  try {
    const result = runMany(
      [
        "node",
        toolingProgram("process/append-command-marker.js"),
        outputPath,
        "a",
        "--",
        "node",
        toolingProgram("process/append-command-marker.js"),
        outputPath,
        "b",
      ],
      tempDir,
    );

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.readFileSync(outputPath, "utf8"), "ab");
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("github/run-many stops at the first failing command group", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "rust-run-many-fail-"));
  const outputPath = path.join(tempDir, "output.txt");

  try {
    const result = runMany(
      [
        "node",
        toolingProgram("process/append-command-marker.js"),
        outputPath,
        "a",
        "--",
        "node",
        toolingProgram("process/exit-with-status.js"),
        "7",
        "--",
        "node",
        toolingProgram("process/append-command-marker.js"),
        outputPath,
        "b",
      ],
      tempDir,
    );

    assert.equal(result.status, 7, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.readFileSync(outputPath, "utf8"), "a");
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("docs and release workflows run command groups with Node", () => {
  const docsWorkflow = readRepoFile(".github", "workflows", "11-docs-build.yml");
  const releaseWorkflow = readRepoFile(".github", "workflows", "release.yml");
  assert.match(
    workflowJobBody(releaseWorkflow, "build-packages"),
    /uses: \.\/\.github\/actions\/release-build-packages/,
  );

  for (const [workflowName, jobBody, command] of [
    [
      "11-docs-build.yml:build-docs",
      workflowJobBody(docsWorkflow, "build-docs"),
      "node tools/support/github/run-many.ts vp run --filter './npm/native' build:ci",
    ],
    [
      "11-docs-build.yml:build-playground",
      workflowJobBody(docsWorkflow, "build-playground"),
      "node tools/support/github/run-many.ts vp run --filter './npm/native' build:ci",
    ],
    [
      "release.yml:build-packages",
      readRepoFile(".github", "actions", "release-build-packages", "action.yml"),
      "node tools/support/github/run-many.ts vp run --filter './npm/cli' build",
    ],
  ] as const) {
    const setupIndex = jobBody.indexOf("uses: ./.github/actions/setup-rust-script");
    const commandIndex = jobBody.indexOf(command);
    assert.ok(setupIndex >= 0, `${workflowName} still runs rust-script task commands`);
    assert.ok(commandIndex > setupIndex, `${workflowName} must run ${commandPath}`);
  }

  assert.doesNotMatch(docsWorkflow, /tools\/moon\/cmd\/github\/run_many/);
  assert.doesNotMatch(releaseWorkflow, /tools\/moon\/cmd\/github\/run_many/);
});
