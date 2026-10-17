import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { hostedOrBlacksmith, readRepoFile } from "../support/github-workflows.ts";
import { createReleasePreflightVerifyOnlyFixture } from "../support/release-preflight-runner-fixture.ts";
import { repoRoot } from "../_helpers/moonbit.ts";

type WorkflowStep = {
  env?: Record<string, string>;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

type WorkflowJob = {
  permissions?: Record<string, string>;
  "runs-on"?: string;
  steps?: WorkflowStep[];
  "timeout-minutes"?: number;
};

test("reusable release preflight verifies evidence and crate plans without registry credentials", () => {
  const source = readRepoFile(".github", "workflows", "release-preflight.yml");
  const workflow = parse(source) as {
    env?: Record<string, unknown>;
    jobs?: Record<string, WorkflowJob>;
    on?: Record<string, unknown>;
    permissions?: Record<string, string>;
  };

  assert.ok(Object.hasOwn(workflow.on ?? {}, "workflow_call"));
  assert.deepEqual(workflow.permissions, {
    actions: "read",
    contents: "read",
    issues: "read",
    "pull-requests": "read",
  });
  assert.equal(workflow.env?.FORCE_JAVASCRIPT_ACTIONS_TO_NODE24, true);
  assert.deepEqual(Object.keys(workflow.jobs ?? {}).sort(), ["validate-crates", "verify"]);

  const verify = workflow.jobs?.verify;
  assert.ok(verify);
  assert.match(verify["runs-on"] ?? "", new RegExp(`^${hostedOrBlacksmith("ubuntu-24.04")}$`));
  assert.equal(verify["timeout-minutes"], 10);
  assert.deepEqual(verify.permissions, {
    actions: "read",
    contents: "read",
    issues: "read",
    "pull-requests": "read",
  });
  const verifyCheckout = verify.steps?.find((step) => step.uses?.startsWith("actions/checkout@"));
  assert.ok(verifyCheckout);
  assert.equal(verifyCheckout.with?.["fetch-depth"], 0);
  assert.equal(verifyCheckout.with?.["persist-credentials"], false);
  const nodeSetup = verify.steps?.find((step) => step.uses?.startsWith("actions/setup-node@"));
  assert.ok(nodeSetup);
  assert.deepEqual(nodeSetup.with, {
    "node-version-file": "package.json",
    "package-manager-cache": false,
  });
  const verification = verify.steps?.find((step) => step.run != null);
  assert.ok(verification);
  assert.equal(
    verification.run,
    "node tools/support/compat/github/release-preflight.ts --verify-only",
  );
  assert.equal(verification.env?.GITHUB_TOKEN, "${{ github.token }}");

  const crateValidation = workflow.jobs?.["validate-crates"];
  assert.ok(crateValidation);
  assert.match(
    crateValidation["runs-on"] ?? "",
    new RegExp(`^${hostedOrBlacksmith("ubuntu-24.04")}$`),
  );
  assert.equal(crateValidation["timeout-minutes"], 45);
  assert.deepEqual(crateValidation.permissions, { contents: "read" });
  const crateCheckout = crateValidation.steps?.find((step) =>
    step.uses?.startsWith("actions/checkout@"),
  );
  assert.ok(crateCheckout);
  assert.equal(crateCheckout.with?.["persist-credentials"], false);
  assert.equal(
    crateValidation.steps?.some((step) => step.uses?.startsWith("wild-linker/action@")),
    false,
  );
  assert.ok(
    crateValidation.steps?.some(
      (step) => step.run === "node tools/support/release/publish_crates/main.ts --dry-run",
    ),
  );
  const stickyCache = crateValidation.steps?.find(
    (step) => step.uses === "./.github/actions/setup-rust-sticky-cache",
  );
  assert.ok(stickyCache);
  assert.equal(stickyCache.with?.key, "release-crates-dry-run");
  assert.equal(stickyCache.with?.["cache-key-suffix"], "${{ runner.os }}-${{ runner.arch }}");

  assert.doesNotMatch(source, /environment:|id-token:\s*write|secrets\.|crates-io-auth-action/);
});

test("both new-run paths finish shared qualification before publication dispatch", () => {
  for (const [file, lookup] of [
    ["pr_pin_runs.rs", "if let Some(id) = find()?"],
    ["pr_watch.rs", "if let Some(id) = find_run("],
  ] as const) {
    const source = readRepoFile("tools", "support", "release", file);
    const reuse = source.indexOf(lookup);
    const qualify = source.indexOf("pr_ci::qualify_check(");
    const dispatch = source.indexOf('"workflow",', qualify);
    assert.ok(reuse >= 0 && qualify > reuse && dispatch > qualify, file);
    assert.match(source.slice(reuse, qualify), /return Ok\(/);
    assert.match(source.slice(qualify, dispatch), /operator\.verify\(\)\?/);
  }
  const shared = readRepoFile("tools", "support", "release", "pr_ci.rs");
  assert.match(shared, /tools\/support\/release\/qualify_check\/main\.ts/);
  assert.match(shared, /pr_budget::child_command\(&mut qualification, budget\)/);
  assert.match(shared, /if !status\.success\(\)/);
});

test("actual publication verify-only refuses absent Check without dispatching cold qualification", () => {
  const root = mkdtempSync(path.join(tmpdir(), "vize-release-read-only-preflight-"));
  try {
    const fixture = createReleasePreflightVerifyOnlyFixture(root, {
      mutateRuns(runs) {
        runs.length = 0;
      },
    });
    const result = spawnSync(
      process.execPath,
      ["tools/support/compat/github/release-preflight.ts", "--verify-only"],
      {
        cwd: repoRoot,
        encoding: "utf8",
        timeout: 10_000,
        env: {
          ...process.env,
          PATH: `${fixture.binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          ...fixture.env,
        },
      },
    );
    assert.ifError(result.error);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Check: missing workflow_dispatch run/);
    assert.doesNotMatch(result.stderr, /Unexpected preflight network request|Failed to dispatch/);
    assert.doesNotMatch(result.stdout, /Dispatching|Release preflight passed/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
