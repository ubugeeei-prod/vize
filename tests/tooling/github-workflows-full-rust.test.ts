import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { parse } from "yaml";
import { checkWorkflowJobSources, readRepoFile, root } from "./support/github-workflows.ts";

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
  outputs?: Record<string, string>;
  steps: Step[];
  "continue-on-error"?: boolean;
};
const check = parse(checkWorkflowJobSources()) as {
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
    "node tools/support/compat/github/run-rust-workspace-tests.ts --cargo-profile ci --capture-typechecker-workspace",
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
    "rust-full-workspace-${{ github.sha }}-${{ needs.full-rust-workspace.outputs.producer-attempt }}",
  );
  assert.deepEqual(workspace.outputs, {
    "producer-attempt": "${{ steps.workspace.outputs.producer-attempt }}",
  });
  assert.equal(workspace.env?.RUST_TEST_THREADS, "4");
  const verification = report.steps.find(
    (step) => step.name === "Verify complete same-head workspace execution",
  );
  assert.equal(verification?.env?.SOURCE_SHA, "${{ github.sha }}");
  assert.equal(verification?.env?.NEEDS_JSON, "${{ toJSON(needs) }}");
  assert.equal(
    verification?.run,
    'node tools/support/compat/github/rust-workspace-inventory.ts verify "$SOURCE_SHA" .artifacts/rust-full-workspace "$PRODUCER_ATTEMPT"',
  );
  const observations = report.steps.at(-1);
  assert.deepEqual(observations?.env, {
    SOURCE_SHA: "${{ github.sha }}",
    PRODUCER_ATTEMPT: "${{ needs.full-rust-workspace.outputs.producer-attempt }}",
  });
  assert.equal(observations?.if, undefined);
  assert.notEqual(observations?.["continue-on-error"], true);
  assert.equal(
    observations?.run,
    'node tests/differential/typechecker-workspace.ts verify "$SOURCE_SHA" .artifacts/rust-full-workspace "$GITHUB_RUN_ID" "$PRODUCER_ATTEMPT"',
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
    'node tools/support/compat/github/rust-workspace-inventory.ts compare .artifacts/rust-full-workspace/workspace-inventory-test.json .artifacts/rust-full-workspace/workspace-inventory-ci.json "$PRODUCER_ATTEMPT"',
  );
});

void test("full-only extraction preserves all seventeen complete original job objects", () => {
  const full = parse(readRepoFile(".github/workflows/full-qualification.yml"));
  const original = structuredClone(full.jobs);
  assert.deepEqual(original["nuxt-module-qualification"], {
    name: "Nuxt module qualification",
    uses: "./.github/workflows/nuxt3-module-build.yml",
  });
  delete original["nuxt-module-qualification"];
  const workspace = original["full-rust-workspace"];
  assert.deepEqual(workspace.outputs, {
    "producer-attempt": "${{ steps.workspace.outputs.producer-attempt }}",
  });
  delete workspace.outputs;
  assert.equal(workspace.env.RUST_TEST_THREADS, "4");
  delete workspace.env.RUST_TEST_THREADS;
  const run = workspace.steps.find(
    (step: { name?: string }) => step.name === "Run and inventory every workspace test and doctest",
  );
  assert.equal(run.id, "workspace");
  assert.equal(
    run.run,
    "node tools/support/compat/github/run-rust-workspace-tests.ts --cargo-profile ci --capture-typechecker-workspace",
  );
  delete run.id;
  run.run = "node tools/support/compat/github/run-rust-workspace-tests.ts --cargo-profile ci";
  const report = original["full-rust-report"];
  const download = report.steps.find((step: { uses?: string }) =>
    step.uses?.startsWith("actions/download-artifact@"),
  );
  assert.equal(
    download.with.name,
    "rust-full-workspace-${{ github.sha }}-${{ needs.full-rust-workspace.outputs.producer-attempt }}",
  );
  download.with.name = "rust-full-workspace-${{ github.sha }}-${{ github.run_attempt }}";
  const compare = report.steps.find(
    (step: { name?: string }) =>
      step.name === "Require the requested same-source test and ci inventories",
  );
  assert.deepEqual(compare.env, {
    PRODUCER_ATTEMPT: "${{ needs.full-rust-workspace.outputs.producer-attempt }}",
  });
  assert.equal(
    compare.run,
    'node tools/support/compat/github/rust-workspace-inventory.ts compare .artifacts/rust-full-workspace/workspace-inventory-test.json .artifacts/rust-full-workspace/workspace-inventory-ci.json "$PRODUCER_ATTEMPT"',
  );
  delete compare.env;
  compare.run = compare.run.replace(' "$PRODUCER_ATTEMPT"', "");
  const execution = report.steps.find(
    (step: { name?: string }) => step.name === "Verify complete same-head workspace execution",
  );
  assert.equal(
    execution.env.PRODUCER_ATTEMPT,
    "${{ needs.full-rust-workspace.outputs.producer-attempt }}",
  );
  assert.equal(
    execution.run,
    'node tools/support/compat/github/rust-workspace-inventory.ts verify "$SOURCE_SHA" .artifacts/rust-full-workspace "$PRODUCER_ATTEMPT"',
  );
  delete execution.env.PRODUCER_ATTEMPT;
  execution.run = execution.run.replace(' "$PRODUCER_ATTEMPT"', "");
  assert.deepEqual(report.steps.pop(), {
    name: "Require real workspace typechecker and TS40 observations",
    env: {
      SOURCE_SHA: "${{ github.sha }}",
      PRODUCER_ATTEMPT: "${{ needs.full-rust-workspace.outputs.producer-attempt }}",
    },
    run: 'node tests/differential/typechecker-workspace.ts verify "$SOURCE_SHA" .artifacts/rust-full-workspace "$GITHUB_RUN_ID" "$PRODUCER_ATTEMPT"',
  });
  // Git source 3d93303: exact complete 17 original job objects, in authored order.
  // Only the explicitly asserted observation additions and Nuxt caller invert.
  assert.equal(Object.keys(original).length, 17);
  assert.equal(
    createHash("sha256").update(JSON.stringify(original)).digest("hex"),
    "4bf5e456fd55af86066c8587c7fb04e665ac9cf7c7441912ff7b384582659d41",
  );
});
