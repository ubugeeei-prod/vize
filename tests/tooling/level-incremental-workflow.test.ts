// P5-9 / P5-1a: the incrementality workflow keeps its shape — TS-42 runs on
// every matching main/davinci push, over the corpus shard, with the seeded
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
  on: { push: { branches: string[]; paths: string[] }; workflow_dispatch: unknown };
  jobs: Record<string, Job>;
};

const workflow = parseYaml(
  fs.readFileSync(path.join(repoRoot, ".github/workflows/davinci-incremental.yml"), "utf8"),
) as Workflow;

const shard = ["splitpanes", "layoutit-grid", "cssgridgenerator"];
const fixtures =
  "--fixtures tests/_fixtures/_projects --fixtures tests/_fixtures/vue-language-tools";
const driver = "node tools/commands/davinci/incremental-equivalence.ts";

function runs(job: Job): string[] {
  return job.steps.flatMap((step) => (step.run ? step.run.trim().split("\n") : []));
}

test("the incrementality workflow triggers on the resident tier and its keyers", () => {
  assert.deepEqual(workflow.on.push.branches, ["main", "davinci"]);
  assert.ok(Object.hasOwn(workflow.on, "workflow_dispatch"));
  assert.ok(!Object.hasOwn(workflow.on, "pull_request"));
  const paths = workflow.on.push.paths;
  for (const required of [
    "crates/vize_incremental/**",
    "davinci/vize_l0/**",
    "davinci/vize_l2/**",
    "davinci/vize_l1_to_l2/**",
    "tools/commands/davinci/incremental-equivalence/**",
    ".github/workflows/davinci-incremental.yml",
  ]) {
    assert.ok(paths.includes(required), `push must trigger on ${required}`);
  }
});

test("TS-42 runs over the hydrated corpus shard and proves the seeded defect caught", () => {
  const job = workflow.jobs["incremental-equivalence"];
  assert.ok(
    job.steps.every((step) => !step.run?.includes("\n")),
    "workflow commands belong in one named step instead of an inline script",
  );
  assert.equal(
    job.steps.some((step) => step.uses === "./.github/actions/setup-rust-script"),
    false,
  );
  assert.ok(job.steps.some((step) => step.uses?.startsWith("voidzero-dev/setup-vp@")));
  assert.deepEqual(runs(job), [
    `git submodule update --init --depth 1 -- ${shard.map((id) => `tests/_fixtures/_git/${id}`).join(" ")}`,
    "cargo test -p vize_incremental",
    "cargo test -p vize_maestro production_alpha_corpus_edits_match_clean -- --ignored --nocapture",
    `${driver} --corpus-shard ${fixtures}`,
    "cargo test -p vize_incremental --features seeded-stale-cache --test equivalence",
    `${driver} --corpus-shard ${fixtures} --seeded`,
  ]);
  const source = fs.readFileSync(
    path.join(repoRoot, "tools/commands/davinci/incremental-equivalence.ts"),
    "utf8",
  );
  assert.ok(
    source.includes(`const CORPUS_SHARD = [${shard.map((id) => `"${id}"`).join(", ")}] as const;`),
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
  assert.ok(
    job.steps.every((step) => !step.run?.includes("\n")),
    "each golden-key command needs an independently visible step",
  );
  assert.deepEqual(
    job.strategy?.matrix.platform.map(({ target }) => target),
    ["linux-x64-gnu", "darwin-arm64"],
  );
  assert.deepEqual(
    runs(job).filter((line) => line.startsWith("cargo test")),
    [
      "cargo test -p vize_l0 --test artifact_keys --test key_manifests --test hash_domains -- --nocapture",
      "cargo test -p vize_l2 --test sfc_summary --test global_summary -- --nocapture",
    ],
  );
});
