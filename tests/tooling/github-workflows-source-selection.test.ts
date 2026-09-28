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
  env?: Record<string, string>;
  steps?: Step[];
  outputs?: Record<string, string>;
  strategy?: { "fail-fast": boolean; matrix: { shard: number[] | string } };
  with?: Record<string, string>;
};
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};
const rust = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml")) as {
  jobs: Record<string, Job>;
};

test("source planning installs the declared Node runtime before TypeScript imports", () => {
  const plan = source.jobs["pr-source-plan"];
  const steps = plan.steps ?? [];
  const setup = steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  const sourcePlan = steps.findIndex((step) => step.name === "Plan source checks");
  const tooling = steps.findIndex((step) => step.name === "Plan tooling test inputs");
  const metadata = steps.findIndex((step) => step.name === "Plan affected Rust crates");
  const toolchain = steps.findIndex((step) => step.uses?.startsWith("dtolnay/rust-toolchain@"));
  assert.ok(setup >= 0 && setup < sourcePlan && sourcePlan < tooling);
  assert.equal(steps[setup].with?.["node-version-file"], "package.json");
  assert.equal(steps[setup].with?.cache, false);
  assert.equal(steps[setup].with?.["run-install"], false);
  assert.ok(sourcePlan < toolchain && toolchain < metadata);
  for (const step of [steps[toolchain], steps[metadata]]) {
    assert.equal(step.if, "${{ steps.plan.outputs.rust == 'true' }}");
  }
  assert.equal(steps[toolchain].with?.toolchain, "1.98.0");
  assert.equal(plan.outputs?.tooling, "${{ steps.tooling-plan.outputs.tooling }}");
  assert.equal(plan.outputs?.["tooling-pure"], "${{ steps.tooling-plan.outputs.tooling-pure }}");
  assert.equal(plan.outputs?.["tooling-full"], "${{ steps.tooling-plan.outputs.tooling-full }}");
  assert.equal(
    plan.outputs?.["tooling-shards"],
    "${{ steps.tooling-plan.outputs.tooling-shards }}",
  );
  assert.equal(
    steps[tooling].env?.TOOLING_TIER,
    "${{ github.event_name == 'merge_group' && 'merge' || 'pr' }}",
  );
  assert.match(steps[tooling].run ?? "", /--github-output/);
  assert.equal(plan.outputs?.["comparison-base"], "${{ steps.comparison.outputs.base }}");
  for (const index of [sourcePlan, tooling, metadata]) {
    assert.equal(steps[index].env?.BASE_SHA, "${{ steps.comparison.outputs.base }}");
  }
  assert.equal(
    source.jobs["pr-rust-source"].with?.["rust-plan"],
    "${{ needs.pr-source-plan.outputs.rust-plan }}",
  );
});

