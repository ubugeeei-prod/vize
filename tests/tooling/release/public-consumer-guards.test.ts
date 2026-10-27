import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

import { readRepoFile } from "../support/github-workflows.ts";

type Job = { if?: string; steps: Array<{ name?: string; run?: string }> };
type Workflow = { jobs: Record<string, Job> };
type Context = {
  github: { repository: string; event_name: string };
  inputs: { phase: string; candidate: string };
};
const repository = "ubugeeei-prod/vize";
const candidate = "6c1c5b386c56745fcb0e7e28feedb930b1a51606";
const filenames = ["release-contracts.yml", "public-native-events.yml"] as const;
const workflows = filenames.map((filename) => ({
  filename,
  workflow: parse(readRepoFile(".github", "workflows", filename)) as Workflow,
}));

function enabled(job: Job, context: Context): boolean {
  // These guards use the common Boolean/property/string subset of Actions.
  // Optional string inputs are empty strings when the dispatch omits them.
  const value: unknown = runInNewContext(job.if ?? "true", context, { timeout: 500 });
  assert.equal(typeof value, "boolean");
  return value as boolean;
}

function context(event: string, phase = "", value = "", repo = repository): Context {
  return {
    github: { repository: repo, event_name: event },
    inputs: { phase, candidate: value },
  };
}

test("explicit public-consumer requests always enter candidate validation", () => {
  const cases = [
    ["pull_request", "", "", repository, false],
    ["workflow_dispatch", "", "", repository, false],
    ["workflow_dispatch", "release-contracts", "", repository, true],
    ["workflow_dispatch", "release-contracts", "invalid", repository, true],
    ["workflow_dispatch", "release-contracts", candidate, repository, true],
    ["workflow_dispatch", "after", "", repository, false],
    ["workflow_dispatch", "after", candidate, repository, true],
    ["workflow_dispatch", "before", candidate, repository, false],
    ["workflow_dispatch", "release-contracts", candidate, "example/fork", false],
  ] as const;
  for (const { filename, workflow } of workflows) {
    const job = workflow.jobs["public-release-contracts"];
    assert.ok(job, filename);
    for (const [event, phase, value, repo, expected] of cases) {
      assert.equal(
        enabled(job, context(event, phase, value, repo)),
        expected,
        filename + ": " + JSON.stringify({ event, phase, candidate: value, repo }),
      );
    }
  }
});

test("selected public-consumer jobs reject missing, malformed, or mismatched candidates", () => {
  for (const { filename, workflow } of workflows) {
    const job = workflow.jobs["public-release-contracts"];
    assert.ok(job, filename);
    const validator = job.steps.find(
      (step) => step.name === "Validate the explicit public candidate",
    );
    assert.ok(validator?.run, filename);
    for (const [value, source, valid] of [
      ["", "", false],
      ["invalid", "invalid", false],
      [candidate, "0".repeat(40), false],
      [candidate, candidate, true],
    ] as const) {
      assert.equal(enabled(job, context("workflow_dispatch", "release-contracts", value)), true);
      const result: SpawnSyncReturns<string> = spawnSync(
        "bash",
        ["-e", "-u", "-o", "pipefail", "-c", validator.run],
        {
          encoding: "utf8",
          env: {
            ...process.env,
            PUBLIC_VERSION: "0.601.0",
            PUBLIC_CANDIDATE: value,
            PUBLIC_SOURCE_SHA: source,
          },
        },
      );
      assert.equal(result.error, undefined, filename);
      assert.equal(result.signal, null, filename);
      assert.equal(result.status === 0, valid, filename + ": " + JSON.stringify({ value, source }));
    }
  }
});

test("ordinary PR and manual release-unit checks remain enabled", () => {
  const wrapper = workflows.find((entry) => entry.filename === "release-contracts.yml")?.workflow;
  assert.ok(wrapper);
  const job = wrapper.jobs.contracts;
  assert.ok(job);
  for (const [event, phase, value, expected] of [
    ["pull_request", "", "", true],
    ["workflow_dispatch", "", "", true],
    ["workflow_dispatch", "release-contracts", "", false],
    ["workflow_dispatch", "release-contracts", candidate, false],
  ] as const) {
    assert.equal(enabled(job, context(event, phase, value)), expected);
  }
  assert.ok(
    job.steps.some((step) =>
      step.run?.includes("tests/tooling/release/public-consumer-guards.test.ts"),
    ),
    "the retained release-contracts suite must execute these actual-expression regressions",
  );
});
