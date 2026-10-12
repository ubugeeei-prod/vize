import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

const root = fileURLToPath(new URL("../../", import.meta.url));
const read = (path) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
const workflow = parse(read(".github/workflows/rust-archive-transport-benchmark.yml"));
const producer = parse(read(".github/actions/measure-rust-archive-window/action.yml"));
const consumer = { runs: { steps: workflow.jobs.consumer.steps.slice(1) } };

void test("Rust transport remains a manual experiment with two fresh arms and all four partitions", () => {
  assert.deepEqual(workflow.on, { workflow_dispatch: null, workflow_call: null });
  assert.deepEqual(workflow.permissions, { contents: "read", actions: "read" });
  assert.deepEqual(workflow.jobs.consumer.strategy.matrix, {
    arm: ["baseline", "candidate"],
    shard: [1, 2, 3, 4],
  });
  assert.equal(workflow.jobs.consumer.strategy["fail-fast"], false);
  assert.deepEqual(workflow.jobs.consumer.env, {
    CARGO_BUILD_JOBS: "12",
    VIZE_NUXT_CONFIG_ITERATIONS: "100",
    RUST_TEST_THREADS: "4",
    VIZE_TEST_DISABLE_TSGO: "1",
    NEXTEST_PROFILE: "pr",
  });
  assert.deepEqual(workflow.jobs.compare.needs, ["producer", "consumer"]);
  assert.equal(workflow.jobs.compare.if, "${{ always() && !cancelled() }}");
});

void test("the existing manual bridge admits only the explicit Rust transport component", () => {
  const bridge = parse(read(".github/workflows/ci-walltime-benchmark.yml"));
  const job = bridge.jobs["rust-archive-window"];
  assert.deepEqual(job.permissions, { contents: "read", actions: "read" });
  assert.equal(job.uses, "./.github/workflows/rust-archive-transport-benchmark.yml");
  assert.equal(job["continue-on-error"], undefined);
  assert.equal(bridge.on.workflow_dispatch.inputs.component.default, "all");
  for (const event of ["pull_request", "push", "schedule", "workflow_dispatch"]) {
    for (const component of bridge.on.workflow_dispatch.inputs.component.options) {
      assert.equal(
        runInNewContext(job.if.slice(3, -3), {
          github: { event_name: event },
          inputs: { component },
        }),
        event === "workflow_dispatch" && component === "rust-archive-window",
      );
    }
  }
});

void test("the original whole ZIP, complete tar proof and all raw observations remain durable", () => {
  const steps = producer.runs.steps;
  assert.equal(steps.find((step) => step.id === "original").env.PAYLOAD_ARTIFACT_ID, "11706049605");
  assert.equal(
    steps.find((step) => step.id === "original").env.PAYLOAD_SOURCE_SHA,
    "1f463e566cee76d093b3f0c41ee30b7bd1314cdb",
  );
  const original = steps.find((step) => step.with?.name.startsWith("rust-window-baseline-"));
  assert.equal(original.with["compression-level"], 0);
  assert.equal(original.with["retention-days"], 14);
  assert.equal(original.if, "${{ always() && steps.original.outcome == 'success' }}");
  assert.match(original.with.path, /original\.zip$/);
  const custody = steps.find((step) => step.with?.name.startsWith("rust-window-custody-"));
  assert.equal(custody.if, "${{ always() }}");
  assert.match(custody.with.path, /trial\/\*\.json/);
  assert.match(custody.with.path, /\*\.stderr/);
  assert.match(custody.with.path, /\*\.resources/);
  const raw = consumer.runs.steps.at(-1);
  assert.equal(raw.if, "${{ always() }}");
  assert.match(raw.with.path, /rust-window-worker/);
});

void test("frozen consumer source, profile, fixtures and pinned decoder remain unchanged", () => {
  const steps = consumer.runs.steps;
  assert.equal(workflow.jobs.consumer.steps[0].uses.startsWith("actions/checkout@"), true);
  assert.equal(
    steps.some((step) => step.uses?.startsWith("./")),
    false,
  );
  const checkout = steps.find((step) => step.uses?.startsWith("actions/checkout@"));
  assert.equal(checkout.with.ref, "e057b3dd9e8ebed0dee7f4239a34daa2b410f7a0");
  assert.equal(checkout.with["persist-credentials"], false);
  assert.equal(steps.find((step) => step.uses?.startsWith("dtolnay/")).with.toolchain, "1.99.0");
  assert.equal(
    steps.find((step) => step.uses?.startsWith("taiki-e/")).with.tool,
    "cargo-nextest@0.9.146",
  );
  const commands = steps.map((step) => step.run || "").join("\n");
  assert.match(commands, /rust-test-archive\.mjs verify/);
  assert.match(
    commands,
    /--profile "\$NEXTEST_PROFILE" --partition "hash:\$SHARD\/4" --no-tests=pass/,
  );
  assert.match(commands, /unpartitioned\.json/);
  assert.equal(
    (
      commands.match(
        /rust_archive_workers\.py" execute "\$GITHUB_SHA" cargo nextest (?:list|run)/g,
      ) || []
    ).length,
    3,
  );
  assert.match(commands, /record .*"\$GITHUB_SHA"/);
  assert.doesNotMatch(commands, /export GITHUB_SHA|GITHUB_SHA=/);
  assert.match(commands, /run\.stdout.*run\.stderr/);
  assert.doesNotMatch(
    commands,
    /cargo (?:build|test|nextest archive)|--filter|--ignore-default-filter|--long|--memory/,
  );
  assert.equal(
    steps.filter((step) => step.uses?.startsWith("actions/download-artifact@")).length,
    2,
  );
});

void test("real whole-frame, source, missing-partition and result controls survive Python optimization", () => {
  for (const options of [[], ["-O"]]) {
    const result = spawnSync(
      "python3",
      [...options, "-B", "tools/support/compat/github/rust_archive_controls.py"],
      {
        cwd: root,
        encoding: "utf8",
        timeout: 30_000,
      },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stderr, /Ran 11 tests/);
  }
});
