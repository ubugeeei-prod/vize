import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

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
    /BENCHMARK_HEAD_SHA:\s*\$\{\{\s*github\.event_name == 'workflow_dispatch' && inputs\.head_sha \|\| github\.event\.pull_request\.head\.sha\s*\}\}/,
  );
  assert.match(
    workflow,
    /BENCHMARK_BASE_SHA:\s*\$\{\{\s*github\.event_name == 'workflow_dispatch' && inputs\.base_sha \|\| github\.event\.pull_request\.base\.sha\s*\}\}/,
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
  assert.match(budgetJob, /path:\s*head[\s\S]*ref:\s*\$\{\{\s*env\.BENCHMARK_HEAD_SHA\s*\}\}/);
  assert.match(budgetJob, /uses:\s*actions\/download-artifact@[0-9a-f]{40}\s*# v8\.0\.1/);
  assert.match(budgetJob, /name:\s*pr-benchmark/);
  assert.match(budgetJob, /name:\s*Read current PR labels/);
  assert.match(budgetJob, /if:\s*\$\{\{\s*github\.event_name == 'pull_request'\s*\}\}/);
  assert.match(budgetJob, /GITHUB_TOKEN:\s*\$\{\{\s*github\.token\s*\}\}/);
  assert.match(budgetJob, /process\.env\.GITHUB_API_URL \?\? "https:\/\/api\.github\.com"/);
  assert.match(budgetJob, /labels\.map\(\(label\) => label\.name\)/);
  assert.match(
    budgetJob,
    /node head\/bench\/enforce-pr-budget\.mjs[\s\S]*--json benchmark-results\.json[\s\S]*--labels-json "\$PR_LABELS_JSON"/,
  );
  assert.match(
    budgetJob,
    /PR_LABELS_JSON:\s*\$\{\{\s*github\.event_name == 'pull_request' && steps\.pr-labels\.outputs\.labels \|\| '\[\]'\s*\}\}/,
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
    /node bench\/comment-pr\.mjs --body benchmark-summary\.md --comment-key "\$BENCHMARK_COMMENT_KEY"/,
  );
});

test("benchmark dispatch validates exact SHA evidence and runs the existing budget", () => {
  const workflow = readRepoFile(".github", "workflows", "benchmark.yml");
  const benchmarkJob = workflowJobBody(workflow, "pr-benchmark");
  const budgetJob = workflowJobBody(workflow, "pr-benchmark-budget");
  const commentJob = workflowJobBody(workflow, "pr-benchmark-comment");

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
  assert.match(workflow, /head_sha from the branch or tag/);
  assert.match(workflow, /passed via --ref, not from a later checkout/);

  assert.match(benchmarkJob, /name:\s*Validate dispatch SHAs/);
  assert.match(benchmarkJob, /if:\s*\$\{\{\s*github\.event_name == 'workflow_dispatch'\s*\}\}/);
  assert.match(benchmarkJob, /full_sha='\^\[0-9a-f\]\{40\}\$'/);
  assert.match(benchmarkJob, /! "\$BENCHMARK_BASE_SHA" =~ \$full_sha/);
  assert.match(benchmarkJob, /! "\$BENCHMARK_HEAD_SHA" =~ \$full_sha/);
  assert.match(benchmarkJob, /RUN_HEAD_SHA:\s*\$\{\{\s*github\.sha\s*\}\}/);
  assert.match(benchmarkJob, /"\$RUN_HEAD_SHA" != "\$BENCHMARK_HEAD_SHA"/);
  assert.match(benchmarkJob, /run head_sha comes from --ref/);
  assert.match(benchmarkJob, /dispatch with a branch or tag/);

  assert.match(budgetJob, /name:\s*pr-benchmark/);
  assert.match(budgetJob, /--labels-json "\$PR_LABELS_JSON"/);
  assert.match(budgetJob, /\|\| '\[\]'/);
  assert.doesNotMatch(commentJob, /github\.event_name == 'workflow_dispatch'/);
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

  // The snapshot commit job only fires on manual non-main branches; scheduled
  // main runs publish artifacts without trying to push back.
  assert.match(
    commitJob,
    /if:\s*\$\{\{\s*github\.event_name == 'workflow_dispatch' && inputs\.commit_results && startsWith\(github\.ref, 'refs\/heads\/'\) && github\.ref_name != 'main'\s*\}\}/,
  );
  assert.match(commitJob, /contents:\s*write/);
  assert.match(commitJob, /docs\/content\/architecture\/performance-blacksmith\.md/);
  assert.match(commitJob, /bench\/results\/tool-benchmark-latest\.json/);
  assert.match(commitJob, /git commit -m "docs: update blacksmith benchmark snapshot"/);
  assert.match(commitJob, /git push origin HEAD:\$\{\{\s*github\.ref_name\s*\}\}/);
  assert.doesNotMatch(commitJob, /codex/i);
});

