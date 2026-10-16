import assert from "node:assert/strict";
import test from "node:test";
import { planCheckInputs } from "../../tools/support/compat/github/plan-check-inputs.ts";

test("prose and tooling metadata do not rebuild product apps", () => {
  assert.deepEqual(planCheckInputs(["docs/content/guide.md", "docs/davinci/decisions/test.md"]), {
    full: false,
    rust: false,
    dependencies: false,
    engines: false,
    apps: false,
  });
  assert.equal(planCheckInputs(["tests/tooling/plan-source-checks.test.ts"]).apps, false);
});

test("product, compiled documentation and cache changes retain app checks", () => {
  for (const path of [
    "davinci/vize_l2/src/lib.rs",
    "crates/vize/src/main.rs",
    "vendor/lightningcss/src/lib.rs",
    "npm/builder/vite/src/index.ts",
    "npm/cli/schemas/vize.config.schema.json",
    "playground/src/App.vue",
    "docs/davinci/plan/canonical-corpus/budgets.toml",
    "docs/davinci/plan/fact-alpha-schemas.md",
    ".github/actions/setup-rust-sticky-cache/action.yml",
  ]) {
    assert.equal(planCheckInputs([path]).apps, true, path);
  }
});

test("dependency and security changes always audit and check engine compatibility", () => {
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

test("scheduled and manual qualification never select away checks", () => {
  assert.equal(Object.values(planCheckInputs([], true)).every(Boolean), true);
});

test("changing selector and task definitions exercises every affected consumer", () => {
  for (const path of [
    ".github/workflows/check.yml",
    "tools/support/compat/github/plan-check-inputs.ts",
    "tools/config/vite-plus/tasks/check.ts",
  ]) {
    const plan = planCheckInputs([path]);
    assert.equal(plan.apps, true, path);
    assert.equal(plan.dependencies, true, path);
    assert.equal(plan.engines, true, path);
  }
  assert.equal(planCheckInputs([".cargo/audit.toml"]).dependencies, true);
});
