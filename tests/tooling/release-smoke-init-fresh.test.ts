import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  managerCommand,
  managerEnv,
  projectEnv,
  runManager,
} from "../../tools/npm/smoke-release-init-project.mjs";
import { PACKAGE_MANAGERS } from "../../tools/npm/smoke-release-init-managers.mjs";
import { FRESH_INIT_MATRIX, PROJECT_SHAPES } from "../../tools/npm/smoke-release-init-shapes.mjs";
import { readRepoFile } from "./support/github-workflows.ts";
import {
  COREPACK_MANAGER_SPECS,
  MANAGER_KEYS,
  SHAPE_KEYS,
  byCodeUnit,
} from "./support/release-smoke-init-contract.ts";

const RUNTIME_PACKAGE_MANAGER_ACTION = "./.github/actions/setup-runtime-package-managers";

test("the fresh-project matrix is data, so new cells need no driver change", () => {
  assert.ok(FRESH_INIT_MATRIX.length > 0, "the fresh-project matrix must run at least one cell");
  for (const cell of FRESH_INIT_MATRIX) {
    const manager = PACKAGE_MANAGERS[cell.packageManager];
    const shape = PROJECT_SHAPES[cell.shape];
    assert.ok(manager, `unknown package manager ${cell.packageManager}`);
    assert.ok(shape, `unknown project shape ${cell.shape}`);
    for (const key of MANAGER_KEYS) {
      assert.ok(key in manager, `${cell.packageManager} is missing ${key}`);
    }
    for (const key of SHAPE_KEYS) {
      assert.ok(key in shape, `${cell.shape} is missing ${key}`);
    }
    // The plan the smoke asserts must be the plan it installs, and the install
    // must leave the project declaring nothing beyond it.
    for (const name of shape.plannedDependencies) {
      assert.ok(
        shape.expectedDevDependencies.includes(name),
        `${cell.shape} plans ${name} but does not expect it in devDependencies`,
      );
      assert.ok(shape.requires.includes(name), `${cell.shape} plans unpacked ${name}`);
    }
    assert.deepEqual(
      [...shape.expectedDevDependencies].sort(byCodeUnit),
      shape.expectedDevDependencies,
      `${cell.shape} devDependency expectation must be sorted for a stable comparison`,
    );
  }
});

test("the fresh-project matrix covers every documented package manager", () => {
  const guide = readRepoFile("docs", "content", "guide", "init.md");
  const packageManagerRow = guide
    .split("\n")
    .find((line) => line.includes("--package-manager <PM>"));
  assert.ok(packageManagerRow, "docs/content/guide/init.md must document package managers");
  const documented = [...packageManagerRow.matchAll(/`([^`]+)`/gu)]
    .map((match) => match[1])
    .filter((value) => value !== "--package-manager <PM>");
  assert.deepEqual(documented, ["pnpm", "npm", "yarn", "bun", "vp"]);
  const matrixManagers = new Set(FRESH_INIT_MATRIX.map((cell) => cell.packageManager));
  for (const manager of documented) {
    assert.ok(matrixManagers.has(manager), `fresh-project matrix does not cover ${manager}`);
  }
});

test("the fresh-project matrix covers every declared project shape", () => {
  const matrixShapes = new Set(FRESH_INIT_MATRIX.map((cell) => cell.shape));
  for (const shape of Object.keys(PROJECT_SHAPES)) {
    assert.ok(matrixShapes.has(shape), `fresh-project matrix does not cover ${shape}`);
  }
});

test("fresh-project package managers use exact Corepack runners where needed", () => {
  for (const [managerId, corepackSpec] of Object.entries(COREPACK_MANAGER_SPECS)) {
    const manager = PACKAGE_MANAGERS[managerId];
    assert.equal(manager.corepackSpec, corepackSpec);
    assert.deepEqual(managerCommand(manager, ["install"]), {
      command: "corepack",
      args: [corepackSpec, "install"],
    });
    assert.equal(managerEnv(manager).COREPACK_ENABLE_PROJECT_SPEC, "0");
  }
  assert.equal(
    managerEnv(PACKAGE_MANAGERS.yarn, { CI: "true" }).YARN_ENABLE_IMMUTABLE_INSTALLS,
    "false",
  );
  assert.equal(
    managerEnv(PACKAGE_MANAGERS.yarn, { YARN_ENABLE_IMMUTABLE_INSTALLS: "true" })
      .YARN_ENABLE_IMMUTABLE_INSTALLS,
    "true",
  );
  for (const managerId of ["npm", "bun", "vp"]) {
    const manager = PACKAGE_MANAGERS[managerId];
    assert.equal("corepackSpec" in manager, false, `${managerId} should keep its direct runner`);
    assert.deepEqual(managerCommand(manager, ["install"]), {
      command: manager.binary,
      args: ["install"],
    });
    assert.equal(managerEnv(manager).COREPACK_ENABLE_PROJECT_SPEC, undefined);
  }
  assert.match(runManager(PACKAGE_MANAGERS.npm, ["--version"], { cwd: process.cwd() }), /^\d+\./u);
});

test("every shape drives a clean, broken, and repaired check", () => {
  for (const shape of Object.values(PROJECT_SHAPES)) {
    const broken = Object.keys(shape.check.broken);
    assert.ok(broken.length > 0, `${shape.id} has no broken variant`);
    const authored = Object.keys(
      shape.files({ typescript: "0", vite: "0", "vite-plus": "0", vue: "0" }),
    );
    for (const name of broken) {
      assert.ok(authored.includes(name), `${shape.id} breaks ${name}, which it never authored`);
    }
    const reported = shape.check.brokenDiagnostics.map((entry) => entry.file);
    assert.deepEqual(reported, broken, `${shape.id} must assert every broken file's diagnostics`);
    for (const entry of shape.check.brokenDiagnostics) {
      assert.ok(
        entry.diagnostics.length > 0,
        `${shape.id} expects no diagnostics for ${entry.file}`,
      );
      for (const diagnostic of entry.diagnostics) {
        // Full authored position and message, never a code-only assertion.
        assert.match(diagnostic, /^error:\d+:\d+ \[TS\d+\] .+\.$/u);
      }
    }
  }
});

