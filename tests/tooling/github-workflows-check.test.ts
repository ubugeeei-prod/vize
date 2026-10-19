import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  checkWorkflowJobSources,
  readRepoFile,
  workflowJobBody,
} from "./support/github-workflows.ts";

test("Check jobs bound runner time explicitly", () => {
  const workflow = parse(checkWorkflowJobSources()) as {
    jobs: Record<string, { "runs-on"?: string; "timeout-minutes"?: number }>;
  };
  for (const [jobName, job] of Object.entries(workflow.jobs)) {
    if (!job["runs-on"]) continue;
    assert.ok(
      Number.isInteger(job["timeout-minutes"]) && (job["timeout-minutes"] ?? 0) > 0,
      `${jobName} must set a finite timeout`,
    );
  }
});

test("SemVer checks run on exact-SHA release dispatches", () => {
  const workflow = checkWorkflowJobSources();
  const job = workflowJobBody(workflow, "semver-checks");

  assert.match(
    job,
    /if:\s*\$\{\{\s*github\.event_name == 'workflow_dispatch' && \(startsWith\(github\.ref, 'refs\/heads\/release\/v'\) \|\| startsWith\(github\.ref, 'refs\/tags\/v'\)\)\s*\}\}/,
  );
  assert.match(job, /name: Use candidate version for SemVer classification/);
  assert.match(job, /: > "\$RUNNER_TEMP\/semver-change-marker\.txt"/);
  assert.match(
    job,
    /BASELINE_REV:\s*\$\{\{\s*github\.event_name == 'pull_request' && github\.event\.pull_request\.base\.sha \|\| \(github\.event_name == 'push' && github\.event\.before \|\| ''\)\s*\}\}/,
  );
  const script = readRepoFile("tools", "commands", "ci", "github", "check-semver.ts");
  assert.match(script, /\["rev-parse", "--verify", "-q", "HEAD\^2"\]/);
  assert.match(script, /\["rev-parse", "HEAD\^1"\]/);
});

test("check workflow gates PRs and merge groups without report dependencies", () => {
  const workflow = checkWorkflowJobSources();
  const reportJob = workflowJobBody(workflow, "test-report");

  assert.match(
    reportJob,
    /if:\s*\$\{\{\s*always\(\) && \(github\.event_name == 'pull_request' \|\| github\.event_name == 'merge_group'\)\s*\}\}/,
  );
  assert.match(reportJob, /contents:\s*read/);
  assert.doesNotMatch(reportJob, /issues:\s*write/);
  assert.doesNotMatch(reportJob, /pull-requests:\s*write/);

  for (const jobName of [
    "iteration-plan",
    "fmt-rust",
    "check-js",
    "security-audit",
    "node-engine-compat",
    "check-vize-apps",
  ]) {
    assert.match(reportJob, new RegExp(`- ${jobName}\\b`));
  }

  assert.match(reportJob, /NEEDS_JSON:\s*"\$\{\{ toJSON\(needs\) \}\}"/);
  assert.match(reportJob, /node tools\/support\/compat\/github\/require-needs-success\.ts/);
  assert.match(reportJob, /uses: actions\/setup-node@/);
  assert.match(reportJob, /node-version-file: package\.json/);
  assert.doesNotMatch(reportJob, /vp install|npm install|pnpm install|report-test-inventory/);
});

test("check workflow runs the editor extension host smoke against a real vize server", () => {
  const workflow = checkWorkflowJobSources();
  const smokeAction = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");

  assert.match(
    workflowJobBody(workflow, "editor-extensions"),
    /uses:\s*\.\/\.github\/actions\/package-editor-extensions/,
  );
  assert.match(
    workflowJobBody(workflow, "editor-host-smoke"),
    /uses:\s*\.\/\.github\/actions\/vscode-host-smoke/,
  );
  assert.match(smokeAction, /cargo build --profile ci -p vize/);
  assert.match(smokeAction, /VIZE_SERVER_PATH:[^\n]*target\/ci\/vize/);
  assert.match(smokeAction, /xvfb-run -a vp run [^\n]*test:vscode-extension:host-real/);
});

