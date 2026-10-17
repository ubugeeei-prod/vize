import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parse, parseAllDocuments } from "yaml";

import { runPackageScriptDirectly, runTask } from "../../tools/config/vite-plus/task-helpers.ts";
import type { PackagePath } from "../../tools/config/vite-plus/task-helpers.ts";
import { testedPackages } from "../../tools/config/vite-plus/task-inputs.ts";
import { testedPackageGroups } from "../../tools/config/vite-plus/tested-package-groups.ts";
import { testAndBenchmarkTasks } from "../../tools/config/vite-plus/tasks/test-benchmark.ts";
import { jsInstallFilters } from "../../tools/support/compat/github/install-js-checks.ts";
import { aggregateFullJsPackageResults } from "../../tools/support/compat/github/require-needs-success.ts";
import {
  planAffectedJs,
  readJsWorkspace,
} from "../../tools/support/compat/github/plan-affected-js.ts";
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
  if?: string;
  env?: Record<string, string>;
  with?: Record<string, unknown>;
};
type Job = {
  strategy?: { matrix?: string | { group: string[] }; "fail-fast"?: boolean };
  steps?: Step[];
  if?: string;
  needs?: string[];
};

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

void test("full qualification runs the complete existing partition independently of project replay", () => {
  const packages = check.jobs["full-js-package-tests"];
  const original = check.jobs["full-js-original-project"];
  assert.deepEqual(packages.strategy, {
    "fail-fast": false,
    matrix: { group: testedPackageGroups.map(({ name }) => name) },
  });
  const full = "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}";
  for (const worker of [packages, original]) {
    assert.equal(worker.if, full);
    assert.equal(worker.needs, undefined, "neither worker waits for the other");
    assert.deepEqual(
      worker.steps?.map(({ uses, if: condition, with: options }) => ({ uses, condition, options })),
      [
        {
          uses: "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
          condition: undefined,
          options: undefined,
        },
        {
          uses: "./.github/actions/setup-js-package-tests",
          condition: undefined,
          options: undefined,
        },
        {
          uses: "./.github/actions/test-js-packages-with-history",
          condition: "${{ !cancelled() }}",
          options: { partition: worker === packages ? "${{ matrix.group }}" : "original-project" },
        },
      ],
    );
  }
  const gate = check.jobs["test-js-packages"];
  assert.equal(
    gate.if,
    "${{ always() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  assert.deepEqual(gate.needs, ["full-js-package-tests", "full-js-original-project"]);
  const report = gate.steps?.at(-1);
  assert.equal(
    report?.run,
    "node tools/support/compat/github/require-needs-success.ts --full-js-packages",
  );
  assert.deepEqual(report?.env, { NEEDS_JSON: "${{ toJSON(needs) }}" });
});

void test("the stable full JS gate refuses every incomplete or unsuccessful upstream", () => {
  const keys = ["full-js-package-tests", "full-js-original-project"];
  const green = Object.fromEntries(keys.map((key) => [key, { result: "success" }]));
  assert.deepEqual(aggregateFullJsPackageResults(green), {
    exitCode: 0,
    message:
      "test-report gate: all 2 needed jobs are accounted for. 2 succeeded; 0 skipped on a pull request by design.",
  });
  for (const key of keys) {
    for (const result of ["failure", "cancelled", "skipped", "pending", "neutral"])
      assert.equal(aggregateFullJsPackageResults({ ...green, [key]: { result } }).exitCode, 1);
    assert.throws(
      () => aggregateFullJsPackageResults({ ...green, [key]: {} }),
      /reported no result/,
    );
    assert.throws(
      () => aggregateFullJsPackageResults({ ...green, [key]: null }),
      /reported no result/,
    );
    assert.throws(() => aggregateFullJsPackageResults({ [key]: green[key] }), /requires exactly/);
  }
  for (const input of [null, [], {}, "success", { ...green, omitted: { result: "success" } }])
    assert.throws(() => aggregateFullJsPackageResults(input), /requires exactly/);
  const executable = path.join(root, "tools/support/compat/github/require-needs-success.ts");
  for (const [needs, status] of [
    [green, 0],
    [{ ...green, "full-js-package-tests": { result: "failure" } }, 1],
  ] as const) {
    const actual = spawnSync(process.execPath, [executable, "--full-js-packages"], {
      env: { ...process.env, NEEDS_JSON: JSON.stringify(needs) },
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(actual.status, status, actual.stderr);
    assert.equal(actual.error, undefined);
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

void test("PR CI runs each affected group once and keeps qualification and conformance separate", () => {
  const packages = jsChecks.jobs["pr-js-package-tests"];
  assert.equal(packages.strategy?.matrix, "${{ fromJSON(inputs.js-plan).matrix }}");
  const full = planAffectedJs(readJsWorkspace(root), ["pnpm-lock.yaml"]);
  assert.deepEqual(
    full.matrix.group,
    testedPackageGroups.map(({ name }) => name),
  );
  const runs = (packages.steps ?? []).filter((step) =>
    step.run?.includes('vp run --workspace-root "test:js:$VIZE_JS_PACKAGE_GROUP"'),
  );
  assert.equal(runs.length, 1);
  assert.equal(runs[0].env?.VIZE_JS_PACKAGE_GROUP, "${{ matrix.group }}");
  assert.equal(runs[0].if, "${{ (inputs.run-js && matrix.group != 'none') }}");
  const cacheKey = (job: Job | undefined) =>
    job?.steps?.find((step) => step.uses === "./.github/actions/setup-rust-sticky-cache")?.with
      ?.key;
  // Trusted check.yml jobs write these roles with the same native builds.
  assert.equal(cacheKey(jsChecks.jobs["pr-js-package-tests"]), "test-js-packages");
  assert.equal(cacheKey(jsChecks.jobs["pr-oxlint-project-qualification"]), "test-js-packages");
  assert.equal(cacheKey(jsChecks.jobs["pr-js-package-conformance"]), "build-js-packages");
  assert.ok(
    check.jobs["full-js-package-tests"].steps?.some(
      (step) => step.uses === "./.github/actions/setup-js-package-tests",
    ),
  );
  const setup = parse(
    readRepoFile(".github", "actions", "setup-js-package-tests", "action.yml"),
  ) as { runs: Job };
  assert.equal(cacheKey(setup.runs), "test-js-packages");
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

await test("UI's actual production Vue consumer has its locked fixture in the filtered install only", () => {
  const configPath = path.join(root, "npm/ui/vite.production-vue.config.ts");
  const source = fs.readFileSync(configPath, "utf8");
  const selected = source.match(
    /const example = fileURLToPath\(new URL\("([^"]+)", import\.meta\.url\)\)/u,
  );
  assert.ok(selected, "the real production configuration selects an example package");
  const fixtureManifest = fileURLToPath(new URL(selected[1], pathToFileURL(configPath)));
  const fixture = path.relative(root, path.dirname(fixtureManifest)).split(path.sep).join("/");
  const fixtureFilter = `./${fixture}...`;
  assert.equal(fixtureFilter, "./npm/builder/vite/example...");
  const installerPlan = planAffectedJs(readJsWorkspace(root), [
    "tools/support/compat/github/install-js-checks.ts",
  ]);
  const groups = testedPackageGroups.map(({ name }) => name);
  assert.deepEqual(installerPlan.groups, groups);
  assert.deepEqual(installerPlan.matrix.group, groups);
  assert.equal(installerPlan.scope, "affected", "shared runtime work is not a full workspace plan");
  assert.equal(installerPlan.native, false, "the installer does not change native product inputs");
  for (const { packages } of testedPackageGroups) {
    for (const pkg of packages) assert.ok(installerPlan.packages.includes(pkg.slice(2)), pkg);
  }
  const ui = manifests.get("./npm/ui");
  assert.ok(ui?.scripts?.test?.includes("vp test run --config vite.production-vue.config.ts"));
  assert.match(source, /assert\.equal\(manifest\.version, "3\.5\.42"\)/u);
  assert.match(source, /assert\.equal\(compiler\.version, "3\.5\.42"\)/u);

  const owner = JSON.parse(fs.readFileSync(fixtureManifest, "utf8")) as Manifest;
  assert.equal(owner.dependencies?.vue, "catalog:vue-production");
  const workspace = parse(readRepoFile("pnpm-workspace.yaml")) as {
    catalogs: { "vue-production": { vue: string } };
  };
  type ProjectLock = {
    catalogs: { "vue-production": { vue: { specifier: string; version: string } } };
    importers: Record<string, { dependencies: { vue: { specifier: string; version: string } } }>;
  };
  const projectLocks = parseAllDocuments(readRepoFile("pnpm-lock.yaml"))
    .map((document) => {
      assert.deepEqual(document.errors, []);
      return document.toJS() as ProjectLock;
    })
    .filter((document) => Object.hasOwn(document.importers, fixture));
  assert.equal(projectLocks.length, 1, "exactly one lock document owns the production fixture");
  const lock = projectLocks[0];
  assert.equal(workspace.catalogs["vue-production"].vue, "3.5.42");
  assert.deepEqual(lock.catalogs["vue-production"].vue, { specifier: "3.5.42", version: "3.5.42" });
  assert.equal(lock.importers[fixture].dependencies.vue.specifier, "catalog:vue-production");
  assert.match(lock.importers[fixture].dependencies.vue.version, /^3\.5\.42(?:\(|$)/u);

  const packageName = (JSON.parse(readRepoFile("package.json")) as Manifest).name;
  for (const { name, packages, prebuilt } of testedPackageGroups) {
    const baseline = [
      packageName,
      "./npm/native...",
      ...[...packages, ...prebuilt].map((pkg) => `${pkg}...`),
    ];
    assert.deepEqual(
      jsInstallFilters(name),
      [...new Set([...baseline, ...(name === "ui" ? [fixtureFilter] : [])])],
      name,
    );
  }
  for (const [group, pkg] of [
    ["ui-conformance", "./npm/ui"],
    ["fresco-conformance", "./npm/fresco-native"],
    ["original-project", "./npm/oxlint"],
  ]) {
    assert.deepEqual(jsInstallFilters(group), [packageName, "./npm/native...", `${pkg}...`], group);
  }
});
