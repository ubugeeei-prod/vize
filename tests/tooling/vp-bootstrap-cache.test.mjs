import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import {
  bootstrapCacheKey,
  bootstrapCachePrefix,
  bootstrapCacheWrite,
} from "../../.github/actions/cache-vp-bootstrap/cache-key.ts";

const root = new URL("../../", import.meta.url);
const workflow = (path) => parse(readFileSync(new URL(path, root), "utf8"));

void test("bootstrap caches rotate daily with compatible pinned-runtime fallback", () => {
  const version = "24.14.0";
  const manager = "pnpm@12.1.0";
  const date = new Date("2026-10-16T09:00:00Z");
  const prefix = bootstrapCachePrefix("Linux", "X64", version, manager);
  assert.equal(bootstrapCacheKey("Linux", "X64", version, manager, date), `${prefix}2026-10-16`);
  assert.ok(
    bootstrapCacheKey("Linux", "X64", version, manager, new Date("2026-10-17")).startsWith(prefix),
  );
  for (const [os, arch, node, pnpm] of [
    ["Windows", "X64", version, manager],
    ["Linux", "ARM64", version, manager],
    ["Linux", "X64", "24.15.0", manager],
    ["Linux", "X64", version, "pnpm@12.2.0"],
  ])
    assert.notEqual(bootstrapCachePrefix(os, arch, node, pnpm), prefix);
  assert.throws(() => bootstrapCacheKey("Linux\n", "X64", version, manager), /runner identity/);
  assert.throws(() => bootstrapCacheKey("Linux", "X64", "lts", manager), /pinned Node/);
  assert.throws(() => bootstrapCacheKey("Linux", "X64", version, "pnpm@latest"), /pinned pnpm/);
});

void test("only a trusted default-branch run saves the bootstrap runtime", () => {
  assert.equal(bootstrapCacheWrite(true, "push", "refs/heads/main", "main"), true);
  assert.equal(bootstrapCacheWrite(true, "schedule", "refs/heads/main", "main"), true);
  assert.equal(bootstrapCacheWrite(true, "workflow_dispatch", "refs/heads/main", "main"), true);
  assert.equal(bootstrapCacheWrite(true, "push", "refs/heads/davinci", "main"), false);
  assert.equal(bootstrapCacheWrite(true, "push", "refs/heads/main", "main\n"), false);
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
  for (const step of action.runs.steps.filter((step) => step.uses?.startsWith("actions/cache"))) {
    assert.equal(
      step.with["restore-keys"],
      "${{ steps.key.outputs.restore }}\nvp-bootstrap-v1-${{ runner.os }}-${{ runner.arch }}-node-\n",
    );
  }

  for (const file of readdirSync(new URL(".github/workflows", root)).filter((file) =>
    /^pr-.*\.yml$/.test(file),
  )) {
    for (const [name, job] of Object.entries(workflow(`.github/workflows/${file}`).jobs)) {
      const steps = job.steps ?? [];
      for (const [index, step] of steps.entries()) {
        if (step.uses !== "./.github/actions/cache-vp-bootstrap") continue;
        assert.equal(
          steps[index + 1]?.uses?.startsWith("voidzero-dev/setup-vp@"),
          true,
          `${file} ${name}`,
        );
        assert.equal(step.if, steps[index + 1].if);
        assert.equal(step.with, undefined);
      }
    }
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
