import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";

import {
  aggregateNeedsResults,
  PULL_REQUEST_SKIPPED_JOBS,
} from "../../tools/github/require-needs-success.mjs";
import { readRepoFile, root } from "./support/github-workflows.ts";

interface WorkflowStep {
  name?: string;
  uses?: string;
  run?: string;
  env?: Record<string, string>;
}

interface WorkflowJob {
  if?: string;
  needs?: string[];
  steps?: WorkflowStep[];
}

interface Workflow {
  jobs?: Record<string, WorkflowJob>;
}

/**
 * The jobs `test-report` aggregates, in workflow order. Kept literal so the
 * expected gate messages below stay stable; the workflow's own list is asserted
 * against this one in "test-report depends on the jobs the ruleset does not
 * require directly".
 */
const NEEDED_JOBS = [
  "nix-flake",
  "fmt-rust",
  "check-js",
  "security-audit",
  "semver-checks",
  "node-engine-compat",
  "check-vize-apps",
  "vue-parity",
  "test-scripts",
  "editor-extensions",
  "editor-host-smoke",
  "build-js-packages",
  "test-js-packages",
  "clippy-and-test",
  "coverage",
  "source-coverage",
  "branch-coverage",
  "playground-test",
];

/**
 * Each `if:` guard used by a job in `test-report`'s `needs:` list, classified by
 * whether the job runs on a pull request. An unclassified guard fails the drift
 * test rather than being guessed at, because guessing wrong either blocks every
 * pull request or reopens the hole this gate closes.
 */
const PULL_REQUEST_BEHAVIOUR_BY_GUARD = new Map([
  ["", "runs"],
  ["${{ github.event_name != 'pull_request' }}", "skips"],
  ["${{ github.event_name == 'pull_request' || github.event_name == 'push' }}", "runs"],
]);

function readCheckWorkflow(): Workflow {
  return parse(readRepoFile(".github", "workflows", "check.yml")) as Workflow;
}

function checkWorkflowJob(name: string): WorkflowJob {
  const job = readCheckWorkflow().jobs?.[name];
  assert.ok(job, `missing check.yml job ${name}`);
  return job;
}

function needsContext(overrides: Record<string, string> = {}): Record<string, { result: string }> {
  return Object.fromEntries(
    NEEDED_JOBS.map((job) => [job, { result: overrides[job] ?? "success" }]),
  );
}

/** The result shape of a healthy pull request: the two event-gated jobs skip. */
function pullRequestContext(
  overrides: Record<string, string> = {},
): Record<string, { result: string }> {
  return needsContext({ "nix-flake": "skipped", "source-coverage": "skipped", ...overrides });
}

test("test-report passes when every needed job succeeded", () => {
  assert.deepEqual(aggregateNeedsResults(needsContext()), {
    exitCode: 0,
    message:
      "test-report gate: all 18 needed jobs are accounted for. 18 succeeded; 0 skipped on a pull request by design.",
  });
});

test("test-report passes when only the event-gated jobs skipped", () => {
  assert.deepEqual(aggregateNeedsResults(pullRequestContext()), {
    exitCode: 0,
    message:
      "test-report gate: all 18 needed jobs are accounted for. 16 succeeded; 2 skipped on a pull request by design: nix-flake, source-coverage.",
  });
});

test("test-report fails when a needed job failed", () => {
  assert.deepEqual(aggregateNeedsResults(pullRequestContext({ "vue-parity": "failure" })), {
    exitCode: 1,
    message: [
      "test-report gate: 1 of 18 needed jobs did not succeed.",
      "  - vue-parity: failure",
      "test-report is a required status check, so it must not pass while a job it aggregates is red.",
      "Only these jobs may skip on a pull request: nix-flake, source-coverage.",
    ].join("\n"),
  });
});

test("test-report fails when a needed job was cancelled", () => {
  assert.deepEqual(
    aggregateNeedsResults(pullRequestContext({ "editor-host-smoke": "cancelled" })),
    {
      exitCode: 1,
      message: [
        "test-report gate: 1 of 18 needed jobs did not succeed.",
        "  - editor-host-smoke: cancelled",
        "test-report is a required status check, so it must not pass while a job it aggregates is red.",
        "Only these jobs may skip on a pull request: nix-flake, source-coverage.",
      ].join("\n"),
    },
  );
});

test("test-report fails when a job skips because its own dependency failed", () => {
  const needs = pullRequestContext({
    "build-js-packages": "failure",
    "playground-test": "skipped",
  });

  assert.deepEqual(aggregateNeedsResults(needs), {
    exitCode: 1,
    message: [
      "test-report gate: 2 of 18 needed jobs did not succeed.",
      "  - build-js-packages: failure",
      "  - playground-test: skipped",
      "test-report is a required status check, so it must not pass while a job it aggregates is red.",
      "Only these jobs may skip on a pull request: nix-flake, source-coverage.",
    ].join("\n"),
  });
});

