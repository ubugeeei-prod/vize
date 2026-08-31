import assert from "node:assert/strict";
import { test } from "node:test";

import {
  buildCommands,
  buildMetadata,
  TOOL_BENCHMARK_RUNNER_LABEL,
} from "../../tools/benchmarks/scripts/compare-tools-metadata.mjs";

const options = {
  runs: 3,
  warmups: 1,
  checkFileCount: 500,
  viteFileCount: 1000,
  nuxtFileCount: 250,
  museaFileCount: 240,
  largeBlocks: 300,
  backend: { corsaPath: null, corsaVersion: null, ready: false },
};

test("Musea corpus size is present in metadata and both reproduction commands", () => {
  assert.deepEqual(buildCommands(3000, options), {
    workflowDispatch:
      "gh workflow run tool-benchmark.yml --ref <branch> -f file_count=3000 -f check_file_count=500 -f vite_file_count=1000 -f nuxt_file_count=250 -f musea_file_count=240 -f large_blocks=300 -f runs=3 -f warmups=1 -f commit_results=true",
    generate: "node tools/benchmarks/scripts/generate.mjs 3000",
    benchmark: `node tools/benchmarks/scripts/compare-tools.mjs --input tools/benchmarks/scripts/__in__ --vize-bin target/release/vize --runs 3 --warmups 1 --check-file-count 500 --vite-file-count 1000 --nuxt-file-count 250 --musea-file-count 240 --large-blocks 300 --runner-label "${TOOL_BENCHMARK_RUNNER_LABEL}" --out tool-benchmark-summary.md --json tool-benchmark-results.json --doc performance-blacksmith.md`,
  });

  const metadata = buildMetadata({
    args: {},
    inputDir: "/tools/benchmarks/scripts/input",
    files: ["App.vue"],
    totalBytes: 100,
    taskList: ["musea"],
    options,
    bins: {},
  });
  assert.equal(metadata.input.museaFileCount, 240);
  assert.match(metadata.commands.workflowDispatch, /-f musea_file_count=240/);
  assert.match(metadata.commands.benchmark, /--musea-file-count 240/);
});