test("the smoke only passes init flags the guide documents", () => {
  const guide = readRepoFile("docs", "content", "guide", "init.md");
  const documented = new Set([...guide.matchAll(/`(--?[a-z-]+)`/gu)].map((match) => match[1]));
  // Added by the driver itself around the shape's own flag set.
  for (const flag of ["--dry-run", "--no-install"]) {
    assert.ok(documented.has(flag), `docs/content/guide/init.md must document ${flag}`);
  }
  for (const shape of Object.values(PROJECT_SHAPES)) {
    for (const flag of shape.initFlags) {
      assert.ok(documented.has(flag), `${shape.id} passes undocumented init flag ${flag}`);
    }
  }
  // The idempotent run the smoke asserts is the one the guide prints verbatim.
  assert.ok(guide.includes("[vize init] nothing to do; the project is already configured"));

  // The guide's non-interactive example is the invocation the smoke stands in
  // for, so every feature it names must be answered explicitly -- either taken
  // or refused. A deferred feature then shows up as a visible `--no-` flag
  // rather than as a silent gap in coverage.
  const example = guide.match(/^vpx vize init (--[^\n]+)$/mu);
  assert.ok(example, "docs/content/guide/init.md must show a non-interactive example");
  for (const shape of Object.values(PROJECT_SHAPES)) {
    for (const flag of example[1].split(" ")) {
      const refused = `--no-${flag.replace(/^--/u, "")}`;
      assert.ok(
        shape.initFlags.includes(flag) || shape.initFlags.includes(refused),
        `${shape.id} neither takes nor refuses the documented ${flag}`,
      );
    }
  }
});

/** Arguments of every `smoke-release-install.mjs` step in a workflow. */
function smokeInstallInvocations(workflow: string): string[][] {
  const parsed = parse(readRepoFile(".github", "workflows", workflow)) as {
    jobs?: Record<string, { steps?: Array<{ run?: string }> }>;
  };
  const invocations: string[][] = [];
  for (const job of Object.values(parsed.jobs ?? {})) {
    for (const step of job.steps ?? []) {
      if (!step.run?.includes("smoke-release-install.mjs")) continue;
      invocations.push(step.run.replace(/\\\n/gu, " ").trim().split(/\s+/u));
    }
  }
  assert.ok(invocations.length > 0, `${workflow} runs no smoke-release-install.mjs step`);
  return invocations;
}

function workflowSteps(workflow: string, jobName: string) {
  const parsed = parse(readRepoFile(".github", "workflows", workflow)) as {
    jobs?: Record<
      string,
      {
        steps?: Array<{
          name?: string;
          run?: string;
          uses?: string;
          with?: Record<string, string>;
        }>;
      }
    >;
  };
  return parsed.jobs?.[jobName]?.steps ?? [];
}

function runtimePackageManagerActionSteps() {
  const parsed = parse(
    readRepoFile(".github", "actions", "setup-runtime-package-managers", "action.yml"),
  ) as {
    runs?: {
      steps?: Array<{
        name?: string;
        run?: string;
        shell?: string;
        uses?: string;
        with?: Record<string, unknown>;
      }>;
    };
  };
  return parsed.runs?.steps ?? [];
}

