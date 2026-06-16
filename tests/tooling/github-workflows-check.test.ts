import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { buildComment } from "../../bench/comment-test-report.mjs";
import { readRepoFile, root, workflowJobBody } from "./support/github-workflows.ts";

test("PR CI jobs cap runtime with explicit timeouts", () => {
  const checkWorkflow = readRepoFile(".github", "workflows", "check.yml");
  const benchmarkWorkflow = readRepoFile(".github", "workflows", "benchmark.yml");
  const toolBenchmarkWorkflow = readRepoFile(".github", "workflows", "tool-benchmark.yml");

  for (const [jobName, minutes] of [
    ["nix-flake", 60],
    ["fmt-rust", 10],
    ["check-js", 15],
    ["security-audit", 20],
    ["semver-checks", 25],
    ["node-engine-compat", 20],
    ["check-vize-apps", 20],
    ["vue-parity", 30],
    ["test-scripts", 15],
    ["editor-extensions", 15],
    ["build-js-packages", 30],
    ["test-js-packages", 30],
    ["clippy-and-test", 30],
    ["coverage", 30],
    ["source-coverage", 40],
    ["branch-coverage", 45],
    ["playground-test", 30],
    ["test-report", 5],
    ["test-report-comment", 5],
  ] as const) {
    assert.match(
      workflowJobBody(checkWorkflow, jobName),
      new RegExp(`timeout-minutes:\\s*${minutes}\\b`),
    );
  }

  for (const [jobName, minutes] of [
    ["pr-benchmark", 30],
    ["pr-benchmark-budget", 5],
    ["pr-benchmark-comment", 5],
  ] as const) {
    assert.match(
      workflowJobBody(benchmarkWorkflow, jobName),
      new RegExp(`timeout-minutes:\\s*${minutes}\\b`),
    );
  }

  for (const [jobName, minutes] of [
    ["tool-benchmark", 75],
    ["tool-benchmark-comment", 5],
    ["tool-benchmark-commit", 5],
  ] as const) {
    assert.match(
      workflowJobBody(toolBenchmarkWorkflow, jobName),
      new RegExp(`timeout-minutes:\\s*${minutes}\\b`),
    );
  }
});

test("check workflow runs declared Node engine compatibility matrix", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const job = workflowJobBody(workflow, "node-engine-compat");

  assert.match(job, /node-version:\s*\["22", "24"\]/);
  assert.match(job, /echo "\$\{\{\s*matrix\.node-version\s*\}\}" > \.node-version\.ci/);
  assert.match(job, /node-version-file:\s*"\.node-version\.ci"/);
  assert.match(
    job,
    /node --test tests\/tooling\/node-engine-matrix\.test\.ts tests\/tooling\/package-manifests\.test\.ts/,
  );
});

test("check workflow comments a detailed PR test report for each head push", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const reportJob = workflowJobBody(workflow, "test-report");
  const commentJob = workflowJobBody(workflow, "test-report-comment");

  assert.match(
    reportJob,
    /if:\s*\$\{\{\s*always\(\) && github\.event_name == 'pull_request'\s*\}\}/,
  );
  assert.match(reportJob, /contents:\s*read/);
  assert.doesNotMatch(reportJob, /issues:\s*write/);
  assert.doesNotMatch(reportJob, /pull-requests:\s*write/);

  for (const jobName of [
    "nix-flake",
    "fmt-rust",
    "check-js",
    "security-audit",
    "semver-checks",
    "node-engine-compat",
    "check-vize-apps",
    "vue-parity",
    "test-scripts",
    "editor-extensions",
    "build-js-packages",
    "test-js-packages",
    "clippy-and-test",
    "coverage",
    "source-coverage",
    "branch-coverage",
    "playground-test",
  ]) {
    assert.match(reportJob, new RegExp(`- ${jobName}\\b`));
  }

  assert.match(
    reportJob,
    /node bench\/test-inventory\.mjs --json test-inventory\.json --markdown "\$GITHUB_STEP_SUMMARY"/,
  );
  assert.match(reportJob, /name:\s*test-inventory/);

  assert.match(commentJob, /needs:\n\s+- test-report\b/);
  assert.match(commentJob, /actions:\s*read/);
  assert.match(commentJob, /contents:\s*read/);
  assert.match(commentJob, /issues:\s*write/);
  assert.match(commentJob, /pull-requests:\s*write/);
  assert.match(commentJob, /ref:\s*\$\{\{\s*github\.event\.pull_request\.base\.sha\s*\}\}/);
  assert.match(commentJob, /uses:\s*actions\/download-artifact@[0-9a-f]{40}\s*# v8\.0\.1/);
  assert.match(commentJob, /name:\s*test-inventory/);
  assert.match(
    commentJob,
    /TEST_REPORT_COMMENT_KEY:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/,
  );
  assert.match(
    commentJob,
    /TEST_REPORT_HEAD_SHA:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/,
  );
  assert.match(
    commentJob,
    /node bench\/comment-test-report\.mjs --inventory test-inventory\.json --summary "\$GITHUB_STEP_SUMMARY"/,
  );
});

