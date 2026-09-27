import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, test } from "node:test";
import {
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const corpus = [
  "tests/tooling/davinci-fact-spec-corpus.test.ts",
  "tests/tooling/davinci-complexity-corpus.test.ts",
  "tests/tooling/davinci-metamorphic-corpus.test.ts",
];
const moon = "tests/tooling/moonbit-warnings.test.ts";
const bench = "tests/tooling/davinci-bench-compare.test.ts";
const pack = "tests/tooling/vite-plus-fast-path.test.ts";
const heavy = [...corpus, moon, bench];
const scopeFixture = mkdtempSync(join(tmpdir(), "vize-heavy-scope-"));
for (const directory of [
  "tests/tooling",
  "tests/tooling/davinci",
  "tests/tooling/release",
  "tests/tooling/_helpers",
]) {
  mkdirSync(join(scopeFixture, directory), { recursive: true });
}
for (const file of heavy) writeFileSync(join(scopeFixture, file), "// scoped subprocess fixture\n");
writeFileSync(join(scopeFixture, moon), 'import "./_helpers/moonbit.ts";\n');
writeFileSync(join(scopeFixture, "tests/tooling/_helpers/moonbit.ts"), "// imported helper\n");
after(() => rmSync(scopeFixture, { recursive: true, force: true }));
const selected = (path) => new Set(planToolingTests([path], { cwd: scopeFixture }).tests);

void test("unrelated guide docs omit audited heavy subprocesses while full merge retains them", () => {
  const pr = selected("docs/content/guide.md");
  const merge = planToolingTests(["docs/content/guide.md"], { tier: "merge", cwd: scopeFixture });
  for (const file of heavy) {
    assert.equal(pr.has(file), false, file);
    assert.ok(merge.tests.includes(file), file);
  }
  assert.deepEqual(merge.tests, toolingTestFiles(scopeFixture));
});

void test("Rust corpus inputs include compiler, hydrated shards, matrix, harness and threshold specs", () => {
  for (const path of [
    "crates/vize_l1/src/parser.rs",
    "crates/vize_croquis/tests/fact_spec/runner.rs",
    "Cargo.toml",
    "Cargo.lock",
    ".cargo/config.toml",
    "rust-toolchain.toml",
    ".github/workflows/pr-source-checks.yml",
    "tests/_fixtures/_git/create-vue",
    "tests/_fixtures/_git/ant-design-vue",
    "tests/fixtures/davinci-matrix/deleted.vue",
    "tools/benchmarks/crates/davinci_harness/src/fixtures.rs",
    "docs/davinci/plan/complexity-metrics.md",
    "docs/davinci/plan/budgets.toml",
    "npm/cli/schemas/vize.config.schema.json",
  ]) {
    const files = selected(path);
    for (const file of corpus) assert.ok(files.has(file), `${path} -> ${file}`);
  }
});

void test("Moon build inputs retain module, library, toolchain and imported helper without unrelated crates", () => {
  assert.equal(selected("crates/vize_l1/src/parser.rs").has(moon), false);
  for (const path of [
    "tools/moon/moon.mod",
    "tools/moon/lib/process.mbt",
    "tools/moon/cmd/new_command/main.mbt",
    "tools/moon/cmd/new_command/moon.pkg",
    ".moonbit-version",
    "tools/nix/moonbit.nix",
    ".github/actions/setup-moonbit/action.yml",
    "tests/tooling/_helpers/moonbit.ts",
  ]) {
    assert.ok(selected(path).has(moon), path);
  }
});

void test("standalone bench comparison retains Rust script, common helper and reports without workspace source", () => {
  assert.equal(selected("crates/vize_l1/src/parser.rs").has(bench), false);
  assert.equal(selected("docs/davinci/plan/phase-4.md").has(bench), false);
  for (const path of [
    "tools/commands/davinci/bench-compare.rs",
    "tools/support/common.rs",
    "tests/_fixtures/davinci-bench-compare/budgets.toml",
    "tests/_fixtures/davinci-bench-compare/baseline/deleted.json",
    "tests/_fixtures/davinci-bench-compare/within-tolerance/current/new.json",
    "rust-toolchain.toml",
    ".cargo/config.toml",
    ".github/actions/setup-rust-script/action.yml",
  ]) {
    assert.ok(selected(path).has(bench), path);
  }
});

void test("real test import closures permit the five scopes while package generation retains fallback", () => {
  const guide = new Set(planToolingTests(["docs/content/guide.md"]).tests);
  for (const file of heavy) assert.equal(guide.has(file), false, file);
  // Generated fixture code imports App.vue relative to a temporary consumer.
  // The current literal-import scanner cannot prove that resolution, so the
  // package test must keep its existing broad coverage.
  assert.ok(guide.has(pack));
  const merge = planToolingTests(["docs/content/guide.md"], { tier: "merge" });
  assert.deepEqual(merge.tests, toolingTestFiles());
});

void test("direct changes, shared inputs and unknown paths retain every heavy scenario", () => {
  for (const file of heavy) assert.ok(selected(file).has(file), file);
  for (const path of ["pnpm-lock.yaml", "package.json", "pnpm-workspace.yaml", "new-root/file"]) {
    const files = selected(path);
    for (const file of heavy) assert.ok(files.has(file), `${path} -> ${file}`);
  }
});

void test("a scoped subprocess test with unresolved runtime imports restores broad inputs", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-heavy-inputs-"));
  try {
    for (const directory of ["tests/tooling", "tests/tooling/davinci", "tests/tooling/release"]) {
      mkdirSync(join(cwd, directory), { recursive: true });
    }
    writeFileSync(join(cwd, moon), 'import "./missing-helper.mjs";\n');
    const plan = planToolingTests(["docs/content/guide.md"], { cwd });
    assert.deepEqual(plan.tests, [moon]);
    writeFileSync(join(cwd, moon), "import(dynamicHelper);\n");
    assert.deepEqual(planToolingTests(["docs/content/guide.md"], { cwd }).tests, [moon]);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