test("selected PR tooling regenerates its plan while merge tooling retains the full receipt path", () => {
  const job = source.jobs["pr-tooling-scripts"];
  const steps = job.steps ?? [];
  assert.deepEqual(job.needs, ["pr-source-plan", "pr-tooling-fast"]);
  assert.equal(
    job.env?.RUN_FULL_TOOLING,
    "${{ github.event_name == 'merge_group' || needs.pr-tooling-fast.outputs.mode != 'fast' }}",
  );
  assert.equal(
    job.strategy?.matrix.shard,
    "${{ fromJSON(needs.pr-source-plan.outputs.tooling-shards) }}",
  );
  assert.equal(job.strategy?.["fail-fast"], false);
  const regenerate = steps.findIndex((step) => step.name === "Regenerate PR tooling plan");
  const build = steps.findIndex((step) => step.name === "Build and install vize CLI");
  const selected = steps.findIndex((step) => step.name === "Test selected PR tooling scripts");
  const full = steps.findIndex((step) => step.name === "Test tooling scripts");
  assert.ok(regenerate >= 0 && regenerate < build && build < selected && selected < full);
  for (const step of [steps[regenerate], steps[selected]]) {
    assert.equal(
      step.if,
      "${{ github.event_name == 'pull_request' && env.RUN_FULL_TOOLING == 'true' && needs.pr-source-plan.outputs.tooling-full == 'true' }}",
    );
  }
  assert.equal(
    steps[regenerate].env?.BASE_SHA,
    "${{ needs.pr-source-plan.outputs.comparison-base }}",
  );
  assert.match(steps[regenerate].run ?? "", /git fetch --no-tags --depth=1 origin "\$BASE_SHA"/);
  assert.match(
    steps[regenerate].run ?? "",
    /--tier pr --output "\$RUNNER_TEMP\/tooling-plan\.json"/,
  );
  assert.equal(steps[selected].env?.VIZE_TOOLING_TEST_PLAN, "${{ runner.temp }}/tooling-plan.json");
  assert.equal(
    steps[selected].env?.SOURCE_LENGTH_BASE_REF,
    "${{ needs.pr-source-plan.outputs.comparison-base }}",
  );
  assert.equal(steps[selected].run, "vp run --workspace-root test:scripts:pr");
  assert.equal(steps[selected].env?.VIZE_LSP_BIN, "${{ github.workspace }}/target/ci/vize");
  assert.equal(steps[selected].env?.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(
    steps[full].if,
    "${{ github.event_name == 'merge_group' && needs.pr-source-plan.outputs.tooling-full == 'true' }}",
  );
  assert.equal(steps[full].run, "vp run --workspace-root test:scripts:merge-shard");
  assert.equal(steps[full].env?.VIZE_TOOLING_MERGE_SHARD, "${{ matrix.shard }}");
  assert.equal(steps[full].env?.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(steps[full].env?.SOURCE_LENGTH_BASE_REF, undefined);
  assert.match(
    steps[build].run ?? "",
    /cargo build --profile ci -p vize && vp exec node tests\/differential\/build-receipt\.mjs/,
  );
  const fast = source.jobs["pr-tooling-fast"];
  const fastSteps = fast.steps ?? [];
  assert.equal(fast.outputs?.mode, "${{ steps.fast-plan.outputs.mode }}");
  const fastPlan = fastSteps.findIndex((step) => step.name === "Plan fast PR tooling inputs");
  const fastBuild = fastSteps.findIndex(
    (step) => step.name === "Build and verify source CLI for fast tooling",
  );
  const fastTests = fastSteps.findIndex(
    (step) => step.name === "Test changed structural PR tooling contracts",
  );
  assert.ok(fastPlan >= 0 && fastPlan < fastBuild && fastBuild < fastTests);
  assert.match(fastSteps[fastBuild].run ?? "", /cargo build --profile ci -p vize/u);
  assert.match(fastSteps[fastBuild].run ?? "", /tests\/differential\/build-receipt\.mjs/u);
  assert.equal(fastSteps[fastTests].env?.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(fastSteps[fastTests].env?.VIZE_LSP_BIN, "${{ github.workspace }}/target/ci/vize");
  assert.ok(source.jobs["source-report"].needs?.includes("pr-tooling-fast"));
  const pure = source.jobs["pr-tooling-pure"];
  const pureSteps = pure.steps ?? [];
  const pureRun = pureSteps.find((step) => step.name === "Test audited pure PR tooling scripts");
  assert.equal(pure.needs, "pr-source-plan");
  assert.equal(pureRun?.run, "vp run --workspace-root test:scripts:pr-pure");
  assert.equal(pureRun?.env?.VIZE_TOOLING_TEST_PLAN, "${{ runner.temp }}/tooling-plan.json");
  assert.ok(
    !pureSteps.some((step) =>
      /cargo build|build:native:test|setup-rust-script/.test(step.run ?? ""),
    ),
  );
});

test("PR browser validation retains tests while every VRT outcome is deferred to merge", () => {
  const steps = source.jobs["pr-playground-test"].steps ?? [];
  const browser = steps.find((step) => step.name === "Run playground tests");
  assert.equal(browser?.if, "${{ env.RUN_PLAYGROUND == 'true' }}");
  for (const name of [
    "Run VRT",
    "Upload VRT report",
    "Upload VRT diff artifacts",
    "Fail if VRT failed",
  ]) {
    const step = steps.find((candidate) => candidate.name === name);
    assert.ok(step?.if?.includes("github.event_name == 'merge_group'"), name);
  }
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
    "pr-rust-fast-clippy",
    "pr-rust-fast-tests",
  ]);
  assert.equal(rust.jobs["rust-source-report"].if, "${{ always() }}");
  assert.equal(
    rust.jobs["rust-source-report"].steps?.at(-1)?.run,
    "node tools/support/compat/github/require-rust-tier.mjs",
  );
});

test("fast Rust source selection is narrow and its two jobs feed the required report", () => {
  const planner = source.jobs["pr-source-plan"];
  assert.equal(planner.outputs?.["rust-fast"], "${{ steps.fast-rust-plan.outputs.rust-fast }}");
  assert.equal(
    planner.outputs?.["rust-fast-plan"],
    "${{ steps.fast-rust-plan.outputs.rust-fast-plan }}",
  );
  assert.equal(
    source.jobs["pr-rust-source"].with?.["rust-fast"],
    "${{ needs.pr-source-plan.outputs.rust-fast == 'true' }}",
  );
  assert.match(rust.jobs["pr-rust-build"].if ?? "", /!inputs\.rust-fast/u);
  assert.match(rust.jobs["pr-rust-shard"].if ?? "", /!inputs\.rust-fast/u);
  for (const job of ["pr-rust-fast-clippy", "pr-rust-fast-tests"]) {
    assert.match(rust.jobs[job].if ?? "", /inputs\.rust-fast/u);
    assert.ok(rust.jobs["rust-source-report"].needs?.includes(job));
  }
  const fastTests = (rust.jobs["pr-rust-fast-tests"].steps ?? [])
    .map((step) => step.run ?? "")
    .join("\n");
  assert.match(fastTests, /cargo nextest archive @packages@ --lib/u);
  assert.match(fastTests, /rust-test-archive\.mjs stamp/u);
  assert.match(fastTests, /rust-test-archive\.mjs verify/u);
  assert.match(fastTests, /cargo nextest run --archive-file/u);
  assert.match(fastTests, /cargo test @packages@ --profile ci --doc/u);
  assert.equal(rust.jobs["rust-source-report"].steps?.at(-1)?.env?.RUST_FAST, "${{ inputs.rust-fast }}");
  assert.equal(rust.jobs["merge-rust-source"].if, "${{ github.event_name == 'merge_group' }}");
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
