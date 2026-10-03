import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

test("native navigation whole responses run in the shared full recipe with strict feature checks", () => {
  const recipe = parse(
    readRepoFile(".github", "actions", "test-rust-workspace-differential", "action.yml"),
  );
  const step = recipe.runs.steps.find(
    (candidate: { name: string }) =>
      candidate.name === "Test native JS and TS navigation responses",
  );
  assert.ok(step);
  assert.equal(step.shell, "bash");
  assert.equal(step.env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.notEqual(step["continue-on-error"], true);
  assert.equal(step.if, undefined);
  assert.deepEqual(step.run.trim().split("\n"), [
    "cargo test --locked -p vize_maestro --features experimental-source-navigation --lib source_project:: -- --nocapture",
    "cargo test --locked -p vize_maestro --features experimental-source-navigation --lib server::native_navigation:: -- --nocapture",
    "cargo check --locked -p vize_maestro --no-default-features --features experimental-source-navigation",
    "cargo clippy --locked -p vize_maestro --features experimental-source-navigation --all-targets -- -D warnings",
  ]);
});
