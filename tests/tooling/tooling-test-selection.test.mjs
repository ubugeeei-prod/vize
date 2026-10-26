import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { mergeOnlyToolingTests } from "../../tools/config/vite-plus/tooling-test-scopes.ts";
import {
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import { toolingTestCommand } from "../../tools/support/compat/github/run-tooling-tests.mjs";

function fixture() {
  const cwd = mkdtempSync(join(tmpdir(), "vize-tooling-inputs-"));
  for (const directory of [
    "tests/tooling/support",
    "tests/tooling/davinci",
    "tests/tooling/release",
    "tools",
    "docs",
  ]) {
    mkdirSync(join(cwd, directory), { recursive: true });
  }
  const write = (file, source = "") => writeFileSync(join(cwd, file), source);
  write("tests/tooling/davinci-phase4-contract-status.test.ts", 'import "./support/plan.ts";');
  write("tests/tooling/support/plan.ts", 'export { value } from "../../../tools/plan-input.mjs";');
  write("tools/plan-input.mjs", "export const value = 1;");
  write("tests/tooling/native.test.ts");
  write("tests/tooling/lsp-smoke.test.ts");
  return { cwd, write, cleanup: () => rmSync(cwd, { recursive: true, force: true }) };
}

void test("a source edit selects only the tests that import or name it", () => {
  const f = fixture();
  try {
    assert.deepEqual(planToolingTests(["davinci/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests, []);
    f.write("tests/tooling/native.test.ts", 'const crate = ["davinci", "vize_l1"];');
    assert.deepEqual(planToolingTests(["davinci/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests, [
      "tests/tooling/native.test.ts",
    ]);
    const docs = planToolingTests(["docs/davinci/plan/phase-4.md"], { cwd: f.cwd });
    assert.ok(docs.tests.includes("tests/tooling/davinci-phase4-contract-status.test.ts"));
  } finally {
    f.cleanup();
  }
});

void test("transitive imports and deleted inputs select the contract", () => {
  const f = fixture();
  try {
    for (const path of ["tools/plan-input.mjs", "tests/tooling/support/plan.ts"]) {
      assert.ok(
        planToolingTests([path], { cwd: f.cwd }).tests.includes(
          "tests/tooling/davinci-phase4-contract-status.test.ts",
        ),
      );
    }
    rmSync(join(f.cwd, "tools/plan-input.mjs"));
    assert.ok(
      planToolingTests(["tools/plan-input.mjs"], { cwd: f.cwd }).tests.includes(
        "tests/tooling/davinci-phase4-contract-status.test.ts",
      ),
    );
  } finally {
    f.cleanup();
  }
});

void test("unknown and shared dependency inputs restore the broad PR suite", () => {
  const f = fixture();
  try {
    for (const paths of [
      [],
      ["new-root/input.ts"],
      ["pnpm-lock.yaml"],
      ["tools/config/vite-plus/task-inputs.ts"],
    ]) {
      assert.equal(planToolingTests(paths, { cwd: f.cwd }).tests.length, 2);
    }
    // This audited plan contract has a curated scope, so an incomplete
    // closure keeps the broad inputs instead of trusting named paths.
    f.write("tests/tooling/davinci-phase4-contract-status.test.ts", "import(variable);");
    assert.deepEqual(planToolingTests(["davinci/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests, [
      "tests/tooling/davinci-phase4-contract-status.test.ts",
    ]);
    // An unscoped test with a computed import runs on the paths it names.
    f.write("tests/tooling/native.test.ts", 'import(join("davinci/vize_l1", variable));');
    f.write("tests/tooling/davinci-phase4-contract-status.test.ts", "");
    assert.deepEqual(planToolingTests(["davinci/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests, [
      "tests/tooling/native.test.ts",
    ]);
  } finally {
    f.cleanup();
  }
});

void test("merge planning restores every scenario, including directly changed deferred files", () => {
  const f = fixture();
  try {
    const paths = ["tests/tooling/lsp-smoke.test.ts"];
    assert.ok(!planToolingTests(paths, { cwd: f.cwd }).tests.includes(paths[0]));
    assert.deepEqual(
      planToolingTests(paths, { cwd: f.cwd, tier: "merge" }).tests,
      toolingTestFiles(f.cwd),
    );
    assert.throws(() => planToolingTests(paths, { cwd: f.cwd, tier: "unknown" }), /tier must/);
  } finally {
    f.cleanup();
  }
});

void test("pure typecheck and LSP helper contracts remain in T0; explicit runtime inventory exists", () => {
  const files = toolingTestFiles();
  for (const file of mergeOnlyToolingTests) assert.ok(files.includes(file), file);
  const plan = planToolingTests(["pnpm-lock.yaml"]);
  assert.ok(plan.tests.includes("tests/tooling/typecheck-dependency-gates.test.ts"));
  assert.ok(plan.tests.includes("tests/tooling/typecheck-baseline-ambient.test.ts"));
  assert.ok(plan.tests.includes("tests/tooling/tooling-test-selection.test.mjs"));
  assert.ok(!plan.tests.includes("tests/tooling/lsp-smoke.test.ts"));
  assert.ok(!plan.tests.includes("tests/tooling/typecheck-baseline-project.test.ts"));
});

void test("the runner keeps shared fixtures serial and rejects unrecognized or duplicate plans", () => {
  const available = ["tests/tooling/example.test.ts"];
  const plan = { version: 1, tier: "pr", tests: available };
  assert.deepEqual(toolingTestCommand(plan, available), [
    "--test",
    "--test-concurrency=1",
    ...available,
  ]);
  for (const tests of [["../../outside.test.ts"], [...available, ...available]]) {
    assert.throws(() => toolingTestCommand({ ...plan, tests }, available), /unrecognized/);
  }
  assert.throws(() => toolingTestCommand({ ...plan, version: 2 }, available), /invalid/);
  assert.throws(
    () => toolingTestCommand({ ...plan, tier: "merge", tests: [] }, available),
    /retain every test/,
  );
});

void test("real Nuxt host inputs reach both strict TypeScript project gates before merge", () => {
  const gates = [
    "tests/tooling/typescript-tooling-project.test.ts",
    "tests/tooling/typescript-ci-gates.test.ts",
  ];
  for (const path of [
    "tools/support/compat/nuxt/nuxt2-module-build.ts",
    "tools/support/compat/nuxt/host-types.ts",
    "tools/support/compat/nuxt/nuxt2-typecheck.ts",
    // Deleted project members still invalidate the strict project checks.
    "tools/support/compat/nuxt/deleted-typecheck-probe.ts",
  ]) {
    const plan = planToolingTests([path], { tier: "pr" });
    assert.equal(plan.reason, "import closures and named repository paths");
    for (const gate of gates) assert.ok(plan.tests.includes(gate), `${path}: ${gate}`);
  }
  const prose = planToolingTests(["docs/content/integrations/nuxt.md"], { tier: "pr" });
  assert.ok(!prose.tests.includes(gates[0]), "prose does not change the TypeScript solution");
  // The CI gate contract dynamically loads an authored historical module.
  // Preserve the selector's existing broad fallback for incomplete imports.
  assert.ok(prose.tests.includes(gates[1]));
});
