//! A plain `<script>` template resolves every name on the component instance —
//! but an instance global read inside a block-bodied handler
//! (`@click="() => { $el.focus() }"`) was never declared, surfacing as
//! `TS2339 … does not exist on the component instance`. `vue-tsc` reports
//! nothing.
//!
//! Pinned to the installed `vue`: the `$el` instance surface is Vue's own.

use super::super::{
    BatchTypeChecker, TypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
    with_workspace_node_modules_override,
};
use std::path::Path;
use vize_s0::{String, cstr};

const HANDLER_BLOCK_INSTANCE_GLOBAL: &str = r#"<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  methods: {
    focus() {
      ;(this.$el as HTMLElement).focus()
    },
  },
})
</script>

<template>
  <button
    @click="
      () => {
        const target = $el as HTMLElement
        target.focus()
      }
    "
  >
    focus
  </button>
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

/// `vue-tsc` reports nothing: `$el` inside the handler's block body reads the
/// component instance like any other template expression.
#[test]
fn instance_globals_resolve_inside_block_bodied_handlers() {
    let Some(snapshot) = options_api_diagnostics(
        "options-api-handler-block-instance-global",
        &[("src/Handler.vue", HANDLER_BLOCK_INSTANCE_GLOBAL)],
    ) else {
        return;
    };
    assert_eq!(
        snapshot,
        Vec::<(String, Option<u32>, String)>::new(),
        "an instance global inside a block-bodied handler resolves on the instance"
    );
}
