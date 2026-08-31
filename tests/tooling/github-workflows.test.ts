import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  hostedOrBlacksmith,
  normalizeRepoText,
  readGithubYamlFiles,
  readRepoFile,
  workflowJobBody,
} from "./support/github-workflows.ts";

test("repo text helpers normalize line endings before workflow matching", () => {
  const workflow = normalizeRepoText("jobs:\r\n  check-js:\r\n    runs-on: ubuntu-24.04\r\n");

  assert.equal(workflowJobBody(workflow, "check-js"), "  check-js:\n    runs-on: ubuntu-24.04\n");
});

test("GitHub workflows opt JavaScript actions into Node 24", () => {
  for (const workflowName of [
    "build-docs.yml",
    "check.yml",
    "deploy-docs.yml",
    "miri.yml",
    "native-smoke.yml",
    "pkg-pr-new.yml",
    "release-preflight.yml",
    "release.yml",
    "title-policy.yml",
    "tool-benchmark.yml",
    "zizmor.yml",
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
    ".github/workflows/build-docs.yml",
    ".github/workflows/check.yml",
    ".github/workflows/deploy-docs.yml",
    ".github/workflows/e2e.yml",
    ".github/workflows/miri.yml",
    ".github/workflows/native-smoke.yml",
    ".github/workflows/release.yml",
    ".github/workflows/tool-benchmark.yml",
  ]) {
    const file = readRepoFile(...relativePath.split("/"));
    assert.doesNotMatch(file, /uses:\s*actions\/cache@v4/, `${relativePath} still uses cache v4`);
  }
});

