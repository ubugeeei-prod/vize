import { readAutomationPrograms } from "./support/automation-source.ts";
import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

type Step = {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
  with?: Record<string, unknown>;
};
test("ordinary source tooling retains complete capacity packets on success and failure without unrelated workspaces", () => {
  const action = parse(
    readAutomationPrograms(".github/actions/test-affected-tooling/action.yml"),
  ) as {
    runs: { steps: Step[] };
  };
  const step = action.runs.steps.find(
    (step) => step.name === "Retain complete native capacity packets and source send trace",
  );
  assert.ok(step);
  assert.equal(
    step.if,
    "${{ always() && hashFiles('target/vize-tests/diagnostic-capacity-*/result.json') != '' }}",
  );
  assert.ok(step.uses?.startsWith("actions/upload-artifact@"));
  const uploadPaths = step.with?.path;
  assert.ok(typeof uploadPaths === "string");
  assert.deepEqual(
    uploadPaths
      .split(/\r?\n/u)
      .map((path) => path.trim())
      .filter(Boolean)
      .sort(),
    [
      "target/vize-tests/diagnostic-capacity-*/result.json",
      "target/ci/vize.differential-build.json",
    ].sort(),
  );
  assert.equal(step.with?.["if-no-files-found"], "error");
  assert.equal(step.with?.["retention-days"], 7);
});
