import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { parse } from "yaml";

import { readRepoFile, root } from "./support/github-workflows.ts";

test("full native workspace tests hydrate the actual pinned Element Plus gitlink first", () => {
  const job = parse(readRepoFile(".github", "workflows", "check.yml")).jobs["clippy-and-test"];
  assert.equal(
    job.if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  const fixture = "tests/_fixtures/_git/element-plus";
  const hydration = job.steps.findIndex((step: { run?: string }) =>
    step.run?.includes(`git submodule update --init --force --depth 1 -- ${fixture}`),
  );
  const native = job.steps.findIndex(
    (step: { uses?: string }) => step.uses === "./.github/actions/test-rust-workspace-differential",
  );
  assert.ok(hydration >= 0 && hydration < native, "hydrate in the same job before native tests");
  assert.equal(job.steps[hydration].if, undefined);
  assert.notEqual(job.steps[hydration]["continue-on-error"], true);
  const gitlink = execFileSync("git", ["ls-tree", "HEAD", fixture], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(gitlink, `160000 commit 20298a21ae54a5417955373b80130ae7c548a6bb\t${fixture}\n`);
  const oracle = readRepoFile(
    "crates",
    "vize_canon",
    "tests",
    "native_syntax_semantics",
    "element_plus.rs",
  );
  assert.match(oracle, /assert_eq!\(pin, "20298a21ae54a5417955373b80130ae7c548a6bb\\n"\)/);
});

const fixture = "tests/_fixtures/_git/element-plus";
const hydrationCommand = `git submodule update --init --force --depth 1 -- ${fixture}`;
const check = () => parse(readRepoFile(".github", "workflows", "check.yml"));

function assertSourceCoverageHydration(job: {
  if?: string;
  steps: Array<{ run?: string; if?: string; "continue-on-error"?: boolean }>;
}) {
  assert.equal(
    job.if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  const consumer = job.steps.findIndex(
    (step) => step.run === "vp run --workspace-root coverage:source",
  );
  const preparation = job.steps.findIndex((step) => step.run?.includes(hydrationCommand));
  assert.ok(preparation >= 0 && preparation < consumer, "hydrate this coverage runner first");
  const step = job.steps[preparation];
  assert.equal(step.if, undefined);
  assert.notEqual(step["continue-on-error"], true);
  assert.equal(step.run, `npm install --global @pkl-community/pkl@0.27.2 && ${hydrationCommand}`);
}

test("whole-workspace source coverage prepares the native fixture in its own runner", () => {
  const job = check().jobs["source-coverage"];
  assertSourceCoverageHydration(job);
  const report = job.steps.find(
    (step: { name?: string }) => step.name === "Source coverage report",
  );
  assert.equal(report.run, "vp run --workspace-root coverage:source");
  assert.deepEqual(report.env, {
    VIZE_NATIVE_SYNTAX_CAPTURE_DIR: "${{ runner.temp }}/source-coverage-native-syntax",
    VIZE_LSP_EXIT_FAILURE_EVIDENCE: "${{ runner.temp }}/source-coverage-lsp-exit",
  });
  assert.equal(report.if, undefined);
  assert.notEqual(report["continue-on-error"], true);
  assert.equal(job.env.VIZE_NATIVE_SYNTAX_CAPTURE_DIR, undefined);
  assert.equal(job.env.VIZE_LSP_EXIT_FAILURE_EVIDENCE, undefined);
  const packets = job.steps.find(
    (step: { name?: string }) => step.name === "Retain native syntax and failed LSP exit packets",
  );
  assert.ok(job.steps.indexOf(report) < job.steps.indexOf(packets));
  assert.equal(packets.if, "${{ always() }}");
  assert.equal(packets.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.deepEqual(packets.with, {
    name: "source-coverage-native-syntax-${{ github.sha }}-${{ github.run_id }}-${{ github.run_attempt }}",
    path: "${{ runner.temp }}/source-coverage-native-syntax/\n${{ runner.temp }}/source-coverage-lsp-exit/\n",
    "if-no-files-found": "warn",
  });
  const taskSource = readRepoFile("tools", "config", "vite-plus", "tasks", "test-benchmark.ts");
  const source = taskSource
    .split("const rustSourceCoverageCommand = [")[1]
    .split("const rustBranchCoverageCommand")[0];
  assert.match(
    source,
    /"cargo llvm-cov nextest --workspace --profile full --test-threads 8 --json --summary-only"/,
  );
  assert.match(taskSource, /"coverage:source": task\(rustSourceCoverageCommand,/);
  const branch = taskSource.split("const rustBranchCoverageCommand = [")[1].split("/**")[0];
  assert.match(
    branch,
    /"cargo \+nightly llvm-cov -p vize_carton -p vize_armature -p vize_atelier_core"/,
  );
  assert.doesNotMatch(branch, /--workspace|-p vize_canon/);
  assert.equal(
    check().jobs["branch-coverage"].steps.some((step: { run?: string }) =>
      step.run?.includes(hydrationCommand),
    ),
    false,
    "the bounded branch suite does not consume this fixture",
  );

  const before = structuredClone(job);
  before.steps = before.steps.filter(
    (step: { run?: string }) => !step.run?.includes(hydrationCommand),
  );
  assert.throws(() => assertSourceCoverageHydration(before));
  for (const alteration of [
    { if: "${{ false }}" },
    { "continue-on-error": true },
    { run: `npm install --global @pkl-community/pkl@0.27.2; ${hydrationCommand}` },
  ]) {
    const unsafe = structuredClone(job);
    Object.assign(
      unsafe.steps.find((step: { run?: string }) => step.run?.includes(hydrationCommand)),
      alteration,
    );
    assert.throws(() => assertSourceCoverageHydration(unsafe));
  }
});

test("the actual preparation restores a pinned gitlink despite empty checkout and newer upstream HEAD", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-coverage-gitlink-"));
  const env = {
    ...process.env,
    GIT_CONFIG_GLOBAL: os.devNull,
    GIT_CONFIG_NOSYSTEM: "1",
    GIT_ALLOW_PROTOCOL: "file",
  };
  const git = (cwd: string, ...args: string[]) =>
    execFileSync("git", args, {
      cwd,
      env,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  const initialize = (cwd: string) => {
    fs.mkdirSync(cwd, { recursive: true });
    git(cwd, "init", "-q", "-b", "main");
    git(cwd, "config", "user.name", "Fixture");
    git(cwd, "config", "user.email", "fixture@example.com");
  };
  const commit = (cwd: string, marker: string) => {
    fs.writeFileSync(path.join(cwd, "README.md"), `${marker}\n`);
    git(cwd, "add", "README.md");
    git(cwd, "commit", "-qm", marker);
    return git(cwd, "rev-parse", "HEAD");
  };
  try {
    const upstream = path.join(directory, "upstream");
    initialize(upstream);
    const pin = commit(upstream, "frozen fixture");
    const newer = commit(upstream, "new upstream HEAD");
    const checkout = path.join(directory, "checkout");
    initialize(checkout);
    git(checkout, "submodule", "add", "-q", pathToFileURL(upstream).href, fixture);
    const actual = path.join(checkout, fixture);
    git(actual, "checkout", "-q", pin);
    git(checkout, "add", ".gitmodules", fixture);
    git(checkout, "commit", "-qm", "pin original fixture");
    git(checkout, "submodule", "deinit", "--force", "--", fixture);
    fs.mkdirSync(actual, { recursive: true });
    const consumer = () => {
      assert.equal(git(actual, "rev-parse", "HEAD"), pin);
      assert.equal(fs.readFileSync(path.join(actual, "README.md"), "utf8"), "frozen fixture\n");
    };
    assert.equal(git(actual, "rev-parse", "HEAD"), git(checkout, "rev-parse", "HEAD"));
    assert.throws(consumer, "the original pin law rejects enclosing Vize HEAD");
    const preparation = check()
      .jobs["source-coverage"].steps.find((step: { run?: string }) =>
        step.run?.includes(hydrationCommand),
      )
      .run.split(" && ")[1];
    execFileSync("bash", ["-e", "-c", preparation], { cwd: checkout, env, stdio: "pipe" });
    consumer();
    assert.equal(
      git(upstream, "rev-parse", "HEAD"),
      newer,
      "preparation leaves upstream untouched",
    );
    git(actual, "checkout", "-q", newer);
    fs.writeFileSync(path.join(actual, "README.md"), "dirty wrong revision\n");
    assert.throws(consumer);
    execFileSync("bash", ["-e", "-c", preparation], { cwd: checkout, env, stdio: "pipe" });
    consumer();

    const missing = path.join(directory, "missing-checkout");
    git(directory, "clone", "-q", pathToFileURL(checkout).href, missing);
    fs.renameSync(upstream, path.join(directory, "unavailable-upstream"));
    const unavailable = spawnSync("bash", ["-e", "-c", preparation], {
      cwd: missing,
      env,
      encoding: "utf8",
    });
    assert.notEqual(unavailable.status, 0, "unavailable fixture fails before coverage");
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
