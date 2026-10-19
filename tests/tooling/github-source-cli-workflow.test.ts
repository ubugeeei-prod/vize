import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { canonicalSourceReport } from "../../tools/support/compat/github/canonical-corpus-selection.mjs";
import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";
import { checkWorkflowJobSources, readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
  env?: Record<string, string>;
  with?: Record<string, string | boolean | number>;
  "continue-on-error"?: boolean;
};
type Job = {
  needs?: string | string[];
  uses?: string;
  if?: string;
  steps?: Step[];
  with?: Record<string, string>;
  env?: Record<string, string>;
  outputs?: Record<string, string>;
};
type Action = { inputs: Record<string, { required?: boolean }>; runs: { steps: Step[] } };
const full = parse(checkWorkflowJobSources()) as {
  jobs: Record<string, Job>;
};
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};
const producer = parse(readRepoFile(".github", "workflows", "full-source-cli.yml")) as {
  jobs: Record<string, Job>;
  on: { workflow_call: { outputs: Record<string, { value: string }> } };
};
const restoreAction = parse(
  readRepoFile(".github", "actions", "restore-source-cli", "action.yml"),
) as Action;
const toolingAction = parse(
  readRepoFile(".github", "actions", "test-full-tooling", "action.yml"),
) as Action;
const sourceNeeds = [
  "pr-source-plan",
  "pr-rust-source",
  "pr-rust-products",
  "pr-js-packages",
  "pr-native-runtime",
  "pr-playground-test",
  "nuxt-module-qualification",
  "canonical-corpus",
  "ci-selection-tests",
  "wasm-release-check",
];
const fullNeeds = ["full-source-cli", "tooling-plan", "test-scripts"];
const fullCondition =
  "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}";
const alwaysFullCondition =
  "${{ always() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}";

