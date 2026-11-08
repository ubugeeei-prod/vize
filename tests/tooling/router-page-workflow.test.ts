import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { readFileSync } from "node:fs";
import { matchesGlob } from "node:path";
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
type Job = {
  needs?: string[];
  uses?: string;
  env: Record<string, string>;
  steps: Step[];
};
type Workflow = {
  on: { pull_request: { paths: string[] } };
  jobs: Record<string, Job>;
};

const root = new URL("../../", import.meta.url);
const workflow = parse(
  readFileSync(new URL(".github/workflows/router-preview-conformance.yml", root), "utf8"),
) as Workflow;
const job = workflow.jobs["page-route-cli-editor"];
const action = parse(
  readFileSync(new URL(".github/actions/check-router-page-types/action.yml", root), "utf8"),
) as { runs: { steps: Step[] } };
const producer = parse(
  readFileSync(new URL(".github/workflows/full-source-cli.yml", root), "utf8"),
) as { jobs: Record<string, { steps: Step[] }> };

void test("page implementation, oracle and action dependencies trigger qualification", () => {
  const patterns = workflow.on.pull_request.paths;
  for (const file of [
    "crates/vize_canon/src/virtual_ts/mapping/model/generated_ranges.rs",
    ".github/actions/check-router-page-types/action.yml",
    "tests/snapshots/check/vue-router-page-route-oracle.ts",
    "tests/fixtures/typechecker/page-route-types/cli-observer.ts",
    "tests/tooling/router-experimental-check.test.ts",
  ]) {
    assert.ok(
      patterns.some((pattern) => matchesGlob(file, pattern)),
      file,
    );
  }
});

void test("the page consumer installs the producer's pinned linker before source restoration", () => {
  const linker = producer.jobs.cli.steps.find(
    (step) => step.uses === "./.github/actions/setup-wild-http-recovery",
  );
  assert.ok(linker?.uses);
  const expected = job.steps.find((step) => step.uses === linker.uses);
  assert.ok(expected, "runtime identity requires the source producer's linker installation");
  assert.deepEqual(expected.with, linker.with);
  assert.ok(
    job.steps.indexOf(expected) <
      job.steps.findIndex((step) => step.uses === "./.github/actions/restore-source-cli"),
  );
});

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
  assert.ok(install);
  assert.equal(install.run, "vp install --frozen-lockfile --prefer-offline");
  assert.ok(!JSON.stringify(job).includes("VIZE_ROUTER_PREVIEW_PROVIDER"));
  assert.ok(!job.steps.some((step) => step.uses?.startsWith("actions/download-artifact@")));
});

void test("manual page qualification rejects absent, shortened and mismatched source identities", () => {
  const guard = job.steps.find((step) => step.name === "Require the reviewed source revision");
  assert.ok(guard?.run);
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
    const result: SpawnSyncReturns<string> = spawnSync(
      "/bin/bash",
      ["-euo", "pipefail", "-c", guard.run],
      {
        encoding: "utf8",
        env: { EXPECTED_SOURCE_SHA: expected, GITHUB_SHA: actual },
      },
    );
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
  assert.ok(check?.env);
  assert.equal(
    check.run,
    "vp node --test --test-concurrency=1 tests/snapshots/check/vue-router-page-route-oracle.ts",
  );
  assert.equal(check.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(check.env.VIZE_TEST_REQUIRE_TSGO, "1");
  for (const step of job.steps) assert.equal(step["continue-on-error"], undefined);
  assert.equal(oracle[0].if, undefined, "a prior failed step must not turn qualification green");
});

void test("all raw packet families and source receipts survive oracle failure", () => {
  const upload = job.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.ok(upload?.with);
  const paths = upload.with.path;
  assert.ok(typeof paths === "string", "upload paths must be a YAML block scalar");
  assert.equal(upload.if, "always()");
  assert.deepEqual(
    paths
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
  assert.equal(upload.with["if-no-files-found"], "error");
  assert.ok(
    job.steps.indexOf(upload) >
      job.steps.findIndex((step) => step.uses === "./.github/actions/check-router-page-types"),
  );
});

void test("full tooling retains Router failure evidence on its original source shard", () => {
  const full = parse(
    readFileSync(new URL(".github/actions/test-full-tooling/action.yml", root), "utf8"),
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
  const expected = job.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.ok(expected?.with);
  const paths = upload.with.path;
  assert.equal(typeof paths, "string");
  assert.deepEqual(
    (paths as string)
      .trim()
      .split("\n")
      .map((line) => line.trim())
      .sort(),
    (expected.with.path as string)
      .trim()
      .split("\n")
      .map((line) => line.trim())
      .sort(),
  );
  const formatter = steps.filter(
    (step) => step.uses === "./.github/actions/upload-formatter-api-corpus-evidence",
  );
  assert.equal(formatter.length, 1);
  assert.equal(formatter[0].if, "${{ always() }}");
  assert.deepEqual(formatter[0].with, { shard: "${{ inputs.shard }}" });
});
