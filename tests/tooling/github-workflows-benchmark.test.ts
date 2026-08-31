import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { hostedOrBlacksmith, readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

type ParsedWorkflow = {
  jobs?: Record<
    string,
    { if?: string; steps?: Array<{ name?: string; with?: Record<string, string> }> }
  >;
};

// Inputs of the step that owns a contract, read as parsed YAML values so the
// assertions survive re-indentation, quoting, and block-scalar rewrites.
function stepInputs(
  workflow: ParsedWorkflow,
  jobName: string,
  stepName: string,
): Record<string, string> {
  const step = workflow.jobs?.[jobName]?.steps?.find((entry) => entry.name === stepName);
  assert.ok(step, `missing step ${stepName}`);
  return step.with ?? {};
}

function workflowStepBody(job: string, stepName: string): string {
  const marker = `\n      - name: ${stepName}\n`;
  const start = job.indexOf(marker);
  assert.notEqual(start, -1, `missing step ${stepName}`);
  const bodyStart = start + 1;
  const nextStep = job.indexOf("\n      - ", bodyStart + marker.length);
  return job.slice(bodyStart, nextStep === -1 ? undefined : nextStep);
}

test("benchmark workflow comments from trusted code after a read-only benchmark run", () => {
  const workflow = readRepoFile(".github", "workflows", "benchmark.yml");
  const benchmarkJob = workflowJobBody(workflow, "pr-benchmark");
  const budgetJob = workflowJobBody(workflow, "pr-benchmark-budget");
  const commentJob = workflowJobBody(workflow, "pr-benchmark-comment");

  assert.match(benchmarkJob, /contents:\s*read/);
  assert.doesNotMatch(benchmarkJob, /issues:\s*write/);
  assert.doesNotMatch(benchmarkJob, /pull-requests:\s*write/);
  assert.match(benchmarkJob, /path:\s*head[\s\S]*ref:\s*\$\{\{\s*env\.BENCHMARK_HEAD_SHA\s*\}\}/);
  assert.match(benchmarkJob, /path:\s*base[\s\S]*ref:\s*\$\{\{\s*env\.BENCHMARK_BASE_SHA\s*\}\}/);
  assert.match(
    workflow,
    /BENCHMARK_HEAD_SHA:\s*\$\{\{[^\n]*github\.event_name == 'workflow_dispatch' && inputs\.head_sha \|\| github\.event\.pull_request\.head\.sha\s*\}\}/,
  );
  assert.match(
    workflow,
    /BENCHMARK_BASE_SHA:\s*\$\{\{[^\n]*github\.event_name == 'workflow_dispatch' && inputs\.base_sha \|\| github\.event\.pull_request\.base\.sha\s*\}\}/,
  );
  assert.match(benchmarkJob, /name:\s*pr-benchmark/);
  assert.doesNotMatch(benchmarkJob, /node base\/tools\/benchmarks\/scripts\/comment-pr\.mjs/);
  assert.doesNotMatch(benchmarkJob, /node tools\/benchmarks\/scripts\/comment-pr\.mjs/);
  assert.match(benchmarkJob, /--threshold "\$VIZE_BENCH_REGRESSION_THRESHOLD_PERCENT"/);

  assert.match(budgetJob, /needs:\n\s+- pr-benchmark\b/);
  assert.match(budgetJob, /actions:\s*read/);
  assert.match(budgetJob, /contents:\s*read/);
  assert.doesNotMatch(budgetJob, /issues:\s*read/);
  assert.doesNotMatch(budgetJob, /issues:\s*write/);
  assert.doesNotMatch(budgetJob, /pull-requests:\s*write/);
  assert.match(budgetJob, /path:\s*head[\s\S]*ref:\s*\$\{\{\s*env\.BENCHMARK_HEAD_SHA\s*\}\}/);
  assert.match(budgetJob, /uses:\s*actions\/download-artifact@[0-9a-f]{40}\s*# v8\.0\.1/);
  assert.match(budgetJob, /name:\s*pr-benchmark/);
  assert.doesNotMatch(budgetJob, /name:\s*Read current PR labels/);
  assert.doesNotMatch(budgetJob, /GITHUB_TOKEN:\s*\$\{\{\s*github\.token\s*\}\}/);
  assert.doesNotMatch(budgetJob, /issues\/\$\{process\.env\.PR_NUMBER\}\/labels/);
  assert.match(
    budgetJob,
    /node head\/tools\/benchmarks\/scripts\/enforce-pr-budget\.mjs[\s\S]*--json benchmark-results\.json[\s\S]*--labels-json "\$PR_LABELS_JSON"/,
  );
  assert.match(
    budgetJob,
    /PR_LABELS_JSON:\s*\$\{\{\s*github\.event_name == 'pull_request' && toJSON\(github\.event\.pull_request\.labels\.\*\.name\) \|\| '\[\]'\s*\}\}/,
  );

  assert.match(commentJob, /needs:\n\s+- pr-benchmark\b/);
  assert.match(
    commentJob,
    /if:\s*\$\{\{\s*github\.event_name == 'pull_request' && github\.event\.pull_request\.head\.repo\.full_name == github\.repository\s*\}\}/,
  );
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
    /node tools\/benchmarks\/scripts\/comment-pr\.mjs --body benchmark-summary\.md --comment-key "\$BENCHMARK_COMMENT_KEY"/,
  );
});

test("benchmark dispatch validates exact SHA evidence and runs the existing budget", () => {
  const workflow = readRepoFile(".github", "workflows", "benchmark.yml");
  const benchmarkJob = workflowJobBody(workflow, "pr-benchmark");
  const budgetJob = workflowJobBody(workflow, "pr-benchmark-budget");
  const commentJob = workflowJobBody(workflow, "pr-benchmark-comment");
  const validateDispatchStep = workflowStepBody(benchmarkJob, "Validate benchmark SHAs");
  const checkoutHeadStep = workflowStepBody(benchmarkJob, "Checkout head");
  const checkBaseStep = workflowStepBody(benchmarkJob, "Check base checkout");

  assert.match(
    workflow,
    /workflow_dispatch:\n\s+inputs:\n\s+base_sha:[\s\S]*required:\s*true[\s\S]*head_sha:[\s\S]*required:\s*true/,
  );
  assert.match(workflow, /base_sha:[\s\S]*type:\s*string/);
  assert.match(workflow, /head_sha:[\s\S]*type:\s*string/);
  assert.match(
    workflow,
    /run-name:\s*"Benchmark [^\n]*inputs\.base_sha[^\n]*inputs\.head_sha[^\n]*PR #\{0\}[^\n]*"/,
  );
  assert.match(
    workflow,
    /group:[^\n]*dispatch-\{0\}-\{1\}[^\n]*inputs\.base_sha[^\n]*inputs\.head_sha/,
  );
  assert.match(workflow, /group:[^\n]*\|\| github\.event\.pull_request\.number[^\n]*\}\}/);
  assert.match(workflow, /head SHA from --ref/);
  assert.match(workflow, /validation rejects mismatches/);

  assert.match(validateDispatchStep, /if:\s*\$\{\{\s*github\.event_name != 'pull_request'\s*\}\}/);
  assert.match(validateDispatchStep, /full_sha='\^\[0-9a-f\]\{40\}\$'/);
  assert.match(validateDispatchStep, /! "\$BENCHMARK_BASE_SHA" =~ \$full_sha/);
  assert.match(validateDispatchStep, /! "\$BENCHMARK_HEAD_SHA" =~ \$full_sha/);
  assert.match(validateDispatchStep, /"\$BENCHMARK_BASE_SHA" == "\$BENCHMARK_HEAD_SHA"/);
  assert.match(validateDispatchStep, /base_sha must differ from head_sha/);
  assert.match(validateDispatchStep, /RUN_HEAD_SHA:\s*\$\{\{\s*github\.sha\s*\}\}/);
  assert.match(validateDispatchStep, /"\$RUN_HEAD_SHA" != "\$BENCHMARK_HEAD_SHA"/);
  assert.match(validateDispatchStep, /run head_sha must match/);
  assert.match(
    checkoutHeadStep,
    /fetch-depth:\s*\$\{\{\s*github\.event_name != 'pull_request' && '0' \|\| '1'\s*\}\}/,
  );
  assert.match(checkBaseStep, /EVENT_NAME:\s*\$\{\{\s*github\.event_name\s*\}\}/);
  assert.match(checkBaseStep, /"\$EVENT_NAME" != "pull_request"/);
  assert.match(checkBaseStep, /merge-base --is-ancestor "\$BASE_SHA" "\$HEAD_SHA"/);
  assert.match(checkBaseStep, /base_sha must be an ancestor of head_sha/);

  const validateIndex = benchmarkJob.indexOf("- name: Validate benchmark SHAs");
  const ancestryIndex = benchmarkJob.indexOf("- name: Check base checkout");
  const benchmarkIndex = benchmarkJob.indexOf("- name: Compare base and head");
  assert.notEqual(validateIndex, -1, "missing SHA validation step");
  assert.notEqual(ancestryIndex, -1, "missing ancestry validation step");
  assert.notEqual(benchmarkIndex, -1, "missing benchmark step");
  assert.ok(validateIndex < ancestryIndex, "SHA equality must be validated before ancestry");
  assert.ok(ancestryIndex < benchmarkIndex, "ancestry must be validated before benchmarking");

  assert.match(budgetJob, /name:\s*pr-benchmark/);
  assert.match(budgetJob, /--labels-json "\$PR_LABELS_JSON"/);
  assert.match(budgetJob, /\|\| '\[\]'/);
  assert.doesNotMatch(commentJob, /github\.event_name == 'workflow_dispatch'/);
});