void test("scheduled tooling owns the CLI producer without delaying independent PR source lanes", () => {
  assert.equal(full.jobs["full-source-cli"].uses, "./.github/workflows/full-source-cli.yml");
  assert.equal(full.jobs["full-source-cli"].if, fullCondition);
  assert.equal(full.jobs["tooling-plan"].if, fullCondition);
  assert.equal(
    full.jobs["test-scripts"].if,
    "${{ !cancelled() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  assert.deepEqual(full.jobs["test-scripts"].needs, ["full-source-cli", "tooling-plan"]);
  assert.equal(source.jobs["pr-source-cli"], undefined);
  assert.equal(source.jobs["pr-tooling-scripts"], undefined);
  for (const name of [
    "canonical-corpus",
    "pr-rust-source",
    "pr-js-packages",
    "pr-native-runtime",
    "pr-playground-test",
    "ci-selection-tests",
  ]) {
    assert.equal(source.jobs[name].needs, "pr-source-plan", name);
  }
  assert.deepEqual(source.jobs["source-report"].needs, sourceNeeds);
  assert.equal(
    source.jobs["source-report"].if,
    "${{ always() && !contains(needs.*.result, 'cancelled') }}",
  );
  const required = full.jobs["test-report"].needs;
  assert.ok(Array.isArray(required) && required.includes("pr-source-checks"));
  for (const name of fullNeeds) assert.ok(!required.includes(name), name);
});

void test("one original CLI/oracle build supplies each digest-strict consumer without receipt restamping", () => {
  const steps = producer.jobs.cli.steps ?? [];
  const build = steps.findIndex(
    (step) => step.name === "Build vize CLI and SFC oracle with original receipts",
  );
  const pack = steps.findIndex((step) => step.name === "Pack original source binaries and receipt");
  const upload = steps.findIndex((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.ok(build >= 0 && build < pack && pack < upload);
  assert.equal(
    steps[build].run,
    "cargo build --profile ci -p vize && vp exec node tests/differential/build-receipt.ts && cat target/ci/vize.differential-build.json && cargo build --profile ci -p vize_atelier_sfc --example cli_build_result_oracle",
  );
  for (const step of [steps[build], steps[pack], steps[upload]]) {
    assert.equal(step.if, undefined);
    assert.notEqual(step["continue-on-error"], true);
  }
  const name = "source-cli-${{ github.run_id }}-${{ steps.pack.outputs.producer-attempt }}";
  assert.equal(steps[upload].with?.name, name);
  assert.equal(steps[upload].with?.["if-no-files-found"], "error");
  for (const output of ["sha256", "receipt-sha256", "producer-attempt"]) {
    assert.equal(
      producer.on.workflow_call.outputs[output].value,
      `\${{ jobs.cli.outputs.${output} }}`,
    );
    assert.equal(producer.jobs.cli.outputs?.[output], `\${{ steps.pack.outputs.${output} }}`);
  }
  const consumer = full.jobs["test-scripts"];
  const worker = consumer.steps?.find(
    (step) => step.uses === "./.github/actions/test-full-tooling",
  );
  assert.equal(worker?.with?.producer, "${{ toJSON(needs.full-source-cli.outputs) }}");
  assert.equal(toolingAction.inputs.producer.required, true);
  const consumers = toolingAction.runs.steps;
  const restore = consumers.findIndex(
    (step) => step.uses === "./.github/actions/restore-source-cli",
  );
  const tests = consumers.findIndex(
    (step) => step.run === "vp run --workspace-root test:scripts:planned",
  );
  assert.ok(restore >= 0 && restore < tests);
  for (const step of [consumers[restore], consumers[tests]]) {
    assert.equal(step.if, undefined);
    assert.notEqual(step["continue-on-error"], true);
  }
  assert.equal(consumers[tests].env?.VIZE_LSP_BIN, "${{ github.workspace }}/target/ci/vize");
  assert.equal(consumers[tests].env?.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(consumers[restore].with?.producer, "${{ inputs.producer }}");
  const [validate, download, install] = restoreAction.runs.steps;
  assert.equal(validate.env?.SOURCE_CLI_PRODUCER_OUTPUTS, "${{ inputs.producer }}");
  assert.match(validate.run ?? "", /source-cli-artifact\.ts producer$/);
  assert.equal(
    download.with?.name,
    "source-cli-${{ github.run_id }}-${{ steps.producer.outputs.producer-attempt }}",
  );
  assert.equal(restoreAction.inputs.producer.required, true);
  assert.equal(download.with?.["digest-mismatch"], "error");
  assert.equal(download.with?.["run-id"], undefined);
  assert.deepEqual(install.env, {
    SOURCE_CLI_SHA256: "${{ steps.producer.outputs.sha256 }}",
    SOURCE_CLI_PRODUCER_ATTEMPT: "${{ steps.producer.outputs.producer-attempt }}",
    SOURCE_CLI_RECEIPT_SHA256: "${{ steps.producer.outputs.receipt-sha256 }}",
  });
  assert.match(
    install.run ?? "",
    /source-cli-artifact\.ts restore "\$RUNNER_TEMP\/source-cli\/source-cli.tar.gz" "\$SOURCE_CLI_SHA256" "\$SOURCE_CLI_PRODUCER_ATTEMPT" "\$SOURCE_CLI_RECEIPT_SHA256" && cat target\/ci\/vize.differential-build.json/,
  );
  for (const step of restoreAction.runs.steps) assert.notEqual(step["continue-on-error"], true);
  assert.doesNotMatch(
    [...consumers, ...restoreAction.runs.steps].map((step) => step.run ?? "").join("\n"),
    /cargo build --profile ci -p vize|build-receipt\.ts/,
  );
  assert.equal(
    consumer.env?.VIZE_SOURCE_CLI_RECEIPT_SHA256,
    "${{ needs.full-source-cli.outputs.receipt-sha256 }}",
  );
});

void test("the full tooling report fails when its producer, plan or workers do not succeed", () => {
  const gate = full.jobs["test-scripts-gate"];
  assert.deepEqual(gate.needs, fullNeeds);
  assert.equal(gate.if, alwaysFullCondition);
  const report = gate.steps?.find(
    (step) => step.uses === "./.github/actions/report-full-js-qualification",
  );
  assert.equal(report?.with?.["needs-json"], "${{ toJSON(needs) }}");
  const reportAction = parse(
    readRepoFile(".github", "actions", "report-full-js-qualification", "action.yml"),
  ) as Action;
  const command = reportAction.runs.steps.find((step) =>
    step.run?.includes("require-needs-success.ts"),
  );
  assert.equal(command?.env?.NEEDS_JSON, "${{ inputs.needs-json }}");
  assert.notEqual(command?.["continue-on-error"], true);
  const needs = Object.fromEntries(fullNeeds.map((name) => [name, { result: "success" }]));
  assert.equal(aggregateNeedsResults(needs).exitCode, 0);
  for (const name of fullNeeds) {
    for (const result of ["failure", "cancelled", "skipped"]) {
      assert.equal(
        aggregateNeedsResults({ ...needs, [name]: { result } }).exitCode,
        1,
        `${name}: ${result}`,
      );
    }
    assert.throws(() => aggregateNeedsResults({ ...needs, [name]: {} }), /reported no result/);
  }
});

void test("the source report still requires every independent product check", () => {
  const needs = Object.fromEntries(
    sourceNeeds.map((name) => [name, { result: "success", outputs: {} }]),
  ) as Record<string, { result: string; outputs: Record<string, string> }>;
  needs["pr-source-plan"].outputs["canonical-corpus"] = "true";
  assert.equal(canonicalSourceReport(needs, "merge_group").exitCode, 0);
  for (const name of sourceNeeds) {
    for (const result of ["failure", "cancelled", "skipped", ""]) {
      needs[name].result = result;
      const outcome = () => canonicalSourceReport(needs, "merge_group");
      if (name === "canonical-corpus" || name === "pr-source-plan" || result === "")
        assert.throws(outcome);
      else assert.equal(outcome().exitCode, 1, `${name}: ${result}`);
    }
    needs[name].result = "success";
  }
});
