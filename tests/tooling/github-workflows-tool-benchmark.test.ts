import assert from "node:assert/strict";
import { test } from "node:test";

import {
  hostedOrBlacksmithExact,
  readRepoFile,
  workflowJobBody,
} from "./support/github-workflows.ts";

test("tool benchmark workflow produces docs artifacts, PR comments, and conventional commits", () => {
  const workflow = readRepoFile(".github", "workflows", "tool-benchmark.yml");
  const benchmarkJob = workflowJobBody(workflow, "tool-benchmark");
  const commentJob = workflowJobBody(workflow, "tool-benchmark-comment");
  const commitJob = workflowJobBody(workflow, "tool-benchmark-commit");

  assert.match(workflow, /\n  workflow_dispatch:\n/);
  assert.match(workflow, /commit_results:[\s\S]*type:\s*boolean[\s\S]*default:\s*false/);
  assert.match(workflow, /VIZE_TOOL_BENCH_FILE_COUNT:/);
  assert.match(workflow, /VIZE_TOOL_BENCH_NUXT_FILE_COUNT:/);
  assert.match(workflow, /musea_file_count:[\s\S]*default:\s*"240"/);
  assert.match(workflow, /VIZE_TOOL_BENCH_MUSEA_FILE_COUNT:/);
  assert.match(workflow, /VIZE_TOOL_BENCH_LARGE_BLOCKS:/);
  const runsOn = benchmarkJob.match(/^\s*runs-on:\s*([^#\s]+)(?:\s*#.*)?$/m)?.[1];
  assert.ok(runsOn, "missing runs-on for the tool-benchmark job");
  assert.match(runsOn, hostedOrBlacksmithExact("ubuntu-24.04"));
  assert.match(benchmarkJob, /contents:\s*read/);
  assert.doesNotMatch(benchmarkJob, /contents:\s*write/);
  assert.doesNotMatch(benchmarkJob, /issues:\s*write/);
  assert.match(benchmarkJob, /uses:\s*\.\/\.github\/actions\/setup-moonbit/);
  const rustScriptSetupIndex = benchmarkJob.indexOf("uses: ./.github/actions/setup-rust-script");
  const vitePluginBuildIndex = benchmarkJob.indexOf(
    "run: vp run --workspace-root build:vite-plugin",
  );
  assert.ok(rustScriptSetupIndex >= 0, "tool benchmark must install rust-script");
  assert.ok(vitePluginBuildIndex >= 0, "tool benchmark must build the Vite plugin");
  assert.ok(
    rustScriptSetupIndex < vitePluginBuildIndex,
    "tool benchmark must install rust-script before build:vite-plugin",
  );
  assert.match(benchmarkJob, /vp run --workspace-root build:native/);
  assert.match(benchmarkJob, /vp run --workspace-root build:vite-plugin/);
  assert.match(benchmarkJob, /vp run --workspace-root build:nuxt-stack/);
  assert.match(
    benchmarkJob,
    /node tools\/benchmarks\/scripts\/generate\.mjs "\$VIZE_TOOL_BENCH_FILE_COUNT"/,
  );
  assert.match(benchmarkJob, /node tools\/benchmarks\/scripts\/compare-tools\.mjs/);
  assert.match(benchmarkJob, /--nuxt-file-count "\$VIZE_TOOL_BENCH_NUXT_FILE_COUNT"/);
  assert.match(benchmarkJob, /--musea-file-count "\$VIZE_TOOL_BENCH_MUSEA_FILE_COUNT"/);
  assert.match(benchmarkJob, /--large-blocks "\$VIZE_TOOL_BENCH_LARGE_BLOCKS"/);
  // The label is written into published benchmark metadata, so it has to name
  // the runner the benchmark actually ran on rather than merely be an accepted
  // label: an independent check passes while the two select different pools.
  const activeBenchmarkJob = benchmarkJob.replace(/^\s*#.*$/gm, "");
  const runnerLabel = activeBenchmarkJob.match(/--runner-label "([^"]+)"/)?.[1];
  assert.ok(runnerLabel, "missing --runner-label for the benchmark run");
  assert.match(runnerLabel, hostedOrBlacksmithExact("ubuntu-24.04"));
  assert.equal(runnerLabel, runsOn, "benchmark metadata must record the runner that produced it");
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
    /node tools\/benchmarks\/scripts\/comment-pr\.mjs --body tool-benchmark-summary\.md --comment-key "\$BENCHMARK_COMMENT_KEY"/,
  );

  // The snapshot commit job only fires on manual non-main branches; scheduled
  // main runs publish artifacts without trying to push back.
  assert.match(
    commitJob,
    /if:\s*\$\{\{\s*github\.event_name == 'workflow_dispatch' && inputs\.commit_results && startsWith\(github\.ref, 'refs\/heads\/'\) && github\.ref_name != 'main'\s*\}\}/,
  );
  assert.match(commitJob, /contents:\s*write/);
  assert.match(commitJob, /docs\/content\/architecture\/performance-blacksmith\.md/);
  assert.match(commitJob, /tools\/benchmarks\/results\/tool-benchmark-latest\.json/);
  assert.match(commitJob, /git commit -m "docs: update blacksmith benchmark snapshot"/);
  assert.match(commitJob, /git push origin HEAD:\$\{\{\s*github\.ref_name\s*\}\}/);
  assert.doesNotMatch(commitJob, /codex/i);
});

test("tool benchmark workflow publishes scheduled artifacts without pushing to protected main", () => {
  const workflow = readRepoFile(".github", "workflows", "tool-benchmark.yml");

  // A weekly cron keeps benchmark artifacts fresh without directly refreshing
  // tools/benchmarks/results/tool-benchmark-latest.json from the protected main branch.
  assert.match(workflow, /\n  schedule:\n/);
  assert.match(workflow, /- cron:\s*"41 5 \* \* 1"/);
});
