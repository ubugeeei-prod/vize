import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  MISSKEY_HMR_EXTERNAL_TEMPLATE,
  MISSKEY_HMR_TARGETS,
} from "../app/dev/misskey-hmr-targets.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

function assertExists(...segments: string[]): void {
  assert.ok(fs.existsSync(path.join(root, ...segments)), segments.join("/"));
}

function shellWords(command: string, label: string): string[] {
  const logicalCommand = command.trim().replace(/\\\r?\n[ \t]*/g, " ");
  assert.doesNotMatch(logicalCommand, /\r?\n/, `${label} must be one logical shell command`);
  return [...logicalCommand.matchAll(/'([^']*)'|"([^"]*)"|(\S+)/g)].map(
    (match) => match[1] ?? match[2] ?? match[3],
  );
}

test("workspace exposes app e2e task aliases with scoped cache inputs", () => {
  const taskInputs = readRepoFile("tools/config/vite-plus/task-inputs.ts");
  const taskGroups = readRepoFile("tools/config/vite-plus/tasks/test-benchmark.ts");
  const testPackage = JSON.parse(readRepoFile("tests", "package.json")) as {
    scripts?: Record<string, string>;
  };

  assert.match(taskInputs, /e2e:\s*\[/);
  assert.match(taskInputs, /"tests\/app\/\*\*"/);
  assert.match(taskInputs, /"tests\/_helpers\/\*\*"/);
  assert.match(taskInputs, /"tests\/_fixtures\/\*\*"/);
  assert.match(taskInputs, /"tests\/snapshots\/\*\*"/);
  assert.match(
    taskGroups,
    /"test:e2e":\s*noCacheTask\(runTasks\("test:e2e:dev", "test:e2e:preview"\)\)/,
  );
  assert.match(taskGroups, /"test:e2e:dev":\s*task\(runInPackages\("test:dev", \["\.\/tests"\]\)/);
  assert.match(
    taskGroups,
    /"test:e2e:preview":\s*task\(runInPackages\("test:preview", \["\.\/tests"\]\)/,
  );
  assert.match(taskGroups, /"test:e2e:vrt":\s*task\(runInPackages\("test:vrt", \["\.\/tests"\]\)/);
  assert.match(taskGroups, /input:\s*cacheInputs\.e2e/);
  assert.match(testPackage.scripts?.["test:dev:ci"] ?? "", /app\/dev\/nuxt-ui\.spec\.ts/);
});

test("fast app readiness aliases use bounded pinned fixtures", () => {
  const testPackage = JSON.parse(readRepoFile("tests", "package.json")) as {
    scripts?: Record<string, string>;
  };
  const scripts = testPackage.scripts ?? {};

  assert.equal(
    scripts["test:readiness"],
    "pnpm test:readiness:check && pnpm test:readiness:lint && pnpm test:readiness:build && pnpm test:readiness:dev",
  );
  for (const [name, drivers] of [
    [
      "test:readiness:check",
      [
        "snapshots/check/compiler-macros.ts",
        "snapshots/check/elk.ts",
        "snapshots/check/misskey.ts",
        "snapshots/check/npmx.ts",
        "snapshots/check/nuxt-ui.ts",
        "snapshots/check/reka-ui.ts",
      ],
    ],
    [
      "test:readiness:lint",
      [
        "tooling/cli-lint-contract.test.ts",
        "snapshots/lint/elk.ts",
        "snapshots/lint/misskey.ts",
        "snapshots/lint/npmx.ts",
        "snapshots/lint/nuxt-ui.ts",
        "snapshots/lint/reka-ui.ts",
      ],
    ],
  ] as const) {
    const script = scripts[name] ?? "";
    const argv = shellWords(script, name);
    assert.equal(
      argv[0],
      "VIZE_TEST_WORKTREE_ID=${VIZE_TEST_WORKTREE_ID:-ci-readiness}",
      `${name} must default the shared readiness worktree`,
    );
    assert.equal(argv[1], "node", `${name} must invoke the Node.js test runner`);
    const driverStart = argv.findIndex((token, index) => index > 1 && token.endsWith(".ts"));
    assert.deepEqual(
      argv.slice(2, driverStart).toSorted(),
      ["--test", "--test-concurrency=1"].toSorted(),
      `${name} must run its drivers serially`,
    );
    assert.deepEqual(
      argv.slice(driverStart),
      drivers,
      `${name} must run exactly the release-blocking drivers`,
    );
  }
  assert.equal(
    scripts["test:readiness:build"],
    "node --test --test-concurrency=1 snapshots/build/generic.ts snapshots/build/elk.ts",
  );
  assert.equal(
    scripts["test:readiness:check:vuefes"],
    "node --test --test-concurrency=1 snapshots/check/vuefes.ts",
    "VueFes authored diagnostics must run in an isolated readiness row",
  );
  // nuxt-ui is the release gate that no pull request could exercise until it
  // joined this smoke: its SSR render output is what broke in #3736, twice, at
  // release time. Each spec gets its own Playwright run so one app's dev server
  // cannot leak into the next, matching test:dev:ci.
  assert.equal(
    scripts["test:readiness:dev"],
    "VIZE_TEST_WORKTREE_ID=${VIZE_TEST_WORKTREE_ID:-ci-readiness} playwright test --config app/playwright.config.ts app/dev/misskey.spec.ts" +
      " && VIZE_TEST_WORKTREE_ID=${VIZE_TEST_WORKTREE_ID:-ci-readiness} playwright test --config app/playwright.config.ts --retries=0 app/dev/nuxt-ui.spec.ts",
  );

  assertExists("tests", "snapshots", "check", "compiler-macros.ts");
  assertExists("tests", "tooling", "cli-lint-contract.test.ts");
  assertExists("tests", "snapshots", "build", "elk.ts");
  assertExists("tests", "snapshots", "build", "generic.ts");
  assertExists("tests", "app", "dev", "misskey.spec.ts");
  assertExists("tests", "app", "dev", "nuxt-ui.spec.ts");

  const misskeyDevSpec = readRepoFile("tests", "app", "dev", "misskey.spec.ts");
  assert.match(
    misskeyDevSpec,
    /test\("authored SFCs hot-update through pure Vite without reloading"/,
    "the pinned pure-Vite readiness spec must exercise authored-source HMR",
  );
  assert.match(misskeyDevSpec, /verifyMisskeyAuthoredSourceHmr/);
  assertExists("tests", "app", "dev", "misskey-hmr.ts");
  const misskeyHmr = readRepoFile("tests", "app", "dev", "misskey-hmr.ts");
  assert.match(misskeyHmr, /export function prepareMisskeyHmrFixture\s*\(/);
  assert.deepEqual(
    MISSKEY_HMR_TARGETS.map(({ marker, moduleSuffix, sourceRelativePath }) => ({
      marker,
      moduleSuffix,
      sourceRelativePath,
    })),
    [
      {
        marker: "data-vize-hmr-direct",
        moduleSuffix: "/src/ui/visitor.vue.ts",
        sourceRelativePath: "src/ui/visitor.vue",
      },
      {
        marker: "data-vize-hmr-dependency",
        moduleSuffix: "/src/components/MkVisitorDashboard.vue.ts",
        sourceRelativePath: "src/components/MkVisitorDashboard.hmr.html",
      },
    ],
  );
  assert.equal(
    MISSKEY_HMR_EXTERNAL_TEMPLATE,
    '<template src="./MkVisitorDashboard.hmr.html"></template>',
  );

  const nuxtUiDevSpec = readRepoFile("tests", "app", "dev", "nuxt-ui.spec.ts");
  assert.match(
    nuxtUiDevSpec,
    /test\("authored component source hot-updates without reloading"/,
    "the pinned Nuxt readiness spec must exercise authored-source HMR",
  );
  assert.match(nuxtUiDevSpec, /verifyNuxtUiAuthoredSourceHmr/);
  assert.match(
    nuxtUiDevSpec,
    /startNuxtUiDevServer\(\)/,
    "the Nuxt readiness spec must boot through the SSR-bridge recovery helper",
  );
  assertExists("tests", "app", "dev", "nuxt-ui-dev-server.ts");
  assertExists("tests", "app", "dev", "nuxt-ui-hmr.ts");
  assertExists("tests", "app", "dev", "source-restore.ts");
  const nuxtUiHmr = readRepoFile("tests", "app", "dev", "nuxt-ui-hmr.ts");
  assert.match(nuxtUiHmr, /waitForNuxtUiWatcherSettle/);
  assert.match(nuxtUiHmr, /watcher debounce window[\s\S]*?warm-up restore[\s\S]*?coalesce/s);

  for (const fixture of ["elk", "misskey", "npmx.dev", "nuxt-ui", "reka-ui"]) {
    const snapshotName = fixture === "npmx.dev" ? "npmx" : fixture;
    assertExists("tests", "snapshots", "check", `${snapshotName}.ts`);
    assertExists("tests", "snapshots", "check", "__snapshots__", `${fixture}-check.snap`);
    assertExists("tests", "snapshots", "lint", `${snapshotName}.ts`);
    assertExists("tests", "snapshots", "lint", "__snapshots__", `${fixture}-lint.snap`);
    const gitlink = execFileSync(
      "git",
      ["ls-files", "--stage", `tests/_fixtures/_git/${fixture}`],
      { cwd: root, encoding: "utf8" },
    );
    assert.match(
      gitlink,
      new RegExp(`^160000 [0-9a-f]{40} 0\\ttests/_fixtures/_git/${fixture}\\n$`),
      `${fixture} must remain pinned to an exact gitlink commit`,
    );
  }
});

test("generic compiler build snapshots stay in the blocking build inventory", () => {
  const testPackage = JSON.parse(readRepoFile("tests", "package.json")) as {
    scripts?: Record<string, string>;
  };
  const readinessWorkflow = readRepoFile(".github", "workflows", "e2e.yml");
  assert.ok(testPackage.scripts?.["test:build"]?.includes("snapshots/build/generic.ts"));
  for (const trigger of [
    "tests/_fixtures/_projects/generic-build/**",
    "tests/snapshots/build/generic.ts",
    "tests/snapshots/build/__snapshots__/generic-*.snap",
  ]) {
    assert.ok(
      readinessWorkflow.includes(`- '${trigger}'`),
      `missing readiness trigger: ${trigger}`,
    );
  }

  assertExists("tests", "snapshots", "build", "generic.ts");
  for (const snapshot of [
    "generic-directive-builtins.snap",
    "generic-fixture-panel.snap",
    "generic-normal-script-bindings.snap",
    "generic-slot-outlet.snap",
  ]) {
    assertExists("tests", "snapshots", "build", "__snapshots__", snapshot);
  }
});

test("real-world browser smoke inventory stays wired into app e2e scripts", () => {
  const testPackage = JSON.parse(readRepoFile("tests", "package.json")) as {
    scripts?: Record<string, string>;
  };
  const devScript = testPackage.scripts?.["test:dev:ci"] ?? "";
  const previewScript = testPackage.scripts?.["test:preview"] ?? "";
  const vrtScript = testPackage.scripts?.["test:vrt"] ?? "";

  const devProjects = ["elk", "misskey", "npmx", "nuxt-ui", "vuefes"];
  for (const project of devProjects) {
    const spec = `app/dev/${project}.spec.ts`;
    assertExists("tests", spec);
    assert.match(devScript, new RegExp(spec.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
  }

  const previewProjects = ["elk", "misskey", "npmx", "vuefes"];
  for (const project of previewProjects) {
    const script = `app/preview/${project}.ts`;
    assertExists("tests", script);
    assert.match(previewScript, new RegExp(script.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
  }

  const vrtProjects = ["elk", "frontend-phpcon", "misskey", "npmx", "vuefes"];
  for (const project of vrtProjects) {
    assertExists("tests", "app", "vrt", `${project}.spec.ts`);
  }
  assert.match(vrtScript, /playwright test --config app\/playwright\.vrt\.config\.ts/);
});
