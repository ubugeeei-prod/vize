import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { runRustWorkerSelection } from "../../tools/support/compat/github/select-rust-workers.mjs";
import { context, fixture } from "./_helpers/rust-worker-selection-fixture.mjs";

// Synthetic metadata and directories exercise output ownership only. They grant
// no actual Rust, archive, corpus, or repaired failed-only execution credit.
function setup(t) {
  const root = fs.mkdtempSync(path.join(tmpdir(), "vize-worker-refusal-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const sha = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  const value = fixture();
  value.run.head_sha = sha;
  for (const job of value.jobs) job.head_sha = sha;
  for (const artifact of value.artifacts) artifact.workflow_run.head_sha = sha;
  const receipt = path.join(root, "worker-selection.json");
  const artifacts = path.join(root, "artifacts");
  const env = {
    GITHUB_WORKFLOW: "Check",
    GITHUB_EVENT_NAME: "merge_group",
    GITHUB_RUN_ID: String(context.runId),
    GITHUB_RUN_ATTEMPT: String(context.attempt),
    GITHUB_SHA: sha,
    GITHUB_REPOSITORY: context.repository,
    GITHUB_API_URL: "https://api.example.invalid",
    GH_TOKEN: "synthetic-test-only",
    GITHUB_OUTPUT: path.join(root, "step-output"),
  };
  const options = {
    fetchImpl: async (url) => {
      const resource = url.pathname;
      const body = resource.endsWith("/jobs")
        ? { jobs: value.jobs }
        : resource.endsWith("/artifacts")
          ? { artifacts: value.artifacts }
          : value.run;
      return new Response(JSON.stringify(body));
    },
  };
  const seed = () => {
    for (const name of ["report.json", "acceptance.json"]) {
      fs.writeFileSync(path.join(root, name), "synthetic stale public output");
    }
  };
  const absent = () => {
    for (const name of ["report.json", "acceptance.json"]) {
      assert.equal(fs.existsSync(path.join(root, name)), false);
    }
  };
  const run = (mode) => runRustWorkerSelection([mode, receipt, artifacts], env, options);
  return { value, receipt, artifacts, env, run, seed, absent };
}

void test("a valid selection write followed by metadata refusal leaves no old public outputs or receipt", async (t) => {
  const value = setup(t);
  const result = await value.run("select");
  assert.deepEqual(JSON.parse(fs.readFileSync(value.receipt, "utf8")), result);
  value.seed();
  value.value.jobs[4].conclusion = "failure";
  await assert.rejects(value.run("select"), /Latest worker.*did not succeed/);
  value.absent();
  assert.equal(fs.existsSync(value.receipt), false);
});

void test("a rejected context after a valid selection clears old outputs before any metadata request", async (t) => {
  const value = setup(t);
  await value.run("select");
  value.seed();
  value.env.GITHUB_EVENT_NAME = "pull_request";
  await assert.rejects(value.run("select"), /Unexpected Rust event context/);
  value.absent();
  assert.equal(fs.existsSync(value.receipt), false);
});

void test("verification keeps its input receipt but clears old public outputs before refused retry", async (t) => {
  const value = setup(t);
  const selection = await value.run("select");
  for (const worker of selection.workers) {
    fs.mkdirSync(path.join(value.artifacts, worker.artifactName), { recursive: true });
  }
  await value.run("verify");
  value.seed();
  value.value.jobs[4].conclusion = "failure";
  await assert.rejects(value.run("verify"), /Latest worker.*did not succeed/);
  value.absent();
  assert.deepEqual(JSON.parse(fs.readFileSync(value.receipt, "utf8")), selection);
});
