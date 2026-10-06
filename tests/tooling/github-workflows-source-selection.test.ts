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
  permissions?: Record<string, string>;
  if?: string;
  needs?: string[] | string;
  steps?: Step[];
  outputs?: Record<string, string>;
  strategy?: { "fail-fast": boolean; matrix: { shard: number[] } | string };
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
  assert.equal(
    plan.outputs?.["tooling-matrix"],
    "${{ steps.tooling-plan.outputs.tooling-matrix }}",
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

test("every isolated tooling runner regenerates its tier and retains the full merge receipt path", () => {
  const job = source.jobs["pr-tooling-scripts"];
  assert.equal(job.strategy?.["fail-fast"], false);
  assert.equal(
    job.strategy?.matrix,
    "${{ fromJSON(needs.pr-source-plan.outputs.tooling-matrix) }}",
  );
  const steps = job.steps ?? [];
  const regenerate = steps.findIndex((step) => step.name === "Regenerate tooling plan");
  const dependencies = steps.findIndex(
    (step) => step.name === "Install fixture and JS dependencies",
  );
  const pkl = steps.findIndex(
    (step) => step.name === "Prepare checksum-pinned Pkl and upstream lint dependencies",
  );
  const build = steps.findIndex((step) => step.name === "Build and install vize CLI");
  const selected = steps.findIndex((step) => step.name === "Test selected PR tooling scripts");
  const full = steps.findIndex((step) => step.name === "Test tooling scripts");
  assert.ok(regenerate >= 0 && regenerate < build && build < selected && selected < full);
  assert.ok(regenerate < dependencies && dependencies < pkl && pkl < build);
  assert.equal(steps[pkl].if, "${{ needs.pr-source-plan.outputs.tooling == 'true' }}");
  assert.equal(steps[pkl].uses, "./.github/actions/install-formatter-css-browser");
  const preparation = parse(
    readRepoFile(".github", "actions", "install-formatter-css-browser", "action.yml"),
  ) as { runs: { steps: Step[] } };
  assert.equal(preparation.runs.steps[0].if, "env.VIZE_TOOLING_TEST_PLAN != ''");
  assert.equal(
    preparation.runs.steps[0].run,
    "node tools/support/compat/github/prepare-pkl-schema.mjs && node tools/support/compat/github/prepare-vue-benchmarks.mjs",
  );
  assert.equal(steps[pkl].env?.VIZE_TOOLING_TEST_PLAN, "${{ runner.temp }}/tooling-plan.json");
  assert.equal(
    steps[pkl].env?.VIZE_TOOLING_TEST_TIER,
    "${{ github.event_name == 'merge_group' && 'merge' || 'pr' }}",
  );
  assert.equal(
    steps[pkl].env?.VIZE_TOOLING_TEST_SHARD,
    "${{ format('{0}/{1}', matrix.index, matrix.total) }}",
  );
  for (const step of [steps[selected]]) {
    assert.equal(
      step.if,
      "${{ github.event_name == 'pull_request' && needs.pr-source-plan.outputs.tooling == 'true' }}",
    );
  }
  assert.equal(steps[regenerate].if, "${{ needs.pr-source-plan.outputs.tooling == 'true' }}");
  assert.equal(
    steps[regenerate].env?.TOOLING_TIER,
    "${{ github.event_name == 'merge_group' && 'merge' || 'pr' }}",
  );
  assert.equal(
    steps[regenerate].env?.BASE_SHA,
    "${{ needs.pr-source-plan.outputs.comparison-base }}",
  );
  assert.match(steps[regenerate].run ?? "", /git fetch --no-tags --depth=1 origin "\$BASE_SHA"/);
  assert.match(
    steps[regenerate].run ?? "",
    /--tier "\$TOOLING_TIER" --output "\$RUNNER_TEMP\/tooling-plan\.json"/,
  );
  assert.equal(steps[selected].env?.VIZE_TOOLING_TEST_PLAN, "${{ runner.temp }}/tooling-plan.json");
  assert.equal(steps[selected].env?.VIZE_TOOLING_TEST_TIER, "pr");
  assert.equal(
    steps[selected].env?.VIZE_TOOLING_TEST_SHARD,
    "${{ format('{0}/{1}', matrix.index, matrix.total) }}",
  );
  assert.equal(
    steps[selected].env?.SOURCE_LENGTH_BASE_REF,
    "${{ needs.pr-source-plan.outputs.comparison-base }}",
  );
  assert.equal(steps[selected].run, "vp run --workspace-root test:scripts:pr");
  assert.equal(
    steps[full].if,
    "${{ github.event_name == 'merge_group' && needs.pr-source-plan.outputs.tooling == 'true' }}",
  );
  assert.equal(steps[full].run, "vp run --workspace-root test:scripts:planned");
  assert.equal(steps[full].env?.SOURCE_LENGTH_BASE_REF, undefined);
  assert.equal(steps[full].env?.VIZE_TOOLING_TEST_PLAN, "${{ runner.temp }}/tooling-plan.json");
  assert.equal(steps[full].env?.VIZE_TOOLING_TEST_TIER, "merge");
  assert.equal(
    steps[full].env?.VIZE_TOOLING_TEST_SHARD,
    "${{ format('{0}/{1}', matrix.index, matrix.total) }}",
  );
  assert.equal(steps[full].env?.VIZE_LSP_BIN, "${{ github.workspace }}/target/ci/vize");
  assert.equal(steps[full].env?.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(source.jobs["source-report"].if, "${{ always() }}");
  assert.ok(source.jobs["source-report"].needs?.includes("pr-tooling-scripts"));
  assert.equal(
    source.jobs["source-report"].steps?.at(-1)?.run,
    "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
  );
  assert.match(
    steps[build].run ?? "",
    /cargo build --profile ci -p vize && vp exec node tests\/differential\/build-receipt\.mjs/,
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
  assert.deepEqual(
    typeof shard.strategy?.matrix === "object" ? shard.strategy.matrix.shard : undefined,
    [1, 2, 3, 4],
  );
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
  const reportSteps = rust.jobs["rust-source-report"].steps ?? [];
  const gate = reportSteps.findIndex((step) => step.name === "Require the complete Rust tier");
  const select = reportSteps.findIndex(
    (step) => step.name === "Select latest source-bound Rust workers",
  );
  const download = reportSteps.findIndex(
    (step) => step.name === "Download the four current full Rust workers",
  );
  const reconcile = reportSteps.findIndex(
    (step) => step.name === "Require all registered typechecker observations",
  );
  assert.ok(gate >= 0 && gate < select && select < download && download < reconcile);
  assert.equal(reportSteps[select].if, reportSteps[download].if);
  assert.equal(reportSteps[reconcile].if, reportSteps[download].if);
  assert.equal(
    reportSteps[download].with?.["artifact-ids"],
    "${{ steps.rust-workers.outputs.artifact-ids }}",
  );
  assert.equal(reportSteps[download].with?.pattern, undefined);
  assert.equal(reportSteps[download].with?.["github-token"], "${{ github.token }}");
  assert.equal(reportSteps[download].with?.["run-id"], "${{ github.run_id }}");
  assert.equal(reportSteps[download].with?.["merge-multiple"], false);
  assert.equal(reportSteps[download].with?.["digest-mismatch"], "error");
  assert.match(reportSteps[select].run ?? "", /select-rust-workers\.mjs select/);
  assert.match(
    reportSteps[reconcile].run ?? "",
    /select-rust-workers\.mjs verify .*\n.*typechecker-shards\.ts aggregate/,
  );
  const check = parse(readRepoFile(".github", "workflows", "check.yml"));
  for (const job of [
    check.jobs["pr-source-checks"],
    source.jobs["pr-rust-source"],
    rust.jobs["rust-source-report"],
  ])
    assert.deepEqual(job.permissions, { contents: "read", actions: "read" });
  assert.equal(reportSteps[gate].if, undefined);
  assert.equal(reportSteps[gate].run, "node tools/support/compat/github/require-rust-tier.mjs");
  assert.deepEqual(reportSteps[gate].env, {
    RUN_RUST: "${{ inputs.run-rust }}",
    NEEDS_JSON: "${{ toJSON(needs) }}",
  });
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
