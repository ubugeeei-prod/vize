import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs, { mkdtempSync, rmSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const commandPath = "tools/commands/ci/github/run-many.rs";

function runMany(args: readonly string[], cwd = repoRoot) {
  return spawnSync("rust-script", [path.join(repoRoot, commandPath), ...args], {
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
        "-e",
        `require('node:fs').appendFileSync(${JSON.stringify(outputPath)}, 'a')`,
        "--",
        "node",
        "-e",
        `require('node:fs').appendFileSync(${JSON.stringify(outputPath)}, 'b')`,
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
        "-e",
        `require('node:fs').appendFileSync(${JSON.stringify(outputPath)}, 'a')`,
        "--",
        "node",
        "-e",
        "process.exit(7)",
        "--",
        "node",
        "-e",
        `require('node:fs').appendFileSync(${JSON.stringify(outputPath)}, 'b')`,
      ],
      tempDir,
    );

    assert.equal(result.status, 7, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.readFileSync(outputPath, "utf8"), "a");
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("docs and release workflows run command groups with Rust Script", () => {
  const docsWorkflow = readRepoFile(".github", "workflows", "build-docs.yml");
  const releaseWorkflow = readRepoFile(".github", "workflows", "release.yml");

  for (const [workflowName, jobBody, command] of [
    [
      "build-docs.yml:build-docs",
      workflowJobBody(docsWorkflow, "build-docs"),
      "rust-script tools/commands/ci/github/run-many.rs vp run --filter './npm/native' build:ci",
    ],
    [
      "build-docs.yml:build-playground",
      workflowJobBody(docsWorkflow, "build-playground"),
      "rust-script tools/commands/ci/github/run-many.rs vp run --filter './npm/native' build:ci",
    ],
    [
      "release.yml:build-release-packages",
      workflowJobBody(releaseWorkflow, "build-release-packages"),
      "rust-script tools/commands/ci/github/run-many.rs vp run --filter './npm/cli' build",
    ],
  ] as const) {
    const setupIndex = jobBody.indexOf("uses: ./.github/actions/setup-rust-script");
    const commandIndex = jobBody.indexOf(command);
    assert.ok(setupIndex >= 0, `${workflowName} must install rust-script`);
    assert.ok(commandIndex >= 0, `${workflowName} must run ${commandPath}`);
    assert.ok(setupIndex < commandIndex, `${workflowName} must install rust-script first`);
  }

  assert.doesNotMatch(docsWorkflow, /tools\/moon\/cmd\/github\/run_many/);
  assert.doesNotMatch(releaseWorkflow, /tools\/moon\/cmd\/github\/run_many/);
});
