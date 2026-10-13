import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { runPackageScriptDirectly, runTask } from "../../tools/config/vite-plus/task-helpers.ts";
import type { PackagePath } from "../../tools/config/vite-plus/task-helpers.ts";
import { testedPackages } from "../../tools/config/vite-plus/task-inputs.ts";
import { testedPackageGroups } from "../../tools/config/vite-plus/tested-package-groups.ts";
import { testAndBenchmarkTasks } from "../../tools/config/vite-plus/tasks/test-benchmark.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Manifest = {
  name: string;
  scripts?: Record<string, string>;
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
  peerDependencies?: Record<string, string>;
  optionalDependencies?: Record<string, string>;
};
type Step = {
  name?: string;
  id?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};
type Job = { strategy?: { matrix?: { group?: string[] } }; steps?: Step[] };

const jsChecks = parse(readRepoFile(".github", "workflows", "pr-js-checks.yml")) as {
  jobs: Record<string, Job>;
};
const check = parse(readRepoFile(".github", "workflows", "check.yml")) as {
  jobs: Record<string, Job>;
};
const command = (name: string) => {
  const entry = (testAndBenchmarkTasks as Record<string, { command?: string; cache?: boolean }>)[
    name
  ];
  assert.ok(entry?.command, `missing task ${name}`);
  assert.equal(entry.cache, false, `${name} always runs`);
  return entry.command;
};

// Every workspace package, so dependency chains through untested packages count.
const manifests = new Map<PackagePath, Manifest>(
  execFileSync("git", ["ls-files", "--", "npm/**/package.json"], { cwd: root, encoding: "utf8" })
    .trim()
    .split("\n")
    .map((file) => [
      `./${path.dirname(file)}` as PackagePath,
      JSON.parse(fs.readFileSync(path.join(root, file), "utf8")) as Manifest,
    ]),
);
const byName = new Map([...manifests].map(([directory, manifest]) => [manifest.name, directory]));
const workspaceDependencies = (pkg: PackagePath) => {
  const manifest = manifests.get(pkg);
  assert.ok(manifest, `${pkg} is a workspace package`);
  return Object.entries({
    ...manifest.dependencies,
    ...manifest.devDependencies,
    ...manifest.peerDependencies,
    ...manifest.optionalDependencies,
  })
    .filter(([, version]) => version.startsWith("workspace:"))
    .map(([name]) => byName.get(name))
    .filter((directory): directory is PackagePath => directory !== undefined);
};
const testedDependencies = (pkg: PackagePath) => {
  const seen = new Set<PackagePath>();
  const pending = workspaceDependencies(pkg);
  for (let next = pending.pop(); next; next = pending.pop()) {
    if (seen.has(next)) continue;
    seen.add(next);
    pending.push(...workspaceDependencies(next));
  }
  return [...seen].filter((dependency) => (testedPackages as PackagePath[]).includes(dependency));
};

void test("PR package groups test every package in testedPackages exactly once", () => {
  const names = testedPackageGroups.map(({ name }) => name);
  assert.equal(new Set(names).size, names.length, "group names are unique");
  for (const { name, packages } of testedPackageGroups)
    assert.ok(packages.length > 0, `${name} tests at least one package`);
  const grouped: PackagePath[] = testedPackageGroups.flatMap(({ packages }) => packages);
  assert.equal(new Set(grouped).size, grouped.length, "no package is tested by two groups");
  assert.deepEqual([...grouped].sort(), [...testedPackages].sort());
  // `vp run --filter ./dir` also selects packages below that directory.
  const selectedTests = (filters: readonly PackagePath[]) =>
    [...manifests]
      .filter(
        ([directory, manifest]) =>
          manifest.scripts?.test &&
          filters.some((filter) => directory === filter || directory.startsWith(`${filter}/`)),
      )
      .map(([directory]) => directory);
  const perGroup = testedPackageGroups.flatMap(({ packages }) => selectedTests(packages));
  assert.equal(new Set(perGroup).size, perGroup.length, "no test script runs in two groups");
  assert.deepEqual(perGroup.sort(), selectedTests(testedPackages).sort());
});

