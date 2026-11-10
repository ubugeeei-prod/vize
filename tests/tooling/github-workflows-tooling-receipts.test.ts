import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";

import { checkWorkflowJobSources, readRepoFile } from "./support/github-workflows.ts";

type Step = {
  id?: string;
  name?: string;
  run?: string;
  uses?: string;
  if?: string;
  with?: Record<string, string>;
  env?: Record<string, string>;
  "continue-on-error"?: boolean;
};
type Workflow = {
  jobs: Record<
    string,
    {
      steps?: Step[];
      needs?: string | string[];
      uses?: string;
      if?: string;
      env?: Record<string, string>;
    }
  >;
};

test("scheduled full tooling requires the original current-source CLI receipt before testing", () => {
  const workflow = parse(checkWorkflowJobSources()) as Workflow;
  const steps = (
    parse(readRepoFile(".github", "actions", "test-full-tooling", "action.yml")) as {
      runs: { steps: Step[] };
    }
  ).runs.steps;
  const build = steps.findIndex((step) => step.uses === "./.github/actions/restore-source-cli");
  const producer =
    (parse(readRepoFile(".github", "workflows", "full-source-cli.yml")) as Workflow).jobs.cli
      .steps ?? [];
  const originalBuild = producer.find((step) =>
    /cargo build --profile ci -p vize/.test(step.run ?? ""),
  );
  assert.ok(originalBuild);
  const tests = steps.findIndex(
    (step) => step.run === "vp run --workspace-root test:scripts:planned",
  );
  assert.ok(build >= 0 && tests > build);
  assert.match(
    originalBuild.run ?? "",
    /cargo build --profile ci -p vize && vp exec node tests\/differential\/build-receipt\.ts/,
  );
  assert.match(
    originalBuild.run ?? "",
    /build-receipt\.ts && cat target\/ci\/vize\.differential-build\.json/,
  );
  for (const step of [originalBuild, steps[build], steps[tests]]) {
    assert.notEqual(step["continue-on-error"], true);
    assert.equal(step.if, undefined);
  }
  assert.equal(workflow.jobs["full-source-cli"].uses, "./.github/workflows/full-source-cli.yml");
  assert.equal(
    workflow.jobs["full-source-cli"].if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  const consumer = workflow.jobs["test-scripts"];
  assert.deepEqual(consumer.needs, ["full-source-cli", "tooling-plan"]);
  assert.equal(
    consumer.if,
    "${{ !cancelled() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  const worker = consumer.steps?.find(
    (step) => step.uses === "./.github/actions/test-full-tooling",
  );
  assert.equal(worker?.with?.producer, "${{ toJSON(needs.full-source-cli.outputs) }}");
  assert.equal(
    consumer.env?.VIZE_SOURCE_CLI_RECEIPT_SHA256,
    "${{ needs.full-source-cli.outputs.receipt-sha256 }}",
  );
  assert.deepEqual(steps[build].with, { producer: "${{ inputs.producer }}" });
  for (const step of steps.slice(0, tests)) {
    assert.equal(step.env?.VIZE_LSP_BIN, undefined);
    assert.equal(step.env?.VIZE_LSP_REQUIRE_SOURCE_BUILD, undefined);
  }
  assert.equal(steps[tests].env?.VIZE_LSP_BIN, "${{ github.workspace }}/target/ci/vize");
  assert.equal(steps[tests].env?.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  const restore = parse(readRepoFile(".github", "actions", "restore-source-cli", "action.yml")) as {
    runs: { steps: Step[] };
  };
  const command = restore.runs.steps.at(-1)?.run ?? "";
  assert.match(command, /source-cli-artifact\.ts restore/);
  assert.match(
    command,
    /"\$SOURCE_CLI_SHA256" "\$SOURCE_CLI_PRODUCER_ATTEMPT" "\$SOURCE_CLI_RECEIPT_SHA256"/,
  );
  assert.match(command, /cat target\/ci\/vize\.differential-build\.json/);
  assert.doesNotMatch(command, /build-receipt\.ts/);
  for (const step of restore.runs.steps) assert.notEqual(step["continue-on-error"], true);
});

test("PR product lanes stay independent of the affected tooling producer", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as Workflow;
  assert.equal(workflow.jobs["pr-source-cli"], undefined);
  assert.equal(workflow.jobs["pr-tooling-scripts"], undefined);
  for (const name of [
    "pr-rust-source",
    "pr-js-packages",
    "pr-native-runtime",
    "pr-playground-test",
  ])
    assert.equal(workflow.jobs[name].needs, "pr-source-plan", name);
  // Only the affected tooling lane builds and consumes the source CLI.
  for (const [name, job] of Object.entries(workflow.jobs)) {
    if (job.uses === "./.github/workflows/full-source-cli.yml")
      assert.equal(name, "affected-tooling-cli");
    for (const step of job.steps ?? []) {
      assert.notEqual(step.uses, "./.github/actions/test-full-tooling");
      assert.notEqual(step.uses, "./.github/actions/restore-source-cli");
      if (step.uses === "./.github/actions/test-affected-tooling")
        assert.equal(name, "affected-tooling");
    }
  }
});

test("the real tooling report requires the source CLI and every selected worker", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as Workflow;
  const report = workflow.jobs["affected-tooling-report"];
  assert.deepEqual(report.needs, ["pr-source-plan", "affected-tooling-cli", "affected-tooling"]);
  const step = report.steps?.find((item) => item.name === "Require every selected tooling worker");
  assert.ok(step?.run);
  assert.deepEqual(step.env, {
    SELECTED: "${{ needs.pr-source-plan.outputs.affected-tooling }}",
    CLI: "${{ needs.affected-tooling-cli.result }}",
    WORKERS: "${{ needs.affected-tooling.result }}",
  });
  assert.equal(step["continue-on-error"], undefined);
  const accepted = new Set(["false/success/skipped", "true/success/success"]);
  for (const selected of ["true", "false", ""])
    for (const cli of ["success", "skipped", "failure", "cancelled", ""])
      for (const workers of ["success", "skipped", "failure", "cancelled", ""]) {
        const result: SpawnSyncReturns<string> = spawnSync(
          "bash",
          ["-euo", "pipefail", "-c", step.run],
          {
            encoding: "utf8",
            env: {
              ...process.env,
              SELECTED: selected,
              CLI: cli,
              WORKERS: workers,
            },
          },
        );
        assert.ifError(result.error);
        const key = [selected, cli, workers].join("/");
        assert.equal(result.status === 0, accepted.has(key), `${key}: ${result.stderr}`);
      }
});

test("affected tooling builds the native addon only for a shard that reads it", () => {
  const action = parse(
    readRepoFile(".github", "actions", "test-affected-tooling", "action.yml"),
  ) as { runs: { steps: Step[] } };
  const plan = action.runs.steps.find((step) => step.id === "plan");
  assert.match(plan?.run ?? "", /tooling-shard-native-build\.mjs/);
  assert.match(plan?.run ?? "", /native-build=\$native_build/);
  const native = action.runs.steps.find(
    (step) => step.name === "Build the native addon required by this shard",
  );
  assert.deepEqual(native, {
    name: "Build the native addon required by this shard",
    if: "${{ steps.plan.outputs.native-build == 'true' }}",
    shell: "bash",
    run: "vp run --workspace-root build:native:test",
  });
  const tests = action.runs.steps.find((step) => step.name === "Test affected tooling scripts");
  assert.equal(
    tests?.run,
    "node tools/commands/ci/verify-tool-layout.ts && node tools/support/compat/github/run-tooling-tests.mjs",
  );
  assert.doesNotMatch(tests?.run ?? "", /build:native:test|test:scripts:pr/);
});
