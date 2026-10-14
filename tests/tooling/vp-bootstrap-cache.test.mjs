import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import {
  bootstrapCacheKey,
  bootstrapCacheSave,
  bootstrapCacheWrite,
  latestLtsVersion,
} from "../../.github/actions/cache-vp-bootstrap/cache-key.mjs";

const root = new URL("../../", import.meta.url);
const workflow = (path) => parse(readFileSync(new URL(path, root), "utf8"));

void test("the bootstrap key is the highest Node.js LTS release for this runner", () => {
  const version = latestLtsVersion([
    { version: "v25.0.0", lts: false },
    { version: "v24.9.0", lts: "Krypton" },
    { version: "v24.14.0", lts: "Krypton" },
    { version: "v22.22.0", lts: "Jod" },
    { version: "v24.15.0-rc.1", lts: "Krypton" },
    { version: "not-a-version", lts: "Krypton" },
  ]);
  assert.equal(version, "24.14.0");
  assert.equal(
    bootstrapCacheKey("Linux", "X64", version),
    "vp-bootstrap-v1-Linux-X64-node-24.14.0",
  );
  assert.throws(() => bootstrapCacheKey("Linux\n", "X64", version), /runner identity/);
  assert.throws(() => latestLtsVersion([{ version: "v25.0.0", lts: false }]), /no LTS/);
});

void test("only a trusted default-branch run saves the bootstrap runtime", () => {
  assert.equal(bootstrapCacheSave("push", "refs/heads/main", "main"), true);
  assert.equal(bootstrapCacheSave("schedule", "refs/heads/main", "main"), true);
  assert.equal(bootstrapCacheSave("workflow_dispatch", "refs/heads/main", "main"), true);
  assert.equal(bootstrapCacheSave("push", "refs/heads/davinci", "main"), false);
  assert.equal(bootstrapCacheSave("pull_request", "refs/pull/1/merge", "main"), false);
  assert.equal(bootstrapCacheSave("merge_group", "refs/heads/main", "main"), false);
  assert.equal(bootstrapCacheSave("push", "refs/heads/main", "main\n"), false);
  assert.equal(bootstrapCacheWrite(true, "push", "refs/heads/main", "main"), true);
  assert.equal(bootstrapCacheWrite(false, "push", "refs/heads/main", "main"), false);
  assert.equal(bootstrapCacheWrite(true, "pull_request", "refs/pull/1/merge", "main"), false);
  assert.equal(bootstrapCacheWrite(true, "merge_group", "refs/heads/main", "main"), false);
});

void test("pull-request setup-vp restores the bootstrap runtime and does not install it twice", () => {
  const action = workflow(".github/actions/cache-vp-bootstrap/action.yml");
  const paths = action.runs.steps
    .filter((step) => step.uses?.startsWith("actions/cache"))
    .map((step) => step.with.path);
  assert.deepEqual(paths, [
    "~/.vite-plus/js_runtime\n~/.vite-plus/package_manager\n",
    "~/.vite-plus/js_runtime\n~/.vite-plus/package_manager\n",
  ]);
  assert.equal(action.runs.steps.filter((step) => step.uses?.includes("setup-vp@")).length, 0);

  const jobs = [
    ["pr-rust-checks.yml", "merge-rust-source"],
    ["pr-rust-checks.yml", "pr-rust-shard"],
    ["pr-rust-checks.yml", "rust-source-report"],
    ["pr-rust-differential.yml", "differential"],
    ["pr-js-checks.yml", "pr-js-package-tests"],
    ["pr-js-checks.yml", "pr-oxlint-project-qualification"],
    ["pr-js-checks.yml", "pr-js-package-conformance"],
    ["pr-source-checks.yml", "pr-tooling-scripts"],
    ["pr-source-checks.yml", "pr-playground-test"],
    ["pr-source-cli.yml", "cli"],
  ];
  for (const [file, name] of jobs) {
    const steps = workflow(`.github/workflows/${file}`).jobs[name].steps;
    const setup = steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
    const cache = steps.findIndex((step) => step.uses === "./.github/actions/cache-vp-bootstrap");
    assert.ok(setup > 0 && cache === setup - 1, `${file} ${name}`);
    assert.equal(steps[cache].if, steps[setup].if);
    assert.equal(steps[cache].with, undefined);
  }

  const seed = workflow(".github/workflows/rust-pr-target-seed.yml");
  assert.deepEqual(Object.keys(seed.on).sort(), ["push", "schedule"]);
  const seedJob = seed.jobs["seed-vp-bootstrap"];
  assert.equal(seedJob.steps.at(-1).uses?.startsWith("voidzero-dev/setup-vp@"), true);
  assert.equal(seedJob.steps.at(-2).uses, "./.github/actions/cache-vp-bootstrap");
  assert.equal(seedJob.steps.at(-2).with.save, "true");
  assert.equal(seedJob.steps.at(-1).with["run-install"], false);
  assert.equal(seedJob.permissions.actions, "write");
  assert.equal(seedJob.permissions.contents, "read");
  assert.notEqual(
    seed.jobs["rust-differential-target-seed"].steps.find(
      (step) => step.uses === "./.github/actions/cache-vp-bootstrap",
    ).with,
    seedJob.steps.at(-2).with,
  );
});