test("test-report fails when an event-gated job ran and failed", () => {
  assert.deepEqual(aggregateNeedsResults(needsContext({ "source-coverage": "failure" })), {
    exitCode: 1,
    message: [
      "test-report gate: 1 of 18 needed jobs did not succeed.",
      "  - source-coverage: failure",
      "test-report is a required status check, so it must not pass while a job it aggregates is red.",
      "Only these jobs may skip on a pull request: nix-flake, source-coverage.",
    ].join("\n"),
  });
});

test("test-report rejects a needs context it cannot read", () => {
  assert.throws(() => aggregateNeedsResults({}), {
    message: "The needs context is empty: test-report must depend on the jobs it gates",
  });
  assert.throws(() => aggregateNeedsResults([]), {
    message: "The needs context must be an object of job results",
  });
  assert.throws(() => aggregateNeedsResults({ "vue-parity": {} }), {
    message: "Job vue-parity reported no result in the needs context",
  });
});

test("the test-report gate step exits non-zero for a red dependency", () => {
  const command = ["tools/commands/ci/github/require-needs-success.rs"];
  const failing = spawnSync("rust-script", command, {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      NEEDS_JSON: JSON.stringify(pullRequestContext({ "vue-parity": "failure" })),
    },
  });

  assert.equal(failing.status, 1);
  assert.equal(failing.stdout, "");
  assert.equal(
    failing.stderr,
    `${[
      "test-report gate: 1 of 18 needed jobs did not succeed.",
      "  - vue-parity: failure",
      "test-report is a required status check, so it must not pass while a job it aggregates is red.",
      "Only these jobs may skip on a pull request: nix-flake, source-coverage.",
    ].join("\n")}\n`,
  );

  const passing = spawnSync("rust-script", command, {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, NEEDS_JSON: JSON.stringify(pullRequestContext()) },
  });

  assert.equal(passing.status, 0);
  assert.equal(passing.stderr, "");
  assert.equal(
    passing.stdout,
    "test-report gate: all 18 needed jobs are accounted for. 16 succeeded; 2 skipped on a pull request by design: nix-flake, source-coverage.\n",
  );
});

test("the test-report gate step reports a missing needs payload", () => {
  const { NEEDS_JSON: _ignored, ...env } = process.env;
  const result = spawnSync("rust-script", ["tools/commands/ci/github/require-needs-success.rs"], {
    cwd: root,
    encoding: "utf8",
    env,
  });

  assert.equal(result.status, 1);
  assert.equal(result.stdout, "");
  assert.equal(
    result.stderr,
    "NEEDS_JSON is required: pass ${{ toJSON(needs) }} to tools/commands/ci/github/require-needs-success.rs\n",
  );
});

test("the pull-request skip list matches the check workflow's event guards", () => {
  const workflow = readCheckWorkflow();
  const unclassified: string[] = [];
  const skipsPullRequests: string[] = [];

  for (const jobName of NEEDED_JOBS) {
    const job = workflow.jobs?.[jobName];
    assert.ok(job, `missing check.yml job ${jobName}`);
    const guard = job.if ?? "";
    const behaviour = PULL_REQUEST_BEHAVIOUR_BY_GUARD.get(guard);
    if (behaviour === undefined) {
      unclassified.push(`${jobName}: ${guard}`);
      continue;
    }
    if (behaviour === "skips") {
      skipsPullRequests.push(jobName);
    }
  }

  assert.deepEqual(unclassified, []);
  assert.deepEqual(skipsPullRequests, ["nix-flake", "source-coverage"]);
  assert.deepEqual(Object.keys(PULL_REQUEST_SKIPPED_JOBS), skipsPullRequests);
});

test("test-report depends on the jobs the ruleset does not require directly", () => {
  assert.deepEqual(checkWorkflowJob("test-report").needs, NEEDED_JOBS);
});

test("test-report collects the inventory before it enforces the gate", () => {
  const steps = checkWorkflowJob("test-report").steps ?? [];

  assert.deepEqual(
    steps.map((step) => ({
      name: step.name ?? null,
      uses: step.uses?.replace(/@[0-9a-f]{40}$/, "") ?? null,
      run: step.run ?? null,
      env: step.env ?? null,
    })),
    [
      { name: null, uses: "actions/checkout", run: null, env: null },
      { name: null, uses: "dtolnay/rust-toolchain", run: null, env: null },
      { name: null, uses: "./.github/actions/setup-rust-script", run: null, env: null },
      {
        name: "Collect test inventory",
        uses: null,
        run: 'node bench/test-inventory.mjs --json test-inventory.json --markdown "$GITHUB_STEP_SUMMARY"',
        env: null,
      },
      { name: "Upload test inventory", uses: "actions/upload-artifact", run: null, env: null },
      {
        name: "Require every needed job to succeed",
        uses: null,
        run: "rust-script tools/commands/ci/github/require-needs-success.rs",
        env: { NEEDS_JSON: "${{ toJSON(needs) }}" },
      },
    ],
  );
});
