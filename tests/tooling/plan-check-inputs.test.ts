import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { planCheckInputs } from "../../tools/support/compat/github/plan-check-inputs.ts";

await test("prose and tooling metadata do not rebuild product apps", () => {
  assert.deepEqual(planCheckInputs(["docs/content/guide.md", "docs/davinci/decisions/test.md"]), {
    full: false,
    rust: false,
    dependencies: false,
    engines: false,
    apps: false,
  });
  assert.equal(planCheckInputs(["tests/tooling/plan-source-checks.test.ts"]).apps, false);
  assert.equal(planCheckInputs(["npm/compose/core/examples/README.md"]).apps, false);
  assert.equal(planCheckInputs(["npm/compose/core/fixtures/invalid.vue"]).apps, false);
  assert.equal(planCheckInputs(["npm/compose/core/examples/invalid.fixture.vue"]).apps, false);
});

await test("product, compiled documentation and cache changes retain app checks", () => {
  for (const path of [
    "davinci/vize_l2/src/lib.rs",
    "crates/vize/src/main.rs",
    "vendor/lightningcss/src/lib.rs",
    "npm/builder/vite/src/index.ts",
    "npm/cli/schemas/vize.config.schema.json",
    "playground/src/App.vue",
    "npm/compose/core/examples/virtual-list.vue",
    "npm/compose/core/src/virtual-list.ts",
    "npm/ui/scripts/reference-docs/composable-examples.ts",
    "docs/theme/components/Guide.vue",
    "examples/html-conformance/src/index.html",
    "docs/davinci/plan/canonical-corpus/budgets.toml",
    "docs/davinci/plan/fact-alpha-schemas.md",
    ".github/actions/setup-rust-sticky-cache/action.yml",
  ]) {
    assert.equal(planCheckInputs([path]).apps, true, path);
  }
});

await test("dependency and security changes always audit and check engine compatibility", () => {
  for (const path of [
    "Cargo.lock",
    "davinci/vize_l1/Cargo.toml",
    "npm/cli/package.json",
    "pnpm-lock.yaml",
    "patches/listhen.patch",
    "tools/support/security/node-forge-proof.ts",
    "tools/commands/ci/npm-audit-retry.ts",
  ]) {
    const plan = planCheckInputs([path]);
    assert.equal(plan.dependencies, true, path);
    assert.equal(plan.engines, true, path);
    assert.equal(plan.apps, true, path);
  }
});

await test("npm Vue examples select the current-source opinionated lint consumer", () => {
  for (const path of [
    "npm/compose/core/examples/virtual-list.vue",
    "npm/compose/future/examples/nested/NewExample.vue",
    "npm/ui/core/src/NewComponent.vue",
  ]) {
    assert.deepEqual(
      planCheckInputs([path]),
      {
        full: false,
        rust: false,
        dependencies: false,
        engines: false,
        apps: true,
      },
      path,
    );
  }
});

await test("npm prose and intentional fixtures stay outside authored source lint", () => {
  for (const path of [
    "npm/compose/core/examples/README.md",
    "npm/compose/core/fixtures/invalid.vue",
    "npm/compose/core/examples/invalid.fixture.vue",
  ]) {
    assert.equal(planCheckInputs([path]).apps, false, path);
  }
});

await test("the actual PR and merge-group planner selects a newly added npm Vue example", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-check-example-plan-"));
  const git = (...args: string[]) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  const script = fileURLToPath(
    new URL("../../tools/support/compat/github/plan-check-inputs.ts", import.meta.url),
  );
  try {
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    git("remote", "add", "origin", cwd);
    writeFileSync(join(cwd, "README.md"), "before\n");
    git("add", ".");
    git("commit", "-qm", "initial source");
    const base = git("rev-parse", "HEAD");
    const examples = join(cwd, "npm/compose/core/examples");
    mkdirSync(examples, { recursive: true });
    writeFileSync(join(examples, "virtual-list.vue"), "<template><div /></template>\n");
    git("add", ".");
    git("commit", "-qm", "add ordinary Vue example");
    const head = git("rev-parse", "HEAD");
    for (const event of ["pull_request", "merge_group"]) {
      const output = join(cwd, `output-${event}`);
      const run = spawnSync(process.execPath, [script], {
        cwd,
        encoding: "utf8",
        env: {
          ...process.env,
          GITHUB_EVENT_NAME: event,
          EVENT_BASE_SHA: base,
          GITHUB_SHA: head,
          PR_HEAD_SHA: head,
          GITHUB_OUTPUT: output,
          GITHUB_STEP_SUMMARY: "",
        },
      });
      assert.equal(run.status, 0, run.stderr);
      assert.equal(
        readFileSync(output, "utf8"),
        "full=false\nrust=false\ndependencies=false\nengines=false\napps=true\n",
        event,
      );
    }
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

await test("scheduled and manual qualification never select away checks", () => {
  assert.equal(Object.values(planCheckInputs([], true)).every(Boolean), true);
});

await test("changing selector and task definitions exercises every affected consumer", () => {
  for (const path of [
    ".github/workflows/check.yml",
    "tools/support/compat/github/plan-check-inputs.ts",
    "tools/support/github/opinionated-source-inputs.ts",
    "tools/config/vite-plus/tasks/check.ts",
  ]) {
    const plan = planCheckInputs([path]);
    assert.equal(plan.apps, true, path);
    assert.equal(plan.dependencies, true, path);
    assert.equal(plan.engines, true, path);
  }
  assert.equal(planCheckInputs([".cargo/audit.toml"]).dependencies, true);
});
