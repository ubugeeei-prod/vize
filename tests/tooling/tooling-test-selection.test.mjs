import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  mergeOnlyToolingTests,
  pureToolingTests,
} from "../../tools/config/vite-plus/tooling-test-scopes.ts";
import {
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import {
  fastStructuralTests,
  planToolingFastPr,
} from "../../tools/support/compat/github/plan-tooling-fast-pr.mjs";
import { fastToolingTestArgs } from "../../tools/support/compat/github/run-tooling-fast-pr.mjs";
import {
  prToolingCohortCommand,
  toolingTestCommand,
} from "../../tools/support/compat/github/run-tooling-tests.mjs";
import {
  formatterEvidenceTest,
  mergeToolingShardCount,
  mergeToolingShardTests,
  partitionMergeToolingTests,
  toolingShardIds,
} from "../../tools/support/compat/github/tooling-merge-shards.mjs";
import { parse } from "yaml";

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

void test("unrelated source edits omit audited plan contracts but keep broad runtime checks", () => {
  const f = fixture();
  try {
    assert.deepEqual(planToolingTests(["crates/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests, [
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

void test("unknown, dynamic, and shared dependency inputs restore the broad PR suite", () => {
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
    f.write("tests/tooling/davinci-phase4-contract-status.test.ts", "import(variable);");
    assert.equal(planToolingTests(["crates/vize_l1/src/lib.rs"], { cwd: f.cwd }).tests.length, 2);
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

void test("audited pure PR tests run once while changed or unknown capabilities stay full", () => {
  const plan = planToolingTests(["Cargo.lock"]);
  const pure = prToolingCohortCommand(plan, "pure").slice(2);
  const full = prToolingCohortCommand(plan, "full").slice(2);
  assert.deepEqual(pure, plan.pureTests);
  assert.ok(pure.length > 0);
  assert.ok(pure.every((file) => pureToolingTests.includes(file)));
  assert.deepEqual([...pure, ...full].sort(), plan.tests);
  assert.equal(new Set([...pure, ...full]).size, plan.tests.length);
  for (const path of [
    "tests/tooling/davinci-generated-ledgers.test.ts",
    "tools/support/compat/davinci/generated-ledgers.mjs",
    "pnpm-lock.yaml",
    "new-root/unknown-file.ts",
  ]) {
    const changed = planToolingTests([path]);
    assert.deepEqual(changed.pureTests, [], path);
    assert.deepEqual(prToolingCohortCommand(changed, "full").slice(2), changed.tests);
  }
  assert.deepEqual(planToolingTests(["Cargo.lock"], { tier: "merge" }).pureTests, []);
  assert.throws(
    () => prToolingCohortCommand({ ...plan, pureTests: ["tests/tooling/unknown.test.ts"] }, "pure"),
    /invalid pure/,
  );
  assert.throws(() => prToolingCohortCommand(plan, "other"), /cohort must/);
});

void test("fast PR tooling admits only measured Rust and structural inputs", () => {
  const paths = [
    "Cargo.lock",
    "crates/vize_atelier_ssr/src/l4.rs",
    "crates/vize_atelier_vapor/src/compile.rs",
    "crates/vize_l0/src/dump/capture.rs",
    "crates/vize_l2/src/dump.rs",
    "docs/davinci/plan/storage-inventory.tsv",
    ...fastStructuralTests,
  ];
  const fast = planToolingFastPr(paths);
  assert.equal(fast.mode, "fast");
  assert.deepEqual(fast.tests, fastStructuralTests);
  assert.deepEqual(fastToolingTestArgs(fast), ["--test", "--test-concurrency=1", ...fastStructuralTests]);
  assert.equal(planToolingTests(paths, { tier: "merge" }).tests.length, toolingTestFiles().length);
  for (const outside of [
    "new-root/unknown.ts",
    "tests/tooling/new-contract.test.ts",
    "tests/tooling/support/davinci-stage-dependencies.ts",
    "tools/support/compat/github/plan-tooling-tests.mjs",
    ".github/workflows/pr-source-checks.yml",
    "npm/native/src/lib.rs",
    "pnpm-lock.yaml",
  ]) {
    assert.equal(planToolingFastPr([...paths, outside]).mode, "full", outside);
  }
  assert.equal(planToolingFastPr([]).mode, "full");
  assert.throws(() => fastToolingTestArgs({ ...fast, tests: [...fast.tests, fast.tests[0]] }), /invalid/);
  assert.throws(() => fastToolingTestArgs({ ...fast, tests: ["tests/tooling/new-contract.test.ts"] }), /invalid/);
});

void test("merge tooling shards cover every test once with isolated serial runners", () => {
  const files = toolingTestFiles();
  const shards = partitionMergeToolingTests(files);
  assert.equal(shards.length, mergeToolingShardCount);
  assert.deepEqual(shards.flat().sort(), files);
  assert.equal(new Set(shards.flat()).size, files.length);
  assert.ok(shards[0].includes(formatterEvidenceTest));
  assert.ok(shards.every((shard) => shard.length > 0));
  const slowest = [
    "tests/tooling/canon-functional-slot-contracts.test.ts",
    "tests/tooling/canon-upstream-diagnostics.test.ts",
    "tests/tooling/real-project-lsp.test.ts",
    "tests/tooling/canon-script-syntax-ownership.test.ts",
  ];
  assert.equal(
    new Set(slowest.map((file) => shards.findIndex((shard) => shard.includes(file)))).size,
    4,
  );
  assert.ok(
    Math.max(...shards.map((shard) => shard.length)) -
      Math.min(...shards.map((shard) => shard.length)) <=
      1,
  );
  for (const shardId of toolingShardIds("merge")) {
    assert.deepEqual(mergeToolingShardTests(files, shardId), shards[shardId - 1]);
  }
  assert.deepEqual(toolingShardIds("pr"), [1]);
  for (const id of [0, mergeToolingShardCount + 1, 1.5, NaN]) {
    assert.throws(() => mergeToolingShardTests(files, id), /shard must/);
  }
  assert.throws(
    () => partitionMergeToolingTests(files.filter((f) => f !== formatterEvidenceTest)),
    /evidence test/,
  );
  assert.throws(() => partitionMergeToolingTests([...files, files[0]]), /unique/);
  const newFile = "tests/tooling/new-unknown-merge-scenario.test.ts";
  const withNewFile = partitionMergeToolingTests([...files, newFile]);
  assert.deepEqual(withNewFile.flat().sort(), [...files, newFile].sort());
});

void test("merge matrix uses the planner's complete shards and the required report", () => {
  const workflow = parse(
    readFileSync(new URL("../../.github/workflows/pr-source-checks.yml", import.meta.url), "utf8"),
  );
  const job = workflow.jobs["pr-tooling-scripts"];
  assert.equal(
    workflow.jobs["pr-source-plan"].outputs["tooling-shards"],
    "${{ steps.tooling-plan.outputs.tooling-shards }}",
  );
  assert.equal(
    job.strategy.matrix.shard,
    "${{ fromJSON(needs.pr-source-plan.outputs.tooling-shards) }}",
  );
  assert.equal(job.strategy["fail-fast"], false);
  const mergeStep = job.steps.find((step) => step.name === "Test tooling scripts");
  assert.equal(mergeStep.run, "vp run --workspace-root test:scripts:merge-shard");
  assert.equal(mergeStep.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(mergeStep.env.VIZE_TOOLING_MERGE_SHARD, "${{ matrix.shard }}");
  assert.match(
    job.steps.find((step) => step.uses === "./.github/actions/upload-formatter-api-corpus-evidence")
      .if,
    /matrix\.shard == 1/u,
  );
  const check = parse(
    readFileSync(new URL("../../.github/workflows/check.yml", import.meta.url), "utf8"),
  );
  assert.ok(check.jobs["test-report"].needs.includes("pr-source-checks"));
  assert.ok(workflow.jobs["source-report"].needs.includes("pr-tooling-pure"));
  assert.ok(workflow.jobs["source-report"].needs.includes("pr-tooling-fast"));
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