test("benchmark schedule gates long-term drift against a fixed commit", () => {
  const workflow = readRepoFile(".github", "workflows", "benchmark.yml");
  const benchmarkJob = workflowJobBody(workflow, "pr-benchmark");
  const budgetJob = workflowJobBody(workflow, "pr-benchmark-budget");
  const validateStep = workflowStepBody(benchmarkJob, "Validate benchmark SHAs");
  const checkoutHeadStep = workflowStepBody(benchmarkJob, "Checkout head");
  const checkBaseStep = workflowStepBody(benchmarkJob, "Check base checkout");
  const provenanceStep = workflowStepBody(benchmarkJob, "Record benchmark build provenance");
  const parsed = parse(workflow) as ParsedWorkflow;
  const cacheBaseInputs = stepInputs(parsed, "pr-benchmark", "Cache base CLI");
  const cacheHeadInputs = stepInputs(parsed, "pr-benchmark", "Cache head CLI");
  const uploadInputs = stepInputs(parsed, "pr-benchmark", "Upload benchmark results");

  assert.match(workflow, /\n  schedule:\n\s+- cron:\s*"29 5 \* \* 2"/);
  assert.match(
    workflow,
    /BENCHMARK_BASE_SHA:\s*\$\{\{\s*github\.event_name == 'schedule' && '[0-9a-f]{40}' \|\| github\.event_name == 'workflow_dispatch'/,
  );
  const baseline = workflow.match(/github\.event_name == 'schedule' && '([0-9a-f]{40})'/)?.[1];
  assert.ok(baseline, "missing exact scheduled benchmark baseline");
  assert.equal(workflow.split(baseline).length - 1, 1, "baseline must have one refresh point");
  assert.match(
    workflow,
    /BENCHMARK_HEAD_SHA:\s*\$\{\{\s*github\.event_name == 'schedule' && github\.sha \|\| github\.event_name == 'workflow_dispatch'/,
  );
  assert.match(validateStep, /github\.event_name != 'pull_request'/);
  assert.match(checkoutHeadStep, /github\.event_name != 'pull_request' && '0' \|\| '1'/);
  assert.match(checkBaseStep, /"\$EVENT_NAME" != "pull_request"/);
  assert.match(checkBaseStep, /merge-base --is-ancestor "\$BASE_SHA" "\$HEAD_SHA"/);
  // Each cached binary is keyed by runner platform, build profile, resolved
  // toolchain, and its own commit, so a stale artifact can never be reused.
  assert.equal(cacheBaseInputs.path, "base/target/ci-opt/vize");
  assert.equal(
    cacheBaseInputs.key,
    "${{ runner.os }}-${{ runner.arch }}-benchmark-base-${{ env.VIZE_BENCH_BUILD_PROFILE_KEY }}-${{ steps.rust-toolchain.outputs.cachekey }}-${{ env.BENCHMARK_BASE_SHA }}",
  );
  assert.equal(cacheHeadInputs.path, "head/target/ci-opt/vize");
  assert.equal(
    cacheHeadInputs.key,
    "${{ runner.os }}-${{ runner.arch }}-benchmark-head-${{ env.VIZE_BENCH_BUILD_PROFILE_KEY }}-${{ steps.rust-toolchain.outputs.cachekey }}-${{ env.BENCHMARK_HEAD_SHA }}",
  );
  assert.match(provenanceStep, /rustc --version --verbose/);
  assert.match(provenanceStep, /printf 'profile=%s\\n' "\$VIZE_BENCH_BUILD_PROFILE_KEY"/);
  assert.match(provenanceStep, /sha256sum base\/target\/ci-opt\/vize head\/target\/ci-opt\/vize/);
  // The provenance file only reaches the budget job if it is an entry of the
  // uploaded artifact's own path list, so compare parsed entries exactly.
  assert.equal(uploadInputs.name, "pr-benchmark");
  const uploadPaths = String(uploadInputs.path ?? "")
    .split("\n")
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
  assert.ok(
    uploadPaths.includes("benchmark-provenance.txt"),
    `provenance must be uploaded: ${uploadPaths.join(", ")}`,
  );

  // Scheduled runs cannot opt out of a missing or unbuildable baseline: only
  // pull_request events provide label names, and every other event supplies an
  // empty label set.
  const budgetHeader = budgetJob.slice(0, budgetJob.indexOf("\n    steps:"));
  assert.doesNotMatch(budgetHeader, /\n    if:/, "scheduled budget must always run");
  assert.match(
    budgetJob,
    /github\.event_name == 'pull_request' && toJSON\(github\.event\.pull_request\.labels\.\*\.name\) \|\| '\[\]'/,
  );
  // Commenting stays scoped to same-repo pull requests, so scheduled runs can
  // never reach the write-permission job regardless of how the guard is worded.
  assert.equal(
    parsed.jobs?.["pr-benchmark-comment"]?.if,
    "${{ github.event_name == 'pull_request' && github.event.pull_request.head.repo.full_name == github.repository }}",
  );
});

test("criterion bench workflow runs an A/B micro-benchmark and a dialect guard", () => {
  const workflow = readRepoFile(".github", "workflows", "criterion-bench.yml");
  const abJob = workflowJobBody(workflow, "criterion-ab");
  const guardJob = workflowJobBody(workflow, "dialect-guard");

  // Only runs on PRs and only when Rust or the bench harness changes.
  assert.match(workflow, /\n  pull_request:\n/);
  assert.match(workflow, /paths:\n\s+- "crates\/\*\*"/);
  assert.match(workflow, /- "tools\/benchmarks\/crates\/\*\*"/);
  assert.match(workflow, /- "Cargo\.lock"/);
  assert.match(workflow, /- "Cargo\.toml"/);
  assert.match(workflow, /- "tools\/benchmarks\/scripts\/criterion-ab\.mjs"/);
  assert.match(workflow, /- "tools\/benchmarks\/scripts\/criterion-impact\.mjs"/);
  assert.match(workflow, /- "tools\/benchmarks\/scripts\/criterion-summary\.mjs"/);
  assert.match(workflow, /- "tools\/benchmarks\/scripts\/dialect-guard\.mjs"/);
  assert.match(workflow, /FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*true/);

  for (const [jobName, minutes] of [
    ["criterion-ab", 120],
    ["dialect-guard", 45],
  ] as const) {
    assert.match(
      workflowJobBody(workflow, jobName),
      new RegExp(`timeout-minutes:\\s*${minutes}\\b`),
    );
  }

  // A/B: alternating base/head criterion baselines compared with critcmp into a
  // shared target dir; report-only by default (no threshold blocks the PR).
  assert.match(abJob, new RegExp(`runs-on:\\s*${hostedOrBlacksmith("ubuntu-24.04")}`));
  assert.match(abJob, /contents:\s*read/);
  assert.doesNotMatch(abJob, /contents:\s*write/);
  const checkoutHead = workflowStepBody(abJob, "Checkout head");
  const checkoutBase = workflowStepBody(abJob, "Checkout base");
  assert.match(checkoutHead, /ref:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/);
  assert.match(checkoutHead, /fetch-depth:\s*0/);
  assert.match(checkoutHead, /persist-credentials:\s*false/);
  assert.match(checkoutBase, /ref:\s*\$\{\{\s*github\.event\.pull_request\.base\.sha\s*\}\}/);
  assert.match(checkoutBase, /persist-credentials:\s*false/);

  const impactStep = workflowStepBody(abJob, "Select affected Criterion suites");
  assert.match(impactStep, /PR_BASE_SHA:\s*\$\{\{\s*github\.event\.pull_request\.base\.sha\s*\}\}/);
  assert.match(impactStep, /PR_HEAD_SHA:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/);
  assert.match(impactStep, /--base-sha "\$PR_BASE_SHA"/);
  assert.match(impactStep, /--head-sha "\$PR_HEAD_SHA"/);

  for (const stepName of [
    "Setup Wild linker",
    "Mount Criterion cache",
    "Cache critcmp",
    "Install critcmp",
  ]) {
    assert.match(
      workflowStepBody(abJob, stepName),
      /if:\s*steps\.impact\.outputs\.has_suites == 'true'/,
      stepName,
    );
  }
  assert.match(
    workflowStepBody(abJob, "Mount Criterion cache"),
    /uses:\s*\.\/head\/\.github\/actions\/setup-rust-sticky-cache/,
  );
  assert.match(
    workflowStepBody(abJob, "Install critcmp"),
    /cargo install critcmp --version 0\.1\.8 --locked/,
  );
  const runStep = workflowStepBody(abJob, "Run criterion A/B");
  assert.match(runStep, /node head\/tools\/benchmarks\/scripts\/criterion-ab\.mjs/);
  assert.match(runStep, /--target-dir "\$GITHUB_WORKSPACE\/head\/target"/);
  const impactPath = impactStep.match(/--out "([^"]+)"/)?.[1];
  const selectionPath = runStep.match(/--selection "([^"]+)"/)?.[1];
  assert.equal(impactPath, "$GITHUB_WORKSPACE/criterion-impact.json");
  assert.equal(selectionPath, impactPath);

  // Dialect guard: build vize with legacy OFF and ON, then assert byte-identical
  // Vue 3 codegen plus a small A/B timing budget.
  assert.match(guardJob, new RegExp(`runs-on:\\s*${hostedOrBlacksmith("ubuntu-24.04")}`));
  assert.match(guardJob, /cargo build --profile ci-opt -p vize --target-dir target\/off/);
  assert.match(
    guardJob,
    /cargo build --profile ci-opt -p vize --features legacy --target-dir target\/on/,
  );
  assert.match(guardJob, /node tools\/benchmarks\/scripts\/generate\.mjs "\$DIALECT_GUARD_FILE_COUNT"/);
  assert.match(guardJob, /node tools\/benchmarks\/scripts\/dialect-guard\.mjs/);
  assert.match(guardJob, /--off-bin target\/off\/ci-opt\/vize/);
  assert.match(guardJob, /--on-bin target\/on\/ci-opt\/vize/);
  assert.match(guardJob, /--threshold "\$DIALECT_GUARD_THRESHOLD_PERCENT"/);
});
