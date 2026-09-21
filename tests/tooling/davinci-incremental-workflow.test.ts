// P5-9 / P5-1a: the incrementality workflow keeps its shape — TS-42 runs on
// every PR touching the resident tier, over the corpus shard, with the seeded
// stale-cache defect proven caught; TS-43 runs on a Linux and a macOS lane.

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type Step = { name?: string; uses?: string; run?: string };
type Job = {
  "runs-on": string;
  strategy?: { matrix: { platform: Array<{ runner: string; target: string }> } };
  steps: Step[];
};
type Workflow = {
  on: Record<"push" | "pull_request", { paths: string[] }>;
  jobs: Record<string, Job>;
};

const workflow = parseYaml(
  fs.readFileSync(path.join(repoRoot, ".github/workflows/davinci-incremental.yml"), "utf8"),
) as Workflow;

const shard = ["splitpanes", "layoutit-grid", "cssgridgenerator"];
const fixtures =
  "--fixtures tests/_fixtures/_projects --fixtures tests/_fixtures/vue-language-tools";
const driver = "rust-script tools/commands/davinci/incremental-equivalence.rs";

function runs(job: Job): string[] {
  return job.steps.flatMap((step) => (step.run ? step.run.trim().split("\n") : []));
}

test("the incrementality workflow triggers on the resident tier and its keyers", () => {
  for (const event of ["push", "pull_request"] as const) {
    const paths = workflow.on[event].paths;
    for (const required of [
      "crates/vize_resident/**",
      "crates/vize_davinci/**",
      "crates/vize_s2/**",
      "crates/vize_s1_to_s2/**",
      "tools/commands/davinci/incremental-equivalence/**",
      ".github/workflows/davinci-incremental.yml",
    ]) {
      assert.ok(paths.includes(required), `${event} must trigger on ${required}`);
    }
  }
});

test("TS-42 runs over the hydrated corpus shard and proves the seeded defect caught", () => {
  const job = workflow.jobs["incremental-equivalence"];
  assert.deepEqual(runs(job), [
    `git submodule update --init --depth 1 -- ${shard.map((id) => `tests/_fixtures/_git/${id}`).join(" ")}`,
    "cargo test -p vize_resident",
    `${driver} --corpus-shard ${fixtures}`,
    "cargo test -p vize_resident --features seeded-stale-cache --test equivalence",
    `${driver} --corpus-shard ${fixtures} --seeded`,
  ]);
  const source = fs.readFileSync(
    path.join(repoRoot, "tools/commands/davinci/incremental-equivalence.rs"),
    "utf8",
  );
  assert.ok(
    source.includes(
      `const CORPUS_SHARD: [&str; 3] = [${shard.map((id) => `"${id}"`).join(", ")}];`,
    ),
    "the driver's shard must be the hydrated one",
  );
  const scripts = fs
    .readdirSync(path.join(repoRoot, "tools/commands/davinci/incremental-equivalence/scripts"))
    .filter((name) => name.endsWith(".edits"));
  assert.deepEqual(scripts.toSorted(), [
    "01-template-keystrokes.edits",
    "02-above-blocks.edits",
    "03-script-edits.edits",
    "04-style-edits.edits",
    "05-length-preserving.edits",
    "06-config-changes.edits",
  ]);
});

test("TS-43 compares the golden keys on a Linux and a macOS lane", () => {
  const job = workflow.jobs["key-stability"];
  assert.deepEqual(
    job.strategy?.matrix.platform.map(({ target }) => target),
    ["linux-x64-gnu", "darwin-arm64"],
  );
  assert.deepEqual(
    runs(job).filter((line) => line.startsWith("cargo test")),
    ["cargo test -p vize_davinci --test artifact_keys --test key_manifests -- --nocapture"],
  );
});