void test("each group holds its tested dependencies or builds their test bundles first", () => {
  for (const { name, packages, prebuilt } of testedPackageGroups) {
    const members = new Set<PackagePath>(packages);
    const required = new Set<PackagePath>();
    for (const pkg of packages)
      for (const dependency of testedDependencies(pkg))
        if (!members.has(dependency)) required.add(dependency);
    // Exactly the outside dependencies: none missing and no unneeded build.
    assert.deepEqual([...prebuilt].sort(), [...required].sort(), name);
    for (const pkg of prebuilt) {
      const scripts = manifests.get(pkg)?.scripts ?? {};
      assert.equal(scripts.build, "vp pack", `${pkg} build is the bundle step`);
      assert.ok(
        scripts.test?.startsWith(`${scripts.build} && `),
        `${pkg} tests start with the same bundle step its dependents read`,
      );
    }
  }
});

void test("group tasks keep the full set's preparation, wrapper, order and concurrency", () => {
  const full = command("test:js");
  const filters = testedPackages.map((pkg) => ` --filter '${pkg}'`).join("");
  assert.ok(full.endsWith(`${filters} test`), "the full task filters testedPackages in order");
  const prefix = full.slice(0, -`${filters} test`.length);
  assert.equal(
    prefix,
    `${runTask("build:native:test")} && VP_RUN_CONCURRENCY_LIMIT=1 node npm/native/scripts/test-preparation.mjs vp run`,
  );
  const [native, packageRun] = prefix.split(" && ");
  for (const { name, packages, prebuilt } of testedPackageGroups) {
    const selected = testedPackages
      .filter((pkg) => (packages as readonly PackagePath[]).includes(pkg))
      .map((pkg) => ` --filter '${pkg}'`)
      .join("");
    assert.equal(
      command(`test:js:${name}`),
      [
        native,
        ...(prebuilt.length > 0 ? [runPackageScriptDirectly("build", prebuilt)] : []),
        `${packageRun}${selected} test`,
      ].join(" && "),
      name,
    );
  }
  assert.equal(
    command("build:oxlint-qualification"),
    `${native} && ${runPackageScriptDirectly("build", ["./npm/oxlint"])}`,
  );
});

void test("PR CI runs every group once and the qualification and conformance jobs separately", () => {
  const groups = jsChecks.jobs["pr-js-package-tests"]?.strategy?.matrix?.group;
  assert.deepEqual(
    groups,
    testedPackageGroups.map(({ name }) => name),
  );
  const cacheKey = (job: Job | undefined) =>
    job?.steps?.find((step) => step.uses === "./.github/actions/setup-rust-sticky-cache")?.with
      ?.key;
  // Trusted check.yml jobs write these roles with the same native builds.
  assert.equal(cacheKey(jsChecks.jobs["pr-js-package-tests"]), "test-js-packages");
  assert.equal(cacheKey(jsChecks.jobs["pr-oxlint-project-qualification"]), "test-js-packages");
  assert.equal(cacheKey(jsChecks.jobs["pr-js-package-conformance"]), "build-js-packages");
  assert.equal(cacheKey(check.jobs["test-js-packages"]), undefined, "uses its setup action");
  assert.equal(cacheKey(check.jobs["build-js-packages"]), "build-js-packages");

  const qualification = jsChecks.jobs["pr-oxlint-project-qualification"]?.steps ?? [];
  const build = qualification.findIndex(
    (step) => step.run === "vp run --workspace-root build:oxlint-qualification",
  );
  const qualify = qualification.findIndex(
    (step) => step.run === "node npm/oxlint/scripts/check-project-transport.mjs",
  );
  assert.ok(build >= 0 && build < qualify, "the job builds the addon and bundles it qualifies");

  const conformance = (jsChecks.jobs["pr-js-package-conformance"]?.steps ?? [])
    .flatMap((step) => step.run ?? [])
    .join(" && ");
  const trusted = (check.jobs["build-js-packages"]?.steps ?? [])
    .flatMap((step) => step.run ?? [])
    .join("\n");
  assert.ok(conformance.includes("vp run --filter './npm/native' build:ci"));
  for (const run of conformance.split(" && ").filter((part) => part.startsWith("vp run --filter")))
    assert.ok(trusted.includes(run), `the trusted build-js-packages job also runs ${run}`);
});