test("test inventory script counts JS, Rust, e2e, VRT, and fixture cases", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-test-inventory-"));
  const inventoryPath = path.join(tempDir, "inventory.json");

  try {
    execFileSync(process.execPath, ["bench/test-inventory.mjs", "--json", inventoryPath], {
      cwd: root,
      stdio: "pipe",
    });

    const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8")) as {
      totalCases: number;
      totalFiles: number;
      areas: Array<{ area: string; files: number; cases: number }>;
      groups: Array<{ area: string; file: string; count: number }>;
    };

    assert.ok(inventory.totalCases > 1_000);
    assert.ok(inventory.totalFiles > 100);

    for (const areaName of ["JS / TS", "Rust", "E2E", "VRT", "Compiler Fixtures"]) {
      const area = inventory.areas.find((candidate) => candidate.area === areaName);
      assert.ok(area, `missing ${areaName} inventory area`);
      assert.ok(area.cases > 0, `${areaName} should have cases`);
    }

    assert.ok(
      inventory.groups.some((group) => group.file === "tests/tooling/github-workflows.test.ts"),
    );
    assert.ok(inventory.groups.some((group) => group.file === "tests/fixtures/vdom/element.pkl"));
    assert.ok(
      inventory.groups.some((group) => group.file === "playground/e2e/vrt/cross-file-ui.spec.ts"),
    );
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("PR test report keeps the test file inventory collapsed with a short toggle", () => {
  const report = buildComment({
    jobs: [
      {
        name: "check-js",
        conclusion: "success",
        started_at: "2026-05-15T00:00:00Z",
        completed_at: "2026-05-15T00:01:00Z",
        html_url: "https://github.com/ubugeeei-prod/vize/actions/runs/1/job/1",
        steps: [],
      },
    ],
    workflowName: "Check",
    runUrl: "https://github.com/ubugeeei-prod/vize/actions/runs/1",
    runId: "1",
    runAttempt: "1",
    sha: "0123456789abcdef",
    inventory: {
      totalCases: 2,
      totalFiles: 1,
      areas: [{ area: "JS / TS", files: 1, cases: 2 }],
      groups: [{ area: "JS / TS", file: "tests/tooling/github-workflows.test.ts", count: 2 }],
    },
  });

  assert.match(report, /Total tracked cases: \*\*2\*\* across \*\*1\*\* files\./);
  assert.match(report, /<details>\n<summary>Files<\/summary>/);
  assert.doesNotMatch(report, /<details open>\n<summary>Test files/);
  assert.doesNotMatch(report, /<summary>Test files \(/);
});

test("check workflow runs JS package unit tests and production dependency audit", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const jsPackageJob = workflowJobBody(workflow, "test-js-packages");
  const auditJob = workflowJobBody(workflow, "security-audit");

  assert.match(jsPackageJob, /vp run --workspace-root test:js/);
  assert.match(jsPackageJob, /key:\s*test-js-packages/);
  assert.match(auditJob, /vp exec pnpm audit --prod --audit-level moderate/);
  assert.match(auditJob, /tool:\s*cargo-audit/);
  assert.match(auditJob, /cargo audit --deny warnings/);
  assert.doesNotMatch(auditJob, /continue-on-error:\s*true/);
});

test("check workflow blocks on Rust source and branch coverage budgets", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const sourceJob = workflowJobBody(workflow, "source-coverage");
  const branchJob = workflowJobBody(workflow, "branch-coverage");

  assert.match(sourceJob, /tool:\s*cargo-llvm-cov/);
  assert.match(sourceJob, /vp install --frozen-lockfile --prefer-offline/);
  assert.match(sourceJob, /vp run --workspace-root coverage:source/);
  assert.match(sourceJob, /source-summary\.json/);
  assert.match(sourceJob, /rust-source-coverage-summary/);

  assert.match(branchJob, /toolchain:\s*nightly/);
  assert.match(branchJob, /tool:\s*cargo-llvm-cov/);
  assert.match(branchJob, /vp install --frozen-lockfile --prefer-offline/);
  assert.match(branchJob, /vp run --workspace-root coverage:source:branch/);
  assert.match(branchJob, /source-branch-summary\.json/);
  assert.match(branchJob, /rust-branch-coverage-summary/);
});

test("check workflow gates Vue parity against official compiler and vue-tsc fixtures", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const job = workflowJobBody(workflow, "vue-parity");

  assert.match(job, /vp install --frozen-lockfile --prefer-offline/);
  assert.match(job, /cargo build --profile ci -p vize/);
  assert.match(job, /vp run --filter '\.\/tests' test:check:fixtures/);
});

test("check workflow only installs Playwright browsers on cache misses", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");

  assert.match(workflow, /- name: Cache Playwright browsers\s+id: cache-playwright/);
  assert.match(
    workflow,
    /- name: Install Playwright browsers\s+if: steps\.cache-playwright\.outputs\.cache-hit != 'true'/,
  );
});

test("check workflow keeps JS checks separate from native and packaging work", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const checkJsJob = workflowJobBody(workflow, "check-js");
  const buildJob = workflowJobBody(workflow, "build-js-packages");
  const playgroundJob = workflowJobBody(workflow, "playground-test");

  assert.match(checkJsJob, /vp run --workspace-root check:ci/);
  assert.doesNotMatch(checkJsJob, /cargo build/);
  assert.match(checkJsJob, /setup-moonbit/);
  assert.doesNotMatch(checkJsJob, /build:packages/);

  assert.match(buildJob, /vp run --filter '\.\/npm\/vize-native' build:ci/);
  assert.match(buildJob, /vp run --workspace-root build:packages/);
  assert.match(buildJob, /name:\s*shared-js-build/);

  assert.match(playgroundJob, /needs:\n\s+- build-js-packages\b/);
  assert.match(playgroundJob, /name:\s*shared-js-build/);
  assert.doesNotMatch(playgroundJob, /name: Build npm packages/);
});

test("check workflow uploads the VRT HTML report when snapshots fail", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");

  assert.match(workflow, /- name: Upload VRT report\s+if: steps\.vrt\.outcome == 'failure'/);
  assert.match(workflow, /name:\s*playground-vrt-report/);
  assert.match(workflow, /path:\s*playground\/playwright-report\//);
  assert.match(workflow, /if-no-files-found:\s*ignore/);
});
