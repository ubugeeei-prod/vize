import assert from "node:assert/strict";
import { textFile } from "../shared/files.ts";
const output = textFile("target/vize-tests/async-component/units.stdout");
const names = [
  "own_slot_scope_does_not_shadow_its_host_component",
  "template_shadowed_async_and_static_sites_do_not_revive_name_fallbacks",
  "mixed_template_sites_keep_only_the_unshadowed_root_site",
  "shadowed_prop_sites_do_not_reuse_the_root_component_contract",
  "shadowed_global_component_sites_do_not_create_provide_ancestry",
  "opaque_loader_reads_preserve_an_independent_proven_component",
  "actual_uncaptured_refused_and_empty_programs_never_alias_absence",
  "unsupported_top_level_try_refuses_the_original_domain",
  "demanded_builtin_walk_retains_the_same_lexical_bindings",
  "loop_lvalues_and_var_hoisting_cannot_keep_stale_loader_identity",
  "cached_identity_keeps_each_use_tdz_independent",
  "long_aliases_and_many_functions_use_indexed_iterative_identity",
  "stable_opaque_functions_and_const_leaves_keep_identity",
  "reassignment_tdz_and_mutable_bindings_refuse_value_authority",
  "closure_parameters_and_hoisted_function_shadow_imports",
  "incomplete_and_two_program_domains_refuse_but_absent_script_preserves",
  "async_component_ancestry_accepts_alias_namespace_and_exact_loader_forms",
  "async_component_ancestry_rejects_fake_opaque_nonliteral_and_uncalled_nested_loaders",
  "async_component_ancestry_keeps_static_import_and_kebab_alias_controls",
];
const lines = output.split(/\r?\n/u);
for (const name of names)
  assert.ok(
    lines.some((line) => line.endsWith("::" + name + " ... ok")),
    name,
  );
assert.ok(output.includes("0 failed;"));
