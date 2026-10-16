import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  uses?: string;
  if?: string;
  run?: string;
  with?: Record<string, unknown>;
  env?: Record<string, string>;
};
type Job = {
  if?: string;
  needs?: string[];
  env?: Record<string, string>;
  steps: Step[];
  "continue-on-error"?: boolean;
};
const check = parse(readRepoFile(".github", "workflows", "check.yml")) as {
  on: { workflow_dispatch: { inputs: Record<string, { default: boolean }> } };
  jobs: Record<string, Job>;
};
const seed = parse(readRepoFile(".github", "workflows", "rust-pr-target-seed.yml")) as {
  jobs: Record<string, Job>;
};
const workspace = check.jobs["full-rust-workspace"];
const tail = check.jobs["clippy-and-test"];
const report = check.jobs["full-rust-report"];
const use = (job: Job, action: string) => job.steps.find((step) => step.uses?.startsWith(action));

void test("full workspace shares the seeded compiler, linker, target path and ci profile", () => {
  const producer = seed.jobs["rust-pr-target-seed"];
  for (const action of ["dtolnay/rust-toolchain@", "wild-linker/action@"])
    assert.deepEqual(use(workspace, action), use(producer, action));
  assert.deepEqual(
    use(workspace, "./.github/actions/setup-rust-sticky-cache")?.with,
    use(producer, "./.github/actions/setup-rust-sticky-cache")?.with,
  );
  assert.equal(workspace.env?.CARGO_BUILD_JOBS, producer.env?.CARGO_BUILD_JOBS);
  const channel = readRepoFile("rust-toolchain.toml").match(/^channel = "([^"]+)"/m)?.[1];
  assert.equal(use(workspace, "dtolnay/rust-toolchain@")?.with?.toolchain, channel);
  const run = workspace.steps.find(
    (step) => step.name === "Run and inventory every workspace test and doctest",
  );
  assert.equal(run?.if, "${{ !cancelled() }}");
  assert.equal(
    run?.run,
    "node tools/support/compat/github/run-rust-workspace-tests.ts --cargo-profile ci",
  );
  assert.equal(use(workspace, "taiki-e/install-action@")?.with?.tool, "cargo-nextest@0.9.146");
  // Existing benchmark callers retain their original default build profile.
  assert.match(
    readRepoFile("tools", "support", "compat", "github", "run-rust-workspace-tests.ts"),
    /"cargo-profile": \{ type: "string", default: "test" \}/,
  );
});

void test("full workspace retains native runtimes, frozen dependencies and result artifacts", () => {
  assert.equal(workspace.env?.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(workspace.env?.VIZE_NUXT_CONFIG_ITERATIONS, "100");
  assert.equal(workspace.env?.VIZE_TEST_DISABLE_TSGO, undefined);
  assert.deepEqual(
    workspace.steps.find(
      (step) => step.name === "Install Pkl CLI and hydrate pinned native fixture",
    ),
    tail.steps.find((step) => step.name === "Install Pkl CLI and hydrate pinned native fixture"),
  );
  assert.equal(use(workspace, "voidzero-dev/setup-vp@")?.with?.["run-install"], true);
  for (const action of [
    "./.github/actions/upload-rust-full-workspace-results",
    "./.github/actions/retain-sfc-runtime",
  ])
    assert.equal(use(workspace, action)?.if, "${{ always() }}");
  const upload = parse(
    readRepoFile(".github", "actions", "upload-rust-full-workspace-results", "action.yml"),
  ) as { runs: { steps: Step[] } };
  assert.equal(
    upload.runs.steps[0].with?.name,
    "rust-full-workspace-${{ github.sha }}-${{ github.run_attempt }}",
  );
  assert.equal(upload.runs.steps[0].with?.path, "target/nextest/full/");
});

void test("independent native tail cannot hide a missing or failed workspace execution", () => {
  assert.equal(workspace.needs, undefined);
  assert.equal(tail.needs, undefined);
  assert.equal(
    workspace.if,
    "${{ !cancelled() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  assert.equal(
    report.if,
    "${{ always() && !cancelled() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  assert.deepEqual(report.needs, ["full-rust-workspace", "clippy-and-test"]);
  for (const job of [workspace, tail, report]) assert.notEqual(job["continue-on-error"], true);
  assert.equal(
    use(tail, "./.github/actions/test-rust-workspace-differential")?.with?.[
      "workspace-already-tested"
    ],
    "true",
  );
  const download = use(report, "actions/download-artifact@");
  assert.equal(
    download?.with?.name,
    "rust-full-workspace-${{ github.sha }}-${{ github.run_attempt }}",
  );
  const verification = report.steps.at(-1);
  assert.equal(verification?.env?.SOURCE_SHA, "${{ github.sha }}");
  assert.equal(verification?.env?.NEEDS_JSON, "${{ toJSON(needs) }}");
  assert.equal(
    verification?.run,
    'node tools/support/compat/github/rust-workspace-inventory.ts verify "$SOURCE_SHA" .artifacts/rust-full-workspace',
  );
});

void test("expensive original-profile comparison is opt-in and requested parity fails closed", () => {
  assert.equal(check.on.workflow_dispatch.inputs["compare-rust-test-profile"].default, false);
  const original = workspace.steps.find(
    (step) => step.name === "Run the original test profile for same-source comparison",
  );
  assert.equal(
    original?.if,
    "${{ !cancelled() && github.event_name == 'workflow_dispatch' && inputs.compare-rust-test-profile }}",
  );
  assert.equal(
    original?.run,
    "node tools/support/compat/github/run-rust-workspace-tests.ts --cargo-profile test --output target/nextest/full/workspace-reference-timing.json",
  );
  const compare = report.steps.find(
    (step) => step.name === "Require the requested same-source test and ci inventories",
  );
  assert.equal(
    compare?.if,
    "${{ github.event_name == 'workflow_dispatch' && inputs.compare-rust-test-profile }}",
  );
  assert.equal(
    compare?.run,
    "node tools/support/compat/github/rust-workspace-inventory.ts compare .artifacts/rust-full-workspace/workspace-inventory-test.json .artifacts/rust-full-workspace/workspace-inventory-ci.json",
  );
});
