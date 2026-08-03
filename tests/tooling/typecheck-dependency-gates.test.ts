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

test("legacy Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    "check_legacy_vue2_class_bindings_cli.rs",
    "check_legacy_vue2_event_alias_cli.rs",
    "check_legacy_vue2_event_payload_cli.rs",
    "check_legacy_vue2_helpers_cli.rs",
    "check_legacy_vue2_no_unused_cli.rs",
    "check_legacy_vue2_template_scope_cli.rs",
    "check_legacy_vue2_vfor_cli.rs",
    "check_vue2_vuetify_props_cli.rs",
  ];

  for (const file of files) {
    const source = readRepoFile("crates", "vize", "tests", file);
    assert.match(source, /corsa_requirement::required_or_skip\(resolve_test_corsa_path\(\)\)/);
    assert.doesNotMatch(source, /let Some\(corsa_path\) = resolve_test_corsa_path\(\)/);
  }
});

test("Nuxt Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    ["check_nuxt2_classic_plugin_bindings_cli.rs", 1],
    ["check_nuxt2_fallback_guidance_cli.rs", 1],
    ["check_nuxt2_plugin_injections_cli.rs", 2],
    ["check_nuxt2_template_globals_cli.rs", 1],
    ["check_nuxt_ambient_cli.rs", 2],
    ["check_nuxt_bridge_shims_cli.rs", 1],
    ["check_nuxt_composition_api_cli.rs", 2],
    ["check_nuxt_composition_api_plugin_augmentation_cli.rs", 1],
    ["check_nuxt_legacy_tsconfig_cli.rs", 3],
    ["check_nuxt_monorepo_cli.rs", 2],
    ["check_nuxt_tsconfig_paths_cli.rs", 2],
  ] as const;

  for (const [file, expectedCalls] of files) {
    const source = readRepoFile("crates", "vize", "tests", file);
    assert.equal(
      source.match(/corsa_requirement::required_or_skip\(resolve_test_corsa_path\(\)\)/g)?.length,
      expectedCalls,
      `${file} should guard all ${expectedCalls} Corsa resolver calls`,
    );
    assert.doesNotMatch(source, /let Some\(corsa_path\) = resolve_test_corsa_path\(\)/);
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
