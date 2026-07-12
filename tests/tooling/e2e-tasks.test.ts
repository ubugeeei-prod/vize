import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

function assertExists(...segments: string[]): void {
  assert.ok(fs.existsSync(path.join(root, ...segments)), segments.join("/"));
}

test("workspace exposes app e2e task aliases with scoped cache inputs", () => {
  const taskInputs = readRepoFile("tools/vite-plus/task-inputs.ts");
  const taskGroups = readRepoFile("tools/vite-plus/tasks/test-benchmark.ts");
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
  assert.equal(
    scripts["test:readiness:check"],
    "node --test --test-concurrency=1 snapshots/check/compiler-macros.ts",
  );
  assert.equal(
    scripts["test:readiness:lint"],
    "node --test --test-concurrency=1 tooling/cli-lint-contract.test.ts",
  );
  assert.equal(
    scripts["test:readiness:build"],
    "node --test --test-concurrency=1 snapshots/build/generic.ts snapshots/build/elk.ts",
  );
  assert.equal(
    scripts["test:readiness:dev"],
    "VIZE_TEST_WORKTREE_ID=${VIZE_TEST_WORKTREE_ID:-ci-readiness} playwright test --config app/playwright.config.ts app/dev/misskey.spec.ts",
  );

  assertExists("tests", "snapshots", "check", "compiler-macros.ts");
  assertExists("tests", "tooling", "cli-lint-contract.test.ts");
  assertExists("tests", "snapshots", "build", "elk.ts");
  assertExists("tests", "snapshots", "build", "generic.ts");
  assertExists("tests", "app", "dev", "misskey.spec.ts");

  for (const fixture of ["elk", "misskey"]) {
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
