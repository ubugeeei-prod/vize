import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { RUST_TEST_COHORT_IDS } from "../../tools/support/compat/github/rust-test-cohorts.ts";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  env?: Record<string, string>;
  with?: Record<string, string | boolean>;
  "continue-on-error"?: boolean;
};
type Job = {
  name?: string;
  permissions?: Record<string, string>;
  if?: string;
  needs?: string[] | string;
  steps?: Step[];
  outputs?: Record<string, string>;
  strategy?: { "fail-fast": boolean; matrix: { shard: number[] } | string };
  with?: Record<string, string>;
  "continue-on-error"?: boolean;
};
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};
const rust = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml")) as {
  jobs: Record<string, Job>;
  on: {
    workflow_call: {
      inputs: Record<string, { type: string; required: boolean; default?: boolean }>;
    };
  };
};

test("source planning uses Node and offline metadata before selecting affected products", () => {
  const plan = source.jobs["pr-source-plan"];
  const steps = plan.steps ?? [];
  const setup = steps.findIndex((step) => step.uses?.startsWith("actions/setup-node@"));
  const sourcePlan = steps.findIndex((step) => step.name === "Plan source checks");
  assert.ok(setup >= 0 && setup < sourcePlan);
  assert.equal(steps[setup].with?.["node-version-file"], "package.json");
  assert.equal(steps[setup].with?.["package-manager-cache"], false);
  assert.equal(steps[setup].with?.cache, undefined);
  assert.ok(steps.every((step) => !step.uses?.includes("setup-vp@")));
  assert.ok(steps.every((step) => !step.uses?.includes("rust-toolchain@")));
  assert.ok(steps.every((step) => !/\b(?:vp|pnpm|npm)(?=\s|$)/.test(step.run ?? "")));
  assert.equal(steps.filter((step) => step.run?.includes("plan-pr-checks.ts")).length, 1);
  assert.match(steps[sourcePlan].run ?? "", /plan-pr-checks\.ts/);
  assert.ok(steps.every((step) => !step.run?.includes("rust-plan.json")));
  assert.equal(plan.outputs?.["comparison-base"], "${{ steps.comparison.outputs.base }}");
  assert.equal(steps[sourcePlan].env?.BASE_SHA, "${{ steps.comparison.outputs.base }}");
  assert.equal(
    source.jobs["pr-rust-source"].with?.["rust-plan"],
    "${{ needs.pr-source-plan.outputs.rust-plan }}",
  );
  assert.equal(
    source.jobs["pr-js-packages"].with?.["js-plan"],
    "${{ needs.pr-source-plan.outputs.js-plan }}",
  );
  const ciSelection = steps.find(
    (step) => step.name === "Test CI selection when its inputs change",
  );
  assert.equal(ciSelection?.if, "${{ steps.plan.outputs.tooling == 'true' }}");
  assert.equal(
    ciSelection?.run,
    "node --test tests/tooling/ci-iteration-plans.test.ts tests/tooling/pr-required-rust.test.ts tests/tooling/rust-test-cohorts.test.ts tests/tooling/rust-test-archive.test.mjs tests/tooling/rust-build-evidence.test.mjs tests/tooling/release-input-scopes.test.mjs tests/tooling/pinned-fixture-hydration.test.ts tests/tooling/native-preparation-identity.test.ts tests/tooling/publish-npm-package*.test.ts",
  );
});

