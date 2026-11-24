import { readAutomationPrograms } from "./support/automation-source.ts";
import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

type Step = {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
  env?: Record<string, string>;
  with?: Record<string, unknown>;
  "continue-on-error"?: boolean;
};

const root = new URL("../../", import.meta.url);
const action = parse(
  readAutomationPrograms(new URL(".github/actions/check-router-page-types/action.yml", root)),
) as { runs: { steps: Step[] } };

void test("the complete page oracle keeps native host and source-build requirements", () => {
  const check = action.runs.steps.find(
    (step) => step.name === "Check complete configured page CLI and editor diagnostics",
  );
  assert.ok(check?.env);
  assert.equal(
    check.run,
    "vp node --test --test-concurrency=1 tests/snapshots/check/vue-router-page-route-oracle.ts",
  );
  assert.equal(check.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(check.env.VIZE_TEST_REQUIRE_TSGO, "1");
});

void test("full tooling retains Router failure evidence on its original source shard", () => {
  const full = parse(
    readAutomationPrograms(new URL(".github/actions/test-full-tooling/action.yml", root)),
  ) as { runs: { steps: Step[] } };
  const steps = full.runs.steps;
  const oracle = steps.filter((step) => step.uses === "./.github/actions/check-router-page-types");
  assert.equal(oracle.length, 1);
  assert.equal(oracle[0].if, "${{ inputs.shard == '1' }}");
  assert.equal(oracle[0]["continue-on-error"], undefined);
  const restore = steps.filter((step) => step.uses === "./.github/actions/restore-source-cli");
  assert.equal(restore.length, 1);
  assert.deepEqual(restore[0].with, { producer: "${{ inputs.producer }}" });
  assert.ok(steps.indexOf(restore[0]) < steps.indexOf(oracle[0]));
  const uploads = steps.filter((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(uploads.length, 1, "the full worker must preserve raw Router evidence");
  const upload = uploads[0];
  assert.equal(upload.if, "${{ always() && inputs.shard == '1' }}");
  assert.equal(upload["continue-on-error"], undefined);
  assert.ok(steps.indexOf(upload) > steps.indexOf(oracle[0]));
  assert.ok(upload.with);
  assert.equal(upload.with["if-no-files-found"], "error");
  const paths = upload.with.path;
  assert.equal(typeof paths, "string");
  assert.deepEqual(
    (paths as string)
      .trim()
      .split("\n")
      .map((line) => line.trim())
      .sort(),
    [
      "target/differential/experimental-router-check/",
      "target/differential/router-page-types/",
      "target/differential/router-page-types-secondary/",
      "target/ci/vize.differential-build.json",
      "target/ci/vize.source-build.json",
      "target/ci/vize.source-use.json",
    ].sort(),
  );
  const formatter = steps.filter(
    (step) => step.uses === "./.github/actions/upload-formatter-api-corpus-evidence",
  );
  assert.equal(formatter.length, 1);
  assert.equal(formatter[0].if, "${{ always() }}");
  assert.deepEqual(formatter[0].with, { shard: "${{ inputs.shard }}" });
});
