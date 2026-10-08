import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, sep } from "node:path";
import { after, test } from "node:test";
import {
  localImportInputs,
  planToolingTests,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const observers = [
  "tests/tooling/art-lint-public-boundaries.test.mjs",
  "tests/tooling/cli-build-enum-bindings-7893.test.mjs",
];
// The changed-path laws need only discovery and import closure, not 900 reads
// of unrelated real test bodies for every individual input control.
const fixtureRoot = mkdtempSync(join(tmpdir(), "vize-product-observer-inputs-"));
after(() => rmSync(fixtureRoot, { recursive: true, force: true }));
for (const directory of ["tests/tooling", "tests/tooling/davinci", "tests/tooling/release"])
  mkdirSync(join(fixtureRoot, directory), { recursive: true });
for (const file of observers) writeFileSync(join(fixtureRoot, file), "export {};\n");
const sourcePlan = (file) => planToolingTests([file], { cwd: fixtureRoot });

test("release prose and decision changes skip only the audited native product observers", () => {
  for (const file of [
    "docs/release/2026-10-08-p0-delivery-ledger.md",
    "docs/release/pr-workflow.md",
    "docs/davinci/decisions/2026-09-27-level-restructure.md",
  ]) {
    const plan = planToolingTests([file]);
    for (const observer of observers) assert.ok(!plan.tests.includes(observer), file);
    // An unaudited native observer retains conservative documentation inputs.
    assert.ok(plan.tests.includes("tests/tooling/lsp-bind-style-code-actions.test.ts"));
    assert.ok(plan.tests.includes("tests/tooling/release/release-readiness.test.ts"));
  }
});

test("source, fixture, build and manually loaded runtime inputs retain both whole observers", () => {
  for (const file of [
    "crates/vize_atelier_sfc/src/lib.rs",
    "davinci/vize_l1_to_l2/src/pass/cfg/source.rs",
    "npm/native/scripts/build-local.mjs",
    "npm/oxlint/src/index.ts",
    "npm/ui/package.json",
    "tests/_fixtures/differential/compiler/enum-template-bindings-7893/Badge.vue.txt",
    "tests/_fixtures/differential/linter/art-public-boundaries-7900/cases.json",
    "tests/tooling/support/enum-template-runtime-7893.mjs",
    "tests/tooling/support/davinci-mounted-trace.mjs",
    "tests/tooling/support/art-lint-source-binding.cjs",
    "tools/support/compat/github/native-setup-capture.mjs",
    ".github/workflows/pr-source-checks.yml",
    ".cargo/config.toml",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "pnpm-lock.yaml",
    "docs/davinci/plan/budgets.toml",
    "docs/davinci/plan/complexity-metrics.md",
    "README.md",
    "unclassified-new-product-input",
  ]) {
    const plan = sourcePlan(file);
    for (const observer of observers) assert.ok(plan.tests.includes(observer), file);
  }
  for (const observer of observers) {
    const imports = localImportInputs(observer);
    assert.ok(imports.complete, observer);
    assert.ok(imports.files.length > 1, observer);
    for (const file of imports.files) {
      assert.ok(sourcePlan(file).tests.includes(observer), file);
    }
  }
});

test("incomplete or nonliteral observer imports retain conservative docs qualification", () => {
  const file = join(fixtureRoot, observers[0]);
  try {
    for (const source of [
      'import "./missing-input.mjs";\n',
      "const source = globalThis.observerInput; await import(source);\n",
    ]) {
      writeFileSync(file, source);
      assert.ok(sourcePlan("docs/release/pr-workflow.md").tests.includes(observers[0]));
    }
    const dependency = join(fixtureRoot, "tests/tooling/support/dependency.mjs");
    mkdirSync(dirname(dependency), { recursive: true });
    writeFileSync(dependency, 'export { value } from "./transitive.mjs";\n');
    writeFileSync(join(dirname(dependency), "transitive.mjs"), "export const value = 1;\n");
    writeFileSync(file, 'import { value } from "./support/dependency.mjs";\n');
    const closure = localImportInputs(observers[0], fixtureRoot);
    assert.ok(closure.complete);
    assert.ok(
      closure.files.some(
        (file) => file.split(sep).join("/") === "tests/tooling/support/transitive.mjs",
      ),
    );
    assert.ok(sourcePlan("tests/tooling/support/transitive.mjs").tests.includes(observers[0]));
  } finally {
    writeFileSync(file, "export {};\n");
  }
});

test("full merge qualification always retains both original product observers", () => {
  for (const paths of [[], ["docs/release/pr-workflow.md"], ["unclassified-input"]]) {
    const plan = planToolingTests(paths, { tier: "merge" });
    assert.equal(plan.tests.length, plan.totalTests);
    assert.equal(plan.deferredTests, 0);
    for (const observer of observers) assert.ok(plan.tests.includes(observer));
  }
});
