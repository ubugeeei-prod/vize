import assert from "node:assert/strict";
import { execFile, execFileSync, spawnSync } from "node:child_process";
import { createServer } from "node:http";
import { promisify } from "node:util";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { hostedOrBlacksmith, readRepoFile } from "../support/github-workflows.ts";
import { createReleasePreflightVerifyOnlyFixture } from "../support/release-preflight-runner-fixture.ts";
import { repoRoot } from "../_helpers/tool-commands.ts";

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
  needs?: string;
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
  assert.deepEqual(Object.keys(workflow.jobs ?? {}).sort(), [
    "validate-crates",
    "verify",
    "wait-check",
  ]);

  const verify = workflow.jobs?.verify;
  assert.ok(verify);
  assert.match(verify["runs-on"] ?? "", new RegExp(`^${hostedOrBlacksmith("ubuntu-24.04")}$`));
  assert.equal(verify["timeout-minutes"], 10);
  assert.equal(verify.needs, "wait-check");
  const wait = workflow.jobs?.["wait-check"];
  assert.ok(wait);
  assert.equal(wait["timeout-minutes"], 345);
  assert.deepEqual(wait.permissions, { actions: "read", contents: "read" });
  assert.deepEqual(wait.steps?.find((step) => step.uses?.startsWith("actions/checkout@"))?.with, {
    "fetch-depth": 2,
    "persist-credentials": false,
    ref: "${{ github.sha }}",
  });
  const waitStep = wait.steps?.find((step) => step.run != null);
  assert.match(waitStep?.run ?? "", /main\.ts --wait-existing/);
  assert.doesNotMatch(waitStep?.run ?? "", /--start|bootstrap|workflow run|rerun/);
  assert.equal(waitStep?.env?.QUALIFICATION_DEADLINE, "${{ inputs.qualification_deadline }}");
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