function requireDailySourceContracts(planWorkflow: typeof source, rustWorkflow: typeof rust) {
  const plan = planWorkflow.jobs["pr-source-plan"];
  assert.equal(plan.outputs?.["source-contracts"], "${{ steps.plan.outputs.source-contracts }}");
  assert.equal(
    planWorkflow.jobs["pr-rust-source"].with?.["run-source-contracts"],
    "${{ needs.pr-source-plan.outputs.source-contracts == 'true' }}",
  );
  assert.deepEqual(rustWorkflow.on.workflow_call.inputs["run-source-contracts"], {
    type: "boolean",
    required: false,
    default: true,
  });
  const job = rustWorkflow.jobs["pr-rust-build"];
  assert.equal(job.if, "${{ inputs.run-rust }}");
  assert.equal(job["continue-on-error"], undefined);
  const steps = job.steps ?? [];
  const contracts = steps.findIndex(
    (step) => step.name === "Require current-tree source, API and reviewed-doc contracts",
  );
  assert.ok(contracts >= 0);
  assert.equal(steps[contracts].if, "${{ inputs.run-source-contracts }}");
  assert.equal(steps[contracts]["continue-on-error"], undefined);
  assert.equal(
    steps[contracts].run,
    "node --test --test-concurrency=1 tests/tooling/davinci-assertion-lint.test.ts tests/tooling/davinci-consumer-migration-surfaces.test.mjs tests/tooling/docs-reviewed-translations.test.ts tests/tooling/typecheck-divergence-report-timeout.test.ts",
  );
  for (const action of [
    "actions/setup-node@",
    "dtolnay/rust-toolchain@",
    "./.github/actions/setup-rust-sticky-cache",
    "./.github/actions/setup-rust-script",
  ]) {
    const setup = steps.findIndex((step) => step.uses?.startsWith(action));
    assert.ok(setup >= 0 && setup < contracts, action);
    if (action.endsWith("setup-rust-script")) {
      assert.equal(steps[setup].if, "${{ inputs.run-source-contracts }}");
      assert.equal(steps[setup]["continue-on-error"], undefined);
    }
  }
  const report = rustWorkflow.jobs["rust-source-report"];
  assert.ok(report.needs?.includes("pr-rust-build"));
  const gate = report.steps?.find((step) => step.name === "Require the complete Rust tier");
  assert.equal(gate?.run, "node tools/support/compat/github/require-rust-tier.ts");
  assert.equal(gate?.["continue-on-error"], undefined);
  assert.ok(
    (plan.steps ?? []).every(
      (step) =>
        !step.uses?.includes("rust-toolchain@") &&
        step.uses !== "./.github/actions/setup-rust-script",
    ),
  );
}

test("the existing required Rust source job runs the original three contracts with strict defaults", () => {
  requireDailySourceContracts(source, rust);
  for (const mutation of [
    "default-false",
    "step-skip",
    "step-advisory",
    "job-skip",
    "job-advisory",
    "report-detach",
    "wrong-command",
  ]) {
    const changed = structuredClone(rust);
    const job = changed.jobs["pr-rust-build"];
    const step = job.steps!.find(
      (candidate) =>
        candidate.name === "Require current-tree source, API and reviewed-doc contracts",
    )!;
    if (mutation === "default-false")
      changed.on.workflow_call.inputs["run-source-contracts"].default = false;
    if (mutation === "step-skip") step.if = "${{ false }}";
    if (mutation === "step-advisory") step["continue-on-error"] = true;
    if (mutation === "job-skip") job.if = "${{ false }}";
    if (mutation === "job-advisory") job["continue-on-error"] = true;
    if (mutation === "report-detach") changed.jobs["rust-source-report"].needs = ["pr-rust-shard"];
    if (mutation === "wrong-command") step.run = "echo contracts passed";
    assert.throws(
      () => requireDailySourceContracts(source, changed),
      assert.AssertionError,
      mutation,
    );
  }
});

test("affected native runtime proofs execute independently of the scheduled tooling campaign", () => {
  const plan = source.jobs["pr-source-plan"];
  assert.equal(
    plan.outputs?.["native-runtime-matrix"],
    "${{ steps.plan.outputs.native-runtime-matrix }}",
  );
  assert.equal(source.jobs["pr-tooling-scripts"], undefined);
  const job = source.jobs["pr-native-runtime"];
  assert.equal(job.needs, "pr-source-plan");
  assert.equal(job.strategy?.["fail-fast"], false);
  assert.equal(
    job.strategy?.matrix,
    "${{ fromJSON(needs.pr-source-plan.outputs.native-runtime-matrix) }}",
  );
  const steps = job.steps ?? [];
  for (const [lane, action] of [
    ["ssr", "test-native-ssr"],
    ["scoped-css", "test-native-scoped-css"],
    ["setup", "test-native-js-setup"],
    ["vapor", "test-native-vapor"],
    ["attributes", "test-native-attribute-values-7502"],
    ["selected-template", "test-native-selected-template"],
  ]) {
    const callers = steps.filter((step) => step.uses === `./.github/actions/${action}`);
    assert.equal(callers.length, 1, lane);
    assert.equal(callers[0].if, `\${{ matrix.lane == '${lane}' }}`);
    assert.notEqual(callers[0]["continue-on-error"], true, lane);
  }
  const install = steps.find((step) => step.name === "Install native runtime oracle dependencies");
  assert.equal(install?.if, "${{ matrix.lane != 'none' }}");
  assert.match(install?.run ?? "", /vp-install\.ts.*npm\/native.*tests/);
  const skip = steps.find((step) => step.name === "Explain unrelated skip");
  assert.equal(skip?.if, "${{ matrix.lane == 'none' }}");
  const report = source.jobs["source-report"];
  assert.equal(report.if, "${{ always() && !contains(needs.*.result, 'cancelled') }}");
  assert.ok(report.needs?.includes("pr-native-runtime"));
  assert.equal(
    report.steps?.at(-1)?.run,
    "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
  );
});