test("check workflow blocks on Rust source and branch coverage budgets", () => {
  const workflow = checkWorkflowJobSources();
  const clippyJob = workflowJobBody(workflow, "rust-clippy-portability");
  const sourceJob = workflowJobBody(workflow, "source-coverage");
  const branchJob = workflowJobBody(workflow, "branch-coverage");
  const jobs = parse(workflow).jobs as Record<string, { env?: Record<string, unknown> }>;

  assert.equal(jobs["rust-clippy-portability"].env?.VIZE_NUXT_CONFIG_ITERATIONS, "100");
  assert.match(clippyJob, /uses: voidzero-dev\/setup-vp@[0-9a-f]{40}/);
  assert.match(clippyJob, /cache:\s*true[\s\S]*run-install:\s*true/);
  assert.equal(jobs["source-coverage"].env?.VIZE_NUXT_CONFIG_ITERATIONS, "100");
  assert.match(sourceJob, /tool:\s*cargo-llvm-cov/);
  assert.match(sourceJob, /vp install --frozen-lockfile --prefer-offline/);
  assert.match(sourceJob, /setup-rust-script[\s\S]*coverage:source/);
  assert.match(sourceJob, /Verify source coverage summary/);
  assert.match(sourceJob, /test -s target\/llvm-cov\/source-summary\.json/);
  assert.match(sourceJob, /source-summary\.json/);
  assert.match(sourceJob, /rust-source-coverage-summary/);
  assert.match(sourceJob, /continue-on-error:\s*true/);

  assert.match(branchJob, /toolchain:\s*nightly/);
  assert.match(branchJob, /tool:\s*cargo-llvm-cov/);
  assert.match(branchJob, /vp install --frozen-lockfile --prefer-offline/);
  assert.match(branchJob, /setup-rust-script[\s\S]*coverage:source:branch/);
  assert.match(branchJob, /Verify branch coverage summary/);
  assert.match(branchJob, /test -s target\/llvm-cov\/source-branch-summary\.json/);
  assert.match(branchJob, /source-branch-summary\.json/);
  assert.match(branchJob, /rust-branch-coverage-summary/);
  assert.match(branchJob, /continue-on-error:\s*true/);
  assert.match(
    branchJob,
    /github\.event_name == 'schedule' \|\| github\.event_name == 'workflow_dispatch'/,
  );
});

test("check workflow only installs Playwright browsers on cache misses", () => {
  const workflow = checkWorkflowJobSources();

  assert.match(workflow, /- name: Cache Playwright browsers\s+id: cache-playwright/);
  assert.match(
    workflow,
    /- name: Install Playwright browsers\s+if: steps\.cache-playwright\.outputs\.cache-hit != 'true'/,
  );
});

test("check workflow builds local native bindings before JS checks", () => {
  const workflow = checkWorkflowJobSources();
  const checkJsJob = workflowJobBody(workflow, "check-js");
  const buildJob = workflowJobBody(workflow, "build-js-packages");
  const playgroundJob = workflowJobBody(workflow, "playground-test");
  assert.match(checkJsJob, /vp run --workspace-root check:ci/);
  assert.match(checkJsJob, /vp run --workspace-root check:iteration/);
  assert.doesNotMatch(checkJsJob, /cargo build/);
  assert.match(checkJsJob, /setup-js-check-runtime/);
  assert.doesNotMatch(checkJsJob, /build:packages/);
  assert.match(buildJob, /vp run --filter '\.\/npm\/native' build:ci/);
  assert.match(buildJob, /vp run --filter '\.\/npm\/ui' check/);
  assert.ok(
    buildJob.indexOf("build:ci") < buildJob.indexOf("check:conformance"),
    "composable renderer conformance needs the locally built native binding",
  );
  assert.match(buildJob, /vp run --filter '\.\/npm\/compose\/core' check:conformance/);
  assert.ok(
    buildJob.indexOf("uses: ./.github/actions/setup-rust-script") <
      buildJob.indexOf("run: vp run --workspace-root build:packages"),
    "build-js-packages must install rust-script before build:packages",
  );
  assert.match(buildJob, /name:\s*shared-js-build/);
  assert.match(playgroundJob, /needs:\n\s+- build-js-packages\b/);
  assert.match(playgroundJob, /name:\s*shared-js-build/);
  assert.doesNotMatch(playgroundJob, /name: Build npm packages/);
});

test("check workflow uploads the VRT HTML report when snapshots fail", () => {
  const workflow = checkWorkflowJobSources();

  assert.match(workflow, /- name: Upload VRT report\s+if: steps\.vrt\.outcome == 'failure'/);
  assert.match(workflow, /name:\s*playground-vrt-report/);
  assert.match(workflow, /path:\s*playground\/playwright-report\//);
  assert.match(workflow, /if-no-files-found:\s*ignore/);
});
