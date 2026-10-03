import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

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
  assert.equal(build.if, "${{ github.event_name == 'pull_request' && inputs.run-rust }}");
  const callers = build.steps.filter((candidate: { uses?: string }) => candidate.uses === action);
  assert.equal(callers.length, 1);
  assert.equal(
    callers[0].if,
    "${{ contains(fromJSON(inputs.rust-plan).packages, 'vize_maestro') }}",
  );
  assert.notEqual(callers[0]["continue-on-error"], true);
  assert.notEqual(build["continue-on-error"], true);
  const report = workflow.jobs["rust-source-report"];
  assert.ok(report.needs.includes("pr-rust-build"));
  assert.equal(report.if, "${{ always() }}");
  const aggregate = report.steps.find(
    (step: { run?: string }) =>
      step.run === "node tools/support/compat/github/require-rust-tier.mjs",
  );
  assert.ok(aggregate);
  assert.equal(aggregate.env.NEEDS_JSON, "${{ toJSON(needs) }}");
  assert.notEqual(aggregate["continue-on-error"], true);
});