test("opted-in PR and merge browser qualification retain tests and enforce merge VRT outcomes", () => {
  const plan = source.jobs["pr-source-plan"];
  assert.equal(plan.outputs?.heavy, "${{ steps.plan.outputs.heavy }}");
  assert.equal(plan.outputs?.playground, "${{ steps.plan.outputs.playground }}");
  assert.equal(
    plan.outputs?.["canonical-corpus"],
    "${{ steps.plan.outputs.heavy == 'true' && steps.canonical-corpus-plan.outputs.canonical-corpus || 'false' }}",
  );
  const tier = plan.steps?.find((step) => step.name === "Plan source checks");
  assert.equal(tier?.env?.PR_LABELS, "${{ toJSON(github.event.pull_request.labels.*.name) }}");
  assert.match(tier?.run ?? "", /plan-pr-checks\.ts/);
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

test("the Rust report waits for the builder and every named test cohort", () => {
  const shard = rust.jobs["pr-rust-shard"];
  // PR shards build in place beside the lanes; only the merge queue waits for its archive.
  assert.deepEqual(shard.needs, ["merge-rust-source"]);
  assert.equal(shard.strategy?.["fail-fast"], false);
  assert.equal(shard.name, "Rust tests (${{ matrix.name }})");
  assert.equal(shard.strategy?.matrix, "${{ fromJSON(inputs.rust-plan).testMatrix }}");
  const steps = shard.steps ?? [];
  const verify = steps.findIndex((step) => step.name === "Verify Rust archive identity");
  const run = steps.findIndex((step) => step.name === "Run Rust test cohort without rebuilding");
  assert.ok(verify >= 0 && verify < run);
  assert.match(steps[run].run ?? "", /-E "\$NEXTEST_FILTER"/);
  assert.doesNotMatch(steps[run].run ?? "", /--partition/);
  assert.deepEqual(rust.jobs["rust-source-report"].needs, [
    "merge-rust-source",
    "merge-rust-differential",
    "pr-rust-build",
    "pr-rust-shard",
  ]);
  assert.equal(
    rust.jobs["rust-source-report"].if,
    "${{ always() && !contains(needs.*.result, 'cancelled') }}",
  );
  const reportSteps = rust.jobs["rust-source-report"].steps ?? [];
  const gate = reportSteps.findIndex((step) => step.name === "Require the complete Rust tier");
  const gateRuntime = reportSteps.findIndex((step) =>
    step.uses?.startsWith("voidzero-dev/setup-vp@"),
  );
  assert.ok(gateRuntime >= 0 && gateRuntime < gate);
  assert.equal(reportSteps[gateRuntime].if, undefined);
  assert.equal(
    reportSteps.filter((step) => step.uses?.startsWith("voidzero-dev/setup-vp@")).length,
    1,
  );
  assert.deepEqual(reportSteps[gateRuntime].with, {
    "node-version-file": "package.json",
    "run-install": false,
  });
  const select = reportSteps.findIndex(
    (step) => step.name === "Select latest source-bound Rust workers",
  );
  const download = reportSteps.findIndex(
    (step) => step.name === "Download every current Rust test cohort",
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
  assert.deepEqual(check.jobs["pr-source-checks"].permissions, {
    contents: "read",
    actions: "read",
    checks: "read",
  });
  for (const job of [source.jobs["pr-rust-source"], rust.jobs["rust-source-report"]])
    assert.deepEqual(job.permissions, { contents: "read", actions: "read" });
  assert.equal(reportSteps[gate].if, undefined);
  assert.equal(reportSteps[gate].run, "node tools/support/compat/github/require-rust-tier.ts");
  assert.deepEqual(reportSteps[gate].env, {
    RUN_RUST: "${{ inputs.run-rust }}",
    NEEDS_JSON: "${{ toJSON(needs) }}",
  });
});

test("merge Rust timing receipts remain available after workspace failure", () => {
  const steps = rust.jobs["merge-rust-source"].steps ?? [];
  const summary = steps.find((step) => step.name === "Summarize Rust test timing evidence");
  const upload = steps.find((step) => step.name === "Upload Rust test timing evidence");
  assert.equal(summary?.if, "${{ !cancelled() && inputs.run-rust }}");
  assert.equal(upload?.if, "${{ !cancelled() && inputs.run-rust }}");
  assert.equal(summary?.env?.BUILD_OUTCOME, "${{ steps.rust-test-build.outcome }}");
  assert.equal(summary?.env?.RUN_OUTCOME, "skipped");
  assert.equal(summary?.env?.RUN_COMMAND, "parallel Clippy and doctest job");
  const doctests = rust.jobs["pr-rust-build"].steps?.find(
    (step) => step.name === "Run affected Rust doctests",
  );
  assert.match(
    doctests?.run ?? "",
    /run-affected-rust\.ts.*cargo test @packages@ --profile ci --doc/,
  );
  assert.equal(upload?.with?.["if-no-files-found"], "error");
  assert.equal(summary?.run, "node tools/support/compat/github/rust-test-timings.mjs");
});

test("unchanged queue gate hydrates the same pinned fixture for all five full Rust cohorts", () => {
  const job = rust.jobs["pr-rust-shard"];
  assert.ok(job.steps);
  const step = job.steps.find(
    (value) => value.name === "Hydrate the required ElementPlus projection fixture",
  )!;
  assert.equal(step.uses, "./.github/actions/hydrate-pinned-fixture");
  assert.equal(
    step.if,
    "${{ github.event_name == 'merge_group' && contains(fromJSON(inputs.rust-plan).packages, 'vize_canon') }}",
  );
  assert.deepEqual(step.with, { fixture: "tests/_fixtures/_git/element-plus" });
  assert.equal(step["continue-on-error"], undefined);
  assert.deepEqual(job.needs, ["merge-rust-source"]);
  assert.deepEqual(job.strategy, {
    "fail-fast": false,
    matrix: "${{ fromJSON(inputs.rust-plan).testMatrix }}",
  });
  assert.deepEqual(RUST_TEST_COHORT_IDS, [
    "compiler",
    "typechecker",
    "language-server",
    "lint-and-format",
    "cli-and-integrations",
  ]);
  const action = parse(
    readRepoFile(".github", "actions", "hydrate-pinned-fixture", "action.yml"),
  ) as {
    runs: { steps: Record<string, unknown>[] };
  };
  const selection = source.jobs["pr-source-plan"].steps?.find(
    (step) => step.name === "Test CI selection when its inputs change",
  );
  assert.ok(selection);
  assert.equal(selection.if, "${{ steps.plan.outputs.tooling == 'true' }}");
  assert.equal(selection["continue-on-error"], undefined);
  assert.deepEqual(selection.run?.split(/\s+/), [
    "node",
    "--test",
    "tests/tooling/ci-iteration-plans.test.ts",
    "tests/tooling/pr-required-rust.test.ts",
    "tests/tooling/rust-test-cohorts.test.ts",
    "tests/tooling/rust-test-archive.test.mjs",
    "tests/tooling/rust-build-evidence.test.mjs",
    "tests/tooling/release-input-scopes.test.mjs",
    "tests/tooling/pinned-fixture-hydration.test.ts",
    "tests/tooling/native-preparation-identity.test.ts",
    "tests/tooling/publish-npm-package*.test.ts",
  ]);
  assert.equal(action.runs.steps.length, 1);
  assert.equal(
    action.runs.steps[0]!.run,
    'node "$GITHUB_ACTION_PATH/hydrate-pinned-fixture.ts" "$GITHUB_WORKSPACE" "$FIXTURE_PATH"',
  );
  assert.equal(action.runs.steps[0]!["continue-on-error"], undefined);
});
