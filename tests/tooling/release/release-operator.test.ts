import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { repoRoot } from "../_helpers/moonbit.ts";
import { writeFakeCommand } from "../support/fake-command.ts";
import { dependencyCommands, dependencyShell } from "../support/release-operator-dependency-law.ts";
import { readRepoFile } from "../support/github-workflows.ts";

const workflow = parse(readRepoFile(".github", "workflows", "release-operator.yml"));
const operator = workflow.jobs.operator;
const invocation = operator.steps.find(
  (step: { env?: Record<string, string> }) => step.env?.GH_TOKEN,
);
const dependencies = operator.steps.find((step: { run?: string }) =>
  step.run?.includes("vp install --frozen-lockfile"),
);

test("hosted operator keeps user authority on protected main and does not deadlock Release", () => {
  assert.deepEqual(Object.keys(workflow.on), ["workflow_dispatch"]);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.equal(workflow.concurrency["cancel-in-progress"], false);
  assert.notEqual(workflow.concurrency.group, "release-candidates");
  assert.equal(operator.environment, "release-operator");
  assert.equal(operator["timeout-minutes"], 360);
  assert.match(operator.if, /github\.ref == 'refs\/heads\/main'/);
  assert.match(operator.if, /github\.repository == 'ubugeeei-prod\/vize'/);
  assert.equal(operator.steps[0].with.ref, "${{ github.sha }}");
  assert.equal(operator.steps[0].with["persist-credentials"], false);
  assert.equal(invocation.env.GH_TOKEN, "${{ secrets.RELEASE_OPERATOR_TOKEN }}");
  assert.equal(invocation.env.GITHUB_TOKEN, invocation.env.GH_TOKEN);
  assert.equal(invocation.env.VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS, "14400");
  assert.equal(
    invocation.env.VIZE_RELEASE_EXPECTED_CUT_SHA,
    "${{ (inputs.operation == 'start' || inputs.operation == 'patch') && github.sha || '' }}",
  );
  assert.equal(invocation.env.GITHUB_ACTOR, undefined);
  assert.equal(invocation.env.GITHUB_TRIGGERING_ACTOR, undefined);
  assert.equal(invocation.env.GITHUB_SHA, undefined);
  assert.deepEqual(workflow.on.workflow_dispatch.inputs.operation.options, [
    "start",
    "patch",
    "resume",
    "retire",
    "recover-consumer",
  ]);
  assert.equal(workflow.on.workflow_dispatch.inputs.operation.default, "start");
  for (const [input, variable] of [
    ["source_head", "RELEASE_SOURCE_HEAD"],
    ["source_tag", "RELEASE_SOURCE_TAG"],
    ["release_run", "RELEASE_RUN"],
    ["operator_run", "RELEASE_ORIGINAL_OPERATOR_RUN"],
    ["consumer_recovery_run", "RELEASE_CONSUMER_RECOVERY_RUN"],
  ]) {
    assert.equal(workflow.on.workflow_dispatch.inputs[input].type, "string");
    assert.equal(workflow.on.workflow_dispatch.inputs[input].default, "");
    assert.equal(workflow.on.workflow_dispatch.inputs[input].required, false);
    assert.equal(invocation.env[variable], `\${{ inputs.${input} }}`);
  }
  assert.equal(
    operator.steps.some(
      (step: { uses?: string }) => step.uses === "./.github/actions/setup-moonbit",
    ),
    false,
  );
  assert.equal(dependencies.run.trim(), "vp install --frozen-lockfile");
});

