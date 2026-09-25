//! A plain `<script>` template resolves every name on the component instance,
//! never on module scope — but an import spelled like a `methods` / `computed`
//! member still typed the template call as the import, so `{{ format(value) }}`
//! reported the import's arity. `vue-tsc` reports nothing.
//!
//! Pinned to the installed `vue`: `defineComponent` method typing is Vue's own.

use super::super::{
    BatchTypeChecker, TypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
    with_workspace_node_modules_override,
};
use std::path::Path;
use vize_s0::{String, cstr};

const FORMAT_MODULE: &str = r#"export function format(value: string, pattern: string): string {
  return value + pattern
}
"#;

const METHOD_SHADOWS_IMPORT: &str = r#"<script lang="ts">
import { defineComponent } from 'vue'
import { format } from './format'

export default defineComponent({
  props: { value: { type: String, required: true } },
  methods: {
    format(date: string): string {
      return format(date, 'yyyy/MM/dd')
    },
  },
})
</script>

<template>
  <span>{{ format(value) }}</span>
</template>
"#;

/// Diagnostics of an Options API project checked against the installed `vue`,
/// or `None` when the run has no Corsa binary or no installed `vue` to pin to.
fn options_api_diagnostics(
    case_name: &str,
    files: &[(&str, &str)],
) -> Option<Vec<(String, Option<u32>, String)>> {
    resolve_test_tsgo_binary()?;
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let test_node_modules = workspace_root.join("tests").join("node_modules");
    if !test_node_modules.join("vue/package.json").is_file() {
        return None;
    }
    let project_root = with_workspace_node_modules_override(
        Some(
            test_node_modules
                .to_str()
                .expect("test node_modules path should be UTF-8"),
        ),
        || create_project_case(case_name, files),
    );

    // The Options API surface is opt-in on the checker, exactly as `vize check`
    // turns it on for a project that authors plain `<script>` components.
    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.enable_options_api();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let mut snapshot: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}:error {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    let _ = std::fs::remove_dir_all(&project_root);
    Some(snapshot)
}

/// `vue-tsc` reports nothing: the template's `format` is the one-argument
/// method, not the two-argument import the method itself calls.
#[test]
fn options_api_method_owns_a_template_name_shared_with_an_import() {
    let Some(snapshot) = options_api_diagnostics(
        "options-api-method-shadows-import",
        &[
            ("src/format.ts", FORMAT_MODULE),
            ("src/Shadow.vue", METHOD_SHADOWS_IMPORT),
        ],
    ) else {
        return;
    };
    assert_eq!(
        snapshot,
        Vec::<(String, Option<u32>, String)>::new(),
        "a `methods` member shadows a same-named import in template scope"
    );
}
