import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { buildComment } from "../../bench/comment-test-report.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function normalizeRepoText(content: string): string {
  return content.replace(/\r\n?/g, "\n");
}

function readRepoFile(...segments: string[]): string {
  return normalizeRepoText(fs.readFileSync(path.join(root, ...segments), "utf8"));
}

function readGithubYamlFiles(): Array<{ relativePath: string; content: string }> {
  const files: Array<{ relativePath: string; content: string }> = [];
  const visit = (directory: string) => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const fullPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(fullPath);
        continue;
      }
      if (!/\.(ya?ml)$/.test(entry.name)) {
        continue;
      }
      files.push({
        relativePath: path.relative(root, fullPath),
        content: normalizeRepoText(fs.readFileSync(fullPath, "utf8")),
      });
    }
  };
  visit(path.join(root, ".github"));
  return files.sort((left, right) => left.relativePath.localeCompare(right.relativePath));
}

function workflowJobBody(workflow: string, jobName: string): string {
  const jobStart = workflow.indexOf(`\n  ${jobName}:\n`);
  assert.notEqual(jobStart, -1, `missing job ${jobName}`);
  const remaining = workflow.slice(jobStart + 1);
  const nextJobMatch = /\n  [a-z0-9-]+:\n/g.exec(remaining.slice(1));
  return remaining.slice(0, nextJobMatch ? nextJobMatch.index + 1 : undefined);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// Runner pairs (hosted GitHub label, equivalent Blacksmith label) accepted by
// the workflow-shape tests. Letting either form match keeps the cross-platform
// coverage assertion strict on _which_ platforms are present without pinning the
// runner-pool vendor; switching back to GitHub-hosted shouldn't break the test
// and vice versa.
function hostedOrBlacksmith(hostedLabel: string): string {
  if (hostedLabel === "ubuntu-24.04") {
    return "(?:ubuntu-24\\.04|blacksmith-\\d+vcpu-ubuntu-2404)";
  }
  if (hostedLabel === "ubuntu-24.04-arm") {
    return "(?:ubuntu-24\\.04-arm|blacksmith-\\d+vcpu-ubuntu-2404-arm)";
  }
  if (hostedLabel === "macos-15") {
    return "(?:macos-15|blacksmith-(?:6|12)vcpu-macos-15)";
  }
  if (hostedLabel === "windows-2025") {
    return "(?:windows-2025|blacksmith-\\d+vcpu-windows-2025)";
  }
  return escapeRegExp(hostedLabel);
}

test("repo text helpers normalize line endings before workflow matching", () => {
  const workflow = normalizeRepoText("jobs:\r\n  check-js:\r\n    runs-on: ubuntu-24.04\r\n");

  assert.equal(workflowJobBody(workflow, "check-js"), "  check-js:\n    runs-on: ubuntu-24.04\n");
});

test("GitHub workflows opt JavaScript actions into Node 24", () => {
  for (const workflowName of [
    "check.yml",
    "deploy-docs.yml",
    "native-smoke.yml",
    "pkg-pr-new.yml",
    "release.yml",
    "tool-benchmark.yml",
  ]) {
    const workflow = readRepoFile(".github", "workflows", workflowName);
    assert.match(workflow, /FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*true/);
  }
});

test("GitHub workflows use the current cache action", () => {
  for (const relativePath of [
    ".github/actions/setup-rust-sticky-cache/action.yml",
    ".github/actions/setup-moonbit/action.yml",
    ".github/workflows/benchmark.yml",
    ".github/workflows/check.yml",
    ".github/workflows/deploy-docs.yml",
    ".github/workflows/e2e.yml",
    ".github/workflows/native-smoke.yml",
    ".github/workflows/release.yml",
    ".github/workflows/tool-benchmark.yml",
  ]) {
    const file = readRepoFile(...relativePath.split("/"));
    assert.doesNotMatch(file, /uses:\s*actions\/cache@v4/, `${relativePath} still uses cache v4`);
  }
});

test("GitHub workflows declare the expected cross-platform runner matrix", () => {
  // We use Blacksmith-hosted runners where compatible and intentionally let
  // any matching vCPU SKU pass — bumping vCPU shouldn't need a test change.
  // macOS Intel and Windows ARM still use GitHub-hosted runners because
  // Blacksmith does not offer equivalent SKUs.
  //
  // We only validate matrix `host:` / `runner:` values here; `runs-on:` is
  // usually templated (`${{ matrix.settings.host }}` or the conditional
  // `github.event_name == 'pull_request' && 'ubuntu-latest' || 'ubuntu-24.04'`)
  // and is asserted shape-by-shape further down.
  const allowedRunnerPattern =
    /^(?:ubuntu-(?:latest|24\.04)(?:-arm)?|blacksmith-\d+vcpu-ubuntu-2404(?:-arm)?|macos-15(?:-intel)?|blacksmith-(?:6|12)vcpu-macos-15|windows-(?:2025|11-arm)|blacksmith-\d+vcpu-windows-2025)$/;
  const matrixRunnerPattern = /(?:runner|host):\s*([A-Za-z0-9._-]+)/g;
  const violations: string[] = [];

  for (const { relativePath, content } of readGithubYamlFiles()) {
    for (const match of content.matchAll(matrixRunnerPattern)) {
      const label = match[1];
      if (!allowedRunnerPattern.test(label)) {
        violations.push(`${relativePath}: ${label}`);
      }
    }
  }

  assert.deepEqual(violations, []);

  const checkWorkflow = readRepoFile(".github", "workflows", "check.yml");
  const nativeWorkflow = readRepoFile(".github", "workflows", "native-smoke.yml");
  const releasePlatforms = readRepoFile("tools", "github", "release-platforms.mjs");

  // check.yml runs every job on the same Linux runner — we accept either the
  // GitHub-hosted label or any Blacksmith Ubuntu SKU so changing vCPU size
  // (or temporarily reverting to GitHub-hosted) doesn't churn this test.
  assert.match(checkWorkflow, new RegExp(`runs-on:\\s*${hostedOrBlacksmith("ubuntu-24.04")}`));
  assert.match(nativeWorkflow, new RegExp(`runner:\\s*${hostedOrBlacksmith("ubuntu-24.04-arm")}`));
  assert.match(nativeWorkflow, new RegExp(`runner:\\s*${hostedOrBlacksmith("macos-15")}`));
  assert.match(nativeWorkflow, new RegExp(`runner:\\s*${hostedOrBlacksmith("windows-2025")}`));
  assert.match(
    releasePlatforms,
    new RegExp(`host:\\s*"${hostedOrBlacksmith("ubuntu-24.04-arm")}"`),
  );
  assert.match(releasePlatforms, new RegExp(`host:\\s*"${hostedOrBlacksmith("macos-15")}"`));
  assert.match(releasePlatforms, new RegExp(`host:\\s*"${hostedOrBlacksmith("windows-2025")}"`));
});

test("GitHub workflows use Node 24-compatible artifact downloads", () => {
  const violations: string[] = [];

  for (const { relativePath, content } of readGithubYamlFiles()) {
    if (/uses:\s*actions\/download-artifact@[0-9a-f]{40}\s*#\s*v[1-6](?:\b|\.)/.test(content)) {
      violations.push(`${relativePath} still uses a Node 20 artifact downloader`);
    }
  }

  assert.deepEqual(violations, []);
});

test("GitHub workflow actions are pinned by full commit SHA", () => {
  const violations: string[] = [];
  const usePattern = /^(\s*-?\s*uses:\s*)(["']?)([^\s"']+)\2\s*(?:#.*)?$/gm;

  for (const { relativePath, content } of readGithubYamlFiles()) {
    for (const match of content.matchAll(usePattern)) {
      const spec = match[3];
      if (spec.startsWith("./")) {
        continue;
      }
      const atIndex = spec.lastIndexOf("@");
      if (atIndex === -1) {
        violations.push(`${relativePath}: ${spec} has no ref`);
        continue;
      }
      const ref = spec.slice(atIndex + 1);
      if (!/^[0-9a-f]{40}$/.test(ref)) {
        violations.push(`${relativePath}: ${spec} is not pinned to a full SHA`);
      }
    }
  }

  assert.deepEqual(violations, []);
});

test("App E2E workflow keeps Blacksmith Testbox dispatch hydration separate", () => {
  const workflow = readRepoFile(".github", "workflows", "e2e.yml");
  const job = workflowJobBody(workflow, "testbox");
  const appJob = workflowJobBody(workflow, "app-e2e");

  assert.match(workflow, /\n  workflow_dispatch:\n/);
  assert.match(workflow, /testbox_id:\n\s+description:\s*Blacksmith Testbox session ID/);
  assert.match(
    job,
    /if:\s*\$\{\{\s*github\.event_name == 'workflow_dispatch' && inputs\.testbox_id != ''\s*\}\}/,
  );
  assert.match(job, /uses:\s*useblacksmith\/begin-testbox@[0-9a-f]{40}\s*# v2/);
  assert.match(job, /testbox_id:\s*\$\{\{\s*inputs\.testbox_id\s*\}\}/);
  assert.match(job, /uses:\s*useblacksmith\/run-testbox@[0-9a-f]{40}\s*# v2/);
  assert.doesNotMatch(job, /vp run --workspace-root test|cargo test --workspace/);
  assert.match(
    appJob,
    /if:\s*\$\{\{\s*github\.event_name != 'workflow_dispatch' \|\| inputs\.testbox_id == ''\s*\}\}/,
  );
});

test("PR CI jobs cap runtime with explicit timeouts", () => {
  const checkWorkflow = readRepoFile(".github", "workflows", "check.yml");
  const benchmarkWorkflow = readRepoFile(".github", "workflows", "benchmark.yml");
  const toolBenchmarkWorkflow = readRepoFile(".github", "workflows", "tool-benchmark.yml");

  for (const [jobName, minutes] of [
    ["nix-flake", 30],
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
    ["coverage", 10],
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

test("release workflow explicitly installs matrix Rust targets", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  for (const jobName of ["build-cli", "build-native-all"]) {
    const job = workflowJobBody(workflow, jobName);
    const setupRust = job.indexOf("name: Setup Rust");
    const installTarget = job.indexOf("name: Install Rust target");
    const cacheRust = job.indexOf("name: Cache Rust");

    assert.notEqual(setupRust, -1, `${jobName} is missing Setup Rust`);
    assert.notEqual(installTarget, -1, `${jobName} is missing Install Rust target`);
    assert.notEqual(cacheRust, -1, `${jobName} is missing Cache Rust`);
    assert.ok(
      setupRust < installTarget && installTarget < cacheRust,
      `${jobName} must install the matrix Rust target before caching/building`,
    );
    assert.match(
      job,
      /run:\s*rustup target add \$\{\{\s*matrix\.settings\.target\s*\}\}/,
      `${jobName} must install the matrix Rust target explicitly`,
    );
  }
});

test("release workflow plans slow platform cadence before building", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const planJob = workflowJobBody(workflow, "plan-release-platforms");
  const buildCliJob = workflowJobBody(workflow, "build-cli");
  const buildNativeJob = workflowJobBody(workflow, "build-native-all");

  assert.match(planJob, /node tools\/github\/release-platforms\.mjs github-output/);
  assert.match(buildCliJob, /needs:\s*plan-release-platforms/);
  assert.match(
    buildCliJob,
    /settings:\s*\$\{\{\s*fromJSON\(needs\.plan-release-platforms\.outputs\.cli_matrix\)\s*\}\}/,
  );
  assert.match(buildNativeJob, /needs:\s*plan-release-platforms/);
  assert.match(
    buildNativeJob,
    /settings:\s*\$\{\{\s*fromJSON\(needs\.plan-release-platforms\.outputs\.native_matrix\)\s*\}\}/,
  );
  assert.match(workflow, /release-platforms\.mjs apply-cadence/);
});

test("Linux Rust CI installs Wild linker before cargo builds", () => {
  for (const workflowName of [
    "benchmark.yml",
    "check.yml",
    "deploy-docs.yml",
    "e2e.yml",
    "native-smoke.yml",
    "release.yml",
    "tool-benchmark.yml",
  ]) {
    const workflow = readRepoFile(".github", "workflows", workflowName);
    assert.match(
      workflow,
      /uses:\s*wild-linker\/action@[0-9a-f]{40}\s*# v0\.9\.0/,
      `${workflowName} should install the pinned Wild linker action`,
    );
    assert.match(workflow, /wild-version:\s*"0\.9\.0"/);
  }
});

test("Blacksmith Rust CI uses sticky disks for Cargo and target caches", () => {
  const action = readRepoFile(".github", "actions", "setup-rust-sticky-cache", "action.yml");

  assert.match(action, /uses:\s*useblacksmith\/stickydisk@[0-9a-f]{40}\s*# v1/);
  assert.match(action, /path:\s*~\/\.cargo\/registry/);
  assert.match(action, /path:\s*~\/\.cargo\/git/);
  assert.match(action, /path:\s*\$\{\{\s*inputs\.target-path\s*\}\}/);
  assert.match(action, /secondary-target-path/);

  for (const workflowName of [
    "check.yml",
    "deploy-docs.yml",
    "e2e.yml",
    "fuzz.yml",
    "tool-benchmark.yml",
  ]) {
    const workflow = readRepoFile(".github", "workflows", workflowName);
    assert.match(
      workflow,
      /uses:\s*\.\/\.github\/actions\/setup-rust-sticky-cache/,
      `${workflowName} should mount Blacksmith sticky disks for Rust work`,
    );
    assert.doesNotMatch(
      workflow,
      /Swatinem\/rust-cache/,
      `${workflowName} should avoid network Rust cache on Linux-only Blacksmith jobs`,
    );
  }

  const benchmarkWorkflow = readRepoFile(".github", "workflows", "benchmark.yml");
  assert.match(benchmarkWorkflow, /uses:\s*\.\/head\/\.github\/actions\/setup-rust-sticky-cache/);
  assert.match(benchmarkWorkflow, /target-path:\s*head\/target/);
  assert.match(benchmarkWorkflow, /secondary-target-path:\s*base\/target/);
  assert.doesNotMatch(benchmarkWorkflow, /Swatinem\/rust-cache/);

  for (const workflowName of ["native-smoke.yml", "release.yml"]) {
    const workflow = readRepoFile(".github", "workflows", workflowName);
    assert.match(
      workflow,
      /(?:if:\s*runner\.os == 'Linux'[\s\S]{0,240}setup-rust-sticky-cache|setup-rust-sticky-cache[\s\S]{0,240}if:\s*runner\.os == 'Linux')/,
    );
    assert.match(
      workflow,
      /(?:if:\s*runner\.os != 'Linux'[\s\S]{0,240}Swatinem\/rust-cache|Swatinem\/rust-cache[\s\S]{0,240}if:\s*runner\.os != 'Linux')/,
    );
  }
});

test("release workflow jobs cap runtime with explicit timeouts", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  for (const [jobName, minutes] of [
    ["plan-release-platforms", 5],
    ["build-cli", 90],
    ["build-editor-extensions", 30],
    ["release-vscode-extension", 15],
    ["build-release-packages", 45],
    ["build-wasm-package", 30],
    ["build-native-all", 90],
    ["smoke-release-packages", 30],
    ["release-npm-native", 30],
    ["release-npm-fresco-native", 20],
    ["release-npm-wasm", 30],
    ["release-npm-vite-plugin", 15],
    ["release-npm-oxlint-plugin", 15],
    ["release-npm-unplugin", 15],
    ["release-npm-fresco", 15],
    ["release-npm-musea-mcp-server", 15],
    ["release-npm-vite-plugin-musea", 15],
    ["release-npm-rspack-plugin", 15],
    ["release-npm-musea-nuxt", 15],
    ["release-npm-nuxt", 15],
    ["release-crates", 30],
    ["create-github-release", 20],
    ["release-npm-cli", 15],
  ] as const) {
    assert.match(
      workflowJobBody(workflow, jobName),
      new RegExp(`timeout-minutes:\\s*${minutes}\\b`),
      `${jobName} should have a ${minutes} minute timeout`,
    );
  }
});

test("release workflow smoke installs npm tarballs before publishing", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const smokeJob = workflowJobBody(workflow, "smoke-release-packages");

  assert.match(
    smokeJob,
    /needs:\s*\[plan-release-platforms, build-release-packages, build-native-all\]/,
  );
  assert.match(smokeJob, /name:\s*Smoke release npm package installs/);
  assert.match(smokeJob, /name:\s*Apply slow platform release cadence/);
  assert.match(smokeJob, /name:\s*Prepare native package tarballs/);
  assert.match(smokeJob, /name:\s*Prepare Fresco native package tarball/);
  assert.match(
    smokeJob,
    /node tools\/npm\/smoke-release-install\.mjs --prepare-manifests --runtime-checks/,
  );

  for (const packageDir of [
    "npm/vize-native",
    "npm/fresco-native",
    "npm/vize",
    "npm/vite-plugin-vize",
    "npm/oxlint-plugin-vize",
    "npm/unplugin-vize",
    "npm/fresco",
    "npm/musea-mcp-server",
    "npm/vite-plugin-musea",
    "npm/rspack-vize-plugin",
    "npm/musea-nuxt",
    "npm/nuxt",
  ]) {
    assert.match(smokeJob, new RegExp(packageDir.replaceAll("/", "\\/")));
  }

  for (const jobName of [
    "release-npm-cli",
    "release-npm-vite-plugin",
    "release-npm-oxlint-plugin",
    "release-npm-unplugin",
    "release-npm-fresco",
    "release-npm-musea-mcp-server",
    "release-npm-vite-plugin-musea",
    "release-npm-rspack-plugin",
    "release-npm-musea-nuxt",
    "release-npm-nuxt",
  ]) {
    assert.match(workflowJobBody(workflow, jobName), /smoke-release-packages/);
  }

  for (const [jobName, smokeStep, publishStep] of [
    [
      "release-npm-native",
      "name: Smoke install native package tarballs",
      "name: Publish platform packages",
    ],
    [
      "release-npm-fresco-native",
      "name: Smoke install Fresco native package tarball",
      "name: Publish",
    ],
    ["release-npm-wasm", "name: Smoke install WASM package tarball", "name: Publish @vizejs/wasm"],
  ] as const) {
    const job = workflowJobBody(workflow, jobName);
    const smokeIndex = job.indexOf(smokeStep);
    const publishIndex = job.indexOf(publishStep);
    assert.notEqual(smokeIndex, -1, `${jobName} is missing ${smokeStep}`);
    assert.notEqual(publishIndex, -1, `${jobName} is missing ${publishStep}`);
    assert.ok(smokeIndex < publishIndex, `${jobName} must smoke install before publishing`);
    if (jobName === "release-npm-native") {
      assert.match(job, /smoke-release-install\.mjs --prepare-manifests --runtime-checks/);
    }
  }
});

test("benchmark workflow comments from trusted code after a read-only benchmark run", () => {
  const workflow = readRepoFile(".github", "workflows", "benchmark.yml");
  const benchmarkJob = workflowJobBody(workflow, "pr-benchmark");
  const budgetJob = workflowJobBody(workflow, "pr-benchmark-budget");
  const commentJob = workflowJobBody(workflow, "pr-benchmark-comment");

  assert.match(benchmarkJob, /contents:\s*read/);
  assert.doesNotMatch(benchmarkJob, /issues:\s*write/);
  assert.doesNotMatch(benchmarkJob, /pull-requests:\s*write/);
  assert.match(
    benchmarkJob,
    /path:\s*head[\s\S]*ref:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/,
  );
  assert.match(
    benchmarkJob,
    /path:\s*base[\s\S]*ref:\s*\$\{\{\s*github\.event\.pull_request\.base\.sha\s*\}\}/,
  );
  assert.match(benchmarkJob, /name:\s*pr-benchmark/);
  assert.doesNotMatch(benchmarkJob, /node base\/bench\/comment-pr\.mjs/);
  assert.doesNotMatch(benchmarkJob, /node bench\/comment-pr\.mjs/);
  assert.match(benchmarkJob, /--threshold "\$VIZE_BENCH_REGRESSION_THRESHOLD_PERCENT"/);

  assert.match(budgetJob, /needs:\n\s+- pr-benchmark\b/);
  assert.match(budgetJob, /actions:\s*read/);
  assert.match(budgetJob, /contents:\s*read/);
  assert.match(budgetJob, /issues:\s*read/);
  assert.doesNotMatch(budgetJob, /issues:\s*write/);
  assert.doesNotMatch(budgetJob, /pull-requests:\s*write/);
  assert.match(
    budgetJob,
    /path:\s*head[\s\S]*ref:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/,
  );
  assert.match(budgetJob, /uses:\s*actions\/download-artifact@[0-9a-f]{40}\s*# v8\.0\.1/);
  assert.match(budgetJob, /name:\s*pr-benchmark/);
  assert.match(budgetJob, /name:\s*Read current PR labels/);
  assert.match(budgetJob, /GITHUB_TOKEN:\s*\$\{\{\s*github\.token\s*\}\}/);
  assert.match(budgetJob, /process\.env\.GITHUB_API_URL \?\? "https:\/\/api\.github\.com"/);
  assert.match(budgetJob, /labels\.map\(\(label\) => label\.name\)/);
  assert.match(
    budgetJob,
    /node head\/bench\/enforce-pr-budget\.mjs[\s\S]*--json benchmark-results\.json[\s\S]*--labels-json "\$PR_LABELS_JSON"/,
  );

  assert.match(commentJob, /needs:\n\s+- pr-benchmark\b/);
  assert.match(commentJob, /actions:\s*read/);
  assert.match(commentJob, /contents:\s*read/);
  assert.match(commentJob, /issues:\s*write/);
  assert.match(commentJob, /pull-requests:\s*write/);
  assert.match(commentJob, /ref:\s*\$\{\{\s*github\.event\.pull_request\.base\.sha\s*\}\}/);
  assert.match(commentJob, /uses:\s*actions\/download-artifact@[0-9a-f]{40}\s*# v8\.0\.1/);
  assert.match(commentJob, /name:\s*pr-benchmark/);
  assert.match(
    commentJob,
    /BENCHMARK_COMMENT_KEY:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/,
  );
  assert.match(
    commentJob,
    /node bench\/comment-pr\.mjs --body benchmark-summary\.md --comment-key "\$BENCHMARK_COMMENT_KEY"/,
  );
});

test("tool benchmark workflow produces docs artifacts, PR comments, and conventional commits", () => {
  const workflow = readRepoFile(".github", "workflows", "tool-benchmark.yml");
  const benchmarkJob = workflowJobBody(workflow, "tool-benchmark");
  const commentJob = workflowJobBody(workflow, "tool-benchmark-comment");
  const commitJob = workflowJobBody(workflow, "tool-benchmark-commit");

  assert.match(workflow, /\n  workflow_dispatch:\n/);
  assert.match(workflow, /commit_results:[\s\S]*type:\s*boolean[\s\S]*default:\s*false/);
  assert.match(workflow, /VIZE_TOOL_BENCH_FILE_COUNT:/);
  assert.match(workflow, /VIZE_TOOL_BENCH_NUXT_FILE_COUNT:/);
  assert.match(workflow, /VIZE_TOOL_BENCH_LARGE_BLOCKS:/);
  assert.match(benchmarkJob, /runs-on:\s*blacksmith-32vcpu-ubuntu-2404/);
  assert.match(benchmarkJob, /contents:\s*read/);
  assert.doesNotMatch(benchmarkJob, /contents:\s*write/);
  assert.doesNotMatch(benchmarkJob, /issues:\s*write/);
  assert.match(benchmarkJob, /uses:\s*\.\/\.github\/actions\/setup-moonbit/);
  assert.match(benchmarkJob, /vp run --workspace-root build:native/);
  assert.match(benchmarkJob, /vp run --workspace-root build:vite-plugin/);
  assert.match(benchmarkJob, /vp run --workspace-root build:nuxt-stack/);
  assert.match(benchmarkJob, /node bench\/generate\.mjs "\$VIZE_TOOL_BENCH_FILE_COUNT"/);
  assert.match(benchmarkJob, /node bench\/compare-tools\.mjs/);
  assert.match(benchmarkJob, /--nuxt-file-count "\$VIZE_TOOL_BENCH_NUXT_FILE_COUNT"/);
  assert.match(benchmarkJob, /--large-blocks "\$VIZE_TOOL_BENCH_LARGE_BLOCKS"/);
  assert.match(benchmarkJob, /--runner-label "blacksmith-32vcpu-ubuntu-2404"/);
  assert.match(benchmarkJob, /--doc performance-blacksmith\.md/);
  assert.match(benchmarkJob, /name:\s*tool-benchmark/);
  assert.match(benchmarkJob, /tool-benchmark-results\.json/);

  assert.match(
    commentJob,
    /if:\s*\$\{\{\s*github\.event_name == 'pull_request' && github\.event\.pull_request\.head\.repo\.full_name == github\.repository\s*\}\}/,
  );
  assert.match(commentJob, /contents:\s*read/);
  assert.match(commentJob, /issues:\s*write/);
  assert.match(commentJob, /pull-requests:\s*write/);
  assert.match(commentJob, /ref:\s*\$\{\{\s*github\.event\.pull_request\.base\.sha\s*\}\}/);
  assert.match(commentJob, /name:\s*tool-benchmark/);
  assert.match(
    commentJob,
    /BENCHMARK_COMMENT_KEY:\s*tool-\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/,
  );
  assert.match(
    commentJob,
    /node bench\/comment-pr\.mjs --body tool-benchmark-summary\.md --comment-key "\$BENCHMARK_COMMENT_KEY"/,
  );

  assert.match(
    commitJob,
    /if:\s*\$\{\{\s*github\.event_name == 'workflow_dispatch' && inputs\.commit_results && startsWith\(github\.ref, 'refs\/heads\/'\)\s*\}\}/,
  );
  assert.match(commitJob, /contents:\s*write/);
  assert.match(commitJob, /docs\/content\/architecture\/performance-blacksmith\.md/);
  assert.match(commitJob, /bench\/results\/tool-benchmark-latest\.json/);
  assert.match(commitJob, /git commit -m "docs: update blacksmith benchmark snapshot"/);
  assert.match(commitJob, /git push origin HEAD:\$\{\{\s*github\.ref_name\s*\}\}/);
  assert.doesNotMatch(commitJob, /codex/i);
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

test("deploy-docs deploy job installs MoonBit before running script-mode helpers", () => {
  const workflow = readRepoFile(".github", "workflows", "deploy-docs.yml");
  const deployJob = workflow.slice(workflow.indexOf("\n  deploy:\n"));
  const setupIndex = deployJob.indexOf("- uses: ./.github/actions/setup-moonbit");
  const moonRunIndex = deployJob.indexOf(
    "run: moon run --target native - -- < tools/moon/scripts/github/create_site_structure.mbtx",
  );

  assert.notEqual(setupIndex, -1);
  assert.notEqual(moonRunIndex, -1);
  assert.ok(setupIndex < moonRunIndex);
});

test("deploy-docs deploy job keeps a full checkout so local actions and scripts remain available", () => {
  const workflow = readRepoFile(".github", "workflows", "deploy-docs.yml");
  const deployJob = workflow.slice(workflow.indexOf("\n  deploy:\n"));

  assert.match(deployJob, /- uses: actions\/checkout@[0-9a-f]{40}\s*# v6/);
  assert.doesNotMatch(deployJob, /sparse-checkout:/);
});

test("WASM build jobs install MoonBit before invoking moon run", () => {
  const cases = [
    {
      workflowName: "check.yml",
      jobName: "playground-test",
      moonRun:
        "run: moon run --target native - -- playground/src/wasm < tools/moon/scripts/github/build_vitrine_wasm.mbtx",
    },
    {
      workflowName: "deploy-docs.yml",
      jobName: "build-playground",
      moonRun:
        "run: moon run --target native - -- npm/vize-wasm playground/src/wasm < tools/moon/scripts/github/build_vitrine_wasm.mbtx",
    },
    {
      workflowName: "release.yml",
      jobName: "build-wasm-package",
      moonRun:
        "run: moon run --target native - -- < tools/moon/scripts/build_vize_wasm_package.mbtx",
    },
  ] as const;

  for (const { workflowName, jobName, moonRun } of cases) {
    const workflow = readRepoFile(".github", "workflows", workflowName);
    const jobStart = workflow.indexOf(`\n  ${jobName}:\n`);
    const remaining = workflow.slice(jobStart + 1);
    const nextJobMatch = /\n  [a-z0-9-]+:\n/g.exec(remaining.slice(1));
    const jobBody = remaining.slice(0, nextJobMatch ? nextJobMatch.index + 1 : undefined);
    const setupIndex = jobBody.indexOf("- uses: ./.github/actions/setup-moonbit");
    const moonRunIndex = jobBody.indexOf(moonRun);

    assert.notEqual(setupIndex, -1, `${workflowName}:${jobName} is missing setup-moonbit`);
    assert.notEqual(moonRunIndex, -1, `${workflowName}:${jobName} is missing the wasm build step`);
    assert.ok(
      setupIndex < moonRunIndex,
      `${workflowName}:${jobName} runs moon before setup-moonbit`,
    );
  }
});

test("setup-moonbit defines explicit Windows and Unix execution paths", () => {
  const action = readRepoFile(".github", "actions", "setup-moonbit", "action.yml");

  assert.match(action, /Cache MoonBit toolchain/);
  assert.match(action, /uses: actions\/cache@[0-9a-f]{40}\s*# v5/);
  assert.match(action, /Setup MSVC toolchain \(Windows\)/);
  assert.match(action, /uses: ilammy\/msvc-dev-cmd@[0-9a-f]{40}\s*# v1/);
  assert.match(action, /Install MoonBit \(Windows\)/);
  assert.match(action, /if: runner\.os == 'Windows'/);
  assert.match(action, /shell: pwsh/);
  assert.match(action, /Install MoonBit \(Unix\)/);
  assert.match(action, /if: runner\.os != 'Windows'/);
  assert.match(action, /shell: bash/);
});

test("setup-moonbit smoke test validates the native async process runtime", () => {
  const installer = readRepoFile(".github", "actions", "setup-moonbit", "install-moonbit.mjs");

  assert.match(installer, /function hasExistingMoonInstall\(\)/);
  assert.match(installer, /\["run", "-q", "--target", "native", "-", "--"\]/);
  assert.match(installer, /"moonbitlang\/async@0\.19\.0\/process"/);
  assert.match(installer, /@process\.run/);
});

test("setup-moonbit patches Darwin secure memcpy macros before smoke testing", () => {
  const installer = readRepoFile(".github", "actions", "setup-moonbit", "install-moonbit.mjs");

  assert.match(installer, /function patchDarwinMoonbitHeader\(\)/);
  assert.match(installer, /os\.type\(\) !== "Darwin"/);
  assert.match(installer, /#undef memcpy/);
  assert.match(installer, /patchDarwinMoonbitHeader\(\);\nsmokeTestMoon\(\);/);
});

test("setup-moonbit writes both command and shell shims on Windows so bash steps can resolve moon", () => {
  const installer = readRepoFile(".github", "actions", "setup-moonbit", "install-moonbit.mjs");

  assert.match(installer, /const shimMoonCmd = path\.join\(shimDir, "moon\.cmd"\);/);
  assert.match(installer, /const shimMoonShell = path\.join\(shimDir, "moon"\);/);
  assert.match(installer, /fs\.writeFileSync\(\s*shimMoonCmd,/);
  assert.match(installer, /fs\.writeFileSync\(\s*shimMoonShell,/);
});

test("release workflow does not pin a separate hard-coded Node version for VS Code publishing", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.doesNotMatch(workflow, /node-version:\s*"24\.14\.0"/);
  assert.match(workflow, /node-version-file:\s*"\.node-version"/);
});

test("release workflow overwrites existing GitHub release assets when a tag is re-driven", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.match(
    workflow,
    /uses: softprops\/action-gh-release@[0-9a-f]{40}\s*# v2[\s\S]*overwrite_files:\s*true/,
  );
});

test("release workflow publishes npm packages through Trusted Publishing only", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.doesNotMatch(workflow, /secrets\.NPM_TOKEN/);
  assert.doesNotMatch(workflow, /NPM_TOKEN/);
  assert.doesNotMatch(workflow, /configure_npm_auth/);

  const npmPublishJobs = [
    "release-npm-native",
    "release-npm-fresco-native",
    "release-npm-wasm",
    "release-npm-vite-plugin",
    "release-npm-oxlint-plugin",
    "release-npm-unplugin",
    "release-npm-fresco",
    "release-npm-musea-mcp-server",
    "release-npm-vite-plugin-musea",
    "release-npm-rspack-plugin",
    "release-npm-musea-nuxt",
    "release-npm-nuxt",
    "release-npm-cli",
  ];

  for (const jobName of npmPublishJobs) {
    const job = workflowJobBody(workflow, jobName);
    assert.match(job, /runs-on:\s*ubuntu-24\.04\b/);
    assert.doesNotMatch(job, /runs-on:\s*blacksmith-/);
    assert.match(job, /environment:\s*npm/);
    assert.match(job, /id-token:\s*write/);
    assert.match(job, /--provenance/);
    assert.doesNotMatch(job, /NODE_AUTH_TOKEN|_authToken/);
  }
});

test("release workflow publishes npm packages from package-specific artifacts", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.doesNotMatch(workflow, /name:\s*release-npm-packages/);

  for (const artifactName of [
    "release-package-vize",
    "release-package-vite-plugin-vize",
    "release-package-oxlint-plugin-vize",
    "release-package-unplugin-vize",
    "release-package-fresco",
    "release-package-musea-mcp-server",
    "release-package-vite-plugin-musea",
    "release-package-rspack-vize-plugin",
    "release-package-musea-nuxt",
    "release-package-nuxt",
    "release-package-vize-wasm",
  ]) {
    assert.match(workflow, new RegExp(`name:\\s*${artifactName}`));
  }

  const downloadTargets = [
    ["release-npm-wasm", "release-package-vize-wasm", "npm/vize-wasm"],
    ["release-npm-vite-plugin", "release-package-vite-plugin-vize", "npm/vite-plugin-vize"],
    ["release-npm-oxlint-plugin", "release-package-oxlint-plugin-vize", "npm/oxlint-plugin-vize"],
    ["release-npm-unplugin", "release-package-unplugin-vize", "npm/unplugin-vize"],
    ["release-npm-fresco", "release-package-fresco", "npm/fresco"],
    ["release-npm-musea-mcp-server", "release-package-musea-mcp-server", "npm/musea-mcp-server"],
    ["release-npm-vite-plugin-musea", "release-package-vite-plugin-musea", "npm/vite-plugin-musea"],
    ["release-npm-rspack-plugin", "release-package-rspack-vize-plugin", "npm/rspack-vize-plugin"],
    ["release-npm-musea-nuxt", "release-package-musea-nuxt", "npm/musea-nuxt"],
    ["release-npm-nuxt", "release-package-nuxt", "npm/nuxt"],
    ["release-npm-cli", "release-package-vize", "npm/vize"],
  ] as const;

  for (const [jobName, artifactName, downloadPath] of downloadTargets) {
    const jobStart = workflow.indexOf(`\n  ${jobName}:\n`);
    assert.notEqual(jobStart, -1, `missing job ${jobName}`);
    const remaining = workflow.slice(jobStart + 1);
    const nextJobMatch = /\n  [a-z0-9-]+:\n/g.exec(remaining.slice(1));
    const jobBody = remaining.slice(0, nextJobMatch ? nextJobMatch.index + 1 : undefined);

    assert.match(jobBody, new RegExp(`name:\\s*${artifactName}`));
    assert.match(jobBody, new RegExp(`path:\\s*${downloadPath.replace("/", "\\/")}`));
  }
});

test("release workflow smokes the wasm package wrapper before publishing", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const buildJob = workflowJobBody(workflow, "build-wasm-package");
  const publishJob = workflowJobBody(workflow, "release-npm-wasm");

  assert.match(buildJob, /runs-on:\s*blacksmith-\d+vcpu-ubuntu-2404/);
  assert.match(buildJob, /npm\/vize-wasm\/index\.js/);
  assert.match(buildJob, /npm\/vize-wasm\/index\.d\.ts/);
  assert.match(buildJob, /tools\/moon\/scripts\/build_vize_wasm_package\.mbtx/);
  assert.match(buildJob, /name:\s*release-package-vize-wasm/);
  assert.match(publishJob, /needs:\s*build-wasm-package/);
  assert.match(publishJob, /name:\s*release-package-vize-wasm/);
  assert.match(publishJob, /path:\s*npm\/vize-wasm/);

  const setupNode = publishJob.indexOf("name: Setup Vite+ and Node.js");
  const download = publishJob.indexOf("name: Download prebuilt WASM package");
  const smoke = publishJob.indexOf("name: Smoke @vizejs/wasm package");
  const publish = publishJob.indexOf("name: Publish @vizejs/wasm");

  assert.notEqual(setupNode, -1);
  assert.notEqual(download, -1);
  assert.notEqual(smoke, -1);
  assert.notEqual(publish, -1);
  assert.ok(setupNode < download && download < smoke && smoke < publish);
  assert.match(publishJob, /node tools\/npm\/smoke-wasm-package\.mjs npm\/vize-wasm/);
});

test("release workflow creates GitHub Releases only after registry publishing succeeds", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const releaseJob = workflowJobBody(workflow, "create-github-release");

  for (const requiredNeed of [
    "build-cli",
    "release-vscode-extension",
    "release-npm-native",
    "release-npm-fresco-native",
    "release-npm-wasm",
    "smoke-release-packages",
    "release-npm-cli",
    "release-npm-vite-plugin",
    "release-npm-oxlint-plugin",
    "release-npm-unplugin",
    "release-npm-fresco",
    "release-npm-musea-mcp-server",
    "release-npm-vite-plugin-musea",
    "release-npm-rspack-plugin",
    "release-npm-musea-nuxt",
    "release-npm-nuxt",
    "release-crates",
  ]) {
    assert.match(releaseJob, new RegExp(`- ${requiredNeed}\\b`));
  }

  const createRelease = releaseJob.indexOf("name: Create Release");
  assert.notEqual(createRelease, -1);
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

test("native smoke workflow covers host platforms before release tags", () => {
  const workflow = readRepoFile(".github", "workflows", "native-smoke.yml");
  const job = workflowJobBody(workflow, "host-native-smoke");

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /schedule:/);
  assert.doesNotMatch(workflow, /pull_request:/);
  assert.match(
    workflow,
    /Full native\/fresh-install smoke is release evidence, not a per-push gate/,
  );
  for (const [runner, target] of [
    [hostedOrBlacksmith("ubuntu-24.04"), "linux-x64-gnu"],
    [hostedOrBlacksmith("ubuntu-24.04-arm"), "linux-arm64-gnu"],
    ["macos-15-intel", "darwin-x64"],
    [hostedOrBlacksmith("macos-15"), "darwin-arm64"],
    [hostedOrBlacksmith("windows-2025"), "win32-x64-msvc"],
    ["windows-11-arm", "win32-arm64-msvc"],
  ] as const) {
    assert.match(job, new RegExp(`runner:\\s*${runner}[\\s\\S]*target:\\s*${target}`));
  }
  assert.match(job, /cargo build --profile ci -p vize/);
  assert.match(job, /vp run --filter '\.\/npm\/vize-native' build:ci/);
  assert.match(job, /require\('\.\/npm\/vize-native'\)/);
  assert.match(job, /smoke-release-install\.mjs --prepare-manifests npm\/vize-native/);
});

test("native smoke workflow fresh-installs runtime tarballs across supported targets", () => {
  const workflow = readRepoFile(".github", "workflows", "native-smoke.yml");
  const job = workflowJobBody(workflow, "fresh-install-smoke");

  for (const [runner, target] of [
    [hostedOrBlacksmith("ubuntu-24.04"), "linux-x64-gnu"],
    [hostedOrBlacksmith("ubuntu-24.04-arm"), "linux-arm64-gnu"],
    ["macos-15-intel", "darwin-x64"],
    [hostedOrBlacksmith("macos-15"), "darwin-arm64"],
    [hostedOrBlacksmith("windows-2025"), "win32-x64-msvc"],
    ["windows-11-arm", "win32-arm64-msvc"],
  ] as const) {
    assert.match(job, new RegExp(`runner:\\s*${runner}[\\s\\S]*target:\\s*${target}`));
  }
  assert.match(job, /node-version:\s*\["22", "24"\]/);
  assert.match(job, /echo "\$\{\{\s*matrix\.node-version\s*\}\}" > \.node-version\.ci/);
  assert.match(job, /node-version-file:\s*"\.node-version\.ci"/);
  assert.match(job, /vp exec napi create-npm-dirs/);
  assert.match(job, /vp exec napi pre-publish -t npm --no-gh-release --skip-optional-publish/);
  assert.match(
    job,
    /smoke-release-install\.mjs --prepare-manifests --runtime-checks[\s\S]*npm\/vize-native npm\/vize-native\/npm\/\*[\s\S]*npm\/vize npm\/vite-plugin-vize/,
  );
});

test("release workflow builds native targets on MoonBit-supported runners", () => {
  const releasePlatforms = readRepoFile("tools", "github", "release-platforms.mjs");

  assert.doesNotMatch(
    releasePlatforms,
    /host:\s*"macos-15-intel"[\s\S]*target:\s*"x86_64-apple-darwin"/,
    "MoonBit native scripts cannot run on macOS Intel runners",
  );

  for (const [host, target] of [
    [hostedOrBlacksmith("macos-15"), "x86_64-apple-darwin"],
    [hostedOrBlacksmith("macos-15"), "aarch64-apple-darwin"],
    [hostedOrBlacksmith("ubuntu-24.04"), "x86_64-unknown-linux-gnu"],
    [hostedOrBlacksmith("ubuntu-24.04-arm"), "aarch64-unknown-linux-gnu"],
    [hostedOrBlacksmith("windows-2025"), "x86_64-pc-windows-msvc"],
    ["windows-11-arm", "aarch64-pc-windows-msvc"],
  ] as const) {
    assert.match(releasePlatforms, new RegExp(`host:\\s*"${host}"[\\s\\S]*target:\\s*"${target}"`));
  }
});

test("release workflow keeps the Windows ARM64 CLI cross build on a compatible hosted runner", () => {
  const releasePlatforms = readRepoFile("tools", "github", "release-platforms.mjs");

  assert.match(
    releasePlatforms,
    /host:\s*"windows-2025",\n\s*target:\s*"aarch64-pc-windows-msvc"/,
    "Blacksmith Windows x64 images expose x64 MSVC SDK libs after setup-moonbit, which breaks ARM64 linking",
  );
  assert.doesNotMatch(
    releasePlatforms,
    /host:\s*"blacksmith-\d+vcpu-windows-2025",\n\s*target:\s*"aarch64-pc-windows-msvc"/,
  );
});

test("release workflow bundles fresco-native binaries into the root package instead of publishing platform packages", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const frescoJobStart = workflow.indexOf("\n  release-npm-fresco-native:\n");
  const nextJobStart = workflow.indexOf("\n  # Build and publish WASM package", frescoJobStart);
  const frescoJob = workflow.slice(frescoJobStart, nextJobStart);

  assert.match(
    frescoJob,
    /Clean bundled native binaries[\s\S]*tools\/moon\/scripts\/github\/clean_node_binaries\.mbtx/,
  );
  assert.match(
    frescoJob,
    /Stage bundled native binaries[\s\S]*tools\/moon\/scripts\/github\/collect_native_artifacts\.mbtx/,
  );
  assert.doesNotMatch(frescoJob, /napi create-npm-dirs/);
  assert.doesNotMatch(frescoJob, /publish_npm_package_dirs\.mbtx/);
});

test("cargo config forces the bundled Rust linker for Windows MSVC targets", () => {
  const cargoConfig = readRepoFile(".cargo", "config.toml");

  assert.match(cargoConfig, /\[target\.x86_64-pc-windows-msvc\]\s*linker = "rust-lld"/);
  assert.match(cargoConfig, /\[target\.aarch64-pc-windows-msvc\]\s*linker = "rust-lld"/);
});

test("release workflow tunes Windows production Rust builds for cold runners", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const profileSteps = [...workflow.matchAll(/- name: Tune Windows release profile/g)];

  assert.equal(profileSteps.length, 2);
  assert.match(
    workflow,
    /Tune Windows release profile[\s\S]*if: runner\.os == 'Windows'[\s\S]*CARGO_PROFILE_RELEASE_LTO=thin/,
  );
  assert.match(
    workflow,
    /Tune Windows release profile[\s\S]*CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16/,
  );
  assert.match(
    workflow,
    /Tune Windows release profile[\s\S]*Build CLI[\s\S]*cargo build --release -p vize --target \$\{\{ matrix\.settings\.target \}\}/,
  );
  assert.match(
    workflow,
    /Tune Windows release profile[\s\S]*Build vize-native[\s\S]*tools\/moon\/scripts\/github\/build_napi_package\.mbtx/,
  );
});

test("release workflow runs GitHub helper scripts with the native target on every runner", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.doesNotMatch(workflow, /MOON_HELPER_TARGET/);
  assert.match(
    workflow,
    /Install cross-compilation tools \(Linux ARM64\)[\s\S]*moon run --target native - -- < tools\/moon\/scripts\/github\/install_cross_compile_tools\.mbtx/,
  );
  assert.match(
    workflow,
    /Create archive \(Windows\)[\s\S]*moon run --target native - -- \$\{\{ matrix\.settings\.target \}\} \$\{\{ matrix\.settings\.archive \}\} vize\.exe < tools\/moon\/scripts\/github\/create_cli_archive\.mbtx/,
  );
  assert.match(workflow, /Build vize-native[\s\S]*moon run --target native - -- npm\/vize-native/);
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
  assert.doesNotMatch(checkJsJob, /setup-moonbit/);
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

test("check and docs workflows use the CI Rust profile for non-release native builds", () => {
  const checkWorkflow = readRepoFile(".github", "workflows", "check.yml");
  const deployDocsWorkflow = readRepoFile(".github", "workflows", "deploy-docs.yml");

  assert.match(checkWorkflow, /cargo build --profile ci -p vize/);
  assert.match(checkWorkflow, /cp target\/ci\/vize \/usr\/local\/bin\/vize/);
  assert.match(checkWorkflow, /vp run --filter '\.\/npm\/vize-native' build:ci/);
  assert.match(deployDocsWorkflow, /vp run --filter '\.\/npm\/vize-native' build:ci/);
});

test("pkg.pr.new workflow publishes built npm packages from the lockfile", () => {
  const workflow = readRepoFile(".github", "workflows", "pkg-pr-new.yml");
  const job = workflowJobBody(workflow, "publish-preview");

  assert.match(job, /timeout-minutes:\s*30/);
  assert.match(job, /vp run --workspace-root build:packages/);
  assert.match(job, /vp exec pkg-pr-new publish --pnpm --packageManager=pnpm --comment=update/);
  assert.doesNotMatch(job, /\b(?:npx|bunx)\b|pnpm dlx|yarn dlx/);
  assert.equal([...job.matchAll(/pkg-pr-new publish/g)].length, 1);

  for (const packagePath of [
    "./npm/vize",
    "./npm/vite-plugin-vize",
    "./npm/oxlint-plugin-vize",
    "./npm/unplugin-vize",
    "./npm/fresco",
    "./npm/musea-mcp-server",
    "./npm/vite-plugin-musea",
    "./npm/rspack-vize-plugin",
    "./npm/musea-nuxt",
    "./npm/nuxt",
  ]) {
    assert.match(job, new RegExp(packagePath.replaceAll("/", "\\/").replace(".", "\\.")));
  }
});
