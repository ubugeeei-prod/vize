import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { canonicalSourceReport } from "../../tools/support/compat/github/canonical-corpus-selection.mjs";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
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
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};
const producer = parse(readRepoFile(".github", "workflows", "pr-source-cli.yml")) as {
  jobs: Record<string, Job>;
  on: { workflow_call: { outputs: Record<string, { value: string }> } };
};
const restoreAction = parse(
  readRepoFile(".github", "actions", "restore-source-cli", "action.yml"),
) as { runs: { steps: Step[] } };
const originalNeeds = [
  "pr-source-plan",
  "pr-rust-source",
  "pr-js-packages",
  "pr-tooling-scripts",
  "pr-playground-test",
  "wit-contracts",
  "canonical-corpus",
];

void test("only tooling depends on the early CLI producer; original independent DAG edges remain", () => {
  assert.deepEqual(source.jobs["pr-source-cli"], {
    needs: "pr-source-plan",
    uses: "./.github/workflows/pr-source-cli.yml",
    with: { "run-cli": "${{ needs.pr-source-plan.outputs.tooling == 'true' }}" },
  });
  assert.deepEqual(source.jobs["pr-tooling-scripts"].needs, ["pr-source-plan", "pr-source-cli"]);
  for (const name of [
    "canonical-corpus",
    "pr-rust-source",
    "pr-js-packages",
    "pr-playground-test",
  ]) {
    assert.equal(source.jobs[name].needs, "pr-source-plan", name);
  }
  assert.equal(source.jobs["wit-contracts"].needs, undefined);
  assert.deepEqual(source.jobs["source-report"].needs, [...originalNeeds, "pr-source-cli"]);
  assert.equal(source.jobs["source-report"].if, "${{ always() }}");
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
    assert.equal(step.if, "${{ inputs.run-cli }}");
    assert.notEqual(step["continue-on-error"], true);
  }
  const name = "source-cli-${{ github.run_id }}-${{ steps.pack.outputs.producer-attempt }}";
  assert.equal(steps[upload].with?.name, name);
  assert.equal(steps[upload].with?.["if-no-files-found"], "error");
  assert.equal(
    producer.on.workflow_call.outputs["receipt-sha256"].value,
    "${{ jobs.cli.outputs.receipt-sha256 }}",
  );
  const consumer = source.jobs["pr-tooling-scripts"];
  const consumers = consumer.steps ?? [];
  const restore = consumers.findIndex(
    (step) => step.uses === "./.github/actions/restore-source-cli",
  );
  const tests = consumers.findIndex((step) => step.name === "Test selected PR tooling scripts");
  assert.ok(restore >= 0 && restore < tests);
  assert.equal(
    restoreAction.runs.steps[1].with?.name,
    "source-cli-${{ github.run_id }}-${{ steps.producer.outputs.producer-attempt }}",
  );
  assert.equal(
    producer.on.workflow_call.outputs["producer-attempt"].value,
    "${{ jobs.cli.outputs.producer-attempt }}",
  );
  assert.equal(
    producer.jobs.cli.outputs?.["producer-attempt"],
    "${{ steps.pack.outputs.producer-attempt }}",
  );
  assert.equal(consumers[restore].with?.producer, "${{ toJSON(needs.pr-source-cli.outputs) }}");
  assert.equal(restoreAction.inputs.producer.required, true);
  assert.equal(restoreAction.runs.steps[1].with?.["digest-mismatch"], "error");
  assert.equal(restoreAction.runs.steps[1].with?.["run-id"], undefined);
  assert.match(restoreAction.runs.steps[2].run ?? "", /source-cli-artifact\.ts restore/);
  assert.doesNotMatch(
    consumers.map((step) => step.run ?? "").join("\n"),
    /cargo build --profile ci -p vize|build-receipt\.ts/,
  );
  assert.equal(
    consumer.env?.VIZE_SOURCE_CLI_RECEIPT_SHA256,
    "${{ needs.pr-source-cli.outputs.receipt-sha256 }}",
  );
});

void test("a missing, cancelled or failed producer remains fatal to the original complete source report", () => {
  const needs = Object.fromEntries(
    [...originalNeeds, "pr-source-cli"].map((name) => [name, { result: "success", outputs: {} }]),
  ) as Record<string, { result: string; outputs: Record<string, string> }>;
  needs["pr-source-plan"].outputs["canonical-corpus"] = "true";
  assert.equal(canonicalSourceReport(needs, "merge_group").exitCode, 0);
  for (const result of ["failure", "cancelled", "skipped"]) {
    needs["pr-source-cli"].result = result;
    assert.equal(canonicalSourceReport(needs, "merge_group").exitCode, 1, result);
  }
  needs["pr-source-cli"].result = "";
  assert.throws(() => canonicalSourceReport(needs, "merge_group"), /reported no result/);
  needs["pr-source-cli"].result = "success";
  for (const name of originalNeeds.filter((name) => name !== "pr-source-plan")) {
    needs[name].result = "failure";
    const outcome = () => canonicalSourceReport(needs, "merge_group");
    if (name === "canonical-corpus") assert.throws(outcome);
    else assert.equal(outcome().exitCode, 1, name);
    needs[name].result = "success";
  }
});
