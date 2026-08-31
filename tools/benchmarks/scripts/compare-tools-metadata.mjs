/**
 * Artifact metadata for the tool comparison benchmark.
 *
 * Every published number must be reproducible from the artifact alone (#3283),
 * so the metadata carries the runner, the entry point (input dir, file count,
 * byte count), the exact versions of every measured binary, and whether the
 * native TypeScript backend was ready — not just the timings.
 */

import os from "node:os";

import { buildFairnessNotes } from "./benchmark-notes.mjs";
import { collectBinaryHashes, collectVersions } from "./benchmark-provenance.mjs";

export const BLACKSMITH_MAX_LABEL = "blacksmith-32vcpu-ubuntu-2404";
export const BLACKSMITH_MAX_SPEC = "32 vCPU / 128 GB RAM / 1.5 TB storage";
export const TOOL_BENCHMARK_RUNNER_LABEL = "ubuntu-24.04";

function githubRunUrl() {
  const server = process.env.GITHUB_SERVER_URL;
  const repo = process.env.GITHUB_REPOSITORY;
  const runId = process.env.GITHUB_RUN_ID;
  if (!server || !repo || !runId) {
    return "";
  }
  return `${server}/${repo}/actions/runs/${runId}`;
}

export function buildCommands(inputFileCount, options) {
  const workflowFlags = [
    `-f file_count=${inputFileCount}`,
    `-f check_file_count=${options.checkFileCount}`,
    `-f vite_file_count=${options.viteFileCount}`,
    `-f nuxt_file_count=${options.nuxtFileCount}`,
    `-f musea_file_count=${options.museaFileCount}`,
    `-f large_blocks=${options.largeBlocks}`,
    `-f runs=${options.runs}`,
    `-f warmups=${options.warmups}`,
    "-f commit_results=true",
  ];
  const compareFlags = [
    "--input tools/benchmarks/scripts/__in__",
    "--vize-bin target/release/vize",
    `--runs ${options.runs}`,
    `--warmups ${options.warmups}`,
    `--check-file-count ${options.checkFileCount}`,
    `--vite-file-count ${options.viteFileCount}`,
    `--nuxt-file-count ${options.nuxtFileCount}`,
    `--musea-file-count ${options.museaFileCount}`,
    `--large-blocks ${options.largeBlocks}`,
    `--runner-label "${TOOL_BENCHMARK_RUNNER_LABEL}"`,
    "--out tool-benchmark-summary.md",
    "--json tool-benchmark-results.json",
    "--doc performance-blacksmith.md",
  ];

  return {
    workflowDispatch: `gh workflow run tool-benchmark.yml --ref <branch> ${workflowFlags.join(" ")}`,
    generate: `node tools/benchmarks/scripts/generate.mjs ${inputFileCount}`,
    benchmark: `node tools/benchmarks/scripts/compare-tools.mjs ${compareFlags.join(" ")}`,
  };
}

export function buildMetadata({ args, inputDir, files, totalBytes, taskList, options, bins }) {
  const runnerLabel = args["runner-label"] ?? process.env.VIZE_BENCH_RUNNER ?? "local";
  const cpus = os.cpus();
  return {
    schemaVersion: 1,
    kind: "tool-comparison",
    generatedAt: new Date().toISOString(),
    commit: {
      sha: args.commit ?? process.env.GITHUB_SHA ?? "",
      ref: args.ref ?? process.env.GITHUB_REF_NAME ?? "",
      repository: args.repository ?? process.env.GITHUB_REPOSITORY ?? "",
      runUrl: args["run-url"] ?? githubRunUrl(),
    },
    runner: {
      label: runnerLabel,
      blacksmithMaxSpec: runnerLabel === BLACKSMITH_MAX_LABEL ? BLACKSMITH_MAX_SPEC : "",
      cpuCount: cpus.length,
      cpuModel: cpus[0]?.model ?? "unknown",
      platform: process.platform,
      arch: process.arch,
      osRelease: os.release(),
      node: process.version,
    },
    versions: collectVersions({ ...bins, corsaVersion: options.backend.corsaVersion }),
    binaries: collectBinaryHashes({ ...bins, corsaPath: options.backend.corsaPath }),
    backend: options.backend,
    input: {
      dir: inputDir,
      fileCount: files.length,
      totalBytes,
      checkFileCount: options.checkFileCount,
      viteFileCount: options.viteFileCount,
      nuxtFileCount: options.nuxtFileCount,
      museaFileCount: options.museaFileCount,
      largeBlocks: options.largeBlocks,
      largeSfcBytes: 0,
    },
    settings: {
      runs: options.runs,
      warmups: options.warmups,
      tasks: taskList,
    },
    commands: buildCommands(files.length, options),
    fairness: buildFairnessNotes(files.length),
  };
}