test("Rust sticky cache skips Blacksmith disks on GitHub-hosted runners", () => {
  const action = parse(
    readRepoFile(".github", "actions", "setup-rust-sticky-cache", "action.yml"),
  ) as {
    runs?: { steps?: Array<{ name?: string; if?: string; run?: string; uses?: string }> };
  };
  const steps = action.runs?.steps ?? [];

  // Every mount has to carry the guard itself: a condition attached to one step
  // does nothing for an unconditional `stickydisk` sibling, which fails on a
  // GitHub-hosted runner instead of being skipped.
  const mounts = steps.filter((step) => step.uses?.startsWith("useblacksmith/stickydisk@"));
  assert.ok(mounts.length > 0, "expected sticky disk mounts");
  for (const mount of mounts) {
    const guard =
      mount.name === "Mount secondary Rust target sticky disk"
        ? "${{ runner.environment != 'github-hosted' && inputs.secondary-target-path != '' }}"
        : "${{ runner.environment != 'github-hosted' }}";
    assert.equal(mount.if, guard, `unguarded sticky disk mount: ${mount.name}`);
  }
  assert.ok(
    mounts.some((mount) => mount.name === "Mount secondary Rust target sticky disk"),
    "the optional secondary mount must keep both guards",
  );

  // Found by what it reports, not by its condition: keying off the condition
  // alone would keep passing once the notice is deleted and any other
  // hosted-only step remains.
  const notice = steps.find((step) => step.run?.includes("skipping Blacksmith sticky disks"));
  assert.ok(notice, "hosted runners must report why sticky disks were skipped");
  assert.equal(notice.if, "${{ runner.environment == 'github-hosted' }}");
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
    /^(?:ubuntu-(?:latest|2[24]\.04)(?:-arm)?|blacksmith-\d+vcpu-ubuntu-2404(?:-arm)?|macos-15(?:-intel)?|blacksmith-(?:6|12)vcpu-macos-15|windows-(?:2025|11-arm)|blacksmith-\d+vcpu-windows-2025)$/;
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
  const releasePlatforms = readRepoFile(
    "tools",
    "commands",
    "ci",
    "github",
    "release-platforms.rs",
  );

  // check.yml runs every job on the same Linux runner — we accept either the
  // GitHub-hosted label or any Blacksmith Ubuntu SKU so changing vCPU size
  // (or temporarily reverting to GitHub-hosted) doesn't churn this test.
  assert.match(checkWorkflow, new RegExp(`runs-on:\\s*${hostedOrBlacksmith("ubuntu-24.04")}`));
  assert.match(nativeWorkflow, new RegExp(`runner:\\s*${hostedOrBlacksmith("ubuntu-22.04-arm")}`));
  assert.match(nativeWorkflow, new RegExp(`runner:\\s*${hostedOrBlacksmith("macos-15")}`));
  assert.match(nativeWorkflow, new RegExp(`runner:\\s*${hostedOrBlacksmith("windows-2025")}`));
  assert.match(
    releasePlatforms,
    new RegExp(`host:\\s*"${hostedOrBlacksmith("ubuntu-22.04-arm")}"`),
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

test("title policy workflow mutates only issue and PR metadata", () => {
  const workflow = readRepoFile(".github", "workflows", "title-policy.yml");
  const job = workflowJobBody(workflow, "issue-pr-title-policy");
  const parsed = parse(workflow) as {
    jobs?: {
      "issue-pr-title-policy"?: {
        steps?: Array<{ uses?: string; with?: { "sparse-checkout"?: string } }>;
      };
    };
  };
  const steps = parsed.jobs?.["issue-pr-title-policy"]?.steps ?? [];
  const sparseCheckoutIndex = steps.findIndex((step) => step.uses?.startsWith("actions/checkout@"));
  const rustScriptSetupIndex = steps.findIndex(
    (step) => step.uses === "./.github/actions/setup-rust-script",
  );
  const sparseCheckout = steps[sparseCheckoutIndex]?.with?.["sparse-checkout"];
  const sparseCheckoutEntries =
    sparseCheckout
      ?.split(/\r?\n/)
      .map((entry) => entry.trim())
      .filter(Boolean) ?? [];

  assert.match(workflow, /\n  issues:\n\s+types:\s+\[opened, edited, reopened\]/);
  assert.match(workflow, /\n  pull_request_target:\n/);
  assert.match(workflow, /issues:\s*write/);
  assert.match(workflow, /pull-requests:\s*write/);
  assert.match(job, /timeout-minutes:\s*5/);
  assert.match(job, /ref:\s*\$\{\{\s*github\.event\.repository\.default_branch\s*\}\}/);
  assert.deepEqual(sparseCheckoutEntries, [
    ".github/actions/setup-rust-script",
    "tools/commands/ci/github/issue-pr-title-policy.rs",
    "tools/support/common.rs",
  ]);
  assert.match(job, /uses:\s*dtolnay\/rust-toolchain@[0-9a-f]{40}/);
  assert.match(job, /uses:\s*\.\/\.github\/actions\/setup-rust-script/);
  assert.match(job, /rust-script tools\/commands\/ci\/github\/issue-pr-title-policy\.rs/);
  assert.doesNotMatch(job, /\.github\/actions\/setup-moonbit/);
  assert.doesNotMatch(job, /tools\/moon\/cmd\/github\/issue_pr_title_policy/);
  assert.doesNotMatch(
    job,
    /moon run --target native tools\/moon\/cmd\/github\/issue_pr_title_policy --/,
  );
  assert.doesNotMatch(job, /\.github\/scripts\/issue-pr-title-policy\.mjs/);
  assert.doesNotMatch(job, /github\.event\.pull_request\.head/);
  assert.ok(sparseCheckoutIndex >= 0, "title policy must use a sparse checkout");
  assert.ok(rustScriptSetupIndex >= 0, "title policy must install rust-script");
  assert.ok(sparseCheckoutIndex < rustScriptSetupIndex);
});

test("App E2E workflow keeps Blacksmith Testbox dispatch hydration separate", () => {
  const workflow = readRepoFile(".github", "workflows", "e2e.yml");
  const job = workflowJobBody(workflow, "testbox");
  const appPlanJob = workflowJobBody(workflow, "app-e2e-plan");
  const appJob = workflowJobBody(workflow, "app-e2e");

  assert.match(workflow, /\n  workflow_dispatch:\n/);
  assert.match(workflow, /testbox_id:\n\s+description:\s*Blacksmith Testbox session ID/);
  assert.match(
    job,
    /if:\s*\$\{\{\s*github\.event_name == 'workflow_dispatch' && inputs\.testbox_id != ''\s*\}\}/,
  );
  assert.match(job, /uses:\s*useblacksmith\/begin-testbox@[0-9a-f]{40}\s*# v2/);
  assert.match(job, /id:\s*begin-testbox/);
  assert.match(job, /testbox_id:\s*\$\{\{\s*inputs\.testbox_id\s*\}\}/);
  assert.match(job, /uses:\s*useblacksmith\/run-testbox@[0-9a-f]{40}\s*# v2/);
  assert.match(
    job,
    /if:\s*\$\{\{\s*always\(\) && steps\.begin-testbox\.outcome == 'success'\s*\}\}/,
  );
  assert.doesNotMatch(job, /vp run --workspace-root test|cargo test --workspace/);
  assert.match(
    appPlanJob,
    /if:\s*\$\{\{\s*github\.event_name != 'pull_request' && \(github\.event_name != 'workflow_dispatch' \|\| inputs\.testbox_id == ''\)\s*\}\}/,
  );
  // The aggregator still has to report on cancelled or failed rows, so it keeps
  // the same gate behind `always()`.
  assert.match(
    appJob,
    /if:\s*\$\{\{\s*always\(\) && github\.event_name != 'pull_request' && \(github\.event_name != 'workflow_dispatch' \|\| inputs\.testbox_id == ''\)\s*\}\}/,
  );
});

test("Linux Rust CI installs Wild linker before cargo builds", () => {
  for (const relativePath of [
    ".github/workflows/benchmark.yml",
    ".github/workflows/build-docs.yml",
    ".github/workflows/check.yml",
    ".github/workflows/criterion-bench.yml",
    // App E2E runs every row through the shared composite action.
    ".github/actions/app-e2e-row/action.yml",
    ".github/workflows/native-smoke.yml",
    ".github/workflows/release-preflight.yml",
    ".github/workflows/release.yml",
    ".github/workflows/tool-benchmark.yml",
  ]) {
    const file = readRepoFile(...relativePath.split("/"));
    assert.match(
      file,
      /uses:\s*wild-linker\/action@[0-9a-f]{40}\s*# v0\.9\.0/,
      `${relativePath} should install the pinned Wild linker action`,
    );
    assert.match(file, /wild-version:\s*"0\.9\.0"/);
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
    "build-docs.yml",
    "check.yml",
    "criterion-bench.yml",
    "e2e.yml",
    "fuzz.yml",
    "miri.yml",
    "release-preflight.yml",
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
