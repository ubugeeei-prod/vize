import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
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

void test("both cache branches restore only the matching runtime schema and pins", () => {
  const action = workflow(".github/actions/cache-vp-bootstrap/action.yml");
  const prefix = bootstrapCachePrefix("Linux", "X64", "24.14.0", "pnpm@12.1.0");
  const previous = bootstrapCacheKey(
    "Linux",
    "X64",
    "24.14.0",
    "pnpm@12.1.0",
    new Date("2026-10-15"),
  );
  const incompatible = [
    "vp-bootstrap-v1-Linux-X64-node-24.14.0",
    bootstrapCacheKey("Windows", "X64", "24.14.0", "pnpm@12.1.0"),
    bootstrapCacheKey("Linux", "ARM64", "24.14.0", "pnpm@12.1.0"),
    bootstrapCacheKey("Linux", "X64", "24.15.0", "pnpm@12.1.0"),
    bootstrapCacheKey("Linux", "X64", "24.14.0", "pnpm@12.2.0"),
  ];
  const caches = action.runs.steps.filter((step) => step.uses?.startsWith("actions/cache"));
  assert.equal(caches.length, 2);
  for (const step of caches) {
    const prefixes = step.with["restore-keys"]
      .replaceAll("${{ steps.key.outputs.restore }}", prefix)
      .replaceAll("${{ runner.os }}", "Linux")
      .replaceAll("${{ runner.arch }}", "X64")
      .trim()
      .split("\n");
    assert.ok(prefixes.some((candidate) => previous.startsWith(candidate)));
    for (const key of incompatible)
      assert.ok(
        prefixes.every((candidate) => !key.startsWith(candidate)),
        `incompatible cache: ${key}`,
      );
  }
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

void test("both Musea callers restore the bootstrap runtime before their unchanged installer", () => {
  const action = workflow(".github/actions/cache-vp-bootstrap/action.yml");
  assert.equal(action.inputs.save.default, "false");
  const restore = action.runs.steps.find((step) => step.uses?.startsWith("actions/cache/restore@"));
  assert.equal(
    restore.if,
    "${{ steps.key.outputs.key != '' && steps.key.outputs.save != 'true' }}",
  );
  const callers = Object.entries(workflow(".github/workflows/musea-browser.yml").jobs).flatMap(
    ([name, job]) =>
      (job.steps ?? []).flatMap((step, index, steps) => {
        if (step.uses !== "./.github/actions/cache-vp-bootstrap") return [];
        assert.deepEqual(step, { uses: "./.github/actions/cache-vp-bootstrap" });
        assert.equal(steps[index - 1]?.uses?.startsWith("actions/checkout@"), true);
        const setup = steps[index + 1];
        assert.equal(setup?.uses?.startsWith("voidzero-dev/setup-vp@"), true);
        assert.equal(step.if, setup.if);
        assert.deepEqual(setup.with, {
          "node-version-file": "package.json",
          cache: true,
          "run-install": false,
        });
        for (const event of ["pull_request", "merge_group", "schedule", "workflow_dispatch"])
          assert.equal(
            bootstrapCacheWrite(step.with?.save === "true", event, "refs/heads/main", "main"),
            false,
          );
        return [name];
      }),
  );
  assert.deepEqual(callers, ["musea-native-inline", "musea-browser"]);
});

const dependencyAction = "voidzero-dev/setup-vp@68f39c78aaf39be468cabd2afb09c162574c84af";
const dependencyConsumers = [
  ["check.yml", "check-js"],
  ["check.yml", "security-audit"],
  ["check.yml", "node-engine-compat", "matrix"],
  ["pr-js-checks.yml", "pr-js-package-tests"],
  ["pr-js-checks.yml", "pr-oxlint-project-qualification"],
  ["pr-js-checks.yml", "pr-js-package-conformance"],
  ["pr-source-checks.yml", "pr-native-runtime"],
  ["pr-source-checks.yml", "pr-playground-test"],
  ["pr-rust-products.yml", "pr-rust-shard"],
  ["pr-rust-differential.yml", "differential"],
  ["rust-pr-target-seed.yml", "rust-differential-target-seed"],
];

function assertDependencyConsumer(job, runtime = "manifest") {
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  const steps = job.steps.filter((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  assert.equal(steps.length, 1);
  const setup = steps[0];
  assert.equal(
    setup.uses,
    dependencyAction,
    "cache-save requires the reviewed official capability",
  );
  assert.equal(setup.with.cache, true, "consumers must retain restoration");
  assert.equal(setup.with["cache-save"], false, "consumers cannot publish the dependency store");
  assert.equal(setup.with.version, "latest", "preserve the previous installer version policy");
  if (runtime === "matrix") {
    assert.deepEqual(job.strategy, {
      "fail-fast": false,
      matrix: { "node-version": ["22", "24"] },
    });
    assert.equal(setup.with["node-version"], "${{ matrix.node-version }}");
    assert.equal(setup.with["node-version-file"], undefined);
  } else {
    assert.equal(setup.with["node-version-file"], "package.json");
    assert.equal(setup.with["node-version"], undefined);
  }
  assert.equal(setup.with["working-directory"], undefined);
  assert.equal(setup.with["cache-dependency-path"], undefined);
  assert.equal(job["continue-on-error"], undefined);
  assert.equal(setup["continue-on-error"], undefined);
  return setup;
}

void test("Linux source consumers retain the same dependency identity without cache publication", () => {
  assert.equal(dependencyConsumers.length, 11, "ten ordinary jobs plus the differential seed");
  for (const [file, name, runtime] of dependencyConsumers) {
    const job = workflow(`.github/workflows/${file}`).jobs[name];
    const setup = assertDependencyConsumer(job, runtime);
    assert.equal(
      setup.with["run-install"],
      ["pr-rust-shard", "rust-differential-target-seed"].includes(name),
    );
    assert.ok(!JSON.stringify(job.needs ?? []).includes("seed-vp-dependencies"));
    for (const change of [
      { cache: false },
      { "cache-save": true },
      { "node-version-file": "other-package.json" },
      { "node-version": "20" },
      { "working-directory": "npm/cli" },
      { "cache-dependency-path": "other-lock.yaml" },
      { version: "" },
    ]) {
      const invalid = structuredClone(job);
      invalid.steps.find((step) => step.uses === dependencyAction).with = {
        ...setup.with,
        ...change,
      };
      assert.throws(() => assertDependencyConsumer(invalid, runtime));
    }
  }
});

void test("one trusted main producer owns the selected Linux source dependency-store writes", () => {
  const seed = workflow(".github/workflows/rust-pr-target-seed.yml");
  const producer = seed.jobs["seed-vp-dependencies"];
  assert.deepEqual(Object.keys(seed.on).sort(), ["push", "schedule"]);
  assert.deepEqual(seed.on.push.branches, ["main"]);
  for (const file of [
    "package.json",
    "pnpm-lock.yaml",
    ".github/workflows/rust-pr-target-seed.yml",
  ])
    assert.ok(seed.on.push.paths.includes(file), `producer must refresh after ${file}`);
  assert.equal(seed.concurrency["cancel-in-progress"], false);
  assert.equal(producer["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.deepEqual(producer.permissions, { contents: "read", actions: "write" });
  assert.equal(producer.needs, undefined, "seed independently of the expensive Rust builds");
  const setup = producer.steps.find((step) => step.uses === dependencyAction);
  assert.deepEqual(setup.with, {
    version: "latest",
    "node-version-file": "package.json",
    cache: true,
    "cache-save": true,
    "run-install": false,
  });
  const saves = Object.entries(seed.jobs).flatMap(([name, job]) =>
    (job.steps ?? [])
      .filter((step) => step.uses?.startsWith("voidzero-dev/setup-vp@") && step.with?.cache)
      .filter((step) => step.with["cache-save"] !== false)
      .map(() => name),
  );
  assert.deepEqual(saves, ["seed-vp-dependencies"]);

  // Exercise the actual job admission expression, including manual and queue
  // events on main: neither may turn a consumer into a trusted writer.
  for (const [event_name, ref, expected] of [
    ["push", "refs/heads/main", true],
    ["schedule", "refs/heads/main", true],
    ["push", "refs/heads/topic", false],
    ["schedule", "refs/heads/topic", false],
    ["pull_request", "refs/pull/1/merge", false],
    ["pull_request_target", "refs/heads/main", false],
    ["merge_group", "refs/heads/main", false],
    ["workflow_dispatch", "refs/heads/main", false],
  ])
    assert.equal(runInNewContext(producer.if, { github: { event_name, ref } }), expected);

  const commands = producer.steps.map((step) => step.run ?? "").join("\n");
  assert.match(commands, /assert\.equal\(process\.platform, 'linux'\)/);
  assert.match(commands, /assert\.equal\(process\.arch, 'x64'\)/);
  assert.match(commands, /process\.versions\.node, manifest\.devEngines\.runtime\.version/);
  assert.match(commands, /rev-parse.*HEAD.*process\.env\.GITHUB_SHA/);
  const final = producer.steps.at(-1);
  assert.equal(final.if, undefined, "failed identity validation must stop the producer");
  assert.equal(final["continue-on-error"], undefined);
  assert.equal(
    final.run,
    "vp install --frozen-lockfile --prefer-offline\ngit diff --exit-code -- package.json pnpm-lock.yaml pnpm-workspace.yaml\n",
  );
});
