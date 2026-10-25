import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  run?: string;
  env?: Record<string, string>;
  uses?: string;
  with?: Record<string, string>;
  "continue-on-error"?: boolean;
};

void test("message profiling executes the exact required native test before merge", () => {
  const action: { runs: { steps: Step[] } } = parse(
    readRepoFile(".github", "actions", "test-native-canon", "action.yml"),
  );
  const steps = action.runs.steps.filter(
    (step) => step.name === "Test complete native diagnostic profiling and source repair",
  );
  assert.equal(steps.length, 1);
  const step = steps[0];
  assert.ok(step);
  assert.equal(step.if, undefined);
  assert.equal(step["continue-on-error"], undefined);
  assert.deepEqual(step.env, {
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_MESSAGE_PROFILE_CAPTURE_DIR: "${{ runner.temp }}/native-snapshot-source/message-profile",
  });
  assert.equal(
    step.run,
    [
      "set -euo pipefail",
      "unset VIZE_TEST_DISABLE_TSGO",
      "cargo test --locked --profile ci -p vize --test integration -- check_message_profile_cli::real_check_message_profile_preserves_complete_native_packets_and_repair --exact --nocapture",
      "",
    ].join("\n"),
  );
  const preserve = action.runs.steps.find(
    (candidate) => candidate.name === "Preserve actual native inputs and responses",
  );
  assert.equal(preserve?.if, "${{ always() }}");
  assert.ok(preserve?.with?.path.includes("${{ runner.temp }}/native-snapshot-source"));
});