test("both new-run paths start exact qualification before release-build dispatch", () => {
  for (const [file, lookup] of [
    ["pr_pin_runs.rs", "if let Some(id) = find()?"],
    ["pr_watch.rs", "if let Some(id) = find_run("],
  ] as const) {
    const source = readRepoFile("tools", "support", "release", file);
    const reuse = source.indexOf(lookup);
    const qualify = source.indexOf("pr_ci::start_check(");
    const dispatch = source.indexOf('"workflow",', qualify);
    assert.ok(reuse >= 0 && qualify > reuse && dispatch > qualify, file);
    assert.match(source.slice(reuse, qualify), /return Ok\(/);
    assert.match(source.slice(qualify, dispatch), /operator\.verify\(\)\?/);
  }
  const shared = readRepoFile("tools", "support", "release", "pr_ci.rs");
  assert.match(shared, /tools\/support\/release\/qualify_check\/main\.ts/);
  assert.match(shared, /pr_budget::child_command\(&mut command, budget\)/);
  assert.match(shared, /if !output\.status\.success\(\)/);
  assert.match(shared, /vize-release-check-start-v1/);
  assert.match(shared, /qualificationDeadline/);
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

test("actual read-only wait CLI refuses missing Q and makes only authenticated GET requests", async (t) => {
  const carrier = parse(readRepoFile(".github", "workflows", "release-preflight-contracts.yml"));
  const checkout = carrier.jobs.contracts.steps.find((step: WorkflowStep) =>
    step.uses?.startsWith("actions/checkout@"),
  );
  assert.deepEqual(checkout.with, { "fetch-depth": 2, "persist-credentials": false });
  const head = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim();
  const cut = execFileSync("git", ["rev-parse", "HEAD^"], {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim();
  const branch = "release/v0.500.0";
  const requests: string[] = [];
  const server = createServer((request, response) => {
    requests.push(`${request.method} ${request.url}`);
    assert.equal(request.headers.authorization, "Bearer unit-token");
    assert.equal(request.method, "GET", "the actual wait must never dispatch or rerun Q");
    response.writeHead(200, { "content-type": "application/json" });
    response.end(
      JSON.stringify(
        request.url?.includes("git/ref/heads/")
          ? { ref: `refs/heads/${branch}`, object: { type: "commit", sha: head } }
          : { workflow_runs: [] },
      ),
    );
  });
  await new Promise<void>((done) => server.listen(0, "127.0.0.1", done));
  t.after(() => server.close());
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  await assert.rejects(
    promisify(execFile)(
      process.execPath,
      [
        path.join(repoRoot, "tools/support/release/qualify_check/main.ts"),
        "--wait-existing",
        "owner/repo",
        branch,
        head,
        cut,
        String(Date.now() + 60_000),
      ],
      {
        cwd: repoRoot,
        timeout: 10_000,
        env: {
          ...process.env,
          GITHUB_SHA: head,
          GITHUB_TOKEN: "unit-token",
          GITHUB_API_URL: `http://127.0.0.1:${address.port}`,
        },
      },
    ),
    (error: unknown) => {
      assert.ok(error && typeof error === "object" && "stderr" in error && "stdout" in error);
      assert.match(String(error.stderr), /exact existing Check is missing/);
      assert.equal(error.stdout, "");
      return true;
    },
  );
  assert.deepEqual(requests, [
    `GET /repos/owner/repo/git/ref/heads/${branch}`,
    `GET /repos/owner/repo/actions/runs?per_page=100&page=1&head_sha=${head}`,
  ]);
});

test("actual immutable-workflow parser distinguishes legacy H and rejects ambiguous dispatch inputs", () => {
  const parser = readRepoFile("tools", "support", "release", "qualify_check", "capability.ts");
  const current = readRepoFile(".github", "workflows", "release.yml");
  const old = parse(current);
  delete old.on.workflow_dispatch.inputs.qualification_deadline;
  const legacy = JSON.stringify(old);
  for (const [source, mode] of [
    [current, "overlap"],
    [legacy, "legacy"],
  ]) {
    const result = spawnSync(
      process.execPath,
      ["--input-type=module-typescript", "--eval", parser],
      {
        cwd: repoRoot,
        input: source,
        encoding: "utf8",
        timeout: 10_000,
      },
    );
    assert.ifError(result.error);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, `${mode}\n`);
    assert.equal(result.stderr, "");
  }
  for (const declaration of [
    null,
    "string",
    { type: "number", required: false, default: "", description: "deadline" },
    { type: "string", required: true, default: "", description: "deadline" },
    { type: "string", required: false, default: "renew", description: "deadline" },
  ]) {
    const malformed = structuredClone(old);
    malformed.on.workflow_dispatch.inputs.qualification_deadline = declaration;
    const result = spawnSync(
      process.execPath,
      ["--input-type=module-typescript", "--eval", parser],
      {
        cwd: repoRoot,
        input: JSON.stringify(malformed),
        encoding: "utf8",
        timeout: 10_000,
      },
    );
    assert.ifError(result.error);
    assert.equal(result.status, 1);
    assert.equal(result.stdout, "");
    assert.match(result.stderr, /qualification_deadline/);
  }
  const unknown = structuredClone(old);
  unknown.on.workflow_dispatch.inputs.unknown_required = { type: "string", required: true };
  const absent = structuredClone(old);
  delete absent.on.workflow_dispatch.inputs.pinned;
  const duplicate = current.replace(
    "      qualification_deadline:",
    "      qualification_deadline: {}\n      qualification_deadline:",
  );
  for (const source of [JSON.stringify(unknown), JSON.stringify(absent), duplicate]) {
    const result = spawnSync(
      process.execPath,
      ["--input-type=module-typescript", "--eval", parser],
      {
        cwd: repoRoot,
        input: source,
        encoding: "utf8",
        timeout: 10_000,
      },
    );
    assert.ifError(result.error);
    assert.equal(result.status, 1);
    assert.equal(result.stdout, "");
  }
  const adapter = readRepoFile("tools", "support", "release", "pr_ci.rs");
  assert.match(
    adapter,
    /"--no-replace-objects",[\s\S]*?"show",[\s\S]*?head}:\.github\/workflows\/release\.yml/,
  );
  assert.match(adapter, /if overlap \{[\s\S]*?command\.arg\("--start"\)/);
  assert.match(adapter, /if !overlap \{[\s\S]*?return Ok\(None\)/);
});
