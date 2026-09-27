import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  env?: Record<string, string>;
  with?: Record<string, string | boolean>;
};
type Job = {
  if?: string;
  needs?: string[] | string;
  steps?: Step[];
  outputs?: Record<string, string>;
  strategy?: { "fail-fast": boolean; matrix: { shard: number[] } };
  with?: Record<string, string>;
};
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};
const rust = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml")) as {
  jobs: Record<string, Job>;
};

test("source planning installs declared runtimes before affected Rust metadata", () => {
  const plan = source.jobs["pr-source-plan"];
  const steps = plan.steps ?? [];
  const setup = steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  const sourcePlan = steps.findIndex((step) => step.name === "Plan source checks");
  const metadata = steps.findIndex((step) => step.name === "Plan affected Rust crates");
  const toolchain = steps.findIndex((step) => step.uses?.startsWith("dtolnay/rust-toolchain@"));
  assert.ok(setup >= 0 && setup < sourcePlan);
  assert.equal(steps[setup].with?.["node-version-file"], "package.json");
  assert.equal(steps[setup].with?.cache, false);
  assert.equal(steps[setup].with?.["run-install"], false);
  assert.ok(sourcePlan < toolchain && toolchain < metadata);
  for (const step of [steps[toolchain], steps[metadata]]) {
    assert.equal(step.if, "${{ steps.plan.outputs.rust == 'true' }}");
  }
  assert.equal(steps[toolchain].with?.toolchain, "1.98.0");
  assert.equal(plan.outputs?.["comparison-base"], "${{ steps.comparison.outputs.base }}");
  for (const index of [sourcePlan, metadata]) {
    assert.equal(steps[index].env?.BASE_SHA, "${{ steps.comparison.outputs.base }}");
  }
  assert.equal(
    source.jobs["pr-rust-source"].with?.["rust-plan"],
    "${{ needs.pr-source-plan.outputs.rust-plan }}",
  );
});

test("the Rust report waits for the builder and all four independently executing shards", () => {
  const shard = rust.jobs["pr-rust-shard"];
  assert.deepEqual(shard.needs, ["pr-rust-build", "merge-rust-source"]);
  assert.equal(shard.strategy?.["fail-fast"], false);
  assert.deepEqual(shard.strategy?.matrix.shard, [1, 2, 3, 4]);
  const steps = shard.steps ?? [];
  const verify = steps.findIndex((step) => step.name === "Verify Rust archive identity");
  const run = steps.findIndex((step) => step.name === "Run Rust test shard without rebuilding");
  assert.ok(verify >= 0 && verify < run);
  assert.match(steps[run].run ?? "", /--partition "hash:\$SHARD\/4"/);
  assert.deepEqual(rust.jobs["rust-source-report"].needs, [
    "merge-rust-source",
    "pr-rust-build",
    "pr-rust-shard",
  ]);
  assert.equal(rust.jobs["rust-source-report"].if, "${{ always() }}");
  assert.equal(
    rust.jobs["rust-source-report"].steps?.at(-1)?.run,
    "node tools/support/compat/github/require-rust-tier.mjs",
  );
});

test("merge Rust timing receipts remain available after workspace failure", () => {
  const steps = rust.jobs["merge-rust-source"].steps ?? [];
  const summary = steps.find((step) => step.name === "Summarize Rust test timing evidence");
  const upload = steps.find((step) => step.name === "Upload Rust test timing evidence");
  assert.equal(summary?.if, "${{ always() && inputs.run-rust }}");
  assert.equal(upload?.if, "${{ always() && inputs.run-rust }}");
  assert.equal(summary?.env?.BUILD_OUTCOME, "${{ steps.rust-test-build.outcome }}");
  assert.equal(summary?.env?.RUN_OUTCOME, "${{ steps.rust-test-run.outcome }}");
  assert.equal(upload?.with?.["if-no-files-found"], "error");
  assert.equal(summary?.run, "node tools/support/compat/github/rust-test-timings.mjs");
});
