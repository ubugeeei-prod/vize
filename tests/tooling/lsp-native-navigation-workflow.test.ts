import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";
import { differentialMatrix } from "../../tools/support/compat/github/rust-differential-lanes.ts";

const action = "./.github/actions/test-native-navigation";

test("native Program and Vue whole responses share the same actual feature validation recipe", () => {
  const recipe = parse(readRepoFile(".github", "actions", "test-native-navigation", "action.yml"));
  assert.equal(recipe.runs.using, "composite");
  assert.equal(recipe.runs.steps.length, 1);
  const step = recipe.runs.steps[0];
  assert.equal(step.shell, "bash");
  assert.notEqual(step["continue-on-error"], true);
  assert.equal(step.if, undefined);
  assert.deepEqual(step.run.trim().split("\n"), [
    "cargo test --locked -p vize_maestro --features experimental-source-navigation --lib source_project:: -- --nocapture",
    "cargo test --locked -p vize_maestro --features experimental-source-navigation --lib server::native_navigation:: -- --nocapture",
    "cargo test --locked -p vize_maestro --features experimental-source-navigation --lib module_link -- --nocapture",
    "cargo test --locked -p vize_maestro --no-default-features --features experimental-source-navigation --lib module_link -- --nocapture",
    "cargo check --locked -p vize_maestro --no-default-features --features experimental-source-navigation",
    "cargo clippy --locked -p vize_maestro --features experimental-source-navigation --all-targets -- -D warnings",
  ]);
  const full = parse(
    readRepoFile(".github", "actions", "test-rust-workspace-differential", "action.yml"),
  );
  const callers = full.runs.steps.filter(
    (candidate: { uses?: string }) => candidate.uses === action,
  );
  assert.equal(callers.length, 1);
  assert.equal(callers[0].env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(callers[0].if, undefined);
  assert.notEqual(callers[0]["continue-on-error"], true);
});

test("affected Maestro source builds require real native feature tests before required Rust aggregation", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml"));
  const build = workflow.jobs["pr-rust-build"];
  assert.equal(build.if, "${{ inputs.run-rust }}");
  const callers = build.steps.filter((candidate: { uses?: string }) => candidate.uses === action);
  assert.equal(callers.length, 1);
  assert.equal(
    callers[0].if,
    "${{ github.event_name == 'pull_request' && contains(fromJSON(inputs.rust-plan).packages, 'vize_maestro') }}",
  );
  assert.notEqual(callers[0]["continue-on-error"], true);
  assert.notEqual(build["continue-on-error"], true);
  const report = workflow.jobs["rust-source-report"];
  assert.ok(report.needs.includes("pr-rust-build"));
  assert.equal(
    report.if,
    "${{ always() && github.event_name == 'merge_group' && !contains(needs.*.result, 'cancelled') }}",
  );
  const aggregate = report.steps.find(
    (step: { run?: string }) =>
      step.run === "node tools/support/compat/github/require-rust-tier.ts",
  );
  assert.ok(aggregate);
  assert.equal(aggregate.env.NEEDS_JSON, "${{ toJSON(needs) }}");
  assert.notEqual(aggregate["continue-on-error"], true);
});

test("affected merge candidates require the same native navigation recipe in the differential lane", () => {
  assert.ok(differentialMatrix(["vize_maestro"]).lane.includes("navigation"));
  assert.equal(differentialMatrix(["vize_atelier_ssr"]).lane.includes("navigation"), false);
  const workflow = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml"));
  const merge = workflow.jobs["merge-rust-differential"];
  assert.equal(merge.if, "${{ github.event_name == 'merge_group' && inputs.run-rust }}");
  assert.equal(merge.with["rust-plan"], "${{ inputs.rust-plan }}");
  assert.ok(workflow.jobs["rust-source-report"].needs.includes("merge-rust-differential"));
  const differential = parse(readRepoFile(".github", "workflows", "pr-rust-differential.yml"));
  const job = differential.jobs.differential;
  assert.equal(job.if, "${{ github.event_name == 'merge_group' && inputs.run-rust }}");
  assert.equal(job.strategy["fail-fast"], false);
  assert.equal(job.strategy.matrix, "${{ fromJSON(inputs.rust-plan).differentialMatrix }}");
  assert.equal(job.env.VIZE_TEST_REQUIRE_TSGO, "1");
  const callers = job.steps.filter((candidate: { uses?: string }) => candidate.uses === action);
  assert.equal(callers.length, 1);
  assert.equal(callers[0].if, "${{ matrix.lane == 'navigation' }}");
  assert.notEqual(callers[0]["continue-on-error"], true);
  assert.notEqual(job["continue-on-error"], true);
});
