import assert from "node:assert/strict";
import { test } from "node:test";

import { testAndBenchmarkTasks } from "../../config/vite-plus/tasks/test-benchmark.ts";
import {
  requireTypecheckDependency,
  stableTypeScriptRuntimePath,
  typecheckDependencySkip,
} from "./support/typecheck-dependency.ts";
import { readRepoFile } from "./support/github-workflows.ts";

test("test:scripts requires its typecheck dependencies", () => {
  const command = (testAndBenchmarkTasks["test:scripts"] as { command: string }).command;
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  assert.match(
    command,
    /&& VIZE_TEST_REQUIRE_TSGO=1 node --test --test-concurrency=1 tests\/tooling\/\*\.test\.ts tests\/tooling\/\*\.test\.mjs$/,
  );
  assert.match(
    workflow,
    /\n  test-scripts:\n[\s\S]*?git submodule update --init --force --depth 1 -- tests\/_fixtures\/_git\/ant-design-vue tests\/_fixtures\/_git\/create-vue && vp install --frozen-lockfile --prefer-offline[\s\S]*?\n  editor-extensions:\n/,
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
    "lsp-fallthrough-attrs.test.ts",
    "lsp-file-create-delete.test.ts",
    "lsp-typecheck-template.test.ts",
    "lsp-watcher-revalidation.test.ts",
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

test("Canon Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    ["check_canon_boolean_tsx_regressions_cli.rs", 2, []],
    ["check_canon_component_derived_props_cli.rs", 1, []],
    ["check_canon_component_refs_cli.rs", 2, []],
    ["check_canon_dynamic_component_props_cli.rs", 1, []],
    ["check_canon_fallthrough_attrs_cli.rs", 3, []],
    ["check_canon_generic_inference_cli.rs", 3, []],
    ["check_canon_generic_props_cli.rs", 1, []],
    ["check_canon_generic_sfc_mount_cli.rs", 1, []],
    ["check_canon_graphql_cli.rs", 2, []],
    ["check_canon_recent_issues_cli.rs", 5, []],
    ["check_canon_recent_type_regressions_cli.rs", 6, []],
    ["check_canon_remapped_key_cli.rs", 1, []],
    ["check_canon_slot_contracts_cli.rs", 1, []],
  ] as const;

  for (const [file, expectedCalls, supportFiles] of files) {
    const source = readRepoFile("crates", "vize", "tests", file);
    const guardSource = [
      source,
      ...supportFiles.map((supportFile) => readRepoFile("crates", "vize", "tests", supportFile)),
    ].join("\n");
    assert.equal(
      guardSource.match(/corsa_requirement::required_or_skip\(resolve_test_corsa_path\(\)\)/g)
        ?.length,
      expectedCalls,
      `${file} should guard all ${expectedCalls} Corsa resolver calls`,
    );
    assert.doesNotMatch(source, /let Some\(corsa_path\) = resolve_test_corsa_path\(\)/);
  }
});

test("project-root Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    ["check_default_run_project_root_cli.rs", 2],
    ["check_symlinked_node_modules_cli.rs", 1],
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

test("basic SFC Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    ["check_default_imports_cli.rs", 2],
    ["check_define_props_cli.rs", 1],
    ["check_function_props_cli.rs", 1],
    ["check_multistatement_v_on_cli.rs", 1],
    ["check_sfc_string_blocks_cli.rs", 1],
    ["options_api_props_long_comment_cli.rs", 1],
    ["options_api_props_spread_cli.rs", 2],
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

test("declaration emit Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    ["build_declaration_emit_cli.rs", 2],
    ["check_declaration_emit_cli.rs", 2],
    ["check_declaration_emit_vue_floor_cli.rs", 1],
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

test("template and TSX Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    ["check_tsx_intrinsic_elements_cli.rs", 2],
    ["check_tsx_sfc_attrs_cli.rs", 3],
    ["check_instance_props_cli.rs", 2],
    ["check_ref_enum_template_cli.rs", 1],
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

test("module import Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    ["check_allowjs_imports_cli.rs", 1],
    ["check_ambient_export_assignment_cli.rs", 1],
    ["check_ambient_imports_cli.rs", 1],
    ["check_directory_self_imports_cli.rs", 1],
    ["check_hoisted_workspace_cli.rs", 1],
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

test("plain script and reference Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    ["check_plain_script_named_exports_cli.rs", 2],
    ["check_plain_script_namespace_cli.rs", 3],
    ["check_reference_types_cli.rs", 3],
    ["check_sfc_import_suppressions_cli.rs", 1],
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

test("project resolution Rust CLI gates use the fail-closed Corsa helper", () => {
  const files = [
    ["check_entry_ignores_cli.rs", 2],
    ["check_package_cwd_cli.rs", 3],
    ["check_tsconfig_types_cli.rs", 2],
    ["check_vue_reexports_cli.rs", 1],
    ["check_workspace_package_boundary_cli.rs", 1],
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

test("main Rust CLI gate uses the fail-closed Corsa resolver", () => {
  const source = readRepoFile("crates", "vize", "tests", "check_cli.rs");
  assert.match(
    source,
    /fn resolve_test_corsa_path\(\) -> Option<String> \{\n    corsa_requirement::required_or_skip\(corsa_path::resolve\(workspace_root\(\)\)\)\n\}/,
  );
  assert.equal(
    source.match(/(?:let Some|if let Some)\(corsa_path\) = resolve_test_corsa_path\(\)/g)?.length,
    35,
    "check_cli.rs should route all 35 Corsa resolver calls through the guarded boundary",
  );
});

test("available dependencies never skip", () => {
  assert.equal(typecheckDependencySkip("/tmp/tsgo", "tsgo", "missing", true), false);
});

test("stable TypeScript runtime path targets the platform package", () => {
  const binExt = process.platform === "win32" ? ".exe" : "";
  assert.equal(
    stableTypeScriptRuntimePath("/repo").replaceAll("\\", "/"),
    `/repo/node_modules/@typescript/typescript-${process.platform}-${process.arch}/lib/tsc${binExt}`,
  );
});

test("an unavailable optional dependency keeps the local skip reason", () => {
  assert.equal(typecheckDependencySkip(undefined, "tsgo", "local opt-out", false), "local opt-out");
});

test("an unavailable required dependency fails closed", () => {
  assert.throws(
    () => typecheckDependencySkip(undefined, "TypeScript 7/Corsa runtime", "missing", true),
    {
      message: "TypeScript 7/Corsa runtime is required when VIZE_TEST_REQUIRE_TSGO=1",
    },
  );
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
