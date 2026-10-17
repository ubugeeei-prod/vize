import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { parse } from "yaml";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  uses?: string;
  if?: string;
  run?: string;
  with?: Record<string, unknown>;
  env?: Record<string, string>;
  "continue-on-error"?: boolean;
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
const tail = check.jobs["rust-native-differential"];
const aggregate = check.jobs["clippy-and-test"];
const producerNames = [
  "rust-clippy-portability",
  "rust-davinci-allocations",
  "rust-native-differential",
];
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
  for (const job of [workspace, tail, aggregate, report])
    assert.notEqual(job["continue-on-error"], true);
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

void test("the same-name Rust aggregate requires every independent producer to succeed", () => {
  assert.deepEqual(aggregate.needs, producerNames);
  assert.equal(
    aggregate.if,
    "${{ always() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  for (const name of producerNames) {
    const producer = check.jobs[name];
    assert.equal(producer.needs, undefined, `${name} must start independently`);
    assert.equal(
      producer.if,
      "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
    );
    assert.equal(producer.env?.VIZE_NUXT_CONFIG_ITERATIONS, "100", name);
    assert.notEqual(producer["continue-on-error"], true, name);
  }
  const step = aggregate.steps.find((candidate) =>
    candidate.run?.includes("require-needs-success.ts"),
  );
  assert.ok(step);
  assert.equal(step.run, "node tools/support/compat/github/require-needs-success.ts");
  assert.deepEqual(step.env, { NEEDS_JSON: "${{ toJSON(needs) }}" });
  assert.equal(step.if, undefined);
  assert.notEqual(step["continue-on-error"], true);
  assert.equal(
    tail.steps.find((candidate) => candidate.name === "Test")?.env?.VIZE_TEST_REQUIRE_TSGO,
    "1",
  );
  assert.equal(use(tail, "./.github/actions/retain-sfc-runtime")?.if, "${{ always() }}");
  const succeeded = Object.fromEntries(producerNames.map((name) => [name, { result: "success" }]));
  const execute = (needs: unknown) =>
    spawnSync(
      process.execPath,
      [path.join(root, "tools/support/compat/github/require-needs-success.ts")],
      { encoding: "utf8", env: { ...process.env, NEEDS_JSON: JSON.stringify(needs) } },
    );
  const success = execute(succeeded);
  assert.ifError(success.error);
  assert.equal(success.status, 0, success.stderr);
  for (const name of producerNames) {
    for (const result of ["failure", "cancelled", "skipped", "in_progress"]) {
      const rejected = execute({ ...succeeded, [name]: { result } });
      assert.ifError(rejected.error);
      assert.equal(rejected.status, 1, `${name}: ${result}`);
      assert.ok(rejected.stderr.includes(`${name}: ${result}`), rejected.stderr);
    }
    const missing = { ...succeeded };
    delete missing[name];
    // The workflow's exact needs vector is checked above; a present producer
    // with no result must also fail the real command rather than count green.
    const malformed = execute({ ...missing, [name]: {} });
    assert.equal(malformed.status, 1, name);
    assert.match(malformed.stderr, /reported no result/);
  }
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