test("tool benchmark workflow publishes scheduled artifacts without pushing to protected main", () => {
  const workflow = readRepoFile(".github", "workflows", "tool-benchmark.yml");

  // A weekly cron keeps benchmark artifacts fresh without directly refreshing
  // bench/results/tool-benchmark-latest.json from the protected main branch.
  assert.match(workflow, /\n  schedule:\n/);
  assert.match(workflow, /- cron:\s*"41 5 \* \* 1"/);
});

test("criterion bench workflow runs an A/B micro-benchmark and a dialect guard", () => {
  const workflow = readRepoFile(".github", "workflows", "criterion-bench.yml");
  const abJob = workflowJobBody(workflow, "criterion-ab");
  const guardJob = workflowJobBody(workflow, "dialect-guard");

  // Only runs on PRs and only when Rust or the bench harness changes.
  assert.match(workflow, /\n  pull_request:\n/);
  assert.match(workflow, /paths:\n\s+- "crates\/\*\*"/);
  assert.match(workflow, /- "bench\/criterion-ab\.mjs"/);
  assert.match(workflow, /- "bench\/dialect-guard\.mjs"/);
  assert.match(workflow, /FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*true/);

  for (const [jobName, minutes] of [
    ["criterion-ab", 45],
    ["dialect-guard", 45],
  ] as const) {
    assert.match(
      workflowJobBody(workflow, jobName),
      new RegExp(`timeout-minutes:\\s*${minutes}\\b`),
    );
  }

  // A/B: alternating base/head criterion baselines compared with critcmp into a
  // shared target dir; report-only by default (no threshold blocks the PR).
  assert.match(abJob, /runs-on:\s*blacksmith-32vcpu-ubuntu-2404/);
  assert.match(abJob, /contents:\s*read/);
  assert.doesNotMatch(abJob, /contents:\s*write/);
  assert.match(
    abJob,
    /path:\s*head[\s\S]*ref:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha\s*\}\}/,
  );
  assert.match(
    abJob,
    /path:\s*base[\s\S]*ref:\s*\$\{\{\s*github\.event\.pull_request\.base\.sha\s*\}\}/,
  );
  assert.match(abJob, /uses:\s*\.\/head\/\.github\/actions\/setup-rust-sticky-cache/);
  assert.match(abJob, /cargo install critcmp --version 0\.1\.8 --locked/);
  assert.match(abJob, /node head\/bench\/criterion-ab\.mjs/);
  assert.match(abJob, /--target-dir "\$GITHUB_WORKSPACE\/head\/target"/);

  // Dialect guard: build vize with legacy OFF and ON, then assert byte-identical
  // Vue 3 codegen plus a small A/B timing budget.
  assert.match(guardJob, /runs-on:\s*blacksmith-32vcpu-ubuntu-2404/);
  assert.match(guardJob, /cargo build --profile ci-opt -p vize --target-dir target\/off/);
  assert.match(
    guardJob,
    /cargo build --profile ci-opt -p vize --features legacy --target-dir target\/on/,
  );
  assert.match(guardJob, /node bench\/generate\.mjs "\$DIALECT_GUARD_FILE_COUNT"/);
  assert.match(guardJob, /node bench\/dialect-guard\.mjs/);
  assert.match(guardJob, /--off-bin target\/off\/ci-opt\/vize/);
  assert.match(guardJob, /--on-bin target\/on\/ci-opt\/vize/);
  assert.match(guardJob, /--threshold "\$DIALECT_GUARD_THRESHOLD_PERCENT"/);
});