test("actual pinned starter refuses stale cuts before mutation and installs the requested isolated frozen graph", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-cut-"));
  const work = path.join(root, "work");
  const remote = path.join(root, "remote.git");
  const bin = path.join(root, "bin");
  const installed = path.join(root, "installed.json");
  fs.mkdirSync(work);
  fs.mkdirSync(bin);
  const git = (...args: string[]) => {
    const result = spawnSync("git", args, { cwd: work, encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  };
  try {
    git("init", "--bare", remote);
    git("init", "-b", "main");
    git("config", "user.name", "Release source fixture");
    git("config", "user.email", "release@example.invalid");
    git("config", "commit.gpgsign", "false");
    git("config", "core.hooksPath", path.join(root, "no-hooks"));
    fs.writeFileSync(path.join(work, ".gitignore"), "node_modules/\n");
    fs.writeFileSync(path.join(work, "pnpm-lock.yaml"), "original frozen graph\n");
    git("add", ".gitignore", "pnpm-lock.yaml");
    git("commit", "-m", "original source");
    const original = git("rev-parse", "HEAD");
    git("remote", "add", "origin", remote);
    git("push", "origin", "main");
    fs.writeFileSync(path.join(work, "pnpm-lock.yaml"), "new frozen graph\n");
    git("commit", "-am", "main advanced during setup");
    git("push", "origin", "main");
    const current = git("rev-parse", "HEAD");
    git("checkout", "--detach", original);
    fs.mkdirSync(path.join(work, "node_modules"));
    const before = git("ls-remote", "--refs", "origin");
    const worktrees = git("worktree", "list", "--porcelain");
    writeFakeCommand(
      bin,
      "gh",
      `
      const args = process.argv.slice(2);
      if (args.join(' ') === 'api user --jq .login') console.log('maintainer');
      else if (args.join(' ') === 'api repos/owner/repo/collaborators/maintainer/permission') console.log(JSON.stringify({role_name:'maintain'}));
      else { console.error('Unexpected GitHub mutation or request', args); process.exit(1); }
    `,
    );
    writeFakeCommand(
      bin,
      "vp",
      `
      const fs = require('node:fs');
      fs.writeFileSync(${JSON.stringify(installed)}, JSON.stringify({
        args: process.argv.slice(2), cwd: process.cwd(),
        lock: fs.readFileSync('pnpm-lock.yaml', 'utf8'),
        borrowedProviders: fs.existsSync('node_modules')
      }));
      process.exit(17);
    `,
    );
    for (const [expected, diagnostic] of [
      [original, /expected release cut differs from current main/],
      ["main", /SHA/],
      ["", /SHA/],
    ] as const) {
      const result = spawnSync(
        "rust-script",
        [path.join(repoRoot, "tools/commands/release/pr.rs"), "start", "minor", "--pin"],
        {
          cwd: work,
          encoding: "utf8",
          env: {
            ...process.env,
            PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
            GITHUB_REPOSITORY: "owner/repo",
            VIZE_RELEASE_EXPECTED_CUT_SHA: expected,
            TMPDIR: root,
          },
        },
      );
      assert.equal(result.error, undefined, `${result.error ?? ""}`);
      assert.notEqual(
        result.status,
        0,
        `${result.error ?? ""}\n${result.stdout}\n${result.stderr}`,
      );
      assert.match(result.stderr, diagnostic);
      assert.equal(git("ls-remote", "--refs", "origin"), before);
      assert.equal(git("worktree", "list", "--porcelain"), worktrees);
      assert.equal(git("status", "--porcelain"), "");
      assert.equal(fs.existsSync(installed), false);
    }
    const acceptedCut = spawnSync(
      "rust-script",
      [path.join(repoRoot, "tools/commands/release/pr.rs"), "start", "minor", "--pin"],
      {
        cwd: work,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
          GITHUB_REPOSITORY: "owner/repo",
          VIZE_RELEASE_EXPECTED_CUT_SHA: current,
          TMPDIR: root,
        },
      },
    );
    assert.equal(acceptedCut.error, undefined);
    assert.notEqual(acceptedCut.status, 0);
    assert.match(acceptedCut.stderr, /vp install --frozen-lockfile failed/);
    const receipt = JSON.parse(fs.readFileSync(installed, "utf8"));
    assert.deepEqual(receipt.args, ["install", "--frozen-lockfile"]);
    assert.notEqual(receipt.cwd, work);
    assert.equal(receipt.lock, "new frozen graph\n");
    assert.equal(receipt.borrowedProviders, false);
    assert.equal(git("-C", receipt.cwd, "rev-parse", "HEAD"), current);
    assert.equal(git("ls-remote", "--refs", "origin"), before);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("hosted entry invokes the unchanged public protocol only for its actual maintainer user", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-operator-"));
  const bin = path.join(root, "bin");
  const capture = path.join(root, "public-command.json");
  const dependencyCapture = path.join(root, "dependencies.jsonl");
  fs.mkdirSync(bin);
  writeFakeCommand(
    bin,
    "gh",
    `
    const args = process.argv.slice(2);
    if (args[0] === 'auth') process.exit(0);
    if (args[1] === 'user') {
      if (args.at(-1) === '.id') console.log('42');
      else console.log(process.env.TEST_OPERATOR_LOGIN);
    } else if (args[1].endsWith('/permission')) console.log(process.env.TEST_OPERATOR_ROLE);
    else process.exit(1);
  `,
  );
  writeFakeCommand(bin, "git", "process.exit(process.argv[2] === 'config' ? 0 : 1);");
  dependencyCommands(bin, capture, dependencyCapture);
  const invoke = (
    operation: string,
    source = "",
    login = "maintainer",
    role = "maintain",
    token = "test-user-token",
    triggeringActor = "maintainer",
    retirement: { head?: string; tag?: string; run?: string; entry?: string } = {},
  ) =>
    spawnSync("bash", ["-c", invocation.run], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        GH_TOKEN: token,
        GITHUB_ACTOR: "maintainer",
        GITHUB_TRIGGERING_ACTOR: triggeringActor,
        GITHUB_REPOSITORY: "ubugeeei-prod/vize",
        RELEASE_OPERATION: operation,
        RELEASE_SOURCE_PR: source,
        RELEASE_SOURCE_CUT: "",
        RELEASE_SOURCE_HEAD: retirement.head ?? "",
        RELEASE_SOURCE_TAG: retirement.tag ?? "",
        RELEASE_RUN: retirement.run ?? "",
        RELEASE_ORIGINAL_OPERATOR_RUN: retirement.entry ?? "",
        RELEASE_CONSUMER_RECOVERY_RUN: "",
        TEST_OPERATOR_LOGIN: login,
        TEST_OPERATOR_ROLE: role,
      },
    });
  try {
    const original = parse(
      readRepoFile(
        "tests",
        "_fixtures",
        "release",
        "retire-moon-447",
        "original-release-operator.yml",
      ),
    ).jobs.operator.steps.find((step: { run?: string }) =>
      step.run?.includes("vp install --frozen-lockfile"),
    );
    assert.throws(
      () => dependencyShell(original.run, root, bin, capture, dependencyCapture),
      assert.AssertionError,
    );
    fs.rmSync(dependencyCapture);
    dependencyShell(dependencies.run, root, bin, capture, dependencyCapture);
    for (const [operation, source, expected] of [
      ["start", "", ["run", "release", "minor", "-y", "--pin"]],
      ["resume", "42", ["run", "release", "--resume", "42", "--pin"]],
    ] as const) {
      const result = invoke(operation, source);
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(JSON.parse(fs.readFileSync(capture, "utf8")), expected);
      fs.rmSync(capture);
    }
    for (const input of [
      ["start", "42"],
      ["resume", ""],
      ["resume", "0"],
      ["resume", "-1"],
      ["resume", "42; touch injected"],
      ["unknown", ""],
      ["start", "", "other-maintainer"],
      ["start", "", ""],
      ["start", "", "maintainer", "write"],
      ["start", "", "maintainer", "admin", ""],
      ["start", "", "maintainer", "admin", "test-user-token", "write-only-rerun-actor"],
    ] as const) {
      const result = invoke(input[0], input[1], input[2], input[3], input[4], input[5]);
      assert.notEqual(result.status, 0, JSON.stringify(input));
      assert.equal(fs.existsSync(capture), false);
      assert.equal(fs.existsSync(path.join(root, "injected")), false);
    }
    const retirement = { head: "a".repeat(40), tag: "v0.440.0", run: "43", entry: "44" };
    const retire = (source = "42", fields = retirement, role = "maintain", actor = "maintainer") =>
      invoke("retire", source, "maintainer", role, "test-user-token", actor, fields);
    const accepted = retire();
    assert.equal(accepted.status, 0, accepted.stderr);
    assert.deepEqual(JSON.parse(fs.readFileSync(capture, "utf8")), [
      "rust-script",
      "tools/commands/release/pr.rs",
      "retire",
      "42",
      retirement.head,
      retirement.tag,
      retirement.run,
      retirement.entry,
    ]);
    fs.rmSync(capture);
    const rejected = [
      ...["", "0", "-1", "042", "42; touch injected"].map((source) => retire(source)),
      ...["", "0", "-1", "043", "43; touch injected"].map((run) =>
        retire("42", { ...retirement, run }),
      ),
      ...["", "0", "-1", "044", "44; touch injected"].map((entry) =>
        retire("42", { ...retirement, entry }),
      ),
      ...["", "main", "a".repeat(39), "a".repeat(41), "g".repeat(40), `${retirement.head}\n`].map(
        (head) => retire("42", { ...retirement, head }),
      ),
      ...["", "v1.440.0", "v0.0.0", "v0.0440.0", "v0.440.1", "v0.440.0-rc.1", "v0.440.0\n"].map(
        (tag) => retire("42", { ...retirement, tag }),
      ),
      retire("42", retirement, "write"),
      retire("42", retirement, "admin", "write-only-rerun-actor"),
    ];
    for (const result of rejected) {
      assert.notEqual(result.status, 0, `${result.stdout}\n${result.stderr}`);
      assert.equal(fs.existsSync(capture), false);
      assert.equal(fs.existsSync(path.join(root, "injected")), false);
    }
    for (const operation of ["start", "resume"]) {
      for (const [field, value] of Object.entries(retirement)) {
        const result = invoke(
          operation,
          operation === "resume" ? "42" : "",
          undefined,
          undefined,
          undefined,
          undefined,
          { [field]: value },
        );
        assert.notEqual(result.status, 0, `${operation}: ${field}`);
        assert.equal(fs.existsSync(capture), false);
      }
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("new publication runs wait for full Check outside the bounded verify-only preflight", () => {
  const runs = readRepoFile("tools", "support", "release", "pr_pin_runs.rs");
  const watcher = readRepoFile("tools", "support", "release", "pr_pin_watch.rs");
  const shared = readRepoFile("tools", "support", "release", "pr_ci.rs");
  const existing = runs.indexOf("if let Some(id) = find()?");
  const qualify = runs.indexOf("pr_ci::qualify_check(");
  const authenticate = runs.indexOf("let current = super::source(");
  const recheckRun = runs.indexOf("if let Some(id) = find()?", existing + 1);
  const dispatch = runs.indexOf('"workflow",');
  assert.ok(
    existing >= 0 &&
      qualify > existing &&
      authenticate > qualify &&
      recheckRun > authenticate &&
      dispatch > recheckRun,
  );
  assert.match(runs.slice(existing, qualify), /return Ok\(id\)/);
  const child = runs.slice(qualify, authenticate);
  assert.match(child, /&candidate\.repository/);
  assert.match(child, /&candidate\.branch/);
  assert.match(child, /&candidate\.head/);
  assert.match(child, /&source\.cut/);
  assert.match(shared, /pr_budget::child_command\(&mut qualification, budget\)/);
  assert.match(shared, /if !status\.success\(\)/);
  assert.match(child, /operator\.verify\(\)\?/);
  const final = runs.slice(authenticate, dispatch);
  assert.match(final, /current\.cut != source\.cut/);
  assert.match(final, /current\.base_version != source\.base_version/);
  assert.match(final, /current\.integration != source\.integration/);
  assert.match(final, /tag_target\(&candidate\.tag, root\)/);
  assert.match(final, /operator\.verify\(\)\?/);
  assert.match(watcher, /runs::find_or_dispatch\(&initial, operator, budget, root\)/);
  const preflight = parse(readRepoFile(".github", "workflows", "release-preflight.yml"));
  assert.equal(preflight.jobs.verify["timeout-minutes"], 10);
  assert.equal(
    preflight.jobs.verify.steps.at(-1).run,
    "node tools/support/compat/github/release-preflight.ts --verify-only",
  );
});
