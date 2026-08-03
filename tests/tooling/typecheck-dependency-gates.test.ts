import assert from "node:assert/strict";
import { test } from "node:test";

import { testAndBenchmarkTasks } from "../../tools/vite-plus/tasks/test-benchmark.ts";
import {
  requireTypecheckDependency,
  typecheckDependencySkip,
} from "./support/typecheck-dependency.ts";
import { readRepoFile } from "./support/github-workflows.ts";

test("test:scripts requires its typecheck dependencies", () => {
  const command = (testAndBenchmarkTasks["test:scripts"] as { command: string }).command;
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  assert.match(
    command,
    /&& VIZE_TEST_REQUIRE_TSGO=1 node --test --test-concurrency=1 tests\/tooling\/\*\.test\.ts$/,
  );
  assert.match(
    workflow,
    /\n  test-scripts:\n[\s\S]*?git submodule update --init --depth 1 -- tests\/_fixtures\/_git\/ant-design-vue && vp install --frozen-lockfile --prefer-offline[\s\S]*?\n  editor-extensions:\n/,
  );
});

test("typecheck and LSP gates do not carry raw dependency skips", () => {
  const files = [
    "check-bench-gate.test.ts",
    "cli-check-collection.test.ts",
    "cli-check-diagnostics.test.ts",
    "cli-check-json-shape.test.ts",
    "cli-check-sub-package-dependency.test.ts",
    "lsp-auto-insertion.test.ts",
    "lsp-concurrent-edit-deadlock.test.ts",
    "lsp-corsa-crash-recovery.test.ts",
    "lsp-typecheck-template.test.ts",
    "typecheck-baseline-project.test.ts",
  ];

  for (const file of files) {
    const source = readRepoFile("tests", "tooling", file);
    assert.doesNotMatch(source, /t\.skip\(["'](?:Corsa|tsgo)/);
    assert.doesNotMatch(source, /skip:\s*(?:CHECKER|checkerPath|!fs\.existsSync\(vueTsc\))/);
    assert.match(source, /(?:requireTypecheckDependency|typecheckDependencySkip)\(/);
  }
});

test("available dependencies never skip", () => {
  assert.equal(typecheckDependencySkip("/tmp/tsgo", "tsgo", "missing", true), false);
});

test("an unavailable optional dependency keeps the local skip reason", () => {
  assert.equal(typecheckDependencySkip(undefined, "tsgo", "local opt-out", false), "local opt-out");
});

test("an unavailable required dependency fails closed", () => {
  assert.throws(() => typecheckDependencySkip(undefined, "tsgo binary", "missing", true), {
    message: "tsgo binary is required when VIZE_TEST_REQUIRE_TSGO=1",
  });
});

test("runtime skip gates use the same fail-closed policy", () => {
  const skipped: string[] = [];
  const t = { skip: (reason: string) => skipped.push(reason) };

  assert.equal(
    requireTypecheckDependency(t, undefined, "Corsa runtime", "local opt-out", false),
    undefined,
  );
  assert.deepEqual(skipped, ["local opt-out"]);
  assert.throws(() => requireTypecheckDependency(t, undefined, "Corsa runtime", "missing", true), {
    message: "Corsa runtime is required when VIZE_TEST_REQUIRE_TSGO=1",
  });
});
