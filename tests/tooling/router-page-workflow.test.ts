import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

const root = new URL("../../", import.meta.url);
const workflow = parse(
  readFileSync(new URL(".github/workflows/router-preview-conformance.yml", root), "utf8"),
);
const job = workflow.jobs["page-route-cli-editor"];
const action = parse(
  readFileSync(new URL(".github/actions/check-router-page-types/action.yml", root), "utf8"),
);

void test("the published page oracle consumes only the same-run original source producer", () => {
  assert.deepEqual(job.needs, ["source-cli"]);
  assert.equal(workflow.jobs["source-cli"].uses, "./.github/workflows/full-source-cli.yml");
  assert.equal(
    job.env.VIZE_SOURCE_CLI_RECEIPT_SHA256,
    "${{ needs.source-cli.outputs.receipt-sha256 }}",
  );
  assert.equal(job.env.VIZE_TEST_BIN, "${{ github.workspace }}/target/ci/vize");
  assert.equal(job.env.VIZE_LSP_BIN, job.env.VIZE_TEST_BIN);
  assert.equal(job.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(job.env.VIZE_TEST_REQUIRE_TSGO, "1");
  const restore = job.steps.filter((step) => step.uses === "./.github/actions/restore-source-cli");
  assert.equal(restore.length, 1);
  assert.deepEqual(restore[0].with, { producer: "${{ toJSON(needs.source-cli.outputs) }}" });
  const install = job.steps.find((step) => step.name === "Install frozen source dependencies");
  assert.equal(install.run, "vp install --frozen-lockfile --prefer-offline");
  assert.ok(!JSON.stringify(job).includes("VIZE_ROUTER_PREVIEW_PROVIDER"));
  assert.ok(!job.steps.some((step) => step.uses?.startsWith("actions/download-artifact@")));
});

void test("manual page qualification rejects absent, shortened and mismatched source identities", () => {
  const guard = job.steps.find((step) => step.name === "Require the reviewed source revision");
  assert.equal(guard.if, "github.event_name == 'workflow_dispatch'");
  assert.deepEqual(guard.env, { EXPECTED_SOURCE_SHA: "${{ inputs.expected-source-sha }}" });
  const reviewed = "1".repeat(40);
  for (const [expected, actual, status] of [
    [reviewed, reviewed, 0],
    ["", reviewed, 1],
    [reviewed.slice(0, 12), reviewed, 1],
    ["2".repeat(40), reviewed, 1],
    ["x".repeat(40), "x".repeat(40), 1],
  ] as const) {
    const result = spawnSync("/bin/bash", ["-euo", "pipefail", "-c", guard.run], {
      encoding: "utf8",
      env: { EXPECTED_SOURCE_SHA: expected, GITHUB_SHA: actual },
    });
    assert.equal(result.status, status, result.stderr);
  }
});

void test("the unchanged complete page oracle keeps native host and source-build requirements", () => {
  const oracle = job.steps.filter(
    (step) => step.uses === "./.github/actions/check-router-page-types",
  );
  assert.equal(oracle.length, 1);
  const check = action.runs.steps.find(
    (step) => step.name === "Check complete configured page CLI and editor diagnostics",
  );
  assert.equal(
    check.run,
    "vp node --test --test-concurrency=1 tests/snapshots/check/vue-router-page-route-oracle.ts",
  );
  assert.equal(check.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(check.env.VIZE_TEST_REQUIRE_TSGO, "1");
  for (const step of job.steps) assert.equal(step["continue-on-error"], undefined);
  assert.equal(oracle[0].if, undefined, "a prior failed step must not turn qualification green");
});

void test("both raw packet families and all source receipts survive oracle failure", () => {
  const upload = job.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(upload.if, "always()");
  assert.deepEqual(upload.with.path.trim().split("\n"), [
    "target/differential/router-page-types/",
    "target/differential/router-page-types-secondary/",
    "target/ci/vize.differential-build.json",
    "target/ci/vize.source-build.json",
    "target/ci/vize.source-use.json",
  ]);
  assert.equal(upload.with["if-no-files-found"], "error");
  assert.ok(
    job.steps.indexOf(upload) >
      job.steps.findIndex((step) => step.uses === "./.github/actions/check-router-page-types"),
  );
});