test("the release runtime smoke runs the fresh-project matrix", () => {
  const runtime = readRepoFile("tools", "npm", "smoke-release-runtime.mjs");
  // The context keys the fresh-project driver needs, independent of the order
  // and line breaks the call site happens to use.
  const freshCall = /runFreshProjectInitChecks\(\{([^}]*)\}\)/u.exec(runtime);
  assert.ok(freshCall, "the runtime smoke must call runFreshProjectInitChecks");
  const context = new Set(
    freshCall[1]
      .split(",")
      .map((entry) => entry.split(":")[0].trim())
      .filter((name) => name.length > 0),
  );
  for (const key of ["tempDir", "vizeBin"]) {
    assert.ok(context.has(key), `runFreshProjectInitChecks must receive ${key}`);
  }
  assert.match(runtime, /from "\.\/smoke-release-init-fresh\.mjs"/u);
  const installer = readRepoFile("tools", "npm", "smoke-release-install.mjs");
  assert.match(
    installer,
    /runRuntimeChecks\(installDir, installable, \{\s*allPackages: packages,/u,
    "fresh-project redirects must see pack-only optional platform tarballs",
  );

  const project = readRepoFile("tools", "npm", "smoke-release-init-project.mjs");
  // The isolation contract: outside the install tree, outside the checkout, and
  // no ancestor that could resolve `vize` for the project.
  assert.match(project, /is inside the install tree/u);
  assert.match(project, /is inside the Vize checkout/u);
  assert.match(project, /would leak into the fresh project/u);
  assert.match(project, /installed vize did not bring project-local TypeScript 7 for Corsa/u);
  assert.match(project, /a missing Corsa runtime silently disabled type checking/u);

  for (const workflow of ["release.yml", "native-smoke.yml"]) {
    // Assert the step's own arguments, so reflowing the YAML command cannot
    // silently drop the packed CLI from the runtime smoke.
    const runtimeSmokeArgs = smokeInstallInvocations(workflow).filter((args) =>
      args.includes("--runtime-checks"),
    );
    assert.ok(
      runtimeSmokeArgs.some(
        (args) => args.includes("--prepare-manifests") && args.includes("npm/cli"),
      ),
      `${workflow} must run the runtime smoke over the packed CLI`,
    );
  }
});

function assertRuntimeSmokePackageManagers(workflow: string, jobName: string) {
  const steps = workflowSteps(workflow, jobName);
  const setupIndex = steps.findIndex((step) => step.uses === RUNTIME_PACKAGE_MANAGER_ACTION);
  const runtimeSmokeIndex = steps.findIndex((step) =>
    step.run?.includes("smoke-release-install.mjs --prepare-manifests --runtime-checks"),
  );
  assert.notEqual(setupIndex, -1, `${workflow} ${jobName} must install runtime managers`);
  assert.notEqual(runtimeSmokeIndex, -1, `${workflow} ${jobName} must run runtime package smoke`);
  assert.ok(
    setupIndex < runtimeSmokeIndex,
    "package managers must be active before the fresh-project matrix runs",
  );
}

function assertRuntimePackageManagerAction() {
  const steps = runtimePackageManagerActionSteps();
  const setupIndex = steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  const shimsIndex = steps.findIndex((step) => step.name === "Enable package manager shims");
  const bunIndex = steps.findIndex((step) => step.name === "Install Bun package manager");
  assert.notEqual(setupIndex, -1, "runtime setup must install Node through setup-vp");
  assert.notEqual(shimsIndex, -1, "runtime setup must expose pnpm/yarn Corepack shims");
  assert.notEqual(bunIndex, -1, "runtime setup must install the bun matrix manager");
  assert.ok(
    setupIndex < shimsIndex && shimsIndex < bunIndex,
    "runtime managers must be installed in Node/Corepack/Bun order",
  );
  assert.equal(
    steps[setupIndex].uses,
    "voidzero-dev/setup-vp@ca1c46663915d6c1042ae23bd39ab85718bfb0fa",
  );
  assert.deepEqual(steps[setupIndex].with, {
    "node-version-file": "${{ inputs.node-version-file }}",
    cache: "${{ inputs.cache }}",
    "run-install": false,
  });
  assert.equal(steps[shimsIndex].shell, "bash");
  assert.equal(steps[shimsIndex].run, "corepack enable");
  assert.equal(steps[bunIndex].uses, "oven-sh/setup-bun@0c5077e51419868618aeaa5fe8019c62421857d6");
  assert.deepEqual(steps[bunIndex].with, { "bun-version": "1.3.14" });
}

test("fresh install runtime smokes expose every package manager they matrix", () => {
  assertRuntimePackageManagerAction();
  assertRuntimeSmokePackageManagers("native-smoke.yml", "fresh-install-smoke");
  assertRuntimeSmokePackageManagers("release.yml", "smoke-release-packages");
});

test("the fresh project runs with the host's Corsa overrides stripped", () => {
  // A host that exported these would let its own runtime satisfy the check and
  // hide a packaging failure, so assert the environment the driver hands out.
  const overrides = ["CORSA_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE"];
  const host = Object.fromEntries(overrides.map((name) => [name, "/host/corsa"]));
  const previous = overrides.map((name) => [name, process.env[name]] as const);
  try {
    Object.assign(process.env, host);
    const stripped = projectEnv();
    for (const name of overrides) {
      assert.ok(!(name in stripped), `projectEnv must drop the host's ${name}`);
    }
    // An override the smoke passes on purpose still reaches the child.
    assert.equal(projectEnv({ CORSA_PATH: "/missing" }).CORSA_PATH, "/missing");
  } finally {
    for (const [name, value] of previous) {
      if (value === undefined) delete process.env[name];
      else process.env[name] = value;
    }
  }
});
