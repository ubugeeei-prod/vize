import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { repoRoot } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const script = path.join(repoRoot, "tools/commands/release/pr.rs");

test("release promotion rejects stale main and immutable tag conflicts atomically", () => {
  const result = spawnSync("rust-script", ["--test", script], { encoding: "utf8" });
  assert.equal(result.status, 0, `${result.error ?? ""}\n${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /8 passed; 0 failed/);
});

test("release-only workflow validates all artifacts before promotion and preserves normal PR cost", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "release.yml"));
  assert.deepEqual(Object.keys(workflow.on), ["workflow_dispatch"]);
  assert.match(workflow["run-name"], /Release \{0\} PR #\{1\} @ \{2\}/);
  assert.ok(workflow["run-name"].endsWith("}}"));
  assert.equal(workflow.jobs.candidate.if, "inputs.release_pr != ''");
  const ready = workflow.jobs["candidate-ready"];
  assert.equal(ready.name, "Release candidate ready");
  assert.equal(
    ready.if,
    undefined,
    "default success dependency condition must reject skipped/failed builds",
  );
  assert.deepEqual(
    [...ready.needs].sort((left: string, right: string) => left.localeCompare(right)),
    [
      "build-cli",
      "build-editor-extensions",
      "build-native-all",
      "build-release-packages",
      "build-wasm-package",
      "candidate-preflight",
      "smoke-release-packages",
    ],
  );
  assert.equal(workflow.jobs["candidate-preflight"].needs, "candidate");
  assert.deepEqual(workflow.jobs["candidate-preflight"].permissions, {
    actions: "write",
    contents: "read",
    issues: "read",
    "pull-requests": "read",
  });
  assert.equal(workflow.jobs["release-preflight"].needs, "candidate-ready");
  assert.equal(
    workflow.jobs["release-preflight"].uses,
    "./.github/workflows/release-promotion.yml",
  );
  const publication = workflow.jobs["create-github-release"].steps.find((step: { uses?: string }) =>
    step.uses?.startsWith("softprops/action-gh-release@"),
  );
  assert.equal(publication.with.tag_name, "${{ inputs.tag_name }}");
  const wasm = workflow.jobs["build-wasm-package"].steps;
  const installed = wasm.findIndex(
    (step: { with?: Record<string, unknown> }) => step.with?.["run-install"] === true,
  );
  const packed = wasm.findIndex((step: { run?: string }) =>
    step.run?.includes("smoke-release-install.rs npm/wasm"),
  );
  assert.ok(
    installed >= 0 && packed > installed,
    "WASM tarball checks must run before the candidate can be promoted",
  );
  const barrier = parse(readRepoFile(".github", "workflows", "release-promotion.yml"));
  assert.deepEqual(barrier.permissions, { contents: "read", "pull-requests": "read" });
  assert.match(barrier.jobs.promotion.steps.at(-1).run, /pr\.rs wait-promotion/);
});

test("Actions candidate authorization checks actual Git ancestry and never creates a tag", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-candidate-test-"));
  const work = path.join(root, "work");
  const remote = path.join(root, "remote.git");
  const bin = path.join(root, "bin");
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
    git("config", "user.name", "Release fixture");
    git("config", "user.email", "release@example.invalid");
    git("config", "commit.gpgsign", "false");
    git("config", "core.hooksPath", path.join(root, "no-hooks"));
    fs.writeFileSync(path.join(work, "Cargo.toml"), '[workspace.package]\nversion = "1.2.2"\n');
    git("add", "Cargo.toml");
    git("commit", "-m", "main");
    const base = git("rev-parse", "HEAD");
    git("remote", "add", "origin", remote);
    git("push", "origin", "main");
    git("checkout", "-b", "release/v1.2.3");
    fs.writeFileSync(path.join(work, "Cargo.toml"), '[workspace.package]\nversion = "1.2.3"\n');
    git("commit", "-am", "chore: release v1.2.3");
    const head = git("rev-parse", "HEAD");
    const pull = {
      number: 42,
      state: "open",
      merged: false,
      draft: false,
      user: { login: "maintainer" },
      head: { sha: head, ref: "release/v1.2.3", repo: { full_name: "owner/repo" } },
      base: { sha: base, ref: "main", repo: { full_name: "owner/repo" } },
    };
    writeFakeCommand(
      bin,
      "gh",
      `
      const args = process.argv.slice(2);
      if (args.join(' ') === 'api repos/owner/repo/pulls/42') {
        console.log(${JSON.stringify(JSON.stringify(pull))});
      } else if (args.join(' ') === 'api repos/owner/repo/collaborators/maintainer/permission') {
        console.log(JSON.stringify({role_name: process.env.TEST_RELEASE_ROLE}));
      } else { console.error('Unexpected GitHub request', args); process.exit(1); }
    `,
    );
    const validate = (role = "maintain", sha = head) =>
      spawnSync("rust-script", [script, "validate", "42", sha, "v1.2.3"], {
        cwd: work,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
          GITHUB_REPOSITORY: "owner/repo",
          GITHUB_SHA: head,
          TEST_RELEASE_ROLE: role,
          GITHUB_OUTPUT: path.join(root, "outputs"),
        },
      });
    fs.writeFileSync(path.join(root, "outputs"), "");
    const accepted = validate();
    assert.equal(accepted.status, 0, `${accepted.error ?? ""}\n${accepted.stderr}`);
    assert.match(fs.readFileSync(path.join(root, "outputs"), "utf8"), new RegExp(`base=${base}`));
    assert.match(validate("write").stderr, /maintain or admin/);
    assert.match(validate("maintain", base).stderr, /head changed/);
    git("checkout", "main");
    git("commit", "--allow-empty", "-m", "main advanced");
    git("push", "origin", "main");
    git("checkout", "release/v1.2.3");
    const stale = validate();
    assert.notEqual(stale.status, 0);
    assert.match(stale.stderr, /main advanced/);
    assert.equal(git("ls-remote", "--tags", "origin"), "");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
