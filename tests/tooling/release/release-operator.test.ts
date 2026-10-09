import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { repoRoot } from "../_helpers/moonbit.ts";
import { writeFakeCommand } from "../support/fake-command.ts";
import { readRepoFile } from "../support/github-workflows.ts";

const workflow = parse(readRepoFile(".github", "workflows", "release-operator.yml"));
const operator = workflow.jobs.operator;
const invocation = operator.steps.at(-1);

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
    "${{ inputs.operation == 'start' && github.sha || '' }}",
  );
  assert.equal(invocation.env.GITHUB_ACTOR, undefined);
  assert.equal(invocation.env.GITHUB_TRIGGERING_ACTOR, undefined);
  assert.equal(invocation.env.GITHUB_SHA, undefined);
  assert.ok(
    operator.steps.some((step: { run?: string }) =>
      step.run?.includes("vp install --frozen-lockfile\nmoon update"),
    ),
  );
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
  writeFakeCommand(
    bin,
    "vp",
    `require('node:fs').writeFileSync(${JSON.stringify(capture)}, JSON.stringify(process.argv.slice(2)));`,
  );
  const invoke = (
    operation: string,
    source = "",
    login = "maintainer",
    role = "maintain",
    token = "test-user-token",
    triggeringActor = "maintainer",
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
        TEST_OPERATOR_LOGIN: login,
        TEST_OPERATOR_ROLE: role,
      },
    });
  try {
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
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
